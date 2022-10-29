use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

use jl_sys::{jl_adopt_thread, jl_gc_safepoint, jl_process_events};

use crate::{error::JlrsResult, memory::stack_frame::StackFrame};

use super::{queue::Receiver, AsyncRuntime, Message, MessageInner};

pub(crate) unsafe fn init_worker<R: AsyncRuntime, const N: usize>(
    worker_id: usize,
    recv_timeout: Duration,
    receiver: Receiver<Message>,
) -> std::thread::JoinHandle<JlrsResult<()>> {
    R::spawn_thread(move || run_async::<R, N>(worker_id, recv_timeout, receiver))
}

fn run_async<R: AsyncRuntime, const N: usize>(
    worker_id: usize,
    recv_timeout: Duration,
    receiver: Receiver<Message>,
) -> JlrsResult<()> {
    let mut base_frame = StackFrame::<N>::new_n();
    R::block_on(
        unsafe { run_inner::<R, N>(recv_timeout, receiver, &mut base_frame) },
        Some(worker_id),
    )
}

async unsafe fn run_inner<R: AsyncRuntime, const N: usize>(
    recv_timeout: Duration,
    receiver: Receiver<Message>,
    base_frame: &mut StackFrame<N>,
) -> JlrsResult<()> {
    let _ = jl_adopt_thread();

    let base_frame: &'static mut StackFrame<N> = std::mem::transmute(base_frame);
    let mut pinned = base_frame.pin();
    let base_frame = pinned.stack_frame();

    let free_stacks = {
        let mut free_stacks = VecDeque::with_capacity(N);
        for i in 0..N {
            free_stacks.push_back(i);
        }

        Rc::new(RefCell::new(free_stacks))
    };

    let running_tasks = {
        let mut running_tasks = Vec::with_capacity(N);
        for _ in 0..N {
            running_tasks.push(None);
        }

        Rc::new(RefCell::new(running_tasks.into_boxed_slice()))
    };

    loop {
        if free_stacks.borrow().len() == 0 {
            jl_process_events();
            R::yield_now().await;
            continue;
        }

        match R::timeout(recv_timeout, receiver.recv()).await {
            None => jl_gc_safepoint(),
            Some(Ok(msg)) => match msg.inner {
                MessageInner::Task(task) => {
                    let idx = free_stacks.borrow_mut().pop_front().unwrap();
                    let stack = base_frame.nth_stack(idx);

                    let task = {
                        let free_stacks = free_stacks.clone();
                        let running_tasks = running_tasks.clone();

                        R::spawn_local(async move {
                            task.call(stack).await;
                            free_stacks.borrow_mut().push_back(idx);
                            running_tasks.borrow_mut()[idx] = None;
                        })
                    };

                    running_tasks.borrow_mut()[idx] = Some(task);
                }
                MessageInner::BlockingTask(task) => {
                    let stack = base_frame.sync_stack();
                    task.call(stack);
                }
                MessageInner::PostBlockingTask(task) => {
                    let idx = free_stacks.borrow_mut().pop_front().unwrap();
                    let stack = base_frame.nth_stack(idx);

                    let task = {
                        let free_stacks = free_stacks.clone();
                        let running_tasks = running_tasks.clone();

                        R::spawn_local(async move {
                            task.post(stack).await;
                            free_stacks.borrow_mut().push_back(idx);
                            running_tasks.borrow_mut()[idx] = None;
                        })
                    };

                    running_tasks.borrow_mut()[idx] = Some(task);
                }
                MessageInner::Include(task) => {
                    let stack = base_frame.sync_stack();
                    task.call(stack);
                }
                // TODO: make this atomic in julia
                MessageInner::ErrorColor(task) => {
                    let stack = base_frame.sync_stack();
                    task.call(stack);
                }
            },
            _ => break,
        }
    }

    // Wait for all tasks to complete without blocking the thread.
    for i in 0..N {
        let task = running_tasks.borrow_mut()[i].take();
        if let Some(task) = task {
            task.await;
            jl_gc_safepoint()
        }
    }

    Ok(())
}

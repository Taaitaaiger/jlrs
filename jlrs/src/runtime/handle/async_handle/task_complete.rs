use std::{cell::RefCell, future::Future, rc::Rc, task::Waker};

#[derive(Clone)]
struct TaskCompleteStateInner {
    waker: Option<Waker>,
    completed: bool,
}

impl TaskCompleteStateInner {
    fn new() -> Self {
        TaskCompleteStateInner {
            waker: None,
            completed: false,
        }
    }
}

#[derive(Clone)]
pub(super) struct TaskCompleteState {
    inner: Rc<RefCell<TaskCompleteStateInner>>,
}

impl TaskCompleteState {
    pub(super) fn new() -> Self {
        TaskCompleteState {
            inner: Rc::new(RefCell::new(TaskCompleteStateInner::new())),
        }
    }

    pub(super) fn complete(&self) {
        let mut borrowed = self.inner.borrow_mut();
        borrowed.completed = true;
        borrowed.waker.as_ref().map(Waker::wake_by_ref);
    }
}

impl Drop for TaskCompleteState {
    fn drop(&mut self) {
        // If complete is still false, we've panicked. If there are no open task slots the
        // executor might be waiting on for TaskComplete to resolve so mark the task as
        // completed.
        if !self.inner.borrow().completed {
            self.complete();
        }
    }
}

pub(super) struct TaskComplete {
    shared_state: TaskCompleteState,
}

impl TaskComplete {
    pub(super) fn new(state: &TaskCompleteState) -> Self {
        TaskComplete {
            shared_state: state.clone(),
        }
    }

    pub(super) fn clear(&self) -> &Self {
        self.shared_state.inner.borrow_mut().completed = false;
        self
    }
}

impl Future for &TaskComplete {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.shared_state.inner.borrow().completed {
            std::task::Poll::Ready(())
        } else if self.shared_state.inner.borrow().waker.is_none() {
            self.shared_state.inner.borrow_mut().waker = Some(cx.waker().clone());
            std::task::Poll::Pending
        } else {
            std::task::Poll::Pending
        }
    }
}

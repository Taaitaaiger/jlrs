use std::{collections::HashMap, ops::AddAssign};

use jl_sys::{jl_adopt_thread, jl_enter_threaded_region};
use jlrs::{
    call::Call,
    data::{
        managed::{
            string::StringRet,
            value::{
                ValueRet,
                tracked::Tracked,
                typed::{TypedValue, TypedValueRet},
            },
        },
        types::{construct_type::ConstructType, foreign_type::mark::Mark},
    },
    memory::gc::{self, write_barrier},
    prelude::{
        ForeignType, JlrsResult, JuliaString, LocalScope, Managed, OpaqueType, Value, WeakValue,
    },
    weak_handle_unchecked,
};
use jlrs_sys::{jlrs_gc_safe_enter, jlrs_get_ptls_states, jlrs_ptls_from_gcstack};

#[derive(Clone, Debug, OpaqueType)]
pub struct OpaqueInt {
    a: i32,
}

impl OpaqueInt {
    pub fn new(value: i32) -> TypedValueRet<OpaqueInt> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        TypedValue::new(weak_handle, OpaqueInt { a: value }).leak()
    }

    pub fn increment(&mut self) {
        self.a += 1;
    }

    pub fn get(&self) -> i32 {
        self.a
    }

    pub fn get_cloned(self) -> i32 {
        self.a
    }
}

#[derive(Clone, OpaqueType)]
pub struct POpaque<T> {
    value: T,
}

impl<T> POpaque<T>
where
    T: 'static
        + Send
        + Sync
        + ConstructType
        + AddAssign
        + Copy
        + jlrs::convert::into_julia::IntoJulia,
{
    pub fn new(value: T) -> TypedValueRet<Self> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        let data = POpaque { value };
        TypedValue::new(weak_handle, data).leak()
    }

    pub fn get(&self) -> T {
        self.value
    }

    pub fn get_cloned(self) -> T {
        self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
    }
}

#[derive(Clone, OpaqueType)]
pub struct POpaqueTwo<T, U> {
    value: T,
    value2: U,
}

impl<T, U> POpaqueTwo<T, U>
where
    T: 'static
        + Send
        + Sync
        + ConstructType
        + AddAssign
        + Copy
        + jlrs::convert::into_julia::IntoJulia,
    U: 'static
        + Send
        + Sync
        + ConstructType
        + AddAssign
        + Copy
        + jlrs::convert::into_julia::IntoJulia,
{
    pub fn new(value: T, value2: U) -> TypedValueRet<Self> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        let data = POpaqueTwo { value, value2 };
        TypedValue::new(weak_handle, data).leak()
    }

    pub fn get_v1(&self) -> T {
        self.value
    }

    pub fn get_v2(&self) -> U {
        self.value2
    }
}

unsafe fn mark_map<M: Mark, P: jlrs::data::types::foreign_type::ForeignType>(
    data: &HashMap<(), M>,
    ptls: jlrs::memory::PTls,
    parent: &P,
) -> usize {
    data.values().map(|v| unsafe { v.mark(ptls, parent) }).sum()
}

#[derive(ForeignType)]
pub struct ForeignThing {
    #[jlrs(mark)]
    a: WeakValue<'static, 'static>,
    #[jlrs(mark_with = mark_map)]
    b: HashMap<(), WeakValue<'static, 'static>>,
}

unsafe impl Send for ForeignThing {}
unsafe impl Sync for ForeignThing {}

impl ForeignThing {
    pub fn new(value: Value<'_, 'static>) -> TypedValueRet<ForeignThing> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        TypedValue::new(
            weak_handle,
            ForeignThing {
                a: value.leak(),
                b: HashMap::default(),
            },
        )
        .leak()
    }

    pub fn get(&self) -> ValueRet {
        unsafe { self.a.assume_owned().leak() }
    }

    pub fn set(&mut self, value: Value) {
        unsafe {
            self.a = value.assume_owned().leak();
            write_barrier(self, value);
        }
    }
}

pub struct UnexportedType;

impl UnexportedType {
    pub fn assoc_func() -> isize {
        1
    }
}

#[repr(C)]
#[derive(Clone, Debug, OpaqueType)]
pub struct Environment {
    s: String,
}
impl Environment {
    pub fn to_string(&self) -> StringRet {
        let handle = unsafe { weak_handle_unchecked!() };
        JuliaString::new(handle, self.s.clone()).leak()
    }
}

#[repr(C)]
#[derive(Clone, Debug, OpaqueType)]
pub struct Action {
    s: String,
}
impl Action {
    pub fn new(s: JuliaString<'_>) -> JlrsResult<TypedValueRet<Self>> {
        let x = Self {
            s: s.as_str()?.to_string(),
        };
        let handle = unsafe { weak_handle_unchecked!() };
        Ok(TypedValue::new(handle, x).leak())
    }
}

/// A structure for callbacks
#[derive(Clone, Debug, ForeignType)]
pub struct Agent {
    #[jlrs(mark)]
    callback: ValueRet,
}
unsafe impl Send for Agent {}
unsafe impl Sync for Agent {}
impl Agent {
    pub fn new(callback: Value<'_, 'static>) -> JlrsResult<TypedValueRet<Self>> {
        let handle = unsafe { weak_handle_unchecked!() };
        let data = Self {
            callback: callback.leak(),
        };
        Ok(TypedValue::new(handle, data).leak())
    }
    fn act(&self, env: Environment) -> Action {
        use jlrs::memory::gc::Gc;
        unsafe {
            gc::gc_unsafe(|handle| {
                handle.local_scope::<_, 3>(|mut frame| {
                    let callback = self.callback.as_value();
                    let s = JuliaString::new(&mut frame, env.s).as_value();
                    let result = callback.call(&mut frame, [s]).expect("Error 1");
                    result.leak().as_value().unbox::<Action>().expect("Not an action")
                })
            })
        }
    }
    async fn async_act(&self, env: Environment) -> Action {
        let handle = unsafe { weak_handle_unchecked!() };
        unsafe {
            gc::gc_unsafe(|handle| {
                handle.local_scope::<_, 3>(|mut frame| {
                    let callback = self.callback.as_value();
                    let s = JuliaString::new(&mut frame, env.s).as_value();
                    let result = gc::gc_safe(|| { callback.call(&mut frame, [s]) }).expect("Error 1");
                    result.leak().as_value().unbox::<Action>().expect("Not an action")
                })
            })
        }
    }
}

fn play_loop(agent: Agent, steps: usize) -> String {
    let env = Environment { s: "".to_string() };
    let mut actions = vec![];
    for _i in 0..steps {
        let Action { s } = agent.act(env.clone());
        actions.push(s);
    }
    actions.join("/")
}
pub fn play(agent: TypedValue<'_, '_, Agent>, steps: isize) -> JlrsResult<StringRet> {
    let agent_r = agent.unbox::<Agent>()?;
    let handle = unsafe { weak_handle_unchecked!() };
    let t = unsafe { gc::gc_safe(|| play_loop(agent_r, steps as usize)) };
    Ok(JuliaString::new(handle, t).leak())
}

#[derive(OpaqueType)]
pub struct Playground {
    runtime: tokio::runtime::Runtime,
}

impl Playground {
    pub fn new() -> TypedValueRet<Self> {
        let handle = unsafe { weak_handle_unchecked!() };
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(16)
            .on_thread_start(|| {
                let pgcstack = unsafe { jl_adopt_thread() };
                let mut ptls = unsafe { jlrs_get_ptls_states() };
                if ptls.is_null() {
                    ptls = unsafe { jlrs_ptls_from_gcstack(pgcstack) };
                }
                unsafe { jlrs_gc_safe_enter(ptls) };
                unsafe { jl_enter_threaded_region() };
            })
            .on_thread_stop(|| {
                let ptls = unsafe { jlrs_get_ptls_states() };
                unsafe { jlrs_gc_safe_enter(ptls) };
            })
            .thread_name("trilliumjl")
            .build()
            .expect("Can't build");
        let data = Self { runtime };
        TypedValue::new(handle, data).leak()
    }
}

async fn multithreaded_play_loop(agent: Agent, steps: usize) -> String {
    use std::sync::Arc;
    use tokio::sync::{mpsc, Barrier};

    let mut handles = vec![];
    let (action_tx, mut action_rx) = mpsc::channel(10);
    let barrier = Arc::new(Barrier::new(steps));
    for i in 0..steps {
        let action_tx = action_tx.clone();
        let agent = agent.clone();
        let barrier = barrier.clone();
        let handle = tokio::task::spawn(async move {
            let s = (i + steps).to_string().repeat(100);
            let _wait_result = barrier.wait().await;
            let Action { s } = agent.async_act(Environment { s }).await;
            let _ = action_tx.send(s).await;
        });
        handles.push(handle);
    }
    let mut acc = String::new();
    for _i in 0..(steps / 2) {
        let Some(s) = action_rx.recv().await else {
            unreachable!();
        };
        acc = acc + &s;
    }
    handles.retain(|h| {
        if h.is_finished() {
            false
        } else {
            h.abort();
            true
        }
    });
    for h in handles.into_iter() {
        let _ = h.await;
    }
    return acc;
}

pub fn multithreaded_play(
    agent: TypedValue<'_, '_, Agent>,
    steps: isize,
    runtime: TypedValue<'_, '_, Playground>,
) -> JlrsResult<StringRet> {
    let agent_r = agent.unbox::<Agent>()?;
    let runtime: Tracked<Playground> = unsafe { runtime.track_shared() }?;
    let t = unsafe {
        gc::gc_safe(|| {
            runtime
                .runtime
                .block_on(multithreaded_play_loop(agent_r, steps as usize))
        })
    };
    let handle = unsafe { weak_handle_unchecked!() };
    Ok(JuliaString::new(handle, t).leak())
}

use std::{collections::HashMap, ops::AddAssign};

use jlrs::{
    call::Call,
    data::{
        managed::string::StringRet,
        managed::value::{
            typed::{TypedValue, TypedValueRet},
            ValueRet,
        },
        types::{construct_type::ConstructType, foreign_type::mark::Mark},
    },
    memory::gc::{self, write_barrier},
    prelude::{
        ForeignType, JlrsResult, JuliaString, LocalScope, Managed, OpaqueType, Value, WeakValue,
    },
    weak_handle_unchecked,
};

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
        unsafe {
            gc::gc_unsafe(|handle| {
                handle.local_scope::<_, 3>(|mut frame| {
                    let callback = self.callback.as_value();
                    let env = Value::new(&mut frame, env);
                    let result = callback.call(&mut frame, [env]).expect("Error 1");
                    result.leak().as_value().unbox::<Action>().unwrap()
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
pub fn play(agent: TypedValue<'_, '_, Agent>, steps: usize) -> JlrsResult<StringRet> {
    let agent_r = agent.unbox::<Agent>()?;
    let handle = unsafe { weak_handle_unchecked!() };
    let t = unsafe { gc::gc_safe(|| play_loop(agent_r, steps)) };
    Ok(JuliaString::new(handle, t).leak())
}

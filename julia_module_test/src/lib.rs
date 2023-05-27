use jlrs::{
    ccall::AsyncCallback,
    data::{
        managed::{
            array::{ArrayRet, TypedArrayUnbound},
            ccall_ref::CCallRef,
            rust_result::{RustResult, RustResultRet},
            value::typed::{TypedValue, TypedValueRef, TypedValueRet},
        },
        types::{
            abstract_types::{AnyType, Number},
            foreign_type::{ForeignType, OpaqueType},
        },
    },
    error::JlrsError,
    memory::gc::{mark_queue_obj, write_barrier},
    prelude::*,
};

unsafe extern "C" fn freestanding_func_trivial() {}

unsafe extern "C" fn freestanding_func_noargs() -> usize {
    0
}

unsafe extern "C" fn freestanding_func_bitsarg(a: usize) -> usize {
    a + 1
}

unsafe extern "C" fn freestanding_func_ref_bitsarg(usize_ref: CCallRef<usize>) -> usize {
    usize_ref.as_ref().unwrap() + 1
}

unsafe extern "C" fn freestanding_func_arrayarg(a: Array) -> usize {
    let elty = a.element_type();

    if let Ok(elty) = elty.cast::<DataType>() {
        elty.size().unwrap_or(std::mem::size_of::<usize>() as _) as _
    } else {
        std::mem::size_of::<usize>()
    }
}

unsafe extern "C" fn freestanding_func_ref_mutarg(module_ref: CCallRef<Module>) -> usize {
    let _module = module_ref.as_managed().unwrap();
    //let target = module.unrooted_target();
    //module.set_global_unchecked("MyGlobal", Value::nothing(&target));
    0
}

unsafe extern "C" fn freestanding_func_ref_any(value_ref: CCallRef<AnyType>) -> usize {
    let _dt = value_ref.as_value_ref().datatype();
    //println!("freestanding_func_ref_any {:?}", value.datatype());
    0
}

unsafe extern "C" fn freestanding_func_ref_abstract(value_ref: CCallRef<Number>) -> usize {
    let _dt = value_ref.as_value().unwrap().datatype();
    //println!("freestanding_func_ref_abstract {:?}", value.datatype());
    0
}

unsafe extern "C" fn freestanding_func_typevaluearg(a: TypedValue<usize>) -> usize {
    a.unbox_unchecked::<usize>()
}

unsafe extern "C" fn freestanding_func_ret_array(dt: DataType) -> ArrayRet {
    CCall::invoke(|mut frame| {
        Array::new_for::<_, _>(frame.as_extended_target(), (2, 2), dt.as_value())
            .unwrap()
            .leak()
    })
}

unsafe extern "C" fn freestanding_func_ret_rust_result(throw_err: Bool) -> RustResultRet<i32> {
    CCall::invoke(|mut frame| {
        if throw_err.as_bool() {
            RustResult::jlrs_error(
                frame.as_extended_target(),
                jlrs::error::JlrsError::exception("Error"),
            )
        } else {
            let v = TypedValue::new(&mut frame, 3);
            RustResult::ok(frame.as_extended_target(), v)
        }
        .leak()
    })
}

#[derive(Clone, Debug)]
struct OpaqueInt {
    a: i32,
}

unsafe impl OpaqueType for OpaqueInt {}

impl OpaqueInt {
    fn new(value: i32) -> TypedValueRet<OpaqueInt> {
        unsafe {
            CCall::invoke(|mut frame| TypedValue::new(&mut frame, OpaqueInt { a: value }).leak())
        }
    }

    fn increment(&mut self) -> RustResultRet<Nothing> {
        self.a += 1;

        unsafe {
            CCall::invoke(|mut frame| {
                let nothing = Value::nothing(&frame).as_typed_unchecked::<Nothing>();
                RustResult::ok(frame.as_extended_target(), nothing).leak()
            })
        }
    }

    fn get(&self) -> RustResultRet<i32> {
        unsafe {
            CCall::invoke(|mut frame| {
                let data = TypedValue::new(&mut frame, self.a);
                RustResult::ok(frame.as_extended_target(), data).leak()
            })
        }
    }

    fn get_cloned(self) -> RustResultRet<i32> {
        unsafe {
            CCall::invoke(|mut frame| {
                let data = TypedValue::new(&mut frame, self.a);
                RustResult::ok(frame.as_extended_target(), data).leak()
            })
        }
    }
}

pub struct ForeignThing {
    a: ValueRef<'static, 'static>,
}

unsafe impl Send for ForeignThing {}

unsafe impl ForeignType for ForeignThing {
    fn mark(ptls: jlrs::memory::PTls, data: &Self) -> usize {
        unsafe { mark_queue_obj(ptls, data.a) as usize }
    }
}

impl ForeignThing {
    fn new(value: Value) -> TypedValueRet<ForeignThing> {
        unsafe {
            CCall::invoke(|mut frame| {
                TypedValue::new(
                    &mut frame,
                    ForeignThing {
                        a: value.assume_owned().leak(),
                    },
                )
                .leak()
            })
        }
    }

    fn get(&self) -> RustResultRet<AnyType> {
        unsafe {
            let leaked = self.a.assume_owned().leak();
            let typed = TypedValueRef::<AnyType>::from_value_ref(leaked).as_managed();
            CCall::invoke(|mut frame| RustResult::ok(frame.as_extended_target(), typed).leak())
        }
    }

    fn set(&mut self, value: Value) -> RustResultRet<Nothing> {
        unsafe {
            self.a = value.assume_owned().leak();
            write_barrier(self, value);

            CCall::invoke(|mut frame| {
                let nothing = Value::nothing(&frame).as_typed_unchecked::<Nothing>();
                RustResult::ok(frame.as_extended_target(), nothing).leak()
            })
        }
    }
}

struct UnexportedType;

impl UnexportedType {
    fn assoc_func() -> isize {
        1
    }
}

fn async_callback(arr: TypedArrayUnbound<isize>) -> JlrsResult<impl AsyncCallback<isize>> {
    let arr = arr.track_shared_unbound()?;
    Ok(move || Ok(arr.as_slice().iter().sum()))
}

fn async_callback_init_err() -> JlrsResult<impl AsyncCallback<isize>> {
    Err(JlrsError::exception("Err"))?;
    Ok(move || Ok(0))
}

fn async_callback_callback_err() -> JlrsResult<impl AsyncCallback<isize>> {
    Ok(move || Err(JlrsError::exception("Err"))?)
}

const CONST_U8: u8 = 1;
static STATIC_U8: u8 = 2;

julia_module! {
    become julia_module_tests_init_fn;

    fn freestanding_func_trivial();
    fn freestanding_func_noargs() -> usize;
    fn freestanding_func_bitsarg(a: usize) -> usize;
    fn freestanding_func_arrayarg(a: Array) -> usize;
    fn freestanding_func_ref_bitsarg(usize_ref: CCallRef<usize>) -> usize;
    fn freestanding_func_ref_any(value_ref: CCallRef<AnyType>) -> usize;
    fn freestanding_func_ref_mutarg(module_ref: CCallRef<Module>) -> usize;
    fn freestanding_func_ref_abstract(value_ref: CCallRef<Number>) -> usize;
    fn freestanding_func_typevaluearg(a: TypedValue<usize>) -> usize;
    fn freestanding_func_ret_array(dt: DataType) -> ArrayRet;
    fn freestanding_func_ret_rust_result(throw_err: Bool) -> RustResultRet<i32>;

    struct OpaqueInt;
    in OpaqueInt fn new(value: i32) -> TypedValueRet<OpaqueInt> as OpaqueInt;
    in OpaqueInt fn increment(&mut self) -> RustResultRet<Nothing> as increment!;
    in OpaqueInt fn get(&self) -> RustResultRet<i32> as unbox_opaque;
    in OpaqueInt fn get_cloned(self) -> RustResultRet<i32>;

    struct ForeignThing;
    in ForeignThing fn new(value: Value) -> TypedValueRet<ForeignThing> as ForeignThing;
    in ForeignThing fn get(&self) -> RustResultRet<AnyType> as extract_inner;
    in ForeignThing fn set(&mut self, value: Value) -> RustResultRet<Nothing> as set_inner!;

    in UnexportedType fn assoc_func() -> isize;

    #[doc = "    async_callback(array::Array{Int})::Int"]
    #[doc = ""]
    #[doc = "...docs for async_callback"]
    async fn async_callback(arr: TypedArrayUnbound<isize>) -> JlrsResult<impl AsyncCallback<isize>>;
    async fn async_callback_init_err() -> JlrsResult<impl AsyncCallback<isize>>;
    async fn async_callback_callback_err() -> JlrsResult<impl AsyncCallback<isize>>;

    const CONST_U8: u8;
    static CONST_U8: u8 as STATIC_CONST_U8;
    const STATIC_U8: u8 as CONST_STATIC_U8;
    static STATIC_U8: u8;

    /*

    struct Foo<T, U> {
        a: T,
        b: U
    }

    impl<T, U> Foo<T, U> {
        
    }

    


    unsafe impl<T, U> ParametricOpaqueType for Foo<T, U> 
    where 
        T: ConstructType, 
        U: ConstructType,
    {
        type Bounds = (
            TypeVarConstructor<Name<'T'>>,
            TypeVarConstructor<Name<'U'>>,
        )
    }

    for T in [f32, f64] {
        struct FFTPlanner<T>;
    }
    
     */
}

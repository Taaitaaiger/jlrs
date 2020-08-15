use jlrs::prelude::*;
use jlrs::util::JULIA;
use jlrs::value::code_instance::CodeInstance;
use jlrs::value::datatype::*;
use jlrs::value::expr::Expr;
use jlrs::value::method::Method;
use jlrs::value::method_instance::MethodInstance;
use jlrs::value::simple_vector::SimpleVector;
use jlrs::value::type_name::TypeName;
use jlrs::value::type_var::TypeVar;
use jlrs::value::union::Union;
use jlrs::value::union_all::UnionAll;

#[test]
fn datatype_methods() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_global, frame| {
            let val = Value::new(frame, 3.0f32)?;
            let dt = val.datatype().unwrap();

            assert_eq!(dt.size(), 4);
            assert_eq!(dt.align(), 4);
            assert_eq!(dt.nbits(), 32);
            assert_eq!(dt.nfields(), 0);
            assert!(dt.isinlinealloc());

            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn datatype_typechecks() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(1, |_global, frame| {
            let val = Value::new(frame, 3.0f32)?;
            let dt = val.datatype().unwrap();

            assert!(!dt.is::<Tuple>());
            assert!(!dt.is::<NamedTuple>());
            assert!(!dt.is::<SimpleVector>());
            assert!(!dt.is::<Mutable>());
            assert!(!dt.is::<MutableDatatype>());
            assert!(dt.is::<Immutable>());
            assert!(!dt.is::<ImmutableDatatype>());
            assert!(!dt.is::<Union>());
            assert!(!dt.is::<TypeVar>());
            assert!(!dt.is::<UnionAll>());
            assert!(!dt.is::<TypeName>());
            assert!(!dt.is::<i8>());
            assert!(!dt.is::<i16>());
            assert!(!dt.is::<i32>());
            assert!(!dt.is::<i64>());
            assert!(!dt.is::<u8>());
            assert!(!dt.is::<u16>());
            assert!(!dt.is::<u32>());
            assert!(!dt.is::<u64>());
            assert!(dt.is::<f32>());
            assert!(!dt.is::<f64>());
            assert!(!dt.is::<bool>());
            assert!(!dt.is::<char>());
            assert!(!dt.is::<Symbol>());
            assert!(!dt.is::<Array>());
            assert!(!dt.is::<Slot>());
            assert!(!dt.is::<Expr>());
            assert!(!dt.is::<GlobalRef>());
            assert!(!dt.is::<GotoNode>());
            assert!(!dt.is::<PhiNode>());
            assert!(!dt.is::<PhiCNode>());
            assert!(!dt.is::<UpsilonNode>());
            assert!(!dt.is::<QuoteNode>());
            assert!(!dt.is::<LineNode>());
            assert!(!dt.is::<MethodInstance>());
            assert!(!dt.is::<CodeInstance>());
            assert!(!dt.is::<Method>());
            assert!(!dt.is::<Module>());
            assert!(!dt.is::<String>());
            assert!(!dt.is::<Pointer>());
            assert!(!dt.is::<Intrinsic>());

            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn function_returns_datatype() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(1, |global, frame| {
            let dt = Module::main(global)
                .submodule("JlrsTests")?
                .function("datatype")?;
            let dt_val = dt.call0(frame)?.unwrap();

            assert!(dt_val.is::<DataType>());
            assert!(dt_val.cast::<DataType>().is_ok());

            Ok(())
        })
        .unwrap();
    })
}

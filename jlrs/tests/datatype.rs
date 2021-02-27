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

        jlrs.scope_with_slots(1, |_global, frame| {
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

        jlrs.scope_with_slots(1, |_global, frame| {
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
        jlrs.scope_with_slots(1, |global, frame| {
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

#[test]
fn datatype_has_typename() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            let tn = dt.type_name();
            let s: String = tn.name().into();

            assert_eq!(s, "TypeVar");

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_has_fieldnames() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            let tn = dt.field_names();

            assert_eq!(tn[0].as_string(), "name");
            assert_eq!(tn[1].as_string(), "lb");
            assert_eq!(tn[2].as_string(), "ub");

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_field_size() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            let sz = dt.field_size(1);
            assert_eq!(sz, 8);

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_field_offset() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            let sz = dt.field_offset(1);
            assert_eq!(sz, 8);

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_pointer_field() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(dt.is_pointer_field(1));

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_isbits() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(!dt.isbits());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_supertype() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(dt.super_type().is_some());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_parameters() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            assert_eq!(
                Value::array_int32_type(global)
                    .cast::<DataType>()
                    .unwrap()
                    .parameters()
                    .len(),
                2
            );

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_instance() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            assert!(Value::array_int32_type(global)
                .cast::<DataType>()
                .unwrap()
                .instance()
                .is_none());

            assert!(DataType::nothing_type(global).instance().is_some());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_ninitialized() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert_eq!(dt.n_initialized(), 3);

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_hash() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(dt.hash() != 0);

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_abstract() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(!dt.is_abstract());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_mutable() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(dt.mutable());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_hasfreetypevast() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(!dt.has_free_type_vars());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_concrete() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(dt.is_concrete_type());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_dispatchtuple() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(!dt.is_dispatch_tuple());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_zeroinit() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(!dt.zeroinit());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn datatype_concrete_subtype() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, _| {
            let dt = DataType::tvar_type(global);
            assert!(dt.has_concrete_subtype());

            Ok(())
        })
        .unwrap();
    })
}

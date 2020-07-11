mod util;

#[cfg(test)]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[derive(Copy, Clone, JuliaTuple, Eq, PartialEq, Debug)]
    #[repr(C)]
    struct UsizeAndIsize(usize, isize);

    #[derive(Copy, Clone, JuliaTuple, Eq, PartialEq, Debug)]
    #[repr(C)]
    struct Usizes(
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    );

    #[derive(Copy, Clone, JuliaTuple, PartialEq, Debug)]
    #[repr(C)]
    struct DifferentTypes(u8, u32, i64, f32, f64, i8, bool, u32, i8, i16);

    /*
    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.WithArray")]
    #[repr(C)]
    struct WithArray<'frame, 'data> {
        id: NotInline<u8>,
        array: Array<'frame, 'data>
    }

    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.WithValue")]
    #[repr(C)]
    struct WithValue<'frame, 'data> {
        id: u8,
        value: Value<'frame, 'data>
    }

    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.WithDataType")]
    #[repr(C)]
    struct WithDataType<'frame> {
        id: u8,
        datatype: DataType<'frame>
    }

    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.WithModule")]
    #[repr(C)]
    struct WithModule<'frame> {
        id: u8,
        module: Module<'frame>
    }

    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.WithSymbol")]
    #[repr(C)]
    struct WithSymbol<'frame> {
        id: u8,
        symbol: Symbol<'frame>
    }
    */

    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.Submodule.MyType")]
    #[repr(C)]
    struct MyType {
        v1: isize,
        v2: f64,
    }

    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.🍔")]
    #[repr(C)]
    struct Hamburger {
        #[jlrs(rename = "🥒")]
        pickle: i32,
        #[jlrs(rename = "🍅")]
        tomato: f32,
    }

    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.NoExist")]
    #[repr(C)]
    struct NoExist {
        foo: i16,
    }

    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.WrongType")]
    #[repr(C)]
    struct WrongType {
        foo: i8,
    }

    #[derive(Copy, Clone, JuliaStruct, PartialEq, Debug)]
    #[jlrs(julia_type = "Main.JlrsDeriveTests.WrongType")]
    #[repr(C)]
    struct WrongRename {
        #[jlrs(rename = "bar")]
        foo: i16,
    }
    
    #[test]
    fn test_union() {
    }

    #[test]
    fn derive_julia_tuple() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .frame(3, |_global, frame| {
                    let s = UsizeAndIsize(3, -4);
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();
                    let second = v.get_nth_field(frame, 1).unwrap();

                    assert_eq!(first.cast::<usize>().unwrap(), 3);
                    assert_eq!(second.cast::<isize>().unwrap(), -4);
                    assert!(v.is::<UsizeAndIsize>());
                    assert_eq!(v.cast::<UsizeAndIsize>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_usizes() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .frame(3, |_global, frame| {
                    let s = Usizes(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();
                    let second = v.get_nth_field(frame, 1).unwrap();

                    assert_eq!(first.cast::<usize>().unwrap(), 1);
                    assert_eq!(second.cast::<usize>().unwrap(), 2);
                    assert!(v.is::<Usizes>());
                    assert_eq!(v.cast::<Usizes>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_different_types() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .frame(3, |_global, frame| {
                    let s = DifferentTypes(21, 293, -7, 12.34, 56.78, -3, true, 12331123, -9, -295);
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();
                    let last = v.get_nth_field(frame, 9).unwrap();

                    assert_eq!(first.cast::<u8>().unwrap(), 21);
                    assert_eq!(last.cast::<i16>().unwrap(), -295);
                    assert!(v.is::<DifferentTypes>());
                    assert_eq!(v.cast::<DifferentTypes>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_julia_struct() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .frame(3, |_global, frame| {
                    let s = MyType { v1: -12, v2: 3.0 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_field(frame, "v1").unwrap();
                    let second = v.get_field(frame, "v2").unwrap();

                    assert_eq!(first.cast::<isize>().unwrap(), -12);
                    assert_eq!(second.cast::<f64>().unwrap(), 3.0);
                    assert!(v.is::<MyType>());
                    assert_eq!(v.cast::<MyType>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_renamed_julia_struct() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .frame(3, |_global, frame| {
                    let s = Hamburger {
                        pickle: -12,
                        tomato: 3.0,
                    };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_field(frame, "🥒").unwrap();
                    let second = v.get_field(frame, "🍅").unwrap();

                    assert_eq!(first.cast::<i32>().unwrap(), -12);
                    assert_eq!(second.cast::<f32>().unwrap(), 3.0);
                    assert!(v.is::<Hamburger>());
                    assert_eq!(v.cast::<Hamburger>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    #[should_panic]
    fn derive_noexist_julia_struct_panics() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .frame(3, |_global, frame| {
                    let s = NoExist { foo: 2 };
                    let _v = Value::new(frame, s).unwrap();
                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    #[should_panic]
    fn derive_wrong_type_julia_struct_panics() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .frame(3, |_global, frame| {
                    let s = WrongType { foo: 2 };
                    let _v = Value::new(frame, s).unwrap();
                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    #[should_panic]
    fn derive_wrong_rename_julia_struct_panics() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .frame(3, |_global, frame| {
                    let s = WrongRename { foo: 2 };
                    let _v = Value::new(frame, s).unwrap();
                    Ok(())
                })
                .unwrap()
        })
    }
}

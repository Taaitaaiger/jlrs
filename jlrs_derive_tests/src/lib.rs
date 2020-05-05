mod util;

#[cfg(test)]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[derive(Copy, Clone, JuliaTuple, Eq, PartialEq, Debug)]
    #[repr(C)]
    struct UsizeAndIsize(usize, isize);

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
    fn derive_julia_tuple() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .frame(3, |_global, frame| {
                    let s = UsizeAndIsize(3, -4);
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();
                    let second = v.get_nth_field(frame, 1).unwrap();

                    assert_eq!(first.try_unbox::<usize>().unwrap(), 3);
                    assert_eq!(second.try_unbox::<isize>().unwrap(), -4);
                    assert!(v.is::<UsizeAndIsize>());
                    assert_eq!(v.try_unbox::<UsizeAndIsize>().unwrap(), s);

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

                    assert_eq!(first.try_unbox::<isize>().unwrap(), -12);
                    assert_eq!(second.try_unbox::<f64>().unwrap(), 3.0);
                    assert!(v.is::<MyType>());
                    assert_eq!(v.try_unbox::<MyType>().unwrap(), s);

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
                    let s = Hamburger { pickle: -12, tomato: 3.0 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_field(frame, "🥒").unwrap();
                    let second = v.get_field(frame, "🍅").unwrap();

                    assert_eq!(first.try_unbox::<i32>().unwrap(), -12);
                    assert_eq!(second.try_unbox::<f32>().unwrap(), 3.0);
                    assert!(v.is::<Hamburger>());
                    assert_eq!(v.try_unbox::<Hamburger>().unwrap(), s);

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
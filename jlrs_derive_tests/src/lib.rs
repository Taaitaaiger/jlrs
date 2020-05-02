#[cfg(test)] 
mod tests {
    use jlrs::prelude::*;
    use jlrs_derive::JuliaTuple;

    #[derive(Copy, Clone, JuliaTuple, Eq, PartialEq, Debug)]
    #[repr(C)]
    struct UsizeAndIsize(usize, isize);
    
    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
        let mut julia = unsafe { Julia::init(16).unwrap() };

        julia.frame(3, |_global, frame| {
            let s = UsizeAndIsize(3, -4);
            let v = Value::new(frame, s).unwrap();
            let first = v.get_nth_field(frame, 0).unwrap();
            let second = v.get_nth_field(frame, 1).unwrap();

            assert_eq!(first.try_unbox::<usize>().unwrap(), 3);
            assert_eq!(second.try_unbox::<isize>().unwrap(), -4);
            assert_eq!(v.try_unbox::<UsizeAndIsize>().unwrap(), s);

            Ok(())
        }).unwrap()
    }
}
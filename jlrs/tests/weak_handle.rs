mod weak_handle {
    use jlrs::{prelude::*, weak_handle};

    fn uses_weak_handle() {
        match weak_handle!() {
            Ok(handle) => {
                handle.local_scope::<_, 0>(|_f| ());
            }
            Err(_e) => {
                panic!()
            }
        }
    }

    #[test]
    fn weak_handle() {
        let julia = Builder::new().start_local().unwrap();

        uses_weak_handle();

        std::mem::drop(julia);
    }
}

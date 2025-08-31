mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{convert::to_symbol::ToSymbol, data::managed::named_tuple::NamedTuple, prelude::*};

    use super::util::JULIA;

    fn create_named_tuple_from_n_pairs() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let name = "foo";
                    let value = Value::new(&mut frame, 1u32);
                    let name = name.to_symbol(&frame);
                    let nt = NamedTuple::from_n_pairs(&mut frame, &[(name, value)])
                        .unwrap()
                        .as_value();
                    assert!(nt.is::<NamedTuple>());
                    assert_eq!(
                        nt.get_field(&mut frame, "foo")
                            .unwrap()
                            .unbox::<u32>()
                            .unwrap(),
                        1u32
                    );
                })
            });
        });
    }

    fn create_named_tuple_from_iter() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let name = "foo";
                    let value = Value::new(&mut frame, 1u32);
                    let name = name.to_symbol(&frame);
                    let nt = NamedTuple::from_iter(&mut frame, [(name, value)].into_iter())
                        .unwrap()
                        .as_value();
                    assert!(nt.is::<NamedTuple>());
                    assert_eq!(
                        nt.get_field(&mut frame, "foo")
                            .unwrap()
                            .unbox::<u32>()
                            .unwrap(),
                        1u32
                    );
                })
            });
        });
    }

    fn create_named_tuple_new() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let name = "foo";
                    let value = Value::new(&mut frame, 1u32);
                    let name = name.to_symbol(&frame);
                    let nt = NamedTuple::new(&mut frame, &[name], &[value])
                        .unwrap()
                        .as_value();
                    assert!(nt.is::<NamedTuple>());
                    assert_eq!(
                        nt.get_field(&mut frame, "foo")
                            .unwrap()
                            .unbox::<u32>()
                            .unwrap(),
                        1u32
                    );
                })
            });
        });
    }

    fn create_named_tuple_macro() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let a_name = "a";
                    let a_value = Value::new(&mut frame, 1u32);
                    let b_value = Value::new(&mut frame, 2u64);
                    let nt = named_tuple!(&mut frame, a_name => a_value, "b" => b_value)
                        .expect("duplicate keys");
                    assert_eq!(
                        nt.as_value()
                            .get_field(&mut frame, a_name)
                            .unwrap()
                            .unbox::<u32>()
                            .unwrap(),
                        1u32
                    );
                    assert_eq!(
                        nt.as_value()
                            .get_field(&mut frame, "b")
                            .unwrap()
                            .unbox::<u64>()
                            .unwrap(),
                        2u64
                    );
                })
            });
        });
    }

    fn named_tuple_contains() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let b_value = Value::new(&mut frame, 2u64);
                    let nt = named_tuple!(&mut frame, "b" => b_value).unwrap();

                    assert!(!nt.contains("a"));
                    assert!(nt.contains("b"));
                })
            });
        });
    }

    fn named_tuple_get() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let val = 2u64;
                    let b_value = Value::new(&mut frame, val);
                    let nt = named_tuple!(&mut frame, "b" => b_value).unwrap();

                    assert!(nt.get(&frame, "a").is_none());
                    let b = nt.get(&mut frame, "b");
                    assert!(b.is_some());
                    assert_eq!(b.unwrap().unbox::<u64>().unwrap(), val);
                })
            });
        });
    }

    fn named_tuple_remove() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let val_a = 1u32;
                    let val_b = 2u64;
                    let a_value = Value::new(&mut frame, val_a);
                    let b_value = Value::new(&mut frame, val_b);
                    let nt = named_tuple!(&mut frame, "a" => a_value, "b" => b_value).unwrap();

                    assert!(nt.get(&frame, "a").is_some());
                    assert!(nt.get(&frame, "b").is_some());

                    let sym = "a".to_symbol(&frame);
                    let nt2 = nt.filter(&mut frame, &[sym]);

                    assert!(nt.get(&frame, "a").is_some());
                    assert!(nt2.get(&frame, "a").is_none());

                    let b = nt2.get(&mut frame, "b");
                    assert!(b.is_some());
                    assert_eq!(b.unwrap(), b_value);
                })
            });
        });
    }

    fn named_tuple_set() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let val_a = 1u32;
                    let val_b = 2u64;
                    let a_value = Value::new(&mut frame, val_a);
                    let b_value = Value::new(&mut frame, val_b);
                    let nt = named_tuple!(&mut frame, "a" => a_value).unwrap();

                    assert!(nt.get(&frame, "a").is_some());
                    assert!(nt.get(&frame, "b").is_none());

                    let sym = "b".to_symbol(&frame);
                    let nt2 = nt.set(&mut frame, sym, b_value);

                    assert!(nt.get(&frame, "b").is_none());
                    assert!(nt2.get(&frame, "a").is_some());

                    let b = nt2.get(&mut frame, "b");
                    assert!(b.is_some());
                    assert_eq!(b.unwrap(), b_value);
                })
            });
        });
    }

    fn named_tuple_filter() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let val_a = 1u32;
                    let val_b = 2u64;
                    let val_c = 3u64;
                    let a_value = Value::new(&mut frame, val_a);
                    let b_value = Value::new(&mut frame, val_b);
                    let c_value = Value::new(&mut frame, val_c);
                    let nt =
                        named_tuple!(&mut frame, "a" => a_value, "b" => b_value, "c" => c_value)
                            .unwrap();
                    let syms = &[
                        "a".to_symbol(&frame),
                        "b".to_symbol(&frame),
                        "d".to_symbol(&frame),
                    ];
                    let nt2 = nt.filter(&mut frame, syms);

                    assert!(nt2.get(&frame, "a").is_none());
                    assert!(nt2.get(&frame, "b").is_none());
                    assert!(nt2.get(&frame, "c").is_some());
                    assert!(nt2.get(&frame, "d").is_none());
                })
            });
        });
    }

    fn named_tuple_extend() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let val_a = 1u32;
                    let val_b = 2u64;
                    let val_c = 3u64;
                    let a_value = Value::new(&mut frame, val_a);
                    let b_value = Value::new(&mut frame, val_b);
                    let c_value = Value::new(&mut frame, val_c);
                    let nt = named_tuple!(&mut frame, "a" => a_value).unwrap();
                    let syms = &["a".to_symbol(&frame), "c".to_symbol(&frame)];
                    let values = &[b_value, c_value];
                    let nt2 = nt.extend(&mut frame, syms, values);

                    assert!(nt2.get(&frame, "a").is_some());
                    assert!(nt2.get(&frame, "b").is_none());
                    assert!(nt2.get(&frame, "c").is_some());
                    assert!(nt2.get(&frame, "d").is_none());

                    assert_eq!(nt2.get(&mut frame, "a").unwrap(), b_value);
                    assert_eq!(nt2.get(&mut frame, "c").unwrap(), c_value);
                })
            });
        });
    }

    fn named_tuple_extend_iter() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let val_a = 1u32;
                    let val_b = 2u64;
                    let val_c = 3u64;
                    let a_value = Value::new(&mut frame, val_a);
                    let b_value = Value::new(&mut frame, val_b);
                    let c_value = Value::new(&mut frame, val_c);
                    let nt = named_tuple!(&mut frame, "a" => a_value).unwrap();
                    let pairs = &[
                        ("a".to_symbol(&frame), b_value),
                        ("c".to_symbol(&frame), c_value),
                    ];
                    let nt2 = nt.extend_iter(&mut frame, pairs.into_iter().copied());

                    assert!(nt2.get(&frame, "a").is_some());
                    assert!(nt2.get(&frame, "b").is_none());
                    assert!(nt2.get(&frame, "c").is_some());
                    assert!(nt2.get(&frame, "d").is_none());

                    assert_eq!(nt2.get(&mut frame, "a").unwrap(), b_value);
                    assert_eq!(nt2.get(&mut frame, "c").unwrap(), c_value);
                })
            });
        });
    }

    fn named_tuple_filter_extend() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let val_a = 1u32;
                    let val_b = 2u64;
                    let val_c = 3u64;
                    let val_d = 4u64;
                    let val_e = 5u64;
                    let val_f = 6u64;

                    let a_value = Value::new(&mut frame, val_a);
                    let b_value = Value::new(&mut frame, val_b);
                    let c_value = Value::new(&mut frame, val_c);
                    let d_value = Value::new(&mut frame, val_d);
                    let e_value = Value::new(&mut frame, val_e);
                    let f_value = Value::new(&mut frame, val_f);

                    let nt =
                        named_tuple!(&mut frame, "a" => a_value, "b" => b_value, "c" => c_value)
                            .unwrap();
                    let to_remove = &["a".to_symbol(&frame), "b".to_symbol(&frame)];
                    let sym_to_add = &[
                        "b".to_symbol(&frame),
                        "c".to_symbol(&frame),
                        "d".to_symbol(&frame),
                    ];
                    let values_to_add = &[d_value, e_value, f_value];

                    let nt2 = nt.filter_extend(&mut frame, to_remove, sym_to_add, values_to_add);

                    assert!(nt2.get(&frame, "a").is_none());
                    assert!(nt2.get(&frame, "b").is_some());
                    assert!(nt2.get(&frame, "c").is_some());
                    assert!(nt2.get(&frame, "d").is_some());

                    assert_eq!(nt2.get(&mut frame, "b").unwrap(), d_value);
                    assert_eq!(nt2.get(&mut frame, "c").unwrap(), e_value);
                    assert_eq!(nt2.get(&mut frame, "d").unwrap(), f_value);
                })
            });
        });
    }

    fn named_tuple_filter_extend_iter() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    let val_a = 1u32;
                    let val_b = 2u64;
                    let val_c = 3u64;
                    let val_d = 4u64;
                    let val_e = 5u64;
                    let val_f = 6u64;

                    let a_value = Value::new(&mut frame, val_a);
                    let b_value = Value::new(&mut frame, val_b);
                    let c_value = Value::new(&mut frame, val_c);
                    let d_value = Value::new(&mut frame, val_d);
                    let e_value = Value::new(&mut frame, val_e);
                    let f_value = Value::new(&mut frame, val_f);

                    let nt =
                        named_tuple!(&mut frame, "a" => a_value, "b" => b_value, "c" => c_value)
                            .unwrap();
                    let to_remove = &["a".to_symbol(&frame), "b".to_symbol(&frame)];
                    let to_add = &[
                        ("b".to_symbol(&frame), d_value),
                        ("c".to_symbol(&frame), e_value),
                        ("d".to_symbol(&frame), f_value),
                    ];

                    let nt2 =
                        nt.filter_extend_iter(&mut frame, to_remove, to_add.into_iter().copied());

                    assert!(nt2.get(&frame, "a").is_none());
                    assert!(nt2.get(&frame, "b").is_some());
                    assert!(nt2.get(&frame, "c").is_some());
                    assert!(nt2.get(&frame, "d").is_some());

                    assert_eq!(nt2.get(&mut frame, "b").unwrap(), d_value);
                    assert_eq!(nt2.get(&mut frame, "c").unwrap(), e_value);
                    assert_eq!(nt2.get(&mut frame, "d").unwrap(), f_value);
                })
            });
        });
    }

    #[test]
    fn named_tuple_tests() {
        create_named_tuple_from_n_pairs();
        create_named_tuple_from_iter();
        create_named_tuple_new();
        create_named_tuple_macro();
        named_tuple_contains();
        named_tuple_get();
        named_tuple_remove();
        named_tuple_set();
        named_tuple_filter();
        named_tuple_extend();
        named_tuple_extend_iter();
        named_tuple_filter_extend();
        named_tuple_filter_extend_iter();
    }
}

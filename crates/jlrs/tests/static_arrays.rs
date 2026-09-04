mod util;
#[cfg(all(feature = "local-rt", feature = "static-arrays"))]
mod tests {
    use jlrs::{
        data::{
            layout::static_arrays::{
                dims::{Dims1D, Dims2D, Dims3D},
                m_array::MArray,
                m_matrix::MMatrix,
                m_vector::MVector,
                s_array::SArray,
                s_matrix::SMatrix,
                s_vector::SVector,
            },
            managed::value::typed::TypedValue,
        },
        prelude::*,
    };

    use super::util::JULIA;

    fn use_static_arrays() {
        JULIA.with(|j| unsafe {
            j.borrow().using("StaticArrays").unwrap();
        });
    }

    fn svector_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 2>(|mut frame| {
                let svector = SVector::new([1.0f32, 2.0f32, 3.0f32]);
                let original = svector.clone();
                let boxed_svector = Value::new(&mut frame, svector);
                assert!(boxed_svector.is::<SVector<f32, 3>>());
                unsafe {
                    let sum_fn = Module::main(&frame)
                        .global(&frame, "sum")
                        .unwrap()
                        .as_value();
                    let sum = sum_fn
                        .call(&mut frame, [boxed_svector])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sum, original.data().iter().sum())
                }

                let unboxed_svector = boxed_svector.unbox::<SVector<f32, 3>>().unwrap();
                assert_eq!(original.data(), unboxed_svector.data());
            })
        });
    }

    fn svector_ref_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 2>(|mut frame| {
                let svector = SVector::new([1.0f32, 2.0f32, 3.0f32]);
                let boxed_svector = Value::new(&mut frame, &svector);
                assert!(boxed_svector.is::<SVector<f32, 3>>());
                unsafe {
                    let sum_fn = Module::main(&frame)
                        .global(&frame, "sum")
                        .unwrap()
                        .as_value();
                    let sum = sum_fn
                        .call(&mut frame, [boxed_svector])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sum, svector.data().iter().sum())
                }

                let unboxed_svector = boxed_svector.unbox::<SVector<f32, 3>>().unwrap();
                assert_eq!(svector.data(), unboxed_svector.data());
            })
        });
    }

    fn mvector_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 2>(|mut frame| {
                let mvector = MVector::new([1.0f32, 2.0f32, 3.0f32]);
                let original = mvector.clone();
                let boxed_mvector = Value::new(&mut frame, mvector);
                assert!(boxed_mvector.is::<MVector<f32, 3>>());
                unsafe {
                    let sum_fn = Module::main(&frame)
                        .global(&frame, "sum")
                        .unwrap()
                        .as_value();
                    let sum = sum_fn
                        .call(&mut frame, [boxed_mvector])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sum, original.data().iter().sum())
                }

                let unboxed_mvector = boxed_mvector.unbox::<MVector<f32, 3>>().unwrap();
                assert_eq!(original.data(), unboxed_mvector.data());
            })
        });
    }

    fn mvector_ref_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 2>(|mut frame| {
                let mvector = MVector::new([1.0f32, 2.0f32, 3.0f32]);
                let boxed_mvector = Value::new(&mut frame, &mvector);

                assert!(boxed_mvector.is::<MVector<f32, 3>>());
                unsafe {
                    let sum_fn = Module::main(&frame)
                        .global(&frame, "sum")
                        .unwrap()
                        .as_value();
                    let sum = sum_fn
                        .call(&mut frame, [boxed_mvector])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sum, mvector.data().iter().sum())
                }

                let unboxed_mvector = boxed_mvector.unbox::<MVector<f32, 3>>().unwrap();
                assert_eq!(mvector.data(), unboxed_mvector.data());
            })
        });
    }

    fn smatrix_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let smatrix = SMatrix::new([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);
                let original = smatrix.clone();
                let boxed_smatrix = Value::new(&mut frame, smatrix);
                assert!(boxed_smatrix.is::<SMatrix<f32, 3, 2>>());

                let unboxed_smatrix = boxed_smatrix.unbox::<SMatrix<f32, 3, 2>>().unwrap();
                assert_eq!(original.data(), unboxed_smatrix.data());
            })
        });
    }

    fn smatrix_ref_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let smatrix = SMatrix::new([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);
                let original = smatrix.clone();
                let boxed_smatrix = Value::new(&mut frame, &smatrix);
                assert!(boxed_smatrix.is::<SMatrix<f32, 3, 2>>());

                let unboxed_smatrix = boxed_smatrix.unbox::<SMatrix<f32, 3, 2>>().unwrap();
                assert_eq!(original.data(), unboxed_smatrix.data());
            })
        });
    }

    fn mmatrix_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let mmatrix = MMatrix::new([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);
                let original = mmatrix.clone();
                let boxed_mmatrix = Value::new(&mut frame, mmatrix);
                assert!(boxed_mmatrix.is::<MMatrix<f32, 3, 2>>());

                let unboxed_mmatrix = boxed_mmatrix.unbox::<MMatrix<f32, 3, 2>>().unwrap();
                assert_eq!(original.data(), unboxed_mmatrix.data());
            })
        });
    }

    fn mmatrix_ref_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let mmatrix = MMatrix::new([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);
                let original = mmatrix.clone();
                let boxed_mmatrix = Value::new(&mut frame, &mmatrix);
                assert!(boxed_mmatrix.is::<MMatrix<f32, 3, 2>>());

                let unboxed_mmatrix = boxed_mmatrix.unbox::<MMatrix<f32, 3, 2>>().unwrap();
                assert_eq!(original.data(), unboxed_mmatrix.data());
            })
        });
    }

    fn sarray_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                type ArrayType = SArray<f32, Dims3D<2, 2, 2>, 8, 3>;
                let sarray = ArrayType::new([
                    1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32, 7.0f32, 8.0f32,
                ]);
                let original = sarray.clone();
                let boxed_sarray = Value::new(&mut frame, sarray);
                assert!(boxed_sarray.is::<ArrayType>());

                let unboxed_sarray = boxed_sarray.unbox::<ArrayType>().unwrap();
                assert_eq!(original.data(), unboxed_sarray.data());
            })
        });
    }

    fn sarray_ref_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                type ArrayType = SArray<f32, Dims3D<2, 2, 2>, 8, 3>;
                let sarray = ArrayType::new([
                    1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32, 7.0f32, 8.0f32,
                ]);
                let boxed_sarray = Value::new(&mut frame, &sarray);
                assert!(boxed_sarray.is::<ArrayType>());

                let unboxed_sarray = boxed_sarray.unbox::<ArrayType>().unwrap();
                assert_eq!(sarray.data(), unboxed_sarray.data());
            })
        });
    }
    fn marray_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                type ArrayType = MArray<f32, Dims3D<2, 2, 2>, 8, 3>;
                let marray = ArrayType::new([
                    1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32, 7.0f32, 8.0f32,
                ]);
                let original = marray.clone();
                let boxed_marray = Value::new(&mut frame, marray);
                assert!(boxed_marray.is::<ArrayType>());

                let unboxed_marray = boxed_marray.unbox::<ArrayType>().unwrap();
                assert_eq!(original.data(), unboxed_marray.data());
            })
        });
    }

    fn marray_ref_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                type ArrayType = MArray<f32, Dims3D<2, 2, 2>, 8, 3>;
                let marray = ArrayType::new([
                    1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32, 7.0f32, 8.0f32,
                ]);
                let boxed_marray = Value::new(&mut frame, &marray);
                assert!(boxed_marray.is::<ArrayType>());

                let unboxed_marray = boxed_marray.unbox::<ArrayType>().unwrap();
                assert_eq!(marray.data(), unboxed_marray.data());
            })
        });
    }

    fn smatrix_get_test() {
        let smatrix = SMatrix::new([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);

        assert_eq!(smatrix.get::<Dims2D<0, 0>>(), &1.0f32);
        assert_eq!(smatrix.get::<Dims2D<1, 0>>(), &2.0f32);
        assert_eq!(smatrix.get::<Dims2D<2, 0>>(), &3.0f32);
        assert_eq!(smatrix.get::<Dims2D<0, 1>>(), &4.0f32);
        assert_eq!(smatrix.get::<Dims2D<1, 1>>(), &5.0f32);
        assert_eq!(smatrix.get::<Dims2D<2, 1>>(), &6.0f32);
    }

    fn mmatrix_get_test() {
        let mmatrix = MMatrix::new([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);

        assert_eq!(mmatrix.get::<Dims2D<0, 0>>(), &1.0f32);
        assert_eq!(mmatrix.get::<Dims2D<1, 0>>(), &2.0f32);
        assert_eq!(mmatrix.get::<Dims2D<2, 0>>(), &3.0f32);
        assert_eq!(mmatrix.get::<Dims2D<0, 1>>(), &4.0f32);
        assert_eq!(mmatrix.get::<Dims2D<1, 1>>(), &5.0f32);
        assert_eq!(mmatrix.get::<Dims2D<2, 1>>(), &6.0f32);
    }

    fn mmatrix_get_mut_test() {
        let mut mmatrix = MMatrix::new([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);

        assert_eq!(mmatrix.get_mut::<Dims2D<0, 0>>(), &mut 1.0f32);
        assert_eq!(mmatrix.get_mut::<Dims2D<1, 0>>(), &mut 2.0f32);
        assert_eq!(mmatrix.get_mut::<Dims2D<2, 0>>(), &mut 3.0f32);
        assert_eq!(mmatrix.get_mut::<Dims2D<0, 1>>(), &mut 4.0f32);
        assert_eq!(mmatrix.get_mut::<Dims2D<1, 1>>(), &mut 5.0f32);
        assert_eq!(mmatrix.get_mut::<Dims2D<2, 1>>(), &mut 6.0f32);
    }

    fn svector_get_test() {
        let svector = SVector::new([1.0f32, 2.0f32]);

        assert_eq!(svector.get::<Dims1D<0>>(), &1.0f32);
        assert_eq!(svector.get::<Dims1D<1>>(), &2.0f32);
    }

    fn mvector_get_test() {
        let mvector = MVector::new([1.0f32, 2.0f32]);

        assert_eq!(mvector.get::<Dims1D<0>>(), &1.0f32);
        assert_eq!(mvector.get::<Dims1D<1>>(), &2.0f32);
    }

    fn mvector_get_mut_test() {
        let mut mvector = MVector::new([1.0f32, 2.0f32]);

        assert_eq!(mvector.get_mut::<Dims1D<0>>(), &mut 1.0f32);
        assert_eq!(mvector.get_mut::<Dims1D<1>>(), &mut 2.0f32);
    }

    fn sarray_get_test() {
        type ArrayType = MArray<f32, Dims3D<1, 2, 3>, 6, 3>;
        let marray = ArrayType::new([1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32]);

        assert_eq!(marray.get::<Dims3D<0, 0, 0>>(), &1.0f32);
        assert_eq!(marray.get::<Dims3D<0, 1, 0>>(), &2.0f32);
        assert_eq!(marray.get::<Dims3D<0, 0, 1>>(), &3.0f32);
        assert_eq!(marray.get::<Dims3D<0, 1, 1>>(), &4.0f32);
        assert_eq!(marray.get::<Dims3D<0, 0, 2>>(), &5.0f32);
        assert_eq!(marray.get::<Dims3D<0, 1, 2>>(), &6.0f32);
    }

    fn marray_get_test() {
        type ArrayType = MArray<f32, Dims3D<2, 2, 2>, 8, 3>;
        let marray = ArrayType::new([
            1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32, 7.0f32, 8.0f32,
        ]);

        assert_eq!(marray.get::<Dims3D<0, 0, 0>>(), &1.0f32);
        assert_eq!(marray.get::<Dims3D<1, 0, 0>>(), &2.0f32);
        assert_eq!(marray.get::<Dims3D<0, 1, 0>>(), &3.0f32);
        assert_eq!(marray.get::<Dims3D<1, 1, 0>>(), &4.0f32);
        assert_eq!(marray.get::<Dims3D<0, 0, 1>>(), &5.0f32);
        assert_eq!(marray.get::<Dims3D<1, 0, 1>>(), &6.0f32);
        assert_eq!(marray.get::<Dims3D<0, 1, 1>>(), &7.0f32);
        assert_eq!(marray.get::<Dims3D<1, 1, 1>>(), &8.0f32);
    }

    fn marray_get_mut_test() {
        type ArrayType = MArray<f32, Dims3D<2, 2, 2>, 8, 3>;
        let mut marray = ArrayType::new([
            1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32, 7.0f32, 8.0f32,
        ]);

        assert_eq!(marray.get_mut::<Dims3D<0, 0, 0>>(), &mut 1.0f32);
        assert_eq!(marray.get_mut::<Dims3D<1, 0, 0>>(), &mut 2.0f32);
        assert_eq!(marray.get_mut::<Dims3D<0, 1, 0>>(), &mut 3.0f32);
        assert_eq!(marray.get_mut::<Dims3D<1, 1, 0>>(), &mut 4.0f32);
        assert_eq!(marray.get_mut::<Dims3D<0, 0, 1>>(), &mut 5.0f32);
        assert_eq!(marray.get_mut::<Dims3D<1, 0, 1>>(), &mut 6.0f32);
        assert_eq!(marray.get_mut::<Dims3D<0, 1, 1>>(), &mut 7.0f32);
        assert_eq!(marray.get_mut::<Dims3D<1, 1, 1>>(), &mut 8.0f32);
    }

    fn smatrix_mapping_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 11>(|mut frame| {
                let smatrix = SMatrix::new([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);
                let boxed_smatrix = Value::new(&mut frame, &smatrix);

                let getindex_fn = Module::base(&frame).global(&mut frame, "getindex").unwrap();
                let one = Value::new(&mut frame, 1isize);
                let two = Value::new(&mut frame, 2isize);
                let three = Value::new(&mut frame, 3isize);

                unsafe {
                    let v_1_1 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, one, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(smatrix.get::<Dims2D<0, 0>>(), &v_1_1);
                    let v_2_1 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, two, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(smatrix.get::<Dims2D<1, 0>>(), &v_2_1);
                    let v_3_1 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, three, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(smatrix.get::<Dims2D<2, 0>>(), &v_3_1);
                    let v_1_2 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, one, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(smatrix.get::<Dims2D<0, 1>>(), &v_1_2);
                    let v_2_2 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, two, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(smatrix.get::<Dims2D<1, 1>>(), &v_2_2);
                    let v_3_2 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, three, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(smatrix.get::<Dims2D<2, 1>>(), &v_3_2);
                }
            })
        });
    }

    fn mmatrix_mapping_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 11>(|mut frame| {
                let mmatrix = MMatrix::new([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);
                let boxed_smatrix = Value::new(&mut frame, &mmatrix);

                let getindex_fn = Module::base(&frame).global(&mut frame, "getindex").unwrap();
                let one = Value::new(&mut frame, 1isize);
                let two = Value::new(&mut frame, 2isize);
                let three = Value::new(&mut frame, 3isize);

                unsafe {
                    let v_1_1 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, one, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(mmatrix.get::<Dims2D<0, 0>>(), &v_1_1);
                    let v_2_1 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, two, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(mmatrix.get::<Dims2D<1, 0>>(), &v_2_1);
                    let v_3_1 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, three, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(mmatrix.get::<Dims2D<2, 0>>(), &v_3_1);
                    let v_1_2 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, one, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(mmatrix.get::<Dims2D<0, 1>>(), &v_1_2);
                    let v_2_2 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, two, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(mmatrix.get::<Dims2D<1, 1>>(), &v_2_2);
                    let v_3_2 = getindex_fn
                        .call(&mut frame, [boxed_smatrix, three, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(mmatrix.get::<Dims2D<2, 1>>(), &v_3_2);
                }
            })
        });
    }

    fn sarray_mapping_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 11>(|mut frame| {
                type ArrayType = SArray<f32, Dims3D<1, 2, 3>, 6, 3>;
                let sarray = ArrayType::new([1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32]);

                let boxed_marray = Value::new(&mut frame, &sarray);
                let getindex_fn = Module::base(&frame).global(&mut frame, "getindex").unwrap();
                let one = Value::new(&mut frame, 1isize);
                let two = Value::new(&mut frame, 2isize);
                let three = Value::new(&mut frame, 3isize);

                unsafe {
                    let v_1_1_1 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, one, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 0, 0>>(), &v_1_1_1);

                    let v_1_2_1 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, two, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 1, 0>>(), &v_1_2_1);

                    let v_1_1_2 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, one, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 0, 1>>(), &v_1_1_2);

                    let v_1_2_2 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, two, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 1, 1>>(), &v_1_2_2);

                    let v_1_1_3 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, one, three])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 0, 2>>(), &v_1_1_3);

                    let v_1_2_3 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, two, three])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 1, 2>>(), &v_1_2_3);
                }
            })
        })
    }

    fn marray_mapping_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 11>(|mut frame| {
                type ArrayType = MArray<f32, Dims3D<1, 2, 3>, 6, 3>;
                let sarray = ArrayType::new([1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32]);

                let boxed_marray = Value::new(&mut frame, &sarray);
                let getindex_fn = Module::base(&frame).global(&mut frame, "getindex").unwrap();
                let one = Value::new(&mut frame, 1isize);
                let two = Value::new(&mut frame, 2isize);
                let three = Value::new(&mut frame, 3isize);

                unsafe {
                    let v_1_1_1 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, one, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 0, 0>>(), &v_1_1_1);

                    let v_1_2_1 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, two, one])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 1, 0>>(), &v_1_2_1);

                    let v_1_1_2 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, one, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 0, 1>>(), &v_1_1_2);

                    let v_1_2_2 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, two, two])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 1, 1>>(), &v_1_2_2);

                    let v_1_1_3 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, one, three])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 0, 2>>(), &v_1_1_3);

                    let v_1_2_3 = getindex_fn
                        .call(&mut frame, [boxed_marray, one, two, three])
                        .unwrap()
                        .unbox::<f32>()
                        .unwrap();
                    assert_eq!(sarray.get::<Dims3D<0, 1, 2>>(), &v_1_2_3);
                }
            })
        })
    }

    fn typed_value_svector_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let svector = SVector::new([1.0f32, 2.0f32, 3.0f32]);
                let boxed_svector = Value::new(&mut frame, svector);
                let typed = boxed_svector.cast::<TypedValue<SVector<f32, 3>>>().unwrap();

                let svector_ref = typed.as_svector_ref();
                assert_eq!(svector_ref.get::<Dims1D<0>>(), &1.0f32);
            })
        });
    }

    fn typed_value_mvector_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let mvector = MVector::new([1.0f32, 2.0f32, 3.0f32]);
                let boxed_mvector = Value::new(&mut frame, mvector);
                let typed = boxed_mvector.cast::<TypedValue<MVector<f32, 3>>>().unwrap();

                let mvector_ref = unsafe { typed.as_mvector_ref() };
                assert_eq!(mvector_ref.get::<Dims1D<0>>(), &1.0f32);
            })
        });
    }

    fn typed_value_mvector_mut_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let mvector = MVector::new([1.0f32, 2.0f32, 3.0f32]);
                let boxed_mvector = Value::new(&mut frame, mvector);
                let typed = boxed_mvector.cast::<TypedValue<MVector<f32, 3>>>().unwrap();

                let mvector_ref = unsafe { typed.as_mvector_mut() };
                assert_eq!(mvector_ref.get_mut::<Dims1D<0>>(), &mut 1.0f32);
            })
        });
    }

    fn typed_value_smatrix_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let smatrix = SMatrix::new([[1.0f32, 2.0f32]]);
                let boxed_smatrix = Value::new(&mut frame, smatrix);
                let typed = boxed_smatrix
                    .cast::<TypedValue<SMatrix<f32, 2, 1>>>()
                    .unwrap();

                let smatrix_ref = typed.as_smatrix_ref();
                assert_eq!(smatrix_ref.get::<Dims2D<0, 0>>(), &1.0f32);
            })
        });
    }

    fn typed_value_mmatrix_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let mmatrix = MMatrix::new([[1.0f32, 2.0f32]]);
                let boxed_mmatrix = Value::new(&mut frame, mmatrix);
                let typed = boxed_mmatrix
                    .cast::<TypedValue<MMatrix<f32, 2, 1>>>()
                    .unwrap();

                let mmatrix_ref = unsafe { typed.as_mmatrix_ref() };
                assert_eq!(mmatrix_ref.get::<Dims2D<0, 0>>(), &1.0f32);
            })
        });
    }

    fn typed_value_mmatrix_mut_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                let mmatrix = MMatrix::new([[1.0f32, 2.0f32]]);
                let boxed_mmatrix = Value::new(&mut frame, mmatrix);
                let typed = boxed_mmatrix
                    .cast::<TypedValue<MMatrix<f32, 2, 1>>>()
                    .unwrap();

                let mmatrix_ref = unsafe { typed.as_mmatrix_mut() };
                assert_eq!(mmatrix_ref.get::<Dims2D<0, 0>>(), &mut 1.0f32);
            })
        });
    }

    fn typed_value_sarray_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                type ArrayType = SArray<f32, Dims3D<1, 2, 3>, 6, 3>;
                let sarray = ArrayType::new([1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32]);
                let boxed_sarray = Value::new(&mut frame, sarray);
                let typed = boxed_sarray.cast::<TypedValue<ArrayType>>().unwrap();

                let sarray_ref = typed.as_sarray_ref();
                assert_eq!(sarray_ref.get::<Dims3D<0, 0, 0>>(), &1.0f32);
            })
        });
    }

    fn typed_value_marray_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                type ArrayType = MArray<f32, Dims3D<1, 2, 3>, 6, 3>;
                let marray = ArrayType::new([1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32]);
                let boxed_marray = Value::new(&mut frame, marray);
                let typed = boxed_marray.cast::<TypedValue<ArrayType>>().unwrap();

                let marray_ref = unsafe { typed.as_marray_ref() };
                assert_eq!(marray_ref.get::<Dims3D<0, 0, 0>>(), &1.0f32);
            })
        });
    }

    fn typed_value_marray_mut_test() {
        JULIA.with(|j| {
            j.borrow().local_scope::<_, 1>(|mut frame| {
                type ArrayType = MArray<f32, Dims3D<1, 2, 3>, 6, 3>;
                let marray = ArrayType::new([1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32]);
                let boxed_marray = Value::new(&mut frame, marray);
                let typed = boxed_marray.cast::<TypedValue<ArrayType>>().unwrap();

                let marray_ref = unsafe { typed.as_marray_mut() };
                assert_eq!(marray_ref.get::<Dims3D<0, 0, 0>>(), &mut 1.0f32);
            })
        });
    }

    #[test]
    fn runtime_test() {
        use_static_arrays();

        svector_test();
        svector_ref_test();
        mvector_test();
        mvector_ref_test();

        smatrix_test();
        smatrix_ref_test();
        mmatrix_test();
        mmatrix_ref_test();

        sarray_test();
        sarray_ref_test();
        marray_test();
        marray_ref_test();

        svector_get_test();
        mvector_get_test();
        mvector_get_mut_test();
        smatrix_get_test();
        mmatrix_get_test();
        mmatrix_get_mut_test();
        sarray_get_test();
        marray_get_test();
        marray_get_mut_test();

        smatrix_mapping_test();
        mmatrix_mapping_test();
        sarray_mapping_test();
        marray_mapping_test();

        typed_value_svector_test();
        typed_value_mvector_test();
        typed_value_mvector_mut_test();

        typed_value_smatrix_test();
        typed_value_mmatrix_test();
        typed_value_mmatrix_mut_test();

        typed_value_sarray_test();
        typed_value_marray_test();
        typed_value_marray_mut_test();
    }
}

mod util;
#[cfg(all(feature = "local-rt", feature = "static-arrays"))]
mod tests {
    use jlrs::{
        data::layout::static_arrays::{
            dims::Dims3D, m_array::MArray, m_matrix::MMatrix, m_vector::MVector, s_array::SArray,
            s_matrix::SMatrix, s_vector::SVector,
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
    }
}

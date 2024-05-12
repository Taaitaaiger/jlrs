// #[cfg(feature = "local-rt")]
// pub(crate) mod tests {
//     use jlrs::{
//         data::managed::array::{data::accessor::AccessorMut1D, TypedVector},
//         prelude::*,
//     };

//     use crate::util::JULIA;

//     fn typed_vector_grow_end() {
//         JULIA.with(|j| {
//             let mut frame = StackFrame::new();
//             let mut jlrs = j.borrow_mut();

//             jlrs.instance(&mut frame)
//                 .scope(|mut frame| {
//                     unsafe {
//                         let data = [1.0f32, 2.0];
//                         let mut arr =
//                             TypedVector::<f32>::from_slice_cloned(&mut frame, data.as_ref(), 2)
//                                 .unwrap()
//                                 .unwrap();

//                         assert_eq!(arr.length(), 2);
//                         let success = arr.bits_data_mut().grow_end(&frame, 1);
//                         assert!(success.is_ok());
//                         assert_eq!(arr.length(), 3);

//                         let success = arr.bits_data_mut().grow_end(&frame, 5);
//                         assert!(success.is_ok());
//                         assert_eq!(arr.length(), 8);

//                         {
//                             let accessor = arr.bits_data();
//                             assert_eq!(accessor.get_uninit(0).unwrap().assume_init(), 1.0);
//                             assert_eq!(accessor.get_uninit(1).unwrap().assume_init(), 2.0);
//                         }
//                     }

//                     Ok(())
//                 })
//                 .unwrap();
//         });
//     }

//     fn typed_vector_grow_end_err() {
//         JULIA.with(|j| {
//             let mut frame = StackFrame::new();
//             let mut jlrs = j.borrow_mut();

//             jlrs.instance(&mut frame)
//                 .scope(|mut frame| {
//                     unsafe {
//                         let data = vec![1.0f32, 2.0];
//                         let mut arr = TypedVector::<f32>::from_vec(&mut frame, data, 2)
//                             .unwrap()
//                             .unwrap();

//                         assert_eq!(arr.length(), 2);
//                         let success = arr.bits_data_mut().grow_end(&frame, 1);
//                         assert!(success.is_err());
//                         assert_eq!(arr.length(), 2);
//                     }

//                     Ok(())
//                 })
//                 .unwrap();
//         });
//     }

//     fn typed_vector_del_end() {
//         JULIA.with(|j| {
//             let mut frame = StackFrame::new();
//             let mut jlrs = j.borrow_mut();

//             jlrs.instance(&mut frame)
//                 .scope(|mut frame| {
//                     unsafe {
//                         let data = [1.0f32, 2.0, 3.0, 4.0];
//                         let mut arr =
//                             TypedVector::<f32>::from_slice_cloned(&mut frame, data.as_ref(), 4)
//                                 .unwrap()
//                                 .unwrap();

//                         assert_eq!(arr.length(), 4);
//                         let success = arr.bits_data_mut().del_end(&frame, 1);
//                         assert!(success.is_ok());
//                         assert_eq!(arr.length(), 3);

//                         let success = arr.bits_data_mut().del_end(&frame, 2);
//                         assert!(success.is_ok());
//                         assert_eq!(arr.length(), 1);

//                         {
//                             let accessor = arr.bits_data();
//                             assert_eq!(*accessor.get(0).unwrap(), 1.0);
//                         }
//                     }

//                     Ok(())
//                 })
//                 .unwrap();
//         });
//     }

//     fn typed_vector_del_end_err() {
//         JULIA.with(|j| {
//             let mut frame = StackFrame::new();
//             let mut jlrs = j.borrow_mut();

//             jlrs.instance(&mut frame)
//                 .scope(|mut frame| {
//                     unsafe {
//                         let data = vec![1.0f32, 2.0];
//                         let mut arr = TypedVector::<f32>::from_vec(&mut frame, data, 2)
//                             .unwrap()
//                             .unwrap();

//                         assert_eq!(arr.length(), 2);
//                         let success = arr.bits_data_mut().del_end(&frame, 1);
//                         assert!(success.is_err());
//                         assert_eq!(arr.length(), 2);
//                     }

//                     Ok(())
//                 })
//                 .unwrap();
//         });
//     }

//     pub(crate) fn array_grow_del_tests() {
//         typed_vector_grow_end();
//         typed_vector_grow_end_err();
//         typed_vector_del_end();
//         typed_vector_del_end_err();
//     }
// }

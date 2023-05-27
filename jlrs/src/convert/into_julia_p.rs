use std::ptr::NonNull;

use jl_sys::jl_new_struct_uninit;

use crate::{
    data::managed::{
        datatype::DataTypeResult,
        private::ManagedPriv,
        value::{Value, ValueResult},
        Managed,
    },
    error::{JlrsResult, TypeError, CANNOT_DISPLAY_TYPE},
    memory::target::{ExtendedTarget, Target},
    private::Private,
};

pub unsafe trait IntoJuliaParametric: Sized {
    /// Returns the associated Julia type of the implementor.
    ///
    /// The layout of that type and the Rust type must match exactly, and it must be an `isbits`
    /// type, otherwise this trait has been implemented incorrectly.
    fn julia_type<'scope, Tgt>(
        target: ExtendedTarget<'scope, '_, '_, Tgt>,
    ) -> JlrsResult<DataTypeResult<'scope, Tgt>>
    where
        Tgt: Target<'scope>;

    #[doc(hidden)]
    fn into_julia<'scope, 'd, Tgt>(
        self,
        target: ExtendedTarget<'scope, '_, '_, Tgt>,
    ) -> JlrsResult<ValueResult<'scope, 'd, Tgt>>
    where
        Tgt: Target<'scope>,
    {
        let (target, frame) = target.split();
        frame.scope(|mut frame| {
            let ty = Self::julia_type(frame.as_extended_target())?;
            match ty {
                Ok(ty) => {
                    if ty.layout().is_none() {
                        Err(TypeError::LayoutNone {
                            ty: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                        })?;
                    }

                    if !ty.is_bits() {
                        todo!()
                    }

                    let instance = ty.instance();
                    match instance {
                        Some(instance) => Ok(Tgt::into_ok(instance.root(target))),
                        // Safety: trait is implemented incorrectly if this is incorrect.
                        None => unsafe {
                            let container = jl_new_struct_uninit(ty.unwrap(Private));
                            container.cast::<Self>().write(self);
                            Ok(target.result_from_ptr::<Value>(
                                Ok(NonNull::new_unchecked(container)),
                                Private,
                            ))
                        },
                    }
                }
                Err(e) => Ok(Tgt::into_err(e.root(target))),
            }
        })
    }
}

/*
#[repr(C)]
// #[jlrs(julia_type = "Base.Complex")]
pub struct JuliaComplex<T> {
    pub re: T,
    pub im: T,
}

unsafe impl<T> IntoJuliaParametric for JuliaComplex<T>
where
    T: ConstructType,
{
    fn julia_type<'scope, Tgt>(
        target: ExtendedTarget<'scope, '_, '_, Tgt>,
    ) -> JlrsResult<DataTypeResult<'scope, Tgt>>
    where
        Tgt: Target<'scope>,
    {
        let (target, frame) = target.split();
        frame.scope(|mut frame| {
            let type_param = T::construct_type(frame.as_extended_target());
            let ty = inline_static_global!(BASE_TYPE, "Base.Complex", &frame)
                .apply_type(&mut frame, [type_param]);

            match ty {
                Ok(ty) => {
                    let dt = ty.cast::<DataType>()?.root(target);
                    Ok(Tgt::into_ok(dt))
                }
                Err(e) => Ok(Tgt::into_err(e.root(target))),
            }
        })
    }
}
*/

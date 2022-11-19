//! Wrapper for `Expr`.

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_expr_t, jl_expr_type};

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{
        array::{ArrayData, ArrayRef},
        private::WrapperPriv,
        symbol::Symbol,
        Ref,
    },
};

/// A compound expression in Julia ASTs.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Expr<'scope>(NonNull<jl_expr_t>, PhantomData<&'scope ()>);

impl<'scope> Expr<'scope> {
    /*
    inspect(Core.Expr):

    head: Symbol (mut)
    args: Vector{Any} (mut)
    */

    /// Returns the head of the expression.
    pub fn head(self) -> Option<Symbol<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let head = self.unwrap_non_null(Private).as_ref().head;
            let head = NonNull::new(head)?;
            Some(Symbol::wrap_non_null(head, Private))
        }
    }

    /// Returns the arguments of the expression.
    pub fn args<'target, T>(self, target: T) -> Option<ArrayData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let args = self.unwrap_non_null(Private).as_ref().args;
            let args = NonNull::new(args)?;
            Some(ArrayRef::wrap(args).root(target))
        }
    }
}

impl_julia_typecheck!(Expr<'scope>, jl_expr_type, 'scope);
impl_debug!(Expr<'_>);

impl<'scope> WrapperPriv<'scope, '_> for Expr<'scope> {
    type Wraps = jl_expr_t;
    type TypeConstructorPriv<'target, 'da> = Expr<'target>;
    const NAME: &'static str = "Expr";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to an [`Expr`] that has not been explicitly rooted.
pub type ExprRef<'scope> = Ref<'scope, 'static, Expr<'scope>>;
impl_valid_layout!(ExprRef, Expr);

use crate::memory::target::target_type::TargetType;

/// `Expr` or `ExprRef`, depending on the target type `T`.
pub type ExprData<'target, T> = <T as TargetType<'target>>::Data<'static, Expr<'target>>;

/// `JuliaResult<Expr>` or `JuliaResultRef<ExprRef>`, depending on the target type `T`.
pub type ExprResult<'target, T> = <T as TargetType<'target>>::Result<'static, Expr<'target>>;

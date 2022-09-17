use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    error::{JuliaResult, JuliaResultRef},
    prelude::Wrapper,
    wrappers::ptr::Ref,
};

use super::context::TaskContext;

pub struct NewGcFrame<'scope, 'base> {
    task_context: &'scope TaskContext<'base>,
    offset: usize,
}

impl<'scope, 'base> NewGcFrame<'scope, 'base> {
    pub(crate) fn root<'data, T: Wrapper<'scope, 'data>>(&mut self, ptr: NonNull<T::Wraps>) -> T {
        todo!()
    }
}

pub struct NewAsyncGcFrame<'scope, 'base> {
    scope_context: NewGcFrame<'scope, 'base>,
}

impl<'scope, 'base> Deref for NewAsyncGcFrame<'scope, 'base> {
    type Target = NewGcFrame<'scope, 'base>;

    fn deref(&self) -> &Self::Target {
        &self.scope_context
    }
}

impl<'scope, 'base> DerefMut for NewAsyncGcFrame<'scope, 'base> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope_context
    }
}

pub struct NewOutput<'target, 'base> {
    task_context: &'target TaskContext<'base>,
    offset: usize,
}

impl<'scope, 'base> NewOutput<'scope, 'base> {
    pub(crate) fn root<'data, T: Wrapper<'scope, 'data>>(self, ptr: NonNull<T::Wraps>) -> T {
        todo!()
    }
}

pub struct Temporary<'scope, 'base> {
    task_context: &'scope TaskContext<'base>,
    offset: usize,
}

impl<'scope, 'base> Temporary<'scope, 'base> {
    pub(crate) fn root<'target, 'data, T: Wrapper<'target, 'data>>(
        &'target mut self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        todo!()
    }
}

pub trait TargetType<'target, 'data, T: Wrapper<'target, 'data>> {
    type Data;
}

impl<'target, 'data, 'base, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T>
    for &mut NewGcFrame<'target, 'base>
{
    type Data = T;
}

impl<'target, 'data, 'base, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T>
    for NewOutput<'target, 'base>
{
    type Data = T;
}

impl<'target, 'data, 'base, 'scope, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T>
    for &'target mut Temporary<'scope, 'base>
{
    type Data = T;
}

impl<'target, 'data, W: Wrapper<'target, 'data>, T: TargetType<'target, 'data, W>>
    TargetType<'target, 'data, W> for &T
{
    type Data = Ref<'target, 'data, W>;
}

pub trait ResultTargetType<'target, 'data, T: Wrapper<'target, 'data>> {
    type Result;
}

impl<'target, 'data, 'base, T: Wrapper<'target, 'data>> ResultTargetType<'target, 'data, T>
    for &mut NewGcFrame<'target, 'base>
{
    type Result = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, 'base, T: Wrapper<'target, 'data>> ResultTargetType<'target, 'data, T>
    for NewOutput<'target, 'base>
{
    type Result = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, 'base, 'scope, T: Wrapper<'target, 'data>> ResultTargetType<'target, 'data, T>
    for &'target mut Temporary<'scope, 'base>
{
    type Result = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, W: Wrapper<'target, 'data>, T: ResultTargetType<'target, 'data, W>>
    ResultTargetType<'target, 'data, W> for &T
{
    type Result = JuliaResultRef<'target, 'data, Ref<'target, 'data, W>>;
}

pub trait ExceptionTargetType<'target, 'data, T> {
    type Exception;
}

impl<'target, 'data, 'base, T> ExceptionTargetType<'target, 'data, T>
    for &mut NewGcFrame<'target, 'base>
{
    type Exception = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, 'base, T> ExceptionTargetType<'target, 'data, T>
    for NewOutput<'target, 'base>
{
    type Exception = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, 'base, 'scope, T> ExceptionTargetType<'target, 'data, T>
    for &'target mut Temporary<'scope, 'base>
{
    type Exception = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, W, T: ExceptionTargetType<'target, 'data, W>>
    ExceptionTargetType<'target, 'data, W> for &T
{
    type Exception = JuliaResultRef<'target, 'data, W>;
}

pub(crate) mod private {
    use std::{cell::RefCell, ptr::NonNull};

    use jl_sys::jl_value_t;

    use crate::{
        error::JuliaResult,
        memory::ledger::Ledger,
        prelude::{ValueRef, Wrapper},
        private::Private,
        wrappers::ptr::{private::WrapperPriv, Ref},
    };

    use super::{NewGcFrame, NewOutput, ResultTargetType, TargetType, Temporary, ExceptionTargetType};

    pub trait TargetBase<'target>: Sized {
        fn ledger(&self, _: Private) -> &'target RefCell<Ledger>;
    }

    impl<'target, 'base> TargetBase<'target> for &mut NewGcFrame<'target, 'base> {
        fn ledger(&self, _: Private) -> &'target RefCell<Ledger> {
            self.task_context.ledger()
        }
    }

    impl<'target, 'base> TargetBase<'target> for NewOutput<'target, 'base> {
        fn ledger(&self, _: Private) -> &'target RefCell<Ledger> {
            self.task_context.ledger()
        }
    }

    impl<'target> TargetBase<'target> for &'target mut Temporary<'_, '_> {
        fn ledger(&self, _: Private) -> &'target RefCell<Ledger> {
            self.task_context.ledger()
        }
    }

    impl<'target, T: TargetBase<'target>> TargetBase<'target> for &T {
        fn ledger(&self, _: Private) -> &'target RefCell<Ledger> {
            T::ledger(self, Private)
        }
    }

    pub trait TargetPriv<'target, 'data, W>:
        TargetBase<'target> + TargetType<'target, 'data, W>
    where
        W: Wrapper<'target, 'data>,
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data;
    }

    impl<'target, 'data, 'base, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W>
        for &mut NewGcFrame<'target, 'base>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            self.root(value)
        }
    }

    impl<'target, 'data, 'base, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W>
        for NewOutput<'target, 'base>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            self.root(value)
        }
    }

    impl<'target, 'data, 'scope, 'base, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W>
        for &'target mut Temporary<'scope, 'base>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            self.root(value)
        }
    }

    impl<'target, 'data, 'base, W: Wrapper<'target, 'data>, T: TargetPriv<'target, 'data, W>>
        TargetPriv<'target, 'data, W> for &T
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            Ref::wrap(value.as_ptr())
        }
    }

    pub trait ResultTargetPriv<'target, 'data, W>:
        TargetBase<'target> + ResultTargetType<'target, 'data, W>
    where
        W: Wrapper<'target, 'data>,
    {
        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result;
    }

    impl<'target, 'data, 'base, W: Wrapper<'target, 'data>> ResultTargetPriv<'target, 'data, W>
        for &mut NewGcFrame<'target, 'base>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
        self,
        result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
        _: Private,
    ) -> Self::Result
        {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, 'base, W: Wrapper<'target, 'data>> ResultTargetPriv<'target, 'data, W>
        for NewOutput<'target, 'base>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
        self,
        result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
        _: Private,
    ) -> Self::Result
        {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, W: Wrapper<'target, 'data>> ResultTargetPriv<'target, 'data, W>
        for &'target mut Temporary<'_, '_>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
        self,
        result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
        _: Private,
    ) -> Self::Result
        {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, W: Wrapper<'target, 'data>, T: ResultTargetPriv<'target, 'data, W>> ResultTargetPriv<'target, 'data, W>
        for &T
    {
        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
        self,
        result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
        _: Private,
    ) -> Self::Result
        {
            match result {
                Ok(t) => Ok(Ref::wrap(t.as_ptr())),
                Err(e) => Err(Ref::wrap(e.as_ptr())),
            }
        }
    }

    
    pub trait ExceptionTargetPriv<'target, 'data, W>:
        TargetBase<'target> + ExceptionTargetType<'target, 'data, W>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception;
    }

    impl<'target, 'data, 'base, W> ExceptionTargetPriv<'target, 'data, W>
        for &mut NewGcFrame<'target, 'base>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
        self,
        result: Result<W, NonNull<jl_value_t>>,
        _: Private,
    ) -> Self::Exception
        {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, 'base, W> ExceptionTargetPriv<'target, 'data, W>
        for NewOutput<'target, 'base>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
        self,
        result: Result<W, NonNull<jl_value_t>>,
        _: Private,
    ) -> Self::Exception
        {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, W> ExceptionTargetPriv<'target, 'data, W>
        for &'target mut Temporary<'_, '_>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
        self,
        result: Result<W, NonNull<jl_value_t>>,
        _: Private,
    ) -> Self::Exception
        {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, W, T: ExceptionTargetPriv<'target, 'data, W>> ExceptionTargetPriv<'target, 'data, W>
        for &T
    {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
        self,
        result: Result<W, NonNull<jl_value_t>>,
        _: Private,
    ) -> Self::Exception
        {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e.as_ptr())),
            }
        }
    }
}

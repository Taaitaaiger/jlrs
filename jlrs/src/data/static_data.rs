//! Static references to global Julia data.
//!
//! Accessing global Julia data through the module system can be expensive. If the global is a
//! constant or never replaced with another value, this data is globally rooted so it's safe to
//! hold on to a reference to this data. This module provides [`StaticGlobal`] and [`StaticRef`],
//! and macros to create and access them.

use std::{
    marker::PhantomData,
    ptr::{null_mut, NonNull},
    sync::atomic::AtomicPtr,
};

use super::types::typecheck::Typecheck;
use crate::{
    data::managed::{module::Module, value::ValueUnbound, Managed},
    gc_safe::GcSafeOnceLock,
    memory::target::Target,
    private::Private, prelude::Value,
};

struct StaticDataInner<T>(ValueUnbound, PhantomData<T>);
unsafe impl<T> Send for StaticDataInner<T> {}
unsafe impl<T> Sync for StaticDataInner<T> {}

/// Static reference to arbitrary managed data. Guaranteed to be initialized at most once.
pub struct StaticGlobal<T> {
    global: GcSafeOnceLock<StaticDataInner<T>>,
    path: &'static str,
}

impl<T> StaticGlobal<T>
where
    T: Managed<'static, 'static> + Typecheck,
{
    /// Define a new static global available at `path`.
    ///
    /// The global is looked up only once when this data is accessedd for the first time. The
    /// `path` argument must be the full path to the data, e.g. `"Main.Submodule.Foo"`.
    #[inline]
    pub const fn new(path: &'static str) -> StaticGlobal<T> {
        StaticGlobal {
            global: GcSafeOnceLock::new(),
            path,
        }
    }

    /// Get the global data, look it up if it doesn't exist yet.
    ///
    /// The global must exist and be an instance of `T`. Otherwise this method will panic.
    pub fn get_or_init<'target, Tgt>(&self, target: &Tgt) -> T
    where
        Tgt: Target<'target>,
    {
        unsafe {
            if let Some(global) = self.global.get() {
                return global.0.cast_unchecked::<T>();
            } else {
                self.init(target)
            }
        }
    }

    #[inline(never)]
    #[cold]
    unsafe fn init<'target, Tgt>(&self, target: &Tgt) -> T
    where
        Tgt: Target<'target>,
    {
        // If multiple threads try to initialize the global, only one calls the init code and
        // the others are parked. We call jlrs_gc_safe_enter to allow the GC to run while a
        // thread is parked, and immediately transition back once we regain control.
        let global = self.global.get_or_init(|| {
            let split_path = self.path.split('.').collect::<Vec<_>>();
            let n_parts = split_path.len();

            let mut module = match split_path[0] {
                "Main" => Module::main(target),
                "Base" => Module::base(target),
                "Core" => Module::core(target),
                pkg => Module::package_root_module(target, pkg).unwrap(),
            };

            if n_parts == 1 {
                let global = module.leak().as_value().cast::<T>().unwrap();
                return StaticDataInner(global.as_value(), PhantomData);
            }

            for i in 1..n_parts - 1 {
                module = module
                    .submodule(target, split_path[i])
                    .unwrap()
                    .as_managed();
            }

            let global = module
                .global(target, split_path[n_parts - 1])
                .unwrap()
                .leak()
                .as_value()
                .cast::<T>()
                .unwrap();

            return StaticDataInner(global.as_value(), PhantomData);
        });

        global.0.cast_unchecked()
    }
}

impl StaticGlobal<ValueUnbound> {
    /// Define a new static global available at `path`.
    ///
    /// The global is looked up only once when this data is accessedd for the first time. The
    /// `path` argument must be the full path to the data, e.g. `"Main.Submodule.Foo"`.
    pub const fn new_value(path: &'static str) -> StaticGlobal<ValueUnbound> {
        StaticGlobal {
            global: GcSafeOnceLock::new(),
            path,
        }
    }
}

/// Static reference to arbitrary managed data. Can be initialized multiple times.
///
/// In general, a `StaticRef` is faster than a `StaticGlobal`.
pub struct StaticRef<T: Managed<'static, 'static>> {
    global: AtomicPtr<T::Wraps>,
    path: &'static str,
}

impl<T> StaticRef<T>
where
    T: Managed<'static, 'static> + Typecheck,
{
    /// Define a new static ref available at `path`.
    ///
    /// The global is looked up if the ref is uninitialized. The `path` argument must be the full
    /// path to the data, e.g. `"Main.Submodule.Foo"`.
    #[inline]
    pub const fn new(path: &'static str) -> StaticRef<T> {
        StaticRef {
            global: AtomicPtr::new(null_mut()),
            path,
        }
    }

    /// Get the global data, look it up if it doesn't exist yet.
    ///
    /// The global must exist and be an instance of `T`. Otherwise this method will panic.
    #[inline]
    pub fn get_or_init<'target, Tgt>(&self, target: &Tgt) -> T
    where
        Tgt: Target<'target>,
    {
        let ptr = self.global.load(atomic::Ordering::Relaxed);
        if ptr.is_null() {
            // It's fine to initialize this multiple times. We're going to store the same data each time.
            self.init(target)
        } else {
            unsafe { T::wrap_non_null(NonNull::new_unchecked(ptr), Private) }
        }
    }

    #[cold]
    #[inline(never)]
    fn init<'target, Tgt>(&self, target: &Tgt) -> T
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let split_path = self.path.split('.').collect::<Vec<_>>();
            let n_parts = split_path.len();

            let mut module = match split_path[0] {
                "Main" => Module::main(target),
                "Base" => Module::base(target),
                "Core" => Module::core(target),
                pkg => Module::package_root_module(target, pkg).unwrap(),
            };

            if n_parts == 1 {
                let global = module.leak().as_value().cast::<T>().unwrap();
                let ptr = global.unwrap(Private);
                self.global.store(ptr, atomic::Ordering::Relaxed);
                return T::wrap_non_null(NonNull::new_unchecked(ptr), Private);
            }

            for i in 1..n_parts - 1 {
                module = module
                    .submodule(target, split_path[i])
                    .unwrap()
                    .as_managed();
            }

            let global = module
                .global(target, split_path[n_parts - 1])
                .unwrap()
                .leak()
                .as_value()
                .cast::<T>()
                .unwrap();

            let ptr = global.unwrap(Private);
            self.global.store(ptr, atomic::Ordering::Relaxed);
            T::wrap_non_null(NonNull::new_unchecked(ptr), Private)
        }
    }

    // Safety: The result of the evaluated command must be globally rooted.
    #[inline]
    pub(crate) unsafe fn get_or_eval<'target, Tgt>(&self, target: &Tgt) -> T
    where
        Tgt: Target<'target>,
    {
        let ptr = self.global.load(atomic::Ordering::Relaxed);
        if ptr.is_null() {
            self.eval(target)
        } else {
            T::wrap_non_null(NonNull::new_unchecked(ptr), Private)
        }
    }

    // Safety: The result of the evaluated command must be globally rooted.
    #[cold]
    #[inline(never)]
    unsafe fn eval<'target, Tgt>(&self, target: &Tgt) -> T
    where
        Tgt: Target<'target>,
    {
        let v = Value::eval_string(target, self.path)
            .unwrap()
            .leak()
            .as_value()
            .cast::<T>()
            .unwrap();

        let ptr = v.unwrap(Private);
        self.global.store(ptr, atomic::Ordering::Relaxed);
        T::wrap_non_null(NonNull::new_unchecked(ptr), Private)
    }
}

/// Define a static global
#[macro_export]
macro_rules! define_static_global {
    ($name:ident, $type:ty, $path:expr) => {
        static $name: $crate::data::static_data::StaticGlobal<$type> =
            $crate::data::static_data::StaticGlobal::new($path);
    };
    (pub $name:ident, $type:ty, $path:expr) => {
        pub static $name: $crate::data::static_data::StaticGlobal<$type> =
            $crate::data::static_data::StaticGlobal::new($path);
    };
    (pub(crate) $name:ident, $type:ty, $path:expr) => {
        pub(crate) static $name: $crate::data::static_data::StaticGlobal<$type> =
            $crate::data::static_data::StaticGlobal::new($path);
    };
    ($name:ident, $path:expr) => {
        static $name: $crate::data::static_data::StaticGlobal<
            $crate::data::managed::value::ValueUnbound,
        > = $crate::data::static_data::StaticGlobal::new_value($path);
    };
    (pub $name:ident, $path:expr) => {
        pub static $name: $crate::data::static_data::StaticGlobal<
            $crate::data::managed::value::ValueUnbound,
        > = $crate::data::static_data::StaticGlobal::new_value($path);
    };
    (pub(crate) $name:ident, $path:expr) => {
        pub(crate) static $name: $crate::data::static_data::StaticGlobal<
            $crate::data::managed::value::ValueUnbound,
        > = $crate::data::static_data::StaticGlobal::new_value($path);
    };
}

/// Define a static ref
#[macro_export]
macro_rules! define_static_ref {
    ($name:ident, $type:ty, $path:expr) => {
        static $name: $crate::data::static_data::StaticRef<$type> =
            $crate::data::static_data::StaticRef::new($path);
    };
    (pub $name:ident, $type:ty, $path:expr) => {
        pub static $name: $crate::data::static_data::StaticRef<$type> =
            $crate::data::static_data::StaticRef::new($path);
    };
    (pub(crate) $name:ident, $type:ty, $path:expr) => {
        pub(crate) static $name: $crate::data::static_data::StaticRef<$type> =
            $crate::data::static_data::StaticRef::new($path);
    };
}

/// Use a previously defined static global
#[macro_export]
macro_rules! static_global {
    ($name:ident, $target:expr) => {{
        $name.get_or_init(&$target)
    }};
}
/// Use a previously defined static ref
#[macro_export]
macro_rules! static_ref {
    ($name:ident, $target:expr) => {{
        $name.get_or_init(&$target)
    }};
}

pub use define_static_global;
pub use define_static_ref;
pub use static_global;
pub use static_ref;

/// Define an inline static global.
///
/// `inline_static_global!(NAME, T, path, target)` is equivalent to
/// `{ define_static_global!(NAME, T, path); static_global!(NAME, target) }`
#[macro_export]
macro_rules! inline_static_global {
    ($name:ident, $type:ty, $path:expr, $target:expr) => {{
        $crate::data::static_data::define_static_global!($name, $type, $path);
        $crate::data::static_data::static_global!($name, $target)
    }};
    ($name:ident, $path:expr, $target:expr) => {{
        $crate::data::static_data::define_static_global!($name, $path);
        $crate::data::static_data::static_global!($name, $target)
    }};
}

/// Define an inline static ref.
///
/// `inline_static_ref!(NAME, T, path, target)` is equivalent to
/// `{ define_static_ref!(NAME, T, path); static_ref!(NAME, target) }`
#[macro_export]
macro_rules! inline_static_ref {
    ($name:ident, $type:ty, $path:expr, $target:expr) => {{
        $crate::data::static_data::define_static_ref!($name, $type, $path);
        $crate::data::static_data::static_ref!($name, $target)
    }};
}

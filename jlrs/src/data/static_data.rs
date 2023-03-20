//! Static references to global Julia data.
//!
//! Accessing global Julia data through the module system can be expensive. If the global is a
//! constant or never replaced with another value, this data is globally rooted so it's safe to
//! hold on to a reference to this data.
//!
//! This module provides [`StaticSymbol`] and [`StaticGlobal`], and macros to create and access
//! them.

use std::marker::PhantomData;

use once_cell::sync::OnceCell;

use super::{managed::symbol::SymbolUnbound, types::typecheck::Typecheck};
use crate::{
    data::managed::{module::Module, symbol::Symbol, value::ValueUnbound, Managed},
    memory::target::Target,
};

struct StaticDataInner<T>(ValueUnbound, PhantomData<T>);
unsafe impl<T> Send for StaticDataInner<T> {}
unsafe impl<T> Sync for StaticDataInner<T> {}

/// Static reference to a [`Symbol`].
///
/// Symbols are immutable and guaranteed to be globally rooted.
pub struct StaticSymbol {
    sym: OnceCell<StaticDataInner<SymbolUnbound>>,
    sym_str: &'static str,
}

impl StaticSymbol {
    /// Define a new static symbol `sym`.
    ///
    /// The symbol is created when it's accessed for the first time.
    pub const fn new(sym: &'static str) -> Self {
        StaticSymbol {
            sym: OnceCell::new(),
            sym_str: sym,
        }
    }

    /// Get the symbol, create it if it doesn't exist yet.
    pub fn get_or_init<'target, Tgt>(&self, target: &Tgt) -> SymbolUnbound
    where
        Tgt: Target<'target>,
    {
        unsafe {
            self.sym
                .get_or_init(|| {
                    StaticDataInner(
                        Symbol::new(target, self.sym_str).leak().as_value(),
                        PhantomData,
                    )
                })
                .0
                .cast_unchecked()
        }
    }
}

/// Static reference to arbitrary managed data.
pub struct StaticGlobal<T> {
    global: OnceCell<StaticDataInner<T>>,
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
    pub const fn new(path: &'static str) -> StaticGlobal<T> {
        StaticGlobal {
            global: OnceCell::new(),
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
            self.global
                .get_or_init(|| {
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
                })
                .0
                .cast_unchecked()
        }
    }
}

impl StaticGlobal<ValueUnbound> {
    /// Define a new static global available at `path`.
    ///
    /// The global is looked up only once when this data is accessedd for the first time. The
    /// `path` argument must be the full path to the data, e.g. `"Main.Submodule.Foo"`.
    pub const fn new_value(path: &'static str) -> StaticGlobal<ValueUnbound> {
        StaticGlobal {
            global: OnceCell::new(),
            path,
        }
    }

    /// Get the global data, look it up if it doesn't exist yet.
    ///
    /// The global must exist. Otherwise this method will panic.
    pub fn get_or_init<'target, Tgt>(&self, target: &Tgt) -> ValueUnbound
    where
        Tgt: Target<'target>,
    {
        unsafe {
            self.global
                .get_or_init(|| {
                    let split_path = self.path.split('.').collect::<Vec<_>>();
                    let n_parts = split_path.len();

                    let mut module = match split_path[0] {
                        "Main" => Module::main(target),
                        "Base" => Module::base(target),
                        "Core" => Module::core(target),
                        pkg => Module::package_root_module(target, pkg).unwrap(),
                    };

                    if n_parts == 1 {
                        let global = module.leak().as_value();
                        return StaticDataInner(global, PhantomData);
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
                        .as_value();
                    return StaticDataInner(global, PhantomData);
                })
                .0
                .cast_unchecked()
        }
    }
}

/// Define a static symbol
#[macro_export]
macro_rules! define_static_symbol {
    ($name:ident, $sym:ident) => {
        static $name: $crate::data::static_data::StaticSymbol =
            $crate::data::static_data::StaticSymbol::new(stringify!($sym));
    };
    (pub $name:ident, $sym:ident) => {
        pub static $name: $crate::data::static_data::StaticSymbol =
            $crate::data::static_data::StaticSymbol::new(stringify!($sym));
    };
    (pub(crate) $name:ident, $sym:ident) => {
        pub(crate) static $name: $crate::data::static_data::StaticSymbol =
            $crate::data::static_data::StaticSymbol::new(stringify!($sym));
    };
}

/// Use a previously defined static symbol
#[macro_export]
macro_rules! static_symbol {
    ($name:ident, $target:expr) => {{
        let sym: $crate::data::managed::symbol::Symbol = $name.get_or_init(&$target);
        sym
    }};
}

/// Define an inline static symbol.
///
/// `inline_static_symbol!(NAME, sym,target)` is equivalent to
/// `{ define_static_symbol!(NAME, sym); static_symbol!(NAME, target) } `
#[macro_export]
macro_rules! inline_static_symbol {
    ($name:ident, $sym:ident, $target:expr) => {{
        static $name: $crate::data::static_data::StaticSymbol =
            $crate::data::static_data::StaticSymbol::new(stringify!($sym));
        let sym: $crate::data::managed::symbol::Symbol = $name.get_or_init(&$target);
        sym
    }};
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

/// Use a previously defined static global
#[macro_export]
macro_rules! static_global {
    ($name:ident, $target:expr) => {{
        $name.get_or_init(&$target)
    }};
}

pub(crate) use define_static_global;
pub(crate) use static_global;

/// Define an inline static global.
///
/// `inline_static_global!(NAME, T, target)` is equivalent to
/// `{ define_static_global!(NAME, T); static_global!(NAME, target) } `
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

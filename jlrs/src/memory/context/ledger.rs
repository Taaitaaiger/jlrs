// The ledger is used to track Julia data. It's implemented in a separate crate and distributed
// as JlrsLedger_jll which is a dependency of JlrsCore. The reason it works this way is because
// multiple packages could use different versions of jlrs and have been compiled with different
// versions of Rust, all of these packages must use the same ledger.
//
// The ledger is not a lock, nothing prevents you from creating a copy of a `Value` that's
// currently tracked. It does work well with opaque types, because tracking is the only way to
// access a managed instance of an opaque type.

use std::ffi::c_void;

use crate::{
    data::managed::{module::Module, private::ManagedPriv, value::Value},
    error::{JlrsError, JlrsResult},
    gc_safe::GcSafeOnceLock,
    memory::target::unrooted::Unrooted,
    private::Private,
};

const API_VERSION: usize = 2;

#[derive(PartialEq, Eq, Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub(crate) enum LedgerResult {
    OkFalse = 0u8,
    OkTrue = 1u8,
    Err = 2u8,
}

pub(crate) static LEDGER: GcSafeOnceLock<Ledger> = GcSafeOnceLock::new();

pub(crate) struct Ledger {
    api_version: unsafe extern "C" fn() -> usize,
    is_borrowed_shared: unsafe extern "C" fn(*const c_void) -> LedgerResult,
    is_borrowed_exclusive: unsafe extern "C" fn(*const c_void) -> LedgerResult,
    is_borrowed: unsafe extern "C" fn(*const c_void) -> LedgerResult,
    borrow_shared_unchecked: unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult,
    unborrow_shared: unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult,
    unborrow_exclusive: unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult,
    try_borrow_shared: unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult,
    try_borrow_exclusive: unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult,
}

pub(crate) unsafe extern "C" fn init_ledger() {
    LEDGER.get_or_init(|| {
        let unrooted = Unrooted::new();
        let module = Module::jlrs_core(&unrooted);

        let module = module.submodule(unrooted, "Ledger").unwrap().as_managed();

        let api_version = *module
            .global(unrooted, "API_VERSION_FN")
            .unwrap()
            .as_value()
            .data_ptr()
            .cast::<Option<unsafe extern "C" fn() -> usize>>()
            .as_ref()
            .as_ref()
            .unwrap();

        let is_borrowed_shared = *module
            .global(unrooted, "IS_BORROWED_SHARED")
            .unwrap()
            .as_value()
            .data_ptr()
            .cast::<Option<unsafe extern "C" fn(*const c_void) -> LedgerResult>>()
            .as_ref()
            .as_ref()
            .unwrap();

        let is_borrowed_exclusive = *module
            .global(unrooted, "IS_BORROWED_EXCLUSIVE")
            .unwrap()
            .as_value()
            .data_ptr()
            .cast::<Option<unsafe extern "C" fn(*const c_void) -> LedgerResult>>()
            .as_ref()
            .as_ref()
            .unwrap();

        let is_borrowed = *module
            .global(unrooted, "IS_BORROWED")
            .unwrap()
            .as_value()
            .data_ptr()
            .cast::<Option<unsafe extern "C" fn(*const c_void) -> LedgerResult>>()
            .as_ref()
            .as_ref()
            .unwrap();

        let try_borrow_shared = *module
            .global(unrooted, "BORROW_SHARED")
            .unwrap()
            .as_value()
            .data_ptr()
            .cast::<Option<unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult>>()
            .as_ref()
            .as_ref()
            .unwrap();

        let try_borrow_exclusive = *module
            .global(unrooted, "BORROW_EXCLUSIVE")
            .unwrap()
            .as_value()
            .data_ptr()
            .cast::<Option<unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult>>()
            .as_ref()
            .as_ref()
            .unwrap();

        let borrow_shared_unchecked = *module
            .global(unrooted, "BORROW_SHARED_UNCHECKED")
            .unwrap()
            .as_value()
            .data_ptr()
            .cast::<Option<unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult>>()
            .as_ref()
            .as_ref()
            .unwrap();

        let unborrow_shared = *module
            .global(unrooted, "UNBORROW_SHARED")
            .unwrap()
            .as_value()
            .data_ptr()
            .cast::<Option<unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult>>()
            .as_ref()
            .as_ref()
            .unwrap();

        let unborrow_exclusive = *module
            .global(unrooted, "UNBORROW_EXCLUSIVE")
            .unwrap()
            .as_value()
            .data_ptr()
            .cast::<Option<unsafe extern "C" fn(ptr: *const c_void) -> LedgerResult>>()
            .as_ref()
            .as_ref()
            .unwrap();

        Ledger {
            api_version,
            is_borrowed_shared,
            is_borrowed_exclusive,
            is_borrowed,
            try_borrow_shared,
            try_borrow_exclusive,
            borrow_shared_unchecked,
            unborrow_shared,
            unborrow_exclusive,
        }
    });

    assert_eq!(Ledger::api_version(), API_VERSION);
}

impl Ledger {
    #[inline]
    pub(crate) fn api_version() -> usize {
        unsafe { (LEDGER.get_unchecked().api_version)() }
    }

    #[inline]
    pub(crate) fn is_borrowed_shared(data: Value) -> JlrsResult<bool> {
        unsafe {
            match (LEDGER.get_unchecked().is_borrowed_shared)(data.data_ptr().as_ptr()) {
                LedgerResult::OkFalse => Ok(false),
                LedgerResult::OkTrue => Ok(true),
                LedgerResult::Err => Err(JlrsError::exception("unexpected error"))?,
            }
        }
    }

    #[inline]
    pub(crate) fn is_borrowed_exclusive(data: Value) -> JlrsResult<bool> {
        unsafe {
            match (LEDGER.get_unchecked().is_borrowed_exclusive)(data.unwrap(Private).cast()) {
                LedgerResult::OkFalse => Ok(false),
                LedgerResult::OkTrue => Ok(true),
                LedgerResult::Err => Err(JlrsError::exception("unexpected error"))?,
            }
        }
    }

    #[inline]
    pub(crate) fn is_borrowed(data: Value) -> JlrsResult<bool> {
        unsafe {
            match (LEDGER.get_unchecked().is_borrowed)(data.data_ptr().as_ptr()) {
                LedgerResult::OkFalse => Ok(false),
                LedgerResult::OkTrue => Ok(true),
                LedgerResult::Err => Err(JlrsError::exception("unexpected error"))?,
            }
        }
    }

    #[inline]
    pub(crate) fn try_borrow_shared(data: Value) -> JlrsResult<bool> {
        unsafe {
            match (LEDGER.get_unchecked().try_borrow_shared)(data.data_ptr().as_ptr()) {
                LedgerResult::OkFalse => Ok(false),
                LedgerResult::OkTrue => Ok(true),
                LedgerResult::Err => Err(JlrsError::exception("already exclusively borrowed"))?,
            }
        }
    }

    #[inline]
    pub(crate) fn try_borrow_exclusive(data: Value) -> JlrsResult<bool> {
        unsafe {
            match (LEDGER.get_unchecked().try_borrow_exclusive)(data.data_ptr().as_ptr()) {
                LedgerResult::OkFalse => Ok(false),
                LedgerResult::OkTrue => Ok(true),
                LedgerResult::Err => Err(JlrsError::exception("already exclusively borrowed"))?,
            }
        }
    }

    #[inline]
    pub(crate) unsafe fn borrow_shared_unchecked(data: Value) -> JlrsResult<bool> {
        unsafe {
            match (LEDGER.get_unchecked().borrow_shared_unchecked)(data.data_ptr().as_ptr()) {
                LedgerResult::OkFalse => Ok(false),
                LedgerResult::OkTrue => Ok(true),
                LedgerResult::Err => Err(JlrsError::exception("already exclusively borrowed"))?,
            }
        }
    }

    #[inline]
    pub(crate) unsafe fn unborrow_shared(data: Value) -> JlrsResult<bool> {
        unsafe {
            match (LEDGER.get_unchecked().unborrow_shared)(data.data_ptr().as_ptr()) {
                LedgerResult::OkFalse => Ok(false),
                LedgerResult::OkTrue => Ok(true),
                LedgerResult::Err => Err(JlrsError::exception("not borrowed"))?,
            }
        }
    }

    #[inline]
    pub(crate) unsafe fn unborrow_exclusive(data: Value) -> JlrsResult<bool> {
        unsafe {
            match (LEDGER.get_unchecked().unborrow_exclusive)(data.data_ptr().as_ptr()) {
                LedgerResult::OkFalse => Ok(false),
                LedgerResult::OkTrue => Ok(true),
                LedgerResult::Err => Err(JlrsError::exception("not borrowed"))?,
            }
        }
    }
}

// Adapted from neon:
// https://github.com/neon-bindings/neon/blob/09c04b3129798b16021549352c74323f629d5bb0/crates/neon/src/types/buffer/lock.rs

use std::{ffi::c_void, ops::Range};

use once_cell::sync::OnceCell;

use crate::error::{AccessError, JlrsResult};

#[derive(Debug, Default)]
pub struct Ledger {
    owned: Vec<Range<*const u8>>,
    shared: Vec<Range<*const u8>>,
}

use std::sync::{Arc, Mutex};
static LEDGER: OnceCell<Arc<Mutex<Ledger>>> = OnceCell::new();

pub(crate) unsafe extern "C" fn init_ledger(ledger_ref: &mut *mut c_void) {
    if ledger_ref.is_null() {
        LEDGER.get_or_init(|| {
            let ledger = Arc::new(Mutex::new(Ledger::default()));
            let cloned = ledger.clone();
            *ledger_ref = Arc::into_raw(ledger) as *mut c_void;
            cloned
        });
    } else {
        LEDGER.get_or_init(|| {
            std::mem::transmute::<&mut *mut c_void, &Arc<Mutex<Ledger>>>(ledger_ref).clone()
        });
    }
}

unsafe impl Send for Ledger {}
unsafe impl Sync for Ledger {}

impl Ledger {
    // TODO: generic?
    pub(crate) fn is_borrowed(range: Range<*const u8>) -> bool {
        let ledger = LEDGER.get().unwrap().lock().expect("Corrupted ledger");
        check_overlap(ledger.shared.as_ref(), &range).is_err()
    }

    pub(crate) fn is_borrowed_mut(range: Range<*const u8>) -> bool {
        let ledger = LEDGER.get().unwrap().lock().expect("Corrupted ledger");
        check_overlap(ledger.owned.as_ref(), &range).is_err()
    }

    pub(crate) fn is_borrowed_any(range: Range<*const u8>) -> bool {
        let ledger = LEDGER.get().unwrap().lock().expect("Corrupted ledger");
        check_overlap(ledger.shared.as_ref(), &range).is_err()
            || check_overlap(ledger.owned.as_ref(), &range).is_err()
    }

    pub(crate) unsafe fn clone_shared(range: Range<*const u8>) {
        LEDGER
            .get()
            .unwrap()
            .lock()
            .expect("Corrupted ledger")
            .shared
            .push(range.clone())
    }

    pub(crate) fn unborrow_shared(range: Range<*const u8>) {
        let mut ledger = LEDGER.get().unwrap().lock().expect("Corrupted ledger");
        let i = ledger.shared.iter().rposition(|r| r == &range).unwrap();
        ledger.shared.remove(i);
    }

    pub(crate) fn unborrow_owned(range: Range<*const u8>) {
        let mut ledger = LEDGER.get().unwrap().lock().expect("Corrupted ledger");
        let i = ledger.owned.iter().rposition(|r| r == &range).unwrap();
        ledger.owned.remove(i);
    }

    // Dynamically check a slice conforms to borrow rules
    pub(crate) fn try_borrow(range: Range<*const u8>) -> JlrsResult<()> {
        LEDGER
            .get()
            .unwrap()
            .lock()
            .expect("Corrupted ledger")
            .try_add_borrow(range.clone())
    }

    // Dynamically check a mutable slice conforms to borrow rules before returning by
    // using interior mutability of the ledger.
    pub(crate) fn try_borrow_mut(range: Range<*const u8>) -> JlrsResult<()> {
        LEDGER
            .get()
            .unwrap()
            .lock()
            .expect("Corrupted ledger")
            .try_add_borrow_mut(range.clone())
    }

    // Try to add an immutable borrow to the ledger
    fn try_add_borrow(&mut self, range: Range<*const u8>) -> JlrsResult<()> {
        // Check if the borrow overlaps with any active mutable borrow
        check_overlap(&self.owned, &range)?;

        // Record a record of the immutable borrow
        self.shared.push(range);

        Ok(())
    }

    // Try to add a mutable borrow to the ledger
    fn try_add_borrow_mut(&mut self, range: Range<*const u8>) -> JlrsResult<()> {
        // Check if the borrow overlaps with any active mutable borrow
        check_overlap(&self.owned, &range)?;

        // Check if the borrow overlaps with any active immutable borrow
        check_overlap(&self.shared, &range)?;

        // Record a record of the mutable borrow
        self.owned.push(range);

        Ok(())
    }
}

fn is_disjoint(a: &Range<*const u8>, b: &Range<*const u8>) -> bool {
    b.start >= a.end || a.start >= b.end
}

fn check_overlap(existing: &[Range<*const u8>], range: &Range<*const u8>) -> JlrsResult<()> {
    if existing.iter().all(|i| is_disjoint(i, range)) {
        Ok(())
    } else {
        Err(AccessError::BorrowError)?
    }
}

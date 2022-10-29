// Adapted from neon:
// https://github.com/neon-bindings/neon/blob/09c04b3129798b16021549352c74323f629d5bb0/crates/neon/src/types/buffer/lock.rs

use std::ops::Range;

use cfg_if::cfg_if;

use crate::error::{AccessError, JlrsResult};

cfg_if! {
    if #[cfg(feature = "unsafe-ledger")] {
        use std::cell::RefCell;
        thread_local! {
            pub(crate) static LEDGER: RefCell<Ledger> = RefCell::default();
        }
    } else {
        use std::sync::Mutex;

        static THREADSAFE_LEDGER: Mutex<Ledger> = Mutex::new(Ledger {
            owned: Vec::new(),
            shared: Vec::new(),
        });
    }
}

#[derive(Debug, Default)]
pub struct Ledger {
    owned: Vec<Range<*const u8>>,
    shared: Vec<Range<*const u8>>,
}

unsafe impl Send for Ledger {}
unsafe impl Sync for Ledger {}

impl Ledger {
    // TODO: generic?
    pub(crate) fn is_borrowed(range: Range<*const u8>) -> bool {
        cfg_if! {
            if #[cfg(feature = "unsafe-ledger")] {
                LEDGER.with(|ledger| {
                    let mut ledger = ledger.borrow_mut();
                    check_overlap(ledger.shared.as_ref(), &range).is_err()
                })
            } else {
                let ledger = THREADSAFE_LEDGER
                    .lock()
                    .expect("Corrupted ledger");

                check_overlap(ledger.shared.as_ref(), &range).is_err()
            }
        }
    }

    pub(crate) fn is_borrowed_mut(range: Range<*const u8>) -> bool {
        cfg_if! {
            if #[cfg(feature = "unsafe-ledger")] {
                LEDGER.with(|ledger| {
                    let mut ledger = ledger.borrow_mut();
                    check_overlap(ledger.owned.as_ref(), &range).is_err()
                })
            } else {
                let ledger = THREADSAFE_LEDGER
                    .lock()
                    .expect("Corrupted ledger");

                check_overlap(ledger.owned.as_ref(), &range).is_err()
            }
        }
    }

    pub(crate) fn is_borrowed_any(range: Range<*const u8>) -> bool {
        cfg_if! {
            if #[cfg(feature = "unsafe-ledger")] {
                LEDGER.with(|ledger| {
                    let mut ledger = ledger.borrow_mut();
                    check_overlap(ledger.owned.as_ref(), &range).is_err() ||
                        check_overlap(ledger.shared.as_ref(), &range).is_err()
                })
            } else {
                let ledger = THREADSAFE_LEDGER
                    .lock()
                    .expect("Corrupted ledger");

                check_overlap(ledger.owned.as_ref(), &range).is_err() ||
                    check_overlap(ledger.shared.as_ref(), &range).is_err()
            }
        }
    }

    pub(crate) unsafe fn clone_shared(range: Range<*const u8>) {
        cfg_if! {
            if #[cfg(feature = "unsafe-ledger")] {
                LEDGER.with(|ledger| {
                    let mut ledger = ledger.borrow_mut();
                    ledger.shared.push(range);
                })
            } else {
                THREADSAFE_LEDGER
                    .lock()
                    .expect("Corrupted ledger")
                    .shared
                    .push(range.clone())
            }
        }
    }

    pub(crate) fn unborrow_shared(range: Range<*const u8>) {
        cfg_if! {
            if #[cfg(feature = "unsafe-ledger")] {
                LEDGER.with(|ledger| {
                    let mut ledger = ledger.borrow_mut();
                    let i = ledger.shared.iter().rposition(|r| r == &range).unwrap();
                    ledger.shared.remove(i);
                })
            } else {
                let mut ledger = THREADSAFE_LEDGER
                    .lock()
                    .expect("Corrupted ledger");

                let i = ledger.shared.iter().rposition(|r| r == &range).unwrap();
                ledger.shared.remove(i);
            }
        }
    }

    pub(crate) fn unborrow_owned(range: Range<*const u8>) {
        cfg_if! {
            if #[cfg(feature = "unsafe-ledger")] {
                LEDGER.with(|ledger| {
                    let mut ledger = ledger.borrow_mut();
                    let i = ledger.owned.iter().rposition(|r| r == &range).unwrap();
                    ledger.owned.remove(i);
                })
            } else {
                let mut ledger = THREADSAFE_LEDGER
                    .lock()
                    .expect("Corrupted ledger");

                let i = ledger.owned.iter().rposition(|r| r == &range).unwrap();
                ledger.owned.remove(i);

            }
        }
    }

    // Dynamically check a slice conforms to borrow rules before returning by
    // using interior mutability of the ledger.
    pub(crate) fn try_borrow(range: Range<*const u8>) -> JlrsResult<()> {
        cfg_if! {
            if #[cfg(feature = "unsafe-ledger")] {
                LEDGER.with(|ledger| {
                    let mut ledger = ledger.borrow_mut();
                    ledger.try_add_borrow(range)
                })
            } else {
                THREADSAFE_LEDGER
                    .lock()
                    .expect("Corrupted ledger")
                    .try_add_borrow(range.clone())
            }
        }
    }

    // Dynamically check a mutable slice conforms to borrow rules before returning by
    // using interior mutability of the ledger.
    pub(crate) fn try_borrow_mut(range: Range<*const u8>) -> JlrsResult<()> {
        cfg_if! {
            if #[cfg(feature = "unsafe-ledger")] {
                LEDGER.with(move |ledger| {
                    let mut ledger = ledger.borrow_mut();
                    ledger.try_add_borrow_mut(range)
                })
            } else {
                THREADSAFE_LEDGER
                    .lock()
                    .expect("Corrupted ledger")
                    .try_add_borrow_mut(range.clone())
            }
        }
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

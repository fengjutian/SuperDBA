//! Recovery-aware locking for the small `Mutex`-guarded state maps in this
//! crate.
//!
//! The state behind these locks is plain `HashMap`/`Option` values that are
//! written in one operation, so a panic while a guard is held cannot leave a
//! cross-field invariant broken. What it *can* do is poison the mutex, and the
//! default `lock().unwrap()` would then turn every later access into a panic —
//! usually on an unrelated background thread, long after the original fault.
//! Recovering the guard keeps the process usable and lets the original panic
//! surface where it happened.

use std::sync::{Mutex, MutexGuard};

/// Lock `mutex`, recovering the guard if the mutex was poisoned.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

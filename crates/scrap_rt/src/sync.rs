//! Synchronization primitives — cfg-gated over std vs parking_lot.

#[cfg(feature = "parking-lot")]
pub(crate) use parking_lot::{Condvar, Mutex};
#[cfg(not(feature = "parking-lot"))]
pub(crate) use std::sync::{Condvar, Mutex};

/// Lock a mutex, abstracting over std (`lock().unwrap()`) vs parking_lot (`lock()`).
macro_rules! mutex_lock {
    ($mutex:expr) => {{
        #[cfg(not(feature = "parking-lot"))]
        {
            $mutex.lock().unwrap()
        }
        #[cfg(feature = "parking-lot")]
        {
            $mutex.lock()
        }
    }};
}

/// Try to lock a mutex, abstracting over std vs parking_lot.
macro_rules! mutex_try_lock {
    ($mutex:expr) => {{
        #[cfg(not(feature = "parking-lot"))]
        {
            $mutex.try_lock().ok()
        }
        #[cfg(feature = "parking-lot")]
        {
            $mutex.try_lock()
        }
    }};
}

/// Wait on a condvar, abstracting over std vs parking_lot.
macro_rules! condvar_wait {
    ($condvar:expr, $guard:expr) => {{
        #[cfg(not(feature = "parking-lot"))]
        {
            $guard = $condvar.wait($guard).unwrap();
        }
        #[cfg(feature = "parking-lot")]
        {
            $condvar.wait(&mut $guard);
        }
    }};
}

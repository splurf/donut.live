use parking_lot::{Condvar, Mutex, MutexGuard, RwLock};
use std::sync::Arc;

/// A shared state, conveniently bundled with its respective condition variable
#[derive(Default)]
pub struct CondLock<T>(Arc<(RwLock<T>, Mutex<bool>, Condvar)>);

impl<T> CondLock<T> {
    /// Acquires the mutex for the boolean predicate
    pub fn lock(&self) -> MutexGuard<bool> {
        self.0 .1.lock()
    }

    /// Blocks the current thread until the condition variable receives a notification and the predicate is false
    pub fn wait(&self) {
        let cvar = &self.0 .2;
        let mut lock = self.lock();
        cvar.wait_while(&mut lock, |m| !*m)
    }

    /// Wakes up all blocked threads on the condition variable
    pub fn notify(&self) {
        self.0 .2.notify_one();
    }
}

impl<T> Clone for CondLock<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> std::ops::Deref for CondLock<T> {
    type Target = RwLock<T>;

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

use parking_lot::{Condvar, Mutex, MutexGuard, RwLock};
use std::sync::Arc;

/// A concurrency-safe wrapper, conveniently bundled with its respective condition variable
#[derive(Default)]
pub struct SignalLock<T> {
    inner: Arc<(RwLock<T>, Mutex<bool>, Condvar)>,
}

impl<T> SignalLock<T> {
    /// Acquires the mutex for the boolean predicate
    pub fn lock(&self) -> MutexGuard<bool> {
        self.inner.1.lock()
    }

    /// Blocks the current thread until the condition variable receives a notification and the predicate is false
    pub fn wait(&self) {
        let cvar = &self.inner.2;
        let mut lock = self.lock();
        cvar.wait_while(&mut lock, |m| !*m)
    }

    /// Wakes up all blocked threads on the condition variable
    pub fn notify(&self) {
        self.inner.2.notify_one();
    }
}

impl<T> Clone for SignalLock<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> std::ops::Deref for SignalLock<T> {
    type Target = RwLock<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

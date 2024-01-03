use {
    crate::Result,
    std::{
        ops::Deref,
        sync::{Arc, Condvar, Mutex, MutexGuard, RwLock},
    },
};

/// A shared state, conveniently bundled with its respective condition variable
#[derive(Default)]
pub struct CondLock<T>(Arc<(RwLock<T>, Mutex<bool>, Condvar)>);

impl<T> CondLock<T> {
    /// Acquires the mutex for the boolean predicate
    pub fn lock(&self) -> Result<MutexGuard<bool>> {
        self.0 .1.lock().map_err(Into::into)
    }

    /// Blocks the current thread until the condition variable receives a notification
    pub fn wait(&self) -> Result<()> {
        let cvar = &self.0 .2;
        let mut started = self.lock()?;
        while !*started {
            started = cvar.wait(started)?;
        }
        Ok(())
    }

    /// Wakes up one blocked thread on the condition variable
    pub fn notify(&self) {
        self.0 .2.notify_one()
    }
}

impl<T> Clone for CondLock<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for CondLock<T> {
    type Target = RwLock<T>;

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

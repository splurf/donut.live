use {
    super::Result,
    std::sync::{Arc, Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/**
 * A shared state conveniently bundled with its respective condition variable
 */
#[derive(Default)]
pub struct CondLock<T>(Arc<(RwLock<T>, Mutex<bool>, Condvar)>);

impl<T> CondLock<T> {
    /** Return a guard of the data with exclusuve read access */
    pub fn read(&self) -> Result<RwLockReadGuard<T>> {
        self.0 .0.read().map_err(Into::into)
    }

    /** Return a guard of the data with exclusuve write access */
    pub fn write(&self) -> Result<RwLockWriteGuard<T>> {
        self.0 .0.write().map_err(Into::into)
    }

    /** Acquires the mutex for the boolean predicate */
    pub fn lock(&self) -> Result<MutexGuard<bool>> {
        self.0 .1.lock().map_err(Into::into)
    }

    /** Blocks the current thread until the condition variable receives a notification */
    pub fn wait(&self) -> Result<()> {
        let cvar = &self.0 .2;
        let mut started = self.lock()?;
        while !*started {
            started = cvar.wait(started)?;
        }
        Ok(())
    }

    /** Wakes up one blocked thread on the condition variable */
    pub fn notify_one(&self) {
        self.0 .2.notify_one()
    }
}

impl<T> Clone for CondLock<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

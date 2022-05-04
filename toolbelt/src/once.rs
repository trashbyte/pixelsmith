use std::sync::Mutex;


/// A simple primitive for ensuring something is done exactly once.
///
/// e.g.
/// ```
/// let mut once_task = DoOnce::new();
/// loop {
///     once_task.do_once(|| {
///         // this closure only runs once
///     })
///     //...
/// }
/// ```
pub struct DoOnce(bool);
impl DoOnce {
    pub const fn new() -> Self { DoOnce(false) }

    /// The passed closure will be called only once, after that calling this will be a no-op.
    pub fn do_once<F: FnOnce()>(&mut self, func: F) {
        if !self.0 {
            func();
            self.0 = true;
        }
    }

    /// Returns true if the task has been run once already
    pub fn done(&self) -> bool { self.0 }
}


/// A simple primitive for ensuring something is done exactly once. DoOnceSync is thread-safe
/// and uses internal mutability, so you can do_once with a const reference.
///
/// e.g.
/// ```
/// let once_task = DoOnceSync::new();
/// loop {
///     once_task.do_once(|| {
///         // this closure only runs once
///     })
///     //...
/// }
/// ```
pub struct DoOnceSync(Mutex<bool>);
impl DoOnceSync {
    pub fn new() -> Self { DoOnceSync(Mutex::new(false)) }

    /// The passed closure will be called only once, after that calling this will be a no-op.
    pub fn do_once<F: FnOnce()>(&self, func: F) {
        let mut lock = self.0.lock().unwrap();
        if !*lock {
            func();
            *lock = true;
        }
    }

    /// Returns true if the task has been run once already
    pub fn done(&self) -> bool { *self.0.lock().unwrap() }
}
unsafe impl Send for DoOnceSync {}
unsafe impl Sync for DoOnceSync {}

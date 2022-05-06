use std::cell::UnsafeCell;
use std::fmt::{Debug, Display, Formatter};
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


/// A simple once-initialized immutable-ish reference for easy global statics.
/// Panics if accessed while uninitialized or initialized twice.
///
/// ```rs
/// const SOME_CONSTANT: UnsafeInitOnce<SomeType> = SOME_CONSTANT::uninitialized();
/// SOME_CONSTANT.initialize(value);
/// SOME_CONSTANT.get()
/// ```
///
/// # Safety
///
/// InitOnce uses a lot of unsafe code internally to access the contents of the UnsafeCell,
/// but the end user doesn't need to worry about &mut aliasing because the API only exposes
/// immutable pointers.
#[repr(transparent)]
pub struct InitOnce<T> {
    inner: UnsafeCell<Option<T>>,
}
impl<T> InitOnce<T> {
    /// Creates a new empty InitOnce. This `fn` is `const` so it can be used in statics.
    pub const fn uninitialized() -> Self {
        InitOnce { inner: UnsafeCell::new(None) }
    }

    /// Attempt to get a reference to the value contained within.
    /// Safely returns `None` if uninitialized.
    pub fn try_get(&self) -> Option<&T> {
        let inner = unsafe { &mut *self.inner.get() };
        inner.as_ref()
    }

    /// Retrieves a reference to the value contained within. Panics if uninitialized.
    pub fn get(&self) -> &T {
        unsafe {
            let r = self.inner.get().as_ref().unwrap();
            match r {
                Some(r) => r,
                None => panic!("Tried to access InitOnce<{}> before initialization", std::any::type_name::<T>())
            }
        }
    }

    /// Inserts a value into this InitOnce. Utilizes interior mutability so only `&self` is required.
    /// Panics if already initialized.
    pub fn initialize(&self, value: T) {
        unsafe {
            let ptr = self.inner.get();
            if (*ptr).is_some() {
                panic!("Tried to initialize InitOnce<{}> a second time", std::any::type_name::<T>());
            }
            ptr.write(Some(value));
        }

    }
}

impl <T: Debug> Debug for InitOnce<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        T::fmt(self.get(), f)
    }
}

impl <T: Display> Display for InitOnce<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        T::fmt(self.get(), f)
    }
}

unsafe impl<T: Sync> Sync for InitOnce<T> {}

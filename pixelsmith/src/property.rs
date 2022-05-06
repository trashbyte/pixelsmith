use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};


struct PropertyBinding<'b, T: 'b> {
    value: &'b mut T,
    lock: Arc<AtomicBool>,
}

impl<'b, T: 'b> PropertyBinding<'b, T> {
    pub fn value(&'b mut self) ->  &'b mut T {
        self.value
    }
}

impl<'b, T> Drop for PropertyBinding<'b, T> {
    fn drop(&mut self) {
        if self.lock.swap(false, Ordering::SeqCst) == false {
            panic!("PropertyBinding<{}>: Tried to drop a lock that was already unlocked!", std::any::type_name::<T>())
        }
    }
}

pub struct Property<T> {
    property: UnsafeCell<T>,
    mut_lock: Arc<AtomicBool>
}

impl<T> Property<T> {
    pub fn new(value: T) -> Self {
        Property {
                property: UnsafeCell::new(value),
                mut_lock: Arc::new(AtomicBool::new(false))
        }
    }
}

impl<T: Copy> Property<T> {
    pub fn bind<F: FnOnce(&mut T)>(&self, f: F) {
        let was_locked = self.mut_lock.swap(true, Ordering::SeqCst);
        if was_locked {
            panic!() // TODO
        }
        let mut orig = unsafe { (*self.property.get()).clone() };
        f(&mut orig);
        unsafe { (*self.property.get()) = orig; };
        self.mut_lock.store(false, Ordering::SeqCst);
    }

    pub fn get(&self) -> T { unsafe { (*self.property.get()).clone() } }

    pub fn set(&self, value: T) {
        let was_locked = self.mut_lock.swap(true, Ordering::SeqCst);
        if was_locked {
            panic!() // TODO
        }
        unsafe { (*self.property.get()) = value; }
        self.mut_lock.store(false, Ordering::SeqCst);
    }
}

#![allow(dead_code)]

use std::borrow::Borrow;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::rc::Rc;


#[repr(transparent)]
pub struct SimpleCell<T: ?Sized>(Rc<RefCell<T>>);

impl<T> SimpleCell<T> {
    pub fn new(value: T) -> Self { SimpleCell(Rc::new(RefCell::new(value))) }

    pub fn replace(&self, value: T) -> T { self.0.replace(value) }

    pub fn try_unwrap(self) -> Result<T, SimpleCell<T>> {
        match Rc::try_unwrap(self.0) {
            Ok(refcell) => Ok(refcell.into_inner()),
            Err(rc) => Err(SimpleCell(rc))
        }
    }

    pub fn replace_with<F>(&self, f: F) -> T where F: FnOnce(&mut T) -> T {
        self.0.replace_with(f)
    }

    pub fn swap(&self, other: &SimpleCell<T>) { self.0.swap(other.0.borrow()) }

    pub fn get(&self) -> Ref<T> {
        RefCell::borrow(Rc::borrow(&self.0))
    }

    pub fn try_get(&self) -> Result<Ref<T>, std::cell::BorrowError> {
        self.0.try_borrow()
    }

    pub fn get_mut(&self) -> RefMut<T> {
        RefCell::borrow_mut(&self.0)
    }

    pub fn try_get_mut(&self) -> Result<RefMut<T>, std::cell::BorrowMutError> {
        self.0.try_borrow_mut()
    }

    pub fn as_ptr(&self) -> *mut T { self.0.as_ptr() }

    pub fn clone(&self) -> Self {
        SimpleCell(self.0.clone())
    }
}

impl<T: Clone + ?Sized> SimpleCell<T> {
    fn boxed(value: &T) -> SimpleCell<Box<T>> {
        SimpleCell(Rc::new(RefCell::new(Box::new(value.clone()))))
    }
}

impl<T: Default> SimpleCell<T> {
    pub fn take(&self) -> T { self.0.take() }
}

impl<T> Deref for SimpleCell<T> {
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        Rc::borrow(&self.0)
    }
}

impl<T: Default> Default for SimpleCell<T> {
    fn default() -> Self { SimpleCell::new(T::default()) }
}

impl<T: Debug> Debug for SimpleCell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T> From<T> for SimpleCell<T> {
    fn from(t: T) -> Self {
        SimpleCell(Rc::from(RefCell::from(t)))
    }
}

impl<T: PartialEq + ?Sized> PartialEq<SimpleCell<T>> for SimpleCell<T> {
    fn eq(&self, other: &SimpleCell<T>) -> bool {
        self.borrow() == other.borrow()
    }
}

impl<T: Eq> Eq for SimpleCell<T> {}

use std::cell::UnsafeCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use parking_lot::{Mutex, MutexGuard};
use imgui::Ui;
use crate::panel::Panel;

pub struct InspectorPanel {
    window_open: bool,
}

impl InspectorPanel {

}

impl super::Panel for Rc<UnsafeCell<InspectorPanel>> {
    fn is_open(&self) -> bool {
        unsafe { (*self.deref().get()).window_open }
    }

    fn open_ref(&mut self) -> &mut bool {
        unsafe { &mut (*self.deref().get()).window_open }
    }

    fn draw(&mut self, ui: &mut Ui) {
        todo!()
    }
}
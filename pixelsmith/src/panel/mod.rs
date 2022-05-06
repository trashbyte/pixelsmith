pub mod inspector;

use std::borrow::BorrowMut;
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use parking_lot::{Mutex, MutexGuard};
pub use inspector::InspectorPanel;


macro_rules! build_panel_set {
    ($($name:ident: $type:ident,)+) => {
        /// A collection of all panels the app uses.
        pub struct PanelSet {
            $(pub $name: std::sync::Arc<parking_lot::Mutex<$type>>),*
        }

        impl PanelSet {
            // /// Returns an Iter over all of the panels in the set.
            // pub fn iter(&self) -> std::slice::Iter<&dyn Panel> {
            //     [
            //         $(self.$name.dyn_ref()),*
            //     ].iter()
            // }
            // /// Returns an Iter over all of the panels in the set.
            // pub fn iter_mut(&self) -> std::slice::Iter<&mut dyn Panel> {
            //     [
            //         $(self.$name.dyn_ref_mut()),*
            //     ].iter()
            // }
        }
    };
}
build_panel_set! {
    inspector: InspectorPanel,
}


pub trait Panel {
    /// True if the panel is open.
    fn is_open(&self) -> bool;
    /// Gets a &mut to the open parameter for imgui to use.
    fn open_ref(&mut self) -> &mut bool;
    /// Sets whether a panel is open.
    fn set_open(&mut self, new_open: bool) { *self.open_ref() = new_open }

    /// Draw the main panel contents with imgui. A panel is in charge of drawing its own window
    /// or not - don't call this from inside of a Window closure, the panel itself needs to do that.
    fn draw(&mut self, ui: &mut imgui::Ui);
}

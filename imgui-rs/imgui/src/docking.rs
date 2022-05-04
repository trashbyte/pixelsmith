use std::ffi::CString;
use std::os::raw::c_char;

use crate::Direction;
use crate::ImStr;
use crate::Ui;

pub struct DockNode {
    id: u32,
}

bitflags::bitflags!(
    #[repr(transparent)]
    pub struct DockNodeFlags: u32 {
        const NONE = sys::ImGuiDockNodeFlags_None;
        const KEEP_ALIVE_ONLY = sys::ImGuiDockNodeFlags_KeepAliveOnly;
        const NO_DOCKING_IN_CENTRAL_NODE = sys::ImGuiDockNodeFlags_NoDockingInCentralNode;
        const PASSTHRU_CENTRAL_NODE = sys::ImGuiDockNodeFlags_PassthruCentralNode;
        const NO_SPLIT = sys::ImGuiDockNodeFlags_NoSplit;
        const NO_RESIZE = sys::ImGuiDockNodeFlags_NoResize;
        const AUTO_HIDE_TAB_BAR = sys::ImGuiDockNodeFlags_AutoHideTabBar;
    }
);

impl DockNode {
    fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn is_split(&self) -> bool {
        unsafe {
            let node = sys::igDockBuilderGetNode(self.id);
            if std::ptr::null() == node {
                false
            } else {
                sys::ImGuiDockNode_IsSplitNode(node)
            }
        }
    }
    /// Dock window into this dockspace
    #[doc(alias = "DockBuilder::DockWindow")]
    pub fn dock_window(&self, window: &ImStr) {
        unsafe { sys::igDockBuilderDockWindow(window.as_ptr(), self.id) }
    }

    #[doc(alias = "DockBuilder::SplitNode")]
    pub fn split<D, O>(&self, split_dir: Direction, size_ratio: f32, dir: D, opposite_dir: O)
        where
            D: FnOnce(DockNode),
            O: FnOnce(DockNode),
    {
        if self.is_split() {
            // Can't split an already split node (need to split the
            // node within)
            return;
        }

        let mut out_id_at_dir: sys::ImGuiID = 0;
        let mut out_id_at_opposite_dir: sys::ImGuiID = 0;
        unsafe {
            sys::igDockBuilderSplitNode(
                self.id,
                split_dir as i32,
                size_ratio,
                &mut out_id_at_dir,
                &mut out_id_at_opposite_dir,
            );
        }

        dir(DockNode::new(out_id_at_dir));
        opposite_dir(DockNode::new(out_id_at_opposite_dir));
    }
}

/// # Docking
impl Ui {
    #[doc(alias = "IsWindowDocked")]
    pub fn is_window_docked(&self) -> bool {
        unsafe { sys::igIsWindowDocked() }
    }

    /// Create dockspace with given label. Returns a handle to the
    /// dockspace which can be used to, say, programatically split or
    /// dock windows into it
    #[doc(alias = "DockSpace")]
    pub fn dockspace<L: AsRef<str>>(&self, label: L, flags: DockNodeFlags) -> DockNode {
        unsafe {
            let id = sys::igGetIDStr(self.scratch_txt(label) as *const c_char);
            sys::igDockSpace(
                id,
                [0.0, 0.0].into(),
                sys::ImGuiDockNodeFlags::from(flags.bits() as i32),
                ::std::ptr::null::<sys::ImGuiWindowClass>(),
            );
            DockNode { id }
        }
    }

    #[doc(alias = "DockSpaceOverViewport")]
    pub fn dockspace_over_viewport(&self, flags: DockNodeFlags) -> sys::ImGuiID {
        unsafe {
            sys::igDockSpaceOverViewport(
                sys::igGetMainViewport(),
                sys::ImGuiDockNodeFlags::from(flags.bits() as i32),
                ::std::ptr::null::<sys::ImGuiWindowClass>(),
            )
        }
    }

    pub fn dock_into_named_window(&self, label: &str, direction: crate::Direction, ratio: f32) {
        unsafe {
            let current = sys::igGetCurrentWindow();
            let cstr = CString::new(label).unwrap();
            let target = sys::igFindWindowByName(cstr.as_ptr());
            sys::igDockContextQueueDock(sys::igGetCurrentContext(),
                                        target,
                                        (*target).DockNode,
                                        current,
                                        direction as sys::ImGuiDir, 1.0-ratio, true
            );
        }
    }

    pub fn dock_into_node(&self, dock_node: sys::ImGuiID) {
        unsafe {
            sys::igDockBuilderDockWindow((*sys::igGetCurrentWindow()).Name, dock_node);
            sys::igDockBuilderFinish(dock_node);
        }
    }

    pub fn hide_dock_node_tabs(&self) {
        unsafe {
            let ptr = sys::igGetWindowDockNode();
            if !ptr.is_null() {
                (*ptr).LocalFlags |= sys::ImGuiDockNodeFlags_HiddenTabBar;
            }
            else {
                eprintln!("DockNode is null");
            }
        }
    }
}

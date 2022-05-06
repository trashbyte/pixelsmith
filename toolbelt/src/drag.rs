use cgmath::{Point2, Vector2, Zero};


#[derive(Debug, Clone)]
pub struct DragState<T> {
    state: Option<T>,
    prev_pos: Option<Point2<f32>>,
}

impl<T> Default for DragState<T> {
    fn default() -> Self { DragState::new() }
}

impl<T> DragState<T> {
    pub const fn new() -> Self {
        DragState {
            state: None,
            prev_pos: None
        }
    }

    pub fn active(&self) -> bool { self.state.is_some() }

    pub fn state(&self) -> &Option<T> { &self.state }

    /// Returns the previous state, if any
    pub fn activate(&mut self, new_state: T, starting_pos: Option<impl Into<Point2<f32>>>) -> Option<T> {
        self.prev_pos = starting_pos.map(|i| i.into());
        self.state.replace(new_state)
    }

    /// Returns the previous state, if any
    pub fn deactivate(&mut self) -> Option<T> {
        self.prev_pos = None;
        self.state.take()
    }

    /// Returns Err if not active, otherwise returns Ok(∆position)
    pub fn update(&mut self, new_pos: impl Into<Point2<f32>>) -> Result<Vector2<f32>, ()> {
        let new_pos = new_pos.into();
        if self.active() {
            match self.prev_pos {
                Some(prev) => {
                    let delta = new_pos - prev;
                    self.prev_pos = Some(new_pos);
                    Ok(delta)
                },
                None => {
                    self.prev_pos = Some(new_pos);
                    Ok(Vector2::zero())
                }
            }
        }
        else { Err(()) }
    }
}

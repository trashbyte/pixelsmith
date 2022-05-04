use crate::lerp;

#[derive(Debug, Copy, Clone, Default)]
pub struct FloatCurvePoint {
    pub time: f32,
    pub value: f32,
    pub arrive_tangent: f32,
    pub leave_tangent: f32,
}
impl FloatCurvePoint {
    pub fn new(time: f32, value: f32, arrive_tangent: f32, leave_tangent: f32) -> Self {
        Self { time, value, arrive_tangent, leave_tangent }
    }
}

// implementation note: points assumed to be sorted in time order
// maintain this invariant in all internal functions
#[derive(Debug, Clone, Default)]
pub struct FloatCurve {
    points: Vec<FloatCurvePoint>,
}

impl FloatCurve {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
        }
    }

    /// returns index of new element
    pub fn add_point_auto(&mut self, time: f32, value: f32) -> usize {
        let idx = self.add_point(time, value, 0.0, 0.0);
        self.calc_auto_tangent_for_point(idx);
        if idx > 0 {
            self.calc_auto_tangent_for_point(idx-1);
        }
        if idx < self.points.len()-1 {
            self.calc_auto_tangent_for_point(idx+1);
        }
        idx
    }

    /// returns index of new element
    pub fn add_point(&mut self, time: f32, value: f32, arrive_tangent: f32, leave_tangent: f32) -> usize {
        if self.points.is_empty() {
            // no points yet, just add this one
            self.points.push(FloatCurvePoint::new(time, value, arrive_tangent, leave_tangent));
            return 0;
        }

        if self.points[0].time > time {
            // target time is before the first point, add to the beginning of the list
            self.points.insert(0, FloatCurvePoint::new(time, value, arrive_tangent, leave_tangent));
            return 0;
        }

        let mut passed_index = None;
        for (i, p) in self.points.iter().enumerate() {
            if p.time < time {
                // passed first point less than target time, insert here
                passed_index = Some(i);
            }
        }

        if let Some(i) = passed_index {
            if i == self.points.len() - 1 {
                // past the last point, add to the end
                self.points.push(FloatCurvePoint::new(time, value, arrive_tangent, leave_tangent));
                return i+1;
            }
            else {
                // insert just after first passed point
                self.points.insert(i+1, FloatCurvePoint::new(time, value, arrive_tangent, leave_tangent));
                return i+1;
            }
        }
        unreachable!();
    }

    pub fn remove_point(&mut self, time: f32) {
        let mut index = None;
        for (i, p) in self.points.iter().enumerate() {
            // use nearly-equal for floats
            if (p.time - time).abs() < 0.0001 {
                index = Some(i);
                break;
            }
        }
        if let Some(i) = index {
            self.points.remove(i);
        }
    }

    pub fn clear_points(&mut self) {
        self.points.clear();
    }

    pub fn get_value(&self, time: f32) -> f32 {
        if self.points.is_empty() {
            // no points, return zero
            return 0.0;
        }

        if time < self.points[0].time {
            // target time is between start and first point, return value at first point
            // (curve is flat outside the points at both ends)
            return self.points[0].value;
        }

        let mut i = 0;
        loop {
            if (self.points[i].time - time).abs() < 0.000_001 {
                // we're sitting right on a point, just return that value
                return self.points[i].value;
            }
            if self.points[i].time > time {
                break; // just passed target time, p[i-1] and p[i] are our two points
            }
            if i == self.points.len()-1 {
                // reached end of points, select last point
                break;
            }
            i += 1;
        }

        if i == self.points.len()-1 {
            if self.points[self.points.len()-1].time > time {
                // between second-to-last and last points
                return solve_two_points(self.points[i-1], self.points[i], time);
            }
            else {
                // target time is after last point, return value at last point
                // (curve is flat outside the points at both ends)
                return self.points[self.points.len()-1].value;
            }
        }

        // at this point we're between two points p[i-1] and p[i]. return cubic interp between points
        solve_two_points(self.points[i-1], self.points[i], time)
    }

    fn calc_auto_tangent_for_point(&mut self, i: usize) {
        if i == 0 || i == self.points.len() - 1 {
            // ends are flat
            self.points[i].leave_tangent = 0.0;
            self.points[i].arrive_tangent = 0.0;
        }
        else {
            let x1 = self.points[i-1].time;
            let x2 = self.points[i+1].time;
            let y1 = self.points[i-1].value;
            let y2 = self.points[i+1].value;
            let slope = (y2-y1)/(x2-x1).max(0.00001);
            self.points[i].leave_tangent = slope;
            self.points[i].arrive_tangent = slope;
        }
    }
}

fn solve_two_points(a: FloatCurvePoint, b: FloatCurvePoint, time: f32) -> f32 {
    let diff = b.time - a.time;
    let alpha = (time - a.time) / diff;
    let p0 = a.value;
    let p3 = b.value;

    let p1 = p0 + (a.leave_tangent * diff * 0.3333);
    let p2 = p3 - (b.arrive_tangent * diff * 0.3333);

    interp_bezier_points(p0, p1, p2, p3, alpha)
}

fn interp_bezier_points(p0: f32, p1: f32, p2: f32, p3: f32, alpha: f32) -> f32 {
    let p01 = lerp(p0, p1, alpha);
    let p12 = lerp(p1, p2, alpha);
    let p23 = lerp(p2, p3, alpha);
    let p012 = lerp(p01, p12, alpha);
    let p123 = lerp(p12, p23, alpha);
    lerp(p012, p123, alpha)
}

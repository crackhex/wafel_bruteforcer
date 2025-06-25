use crate::bruteforce_params::Weights;
use wafel_api::Game;

/// Contains the information about Mario which is used when checking for bounds. These fields
/// mirror the fields of the weights and bounds structs, and should be updated accordingly if
/// more fields are added to the bounds and weights structs.
#[derive(Default)]
pub struct CommonMarioData {
    pub pos: [f32; 3],
    pub face_angle: [i16; 3],
    pub angle_vel: [i16; 3],
    pub forward_vel: f32,
}
impl CommonMarioData {
    pub fn new(pos: [f32; 3], face_angle: [i16; 3], angle_vel: [i16; 3], forward_vel: f32) -> Self {
        Self {
            pos,
            face_angle,
            angle_vel,
            forward_vel,
        }
    }
    pub fn new_from_game(game: &Game) -> Self {
        Self {
            pos: game.read("gMarioState.pos").as_f32_3(),
            face_angle: game.read("gMarioState.faceAngle").as_i16_3(),
            angle_vel: game.read("gMarioState.angleVel").as_i16_3(),
            forward_vel: game.read("gMarioState.forwardVel").as_f32(),
        }
    }
}

pub trait InBoundsTrait {
    fn check_if_all_true(&self) -> bool;
    fn adjust_weights(&self, weights: &mut Weights, bound_penalty: impl Into<f64>);
}

pub struct Bounds {
    pub pos_limits: [(f32, f32); 3],
    pub face_angle_limits: [(i32, i32); 3],
    pub angle_vel_limits: [(i16, i16); 3],
    pub hspd_limits: (f32, f32),
}

impl Bounds {
    pub fn new(
        pos_limits: [(f32, f32); 3],
        face_angle_limits: [(i32, i32); 3],
        angle_vel_limits: [(i16, i16); 3],
        hspd_limits: (f32, f32),
    ) -> Self {
        Bounds {
            pos_limits,
            face_angle_limits,
            angle_vel_limits,
            hspd_limits,
        }
    }
}

pub struct IsInBounds {
    pub pos_limits: InPosBounds,
    pub angle_vel_limits: InAngleVelBounds,
    pub face_angle_limits: InFaceAngleBounds,
    pub hspd_limits: InHspdBounds,
}

impl IsInBounds {
    pub const fn new() -> Self {
        Self {
            pos_limits: InPosBounds::new(),
            angle_vel_limits: InAngleVelBounds::new(),
            face_angle_limits: InFaceAngleBounds::new(),
            hspd_limits: InHspdBounds::new(),
        }
    }

    pub(crate) fn update_from_mario_data(&mut self, mario_data: &CommonMarioData, bounds: &Bounds) {
        self.pos_limits
            .evaluate_within_bounds(mario_data.pos, bounds.pos_limits);
        self.face_angle_limits
            .evaluate_within_bounds(mario_data.face_angle, bounds.face_angle_limits);
        self.angle_vel_limits
            .evaluate_within_bounds(mario_data.angle_vel, &bounds.angle_vel_limits);
        self.hspd_limits
            .evaluate_within_bounds(mario_data.forward_vel, bounds.hspd_limits);
    }

    /// Takes in Game reference and Bounds reference, determines if Mario is in bounds, and
    /// returns an Owned instance of IsInBounds containing the results
    pub fn new_from_game(game: &Game, bounds: &Bounds) -> Self {
        let pos = game.read("gMarioState.pos").as_f32_3();
        let angle = game.read("gMarioState.faceAngle").as_i16_3();
        let angle_vel = game.read("gMarioState.angleVel").as_i16_3();
        let hspd = game.read("gMarioState.forwardVel").as_f32();
        let mario_data = CommonMarioData::new(pos, angle, angle_vel, hspd);
        IsInBounds::new_from_mario_data(&mario_data, bounds)
    }

    /// Checks if the given data from CommonMarioData are within the specified bounds
    /// given by Bounds and returns an instance of IsInBounds containing the results.
    pub(crate) fn new_from_mario_data(mario_data: &CommonMarioData, bounds: &Bounds) -> Self {
        let mut is_in_bounds: Self = Self::new();
        is_in_bounds.update_from_mario_data(mario_data, bounds);
        is_in_bounds
    }

    pub(crate) fn check_if_all_true(&self) -> bool {
        self.pos_limits.check_if_all_true()
            && self.angle_vel_limits.check_if_all_true()
            && self.face_angle_limits.check_if_all_true()
            && self.hspd_limits.hspd
    }
}

#[derive(Default)]
pub struct InPosBounds {
    pub pos_x: bool,
    pub pos_y: bool,
    pub pos_z: bool,
}

impl InPosBounds {
    pub const fn new() -> Self {
        InPosBounds {
            pos_x: true,
            pos_y: true,
            pos_z: true,
        }
    }
    pub const fn evaluate_within_bounds(&mut self, pos: [f32; 3], pos_limits: [(f32, f32); 3]) {
        self.pos_x = (pos_limits[0].0 < pos[0]) && (pos[0] < pos_limits[0].1);
        self.pos_y = (pos_limits[1].0 < pos[1]) && (pos[1] < pos_limits[1].1);
        self.pos_z = (pos_limits[2].0 < pos[2]) && (pos[2] < pos_limits[2].1);
    }
}
impl InBoundsTrait for InPosBounds {
    /// Takes in a mutable reference to Self and checks if the given position is within
    /// the specified position limits. The fields for the struct are then set accordingly.
    fn check_if_all_true(&self) -> bool {
        self.pos_x && self.pos_y && self.pos_z
    }

    fn adjust_weights(&self, weights: &mut Weights, bound_penalty: impl Into<f64>) {
        let bound_penalty: f64 = bound_penalty.into();
        for (i, in_bounds_pos) in self.into_iter().enumerate() {
            if !in_bounds_pos {
                weights.pos_weights[i] += 1.0;
                weights.pos_weights[i] *= bound_penalty;
            }
        }
    }
}

impl IntoIterator for InPosBounds {
    type Item = bool;
    type IntoIter = std::array::IntoIter<bool, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [self.pos_x, self.pos_y, self.pos_z].into_iter()
    }
}

impl IntoIterator for &InPosBounds {
    type Item = bool;
    type IntoIter = std::array::IntoIter<bool, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [self.pos_x, self.pos_y, self.pos_z].into_iter()
    }
}

#[derive(Default)]
pub struct InAngleVelBounds {
    pub angle_vel_x: bool,
    pub angle_vel_y: bool,
    pub angle_vel_z: bool,
}
impl InAngleVelBounds {
    pub const fn new() -> Self {
        InAngleVelBounds {
            angle_vel_x: true,
            angle_vel_y: true,
            angle_vel_z: true,
        }
    }
    /// Takes in a mutable reference to Self and checks if the given angle velocities are within
    /// the specified angle velocity limits. The fields for the struct are then set accordingly.
    pub const fn evaluate_within_bounds(
        &mut self,
        angle_vel: [i16; 3],
        angle_vel_limits: &[(i16, i16); 3],
    ) {
        self.angle_vel_x =
            (angle_vel_limits[0].0 < angle_vel[0]) && (angle_vel[0] < angle_vel_limits[0].1);
        self.angle_vel_y =
            (angle_vel_limits[1].0 < angle_vel[1]) && (angle_vel[1] < angle_vel_limits[1].1);
        self.angle_vel_z =
            (angle_vel_limits[2].0 < angle_vel[2]) && (angle_vel[2] < angle_vel_limits[2].1);
    }
}
impl InBoundsTrait for InAngleVelBounds {
    fn check_if_all_true(&self) -> bool {
        self.angle_vel_x && self.angle_vel_y && self.angle_vel_z
    }

    fn adjust_weights(&self, weights: &mut Weights, bound_penalty: impl Into<f64>) {
        let bound_penalty: f64 = bound_penalty.into();
        for (i, in_bounds_pos) in self.into_iter().enumerate() {
            if !in_bounds_pos {
                weights.angle_vel_weights[i] += 1.0;
                weights.angle_vel_weights[i] *= bound_penalty;
            }
        }
    }
}

impl IntoIterator for InAngleVelBounds {
    type Item = bool;
    type IntoIter = std::array::IntoIter<bool, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [self.angle_vel_x, self.angle_vel_y, self.angle_vel_z].into_iter()
    }
}

impl IntoIterator for &InAngleVelBounds {
    type Item = bool;
    type IntoIter = std::array::IntoIter<bool, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [self.angle_vel_x, self.angle_vel_y, self.angle_vel_z].into_iter()
    }
}

#[derive(Default)]
pub struct InFaceAngleBounds {
    pub face_angle_x: bool,
    pub face_angle_y: bool,
    pub face_angle_z: bool,
}

impl InFaceAngleBounds {
    pub const fn new() -> Self {
        InFaceAngleBounds {
            face_angle_x: true,
            face_angle_y: true,
            face_angle_z: true,
        }
    }

    /// Takes in a mutable reference to Self and checks if the given face angles are within
    /// the specified face angle limits. The fields for the struct are then set accordingly.
    pub const fn evaluate_within_bounds(
        &mut self,
        angle: [i16; 3],
        face_angle_limits: [(i32, i32); 3],
    ) {
        self.face_angle_x = (face_angle_limits[0].0 < (angle[0] as i32))
            && ((angle[0] as i32) < face_angle_limits[0].1);

        self.face_angle_y = (face_angle_limits[1].0 < (angle[1] as u16) as i32)
            && (((angle[1] as u16) as i32) < face_angle_limits[1].1);

        self.face_angle_x = (face_angle_limits[2].0 < (angle[2] as i32))
            && ((angle[2] as i32) < face_angle_limits[2].1);
    }
}
impl InBoundsTrait for InFaceAngleBounds {
    fn check_if_all_true(&self) -> bool {
        self.face_angle_x && self.face_angle_y && self.face_angle_z
    }

    fn adjust_weights(&self, weights: &mut Weights, bound_penalty: impl Into<f64>) {
        let bound_penalty: f64 = bound_penalty.into();
        for (i, in_bounds_pos) in self.into_iter().enumerate() {
            if !in_bounds_pos {
                weights.face_angle_weights[i] += 1.0;
                weights.face_angle_weights[i] *= bound_penalty;
            }
        }
    }
}

impl IntoIterator for InFaceAngleBounds {
    type Item = bool;
    type IntoIter = std::array::IntoIter<bool, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [self.face_angle_x, self.face_angle_y, self.face_angle_z].into_iter()
    }
}

impl IntoIterator for &InFaceAngleBounds {
    type Item = bool;
    type IntoIter = std::array::IntoIter<bool, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [self.face_angle_x, self.face_angle_y, self.face_angle_z].into_iter()
    }
}
pub struct InHspdBounds {
    pub hspd: bool,
}

impl InHspdBounds {
    pub const fn new() -> Self {
        InHspdBounds { hspd: true }
    }

    /// Takes in a mutable reference to Self and checks if the given horizontal speed is within
    /// the specified forward velocity limits. The fields for the struct are then set accordingly.
    pub const fn evaluate_within_bounds(&mut self, hspd: f32, hspd_limits: (f32, f32)) {
        self.hspd = (hspd_limits.0 < hspd) && (hspd < hspd_limits.1);
    }
}

impl InBoundsTrait for InHspdBounds {
    fn check_if_all_true(&self) -> bool {
        self.hspd
    }

    fn adjust_weights(&self, weights: &mut Weights, bound_penalty: impl Into<f64>) {
        let bound_penalty: f64 = bound_penalty.into();
        if !self.hspd {
            weights.hspd_weight += 1.0;
            weights.hspd_weight *= bound_penalty;
        }
    }
}

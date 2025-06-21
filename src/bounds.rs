use crate::{
    ANGLE_VEL_LIMITS, ANGLE_VEL_WEIGHTS, FACE_ANGLE_LIMITS, FACE_ANGLE_WEIGHTS, HSPD_LIMITS,
    HSPD_WEIGHT, POS_LIMITS, POS_WEIGHTS,
};
use wafel_api::Game;

/// Contains the information about Mario which is used when checking for bounds. These fields
/// mirror the fields of the weights and bounds structs, and should be updated accordingly if
/// more fields are added to the bounds and weights structs.
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

pub struct Weights {
    pub pos_weights: [f64; 3],
    pub face_angle_weights: [f64; 3],
    pub angle_vel_weights: [f64; 3],
    pub hspd_weight: f64,
}
impl Weights {
    pub fn new() -> Self {
        Self {
            pos_weights: POS_WEIGHTS,
            face_angle_weights: FACE_ANGLE_WEIGHTS,
            angle_vel_weights: ANGLE_VEL_WEIGHTS,
            hspd_weight: HSPD_WEIGHT,
        }
    }
    /// Takes in an instance of IsInBounds and a mutable reference to Weights, and adjusts the weights
    /// based on the data given by in_bounds. This is used to penalize the score for not being within
    /// the specified bounds, without failing the score entirely.
    pub fn adjust_weights(&mut self, in_bounds: &IsInBounds) {
        // 0 for testing //check_limits(game);
        for (i, in_bounds_pos) in (&in_bounds.pos_limits).into_iter().enumerate() {
            if !in_bounds_pos {
                self.pos_weights[i] += 1.0;
                self.pos_weights[i] *= 10000.0;
            }
        }
        for (i, in_bounds_face_angle) in (&in_bounds.face_angle_limits).into_iter().enumerate() {
            if !in_bounds_face_angle {
                self.face_angle_weights[i] += 1.0;
                self.face_angle_weights[i] *= 10000.0;
            }
        }
        for (i, in_bounds_angle_vel) in (&in_bounds.angle_vel_limits).into_iter().enumerate() {
            if !in_bounds_angle_vel {
                self.angle_vel_weights[i] += 1.0;
                self.angle_vel_weights[i] *= 10000.0;
            }
        }
        if !in_bounds.hspd_limits.hspd {
            self.hspd_weight += 1.0;
            self.hspd_weight *= 1000.0;
        }
    }
}
pub struct Bounds {
    pub pos_limits: [(f32, f32); 3],
    pub face_angle_limits: [(i32, i32); 3],
    pub angle_vel_limits: [(i16, i16); 3],
    pub hspd_limits: (f32, f32),
}

impl Bounds {
    pub fn new() -> Self {
        Bounds {
            pos_limits: POS_LIMITS,
            face_angle_limits: FACE_ANGLE_LIMITS,
            angle_vel_limits: ANGLE_VEL_LIMITS,
            hspd_limits: HSPD_LIMITS,
        }
    }
}

pub struct IsInBounds {
    pos_limits: InPosBounds,
    angle_vel_limits: InAngleVelBounds,
    face_angle_limits: InFaceAngleBounds,
    hspd_limits: InHspdBounds,
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

    pub(crate) const fn update_from_mario_data(
        &mut self,
        mario_data: &CommonMarioData,
        bounds: &Bounds,
    ) {
        self.pos_limits
            .update_in_pos_bounds(mario_data.pos, bounds.pos_limits);
        self.face_angle_limits
            .update_in_face_angle_bounds(mario_data.face_angle, bounds.face_angle_limits);
        self.angle_vel_limits
            .update_in_angle_vel_bounds(mario_data.angle_vel, &bounds.angle_vel_limits);
        self.hspd_limits
            .update_in_hspd_bounds(mario_data.forward_vel, bounds.hspd_limits);
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
    pub(crate) const fn new_from_mario_data(mario_data: &CommonMarioData, bounds: &Bounds) -> Self {
        let mut is_in_bounds: Self = Self::new();
        is_in_bounds.update_from_mario_data(mario_data, bounds);
        is_in_bounds
    }
}

pub struct InPosBounds {
    pub pos_x: bool,
    pub pos_y: bool,
    pub pos_z: bool,
}

impl Default for InPosBounds {
    fn default() -> Self {
        Self::new()
    }
}

impl InPosBounds {
    pub const fn new() -> Self {
        InPosBounds {
            pos_x: true,
            pos_y: true,
            pos_z: true,
        }
    }

    /// Takes in a mutable reference to Self and checks if the given position is within
    /// the specified position limits. The fields for the struct are then set accordingly.
    pub const fn update_in_pos_bounds(&mut self, pos: [f32; 3], pos_limits: [(f32, f32); 3]) {
        if !((pos_limits[0].0 < pos[0]) && (pos[0] < pos_limits[0].1)) {
            self.pos_x = false;
        }
        if !((pos_limits[1].0 < pos[1]) && (pos[1] < pos_limits[1].1)) {
            self.pos_y = false;
        }
        if !((pos_limits[2].0 < pos[2]) && (pos[2] < pos_limits[2].1)) {
            self.pos_z = false;
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

pub struct InAngleVelBounds {
    pub angle_vel_x: bool,
    pub angle_vel_y: bool,
    pub angle_vel_z: bool,
}

impl Default for InAngleVelBounds {
    fn default() -> Self {
        Self::new()
    }
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
    pub const fn update_in_angle_vel_bounds(
        &mut self,
        angle_vel: [i16; 3],
        angle_vel_limits: &[(i16, i16); 3],
    ) {
        if !((angle_vel_limits[0].0 < angle_vel[0]) && (angle_vel[0] < angle_vel_limits[0].1)) {
            self.angle_vel_x = false;
        }
        if !((angle_vel_limits[1].0 < angle_vel[1]) && (angle_vel[1] < angle_vel_limits[1].1)) {
            self.angle_vel_y = false;
        }
        if !((angle_vel_limits[2].0 < angle_vel[2]) && (angle_vel[2] < angle_vel_limits[2].1)) {
            self.angle_vel_z = false;
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

pub struct InFaceAngleBounds {
    pub face_angle_x: bool,
    pub face_angle_y: bool,
    pub face_angle_z: bool,
}

impl Default for InFaceAngleBounds {
    fn default() -> Self {
        Self::new()
    }
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
    pub const fn update_in_face_angle_bounds(
        &mut self,
        angle: [i16; 3],
        face_angle_limits: [(i32, i32); 3],
    ) {
        if !((face_angle_limits[0].0 < (angle[0] as i32))
            && ((angle[0] as i32) < face_angle_limits[0].1))
        {
            self.face_angle_x = false;
        }
        if !((face_angle_limits[1].0 < ((angle[1] as u16) as i32))
            && (((angle[1] as u16) as i32) < face_angle_limits[1].1))
        {
            self.face_angle_y = false;
        }
        if !((face_angle_limits[2].0 < (angle[2] as i32))
            && ((angle[2] as i32) < face_angle_limits[2].1))
        {
            self.face_angle_z = false;
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
    pub const fn update_in_hspd_bounds(&mut self, hspd: f32, hspd_limits: (f32, f32)) {
        if !((hspd_limits.0 < hspd) && (hspd < hspd_limits.1)) {
            self.hspd = false;
        }
    }
}

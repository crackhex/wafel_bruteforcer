use crate::{
    ANGLE_VEL_LIMITS, ANGLE_VEL_WEIGHTS, FACE_ANGLE_LIMITS, FACE_ANGLE_WEIGHTS, HSPD_LIMITS,
    HSPD_WEIGHT, POS_LIMITS, POS_WEIGHTS,
};
use wafel_api::Game;

pub struct CommonMarioData {
    pub pos: [f32; 3],
    pub face_angle: [i16; 3],
    pub angle_vel: [i16; 3],
    pub forward_vel: f32,
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
    pub fn new() -> Self {
        Self {
            pos_limits: InPosBounds::new(),
            angle_vel_limits: InAngleVelBounds::new(),
            face_angle_limits: InFaceAngleBounds::new(),
            hspd_limits: InHspdBounds::new(),
        }
    }

    pub fn check_all_limits(&mut self, mario_data: &CommonMarioData, bounds: &Bounds) {
        self.pos_limits
            .check_pos_limits(mario_data.pos, bounds.pos_limits);
        self.face_angle_limits
            .check_face_angle_limits(mario_data.face_angle, bounds.face_angle_limits);
        self.angle_vel_limits
            .check_angle_vel_limits(mario_data.angle_vel, &bounds.angle_vel_limits);
        self.hspd_limits
            .check_hspd_limits(mario_data.forward_vel, bounds.hspd_limits);
    }

    /// Takes in Game reference and Bounds reference, determines if Mario is in bounds, and
    /// returns an Owned instance of IsInBounds containing the results
    pub fn check_bounds_from_game(game: &Game, bounds: &Bounds) -> Self {
        let pos = game.read("gMarioState.pos").as_f32_3();
        let angle = game.read("gMarioState.faceAngle").as_i16_3();
        let angle_vel = game.read("gMarioState.angleVel").as_i16_3();
        let hspd = game.read("gMarioState.forwardVel").as_f32();
        IsInBounds::create_in_bounds(pos, angle, angle_vel, hspd, bounds)
    }

    /// Checks if the given position, angles, angle velocities, and horizontal speed are within
    /// the specified bounds, and returns an instance of IsInBounds containing the results.
    fn create_in_bounds(
        pos: [f32; 3],
        angle: [i16; 3],
        angle_vel: [i16; 3],
        hspd: f32,
        bounds: &Bounds,
    ) -> Self {
        let mut is_in_bounds: Self = Self::new();
        is_in_bounds.check_all_limits(
            &CommonMarioData {
                pos,
                face_angle: angle,
                angle_vel,
                forward_vel: hspd,
            },
            bounds,
        );
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
    pub fn new() -> Self {
        InPosBounds {
            pos_x: true,
            pos_y: true,
            pos_z: true,
        }
    }

    pub fn check_pos_limits(&mut self, pos: [f32; 3], pos_limits: [(f32, f32); 3]) {
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

impl<'a> IntoIterator for &'a InPosBounds {
    type Item = &'a bool;
    type IntoIter = std::array::IntoIter<&'a bool, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [&self.pos_x, &self.pos_y, &self.pos_z].into_iter()
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
    pub fn new() -> Self {
        InAngleVelBounds {
            angle_vel_x: true,
            angle_vel_y: true,
            angle_vel_z: true,
        }
    }
    pub fn check_angle_vel_limits(
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

impl<'a> IntoIterator for &'a InAngleVelBounds {
    type Item = &'a bool;
    type IntoIter = std::array::IntoIter<&'a bool, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [&self.angle_vel_x, &self.angle_vel_y, &self.angle_vel_z].into_iter()
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
    pub fn new() -> Self {
        InFaceAngleBounds {
            face_angle_x: true,
            face_angle_y: true,
            face_angle_z: true,
        }
    }

    pub fn check_face_angle_limits(&mut self, angle: [i16; 3], face_angle_limits: [(i32, i32); 3]) {
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

impl<'a> IntoIterator for &'a InFaceAngleBounds {
    type Item = &'a bool;
    type IntoIter = std::array::IntoIter<&'a bool, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [&self.face_angle_x, &self.face_angle_y, &self.face_angle_z].into_iter()
    }
}
pub struct InHspdBounds {
    pub hspd: bool,
}

impl InHspdBounds {
    pub fn new() -> Self {
        InHspdBounds { hspd: true }
    }

    pub fn check_hspd_limits(&mut self, hspd: f32, hspd_limits: (f32, f32)) {
        if !((hspd_limits.0 < hspd) && (hspd < hspd_limits.1)) {
            self.hspd = false;
        }
    }
}

pub fn adjust_weights(in_bounds: &IsInBounds, weights: &mut Weights) {
    // 0 for testing //check_limits(game);
    for (i, in_bounds_pos) in (&in_bounds.pos_limits).into_iter().enumerate() {
        if !in_bounds_pos {
            weights.pos_weights[i] += 1.0;
            weights.pos_weights[i] *= 10000.0;
        }
    }
    for (i, in_bounds_face_angle) in (&in_bounds.face_angle_limits).into_iter().enumerate() {
        if !in_bounds_face_angle {
            weights.face_angle_weights[i] += 1.0;
            weights.face_angle_weights[i] *= 10000.0;
        }
    }
    for (i, in_bounds_angle_vel) in (&in_bounds.angle_vel_limits).into_iter().enumerate() {
        if !in_bounds_angle_vel {
            weights.angle_vel_weights[i] += 1.0;
            weights.angle_vel_weights[i] *= 10000.0;
        }
    }
    if !in_bounds.hspd_limits.hspd {
        weights.hspd_weight += 1.0;
        weights.hspd_weight *= 1000.0;
    }
}

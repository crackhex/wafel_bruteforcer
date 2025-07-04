use crate::bounds::{InBoundsTrait, IsInBounds};

/// Configuration for the bruteforcer, containing all the necessary parameters to run the bruteforce
pub(crate) struct BruteforceConfig {
    pub start_frame: u32,
    pub end_frame: u32,
    pub perm_freq: f32,
    pub perm_size: u8,
    pub wafel_path: &'static str,
    pub version: &'static str,
    pub output_name: &'static str,
    pub thread_num: u16,
    pub bound_penalty: f64,
    pub bound_correction: bool,
}
impl BruteforceConfig {
    pub fn new(
        start_frame: u32,
        end_frame: u32,
        perm_freq: f32,
        perm_size: u8,
        wafel_path: &'static str,
        version: &'static str,
        output_name: &'static str,
        thread_num: u16,
        bound_penalty: f64,
        bound_correction: bool,
    ) -> Self {
        Self {
            start_frame,
            end_frame,
            perm_freq,
            perm_size,
            wafel_path,
            version,
            output_name,
            thread_num,
            bound_penalty,
            bound_correction,
        }
    }
}
pub struct Target {
    pub pos: [f32; 3],
    pub face_angle: [i32; 3],
    pub angle_vel: [i16; 3],
    pub hspd: f32,
    pub coins: u16,
}

impl Target {
    pub fn new(
        pos: [f32; 3],
        face_angle: [i32; 3],
        angle_vel: [i16; 3],
        hspd: f32,
        coins: u16,
    ) -> Self {
        Self {
            pos,
            face_angle,
            angle_vel,
            hspd,
            coins,
        }
    }
}
#[derive(Clone)]
pub struct Weights {
    pub pos_weights: [f64; 3],
    pub face_angle_weights: [f64; 3],
    pub angle_vel_weights: [f64; 3],
    pub hspd_weight: f64,
}
impl Weights {
    pub fn new(
        pos_weights: [f64; 3],
        face_angle_weights: [f64; 3],
        angle_vel_weights: [f64; 3],
        hspd_weight: f64,
    ) -> Self {
        Self {
            pos_weights,
            face_angle_weights,
            angle_vel_weights,
            hspd_weight,
        }
    }
    /// Takes in an instance of IsInBounds and a mutable reference to Weights, and adjusts the weights
    /// based on the data given by in_bounds. This is used to penalize the score for not being within
    /// the specified bounds, without failing the score entirely.
    pub fn penalise_bounds(&mut self, in_bounds: &IsInBounds, bound_penalty: f64) {
        // 0 for testing //check_limits(game);
        in_bounds.pos_limits.adjust_weights(self, bound_penalty);
        in_bounds
            .face_angle_limits
            .adjust_weights(self, bound_penalty);
        in_bounds
            .angle_vel_limits
            .adjust_weights(self, bound_penalty);
        in_bounds.hspd_limits.adjust_weights(self, bound_penalty);
    }
}

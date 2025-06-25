use crate::WAFEL_PATH;
use crate::bounds::{Bounds, CommonMarioData, IsInBounds};
use crate::bruteforce_params::{BruteforceConfig, Target, Weights};
use crate::{
    ANGLE_VEL_LIMITS,
    FACE_ANGLE_LIMITS, GAME_CREATION_LOCK, HSPD_LIMITS,
    M64File, POS_LIMITS,
};
use crate::{NUM_THREADS, VERSION};
use rand::random_range;
use std::fs::copy;
use std::path::Path;
use wafel_api::Value;
use wafel_api::{Game, save_m64};
use wafel_api::{Input, SaveState};
// Spawn more dlls

pub fn spawn_dlls() {
    println!("spawning DLLs...");
    let path = format!("{WAFEL_PATH}libsm64\\sm64_{VERSION}.dll");
    //let m64_path = &*format!("{INP_NAME}.m64");
    for i in 0..NUM_THREADS - 1 {
        let path_copy = format!("{WAFEL_PATH}libsm64\\sm64_{VERSION}{i}.dll");
        // let m64_copy = &*format!("{INP_NAME}.{i}.m64");
        if !Path::new(&path_copy).is_file() {
            println!("{i}");
            copy(&path, path_copy).unwrap();
        }
        //if !Path::new(m64_copy).is_file() {
        // copy(m64_path, m64_copy).unwrap();
        // }
    }
}

// fast forwards to start frame
pub fn set_inputs(game: &mut Game, input: &Input) {
    game.write(
        "gControllerPads[0].stick_x",
        Value::Int(input.stick_x.into()),
    );
    game.write(
        "gControllerPads[0].stick_y",
        Value::Int(input.stick_y.into()),
    );
    game.write(
        "gControllerPads[0].button",
        Value::Int(input.buttons.into()),
    );
}
pub fn calculate_score_bound_correction(
    bound_correction: bool,
    mario_data: &CommonMarioData,
    weights: &Weights,
    target: &Target,
    in_bounds: &mut IsInBounds,
) -> f64 {
    let mut result: f64 = f64::INFINITY;
    if bound_correction {
        let mut new_weights = weights.clone();
        new_weights.penalise_bounds(in_bounds);
        result = calculate_score(mario_data, &new_weights, target);
    } else if in_bounds.check_if_all_true() {
        result = calculate_score(mario_data, weights, target);
    }
    result
}

// Checking if mario falls within the limits set
pub fn calculate_score(mario_data: &CommonMarioData, weights: &Weights, target: &Target) -> f64 {
    let mut result: f64 = 0.0;
    for i in 0..3 {
        result += (target.pos[i] - mario_data.pos[i]).abs() as f64 * weights.pos_weights[i];
        result += (target.angle_vel[i] - (mario_data.angle_vel[i])).abs() as f64
            * weights.angle_vel_weights[i];
    }
    result += (target.hspd - mario_data.forward_vel).abs() as f64 * weights.hspd_weight;
    result += (target.face_angle[0] as f64 - mario_data.face_angle[0] as f64).abs()
        * weights.face_angle_weights[0];
    result += (target.face_angle[1] - (mario_data.face_angle[1] as u16) as i32).abs() as f64
        * weights.face_angle_weights[1];
    result += (target.face_angle[2] as f64 - mario_data.face_angle[2] as f64).abs()
        * weights.face_angle_weights[2];
    result
}

// Adjust weights for fitness calculations and run
pub fn bruteforce_main(
    m64: M64File,
    weights: Weights,
    target: Target,
    brute_config: BruteforceConfig,
) {
    // LOCK: Only protect this block
    // Create a new game instance with the DLL for this thread
    let mut game = {
        let _lock = GAME_CREATION_LOCK.lock().unwrap();
        unsafe {
            Game::new(&format!(
                "{}libsm64\\sm64_{}_{}.dll",
                brute_config.wafel_path, brute_config.version, brute_config.thread_num
            ))
        }
    };

    println!("Thread {}: Created game instance", brute_config.thread_num);
    let mut start_st = game.save_state();

    for frame in 0..brute_config.end_frame + 1 {
        set_inputs(&mut game, &m64.1[frame as usize]);
        game.advance();
        if frame == brute_config.start_frame - 1 {
            // Save the state at the start frame for bruteforcing
            start_st = game.save_state();
        }
    }

    let mario_data = CommonMarioData::new_from_game(&game);
    let bounds = Bounds::new(POS_LIMITS, FACE_ANGLE_LIMITS, ANGLE_VEL_LIMITS, HSPD_LIMITS);
    let mut in_bounds = IsInBounds::new_from_mario_data(&mario_data, &bounds);
    let result = calculate_score_bound_correction(
        brute_config.bound_correction,
        &mario_data,
        &weights,
        &target,
        &mut in_bounds,
    );
    println!(
        "Thread {}: Initial score: {}, at frame {}",
        brute_config.thread_num, result, brute_config.end_frame
    );
    println!(
        "Position: {:?}, Face Angle: {:?}, Angle Vel: {:?}, Forward Vel: {}",
        mario_data.pos, mario_data.face_angle, mario_data.angle_vel, mario_data.forward_vel
    );
    bruteforce_loop(
        m64,
        game,
        start_st,
        in_bounds,
        bounds,
        weights,
        target,
        result,
        brute_config,
    );
}

fn bruteforce_loop(
    mut m64: M64File,
    mut game: Game,
    start_st: SaveState,
    mut in_bounds: IsInBounds,
    bounds: Bounds,
    weights: Weights,
    target: Target,
    mut result: f64,
    brute_config: BruteforceConfig,
) -> f64 {
    let mut count = 0;
    loop {
        game.load_state(&start_st);

        // Perturb the inputs for the current frame
        let mut m64_perturb: M64File = m64.clone();
        perturb_inputs_simple(
            &mut m64_perturb.1,
            brute_config.start_frame,
            brute_config.end_frame + 1,
            brute_config.perm_size,
            brute_config.perm_freq,
        );
        // Set the perturbed inputs and advance the game
        for frame in brute_config.start_frame..brute_config.end_frame + 1 {
            set_inputs(&mut game, &m64_perturb.1[frame as usize]);
            game.advance();
        }
        let mario_data = CommonMarioData::new_from_game(&game);
        in_bounds.update_from_mario_data(&mario_data, &bounds);
        // todo: Pull into a function
        let new_score = calculate_score_bound_correction(
            brute_config.bound_correction,
            &mario_data,
            &weights,
            &target,
            &mut in_bounds,
        );
        if new_score < result {
            // If the new score is better, update the inputs and result
            result = new_score;
            println!(
                "Thread {}: New best score: {}, at frame {}",
                brute_config.thread_num, result, brute_config.end_frame
            );
            println!(
                "Position: {:?}, Face Angle: {:?}, Angle Vel: {:?}, Forward Vel: {}",
                mario_data.pos,
                mario_data.face_angle[1] as u16,
                mario_data.angle_vel,
                mario_data.forward_vel
            );
            m64 = m64_perturb;
        }
        count += 1;
        if count % 10000 == 0 {
            save_m64(brute_config.output_name, &m64.0, &m64.1);
            println!(
                "Thread {}: Saved m64 after {count} iterations",
                brute_config.thread_num
            );
        }
    }
}

fn perturb_inputs_simple(
    inputs: &mut [Input],
    start_frame: u32,
    end_frame: u32,
    perm_size: u8,
    perm_freq: f32,
) {
    for frame in start_frame..end_frame {
        let input = &mut inputs[frame as usize];
        let random_f32 = random_range(0.0f32..1.0f32);
        if random_f32 > perm_freq {
            continue;
        }
        let rand_x = random_range(-(perm_size as i8)..(perm_size as i8));
        let rand_y = random_range(-(perm_size as i8)..(perm_size as i8));
        // Ensure stick positions don't overflow
        input.stick_x = input.stick_x.saturating_add(rand_x);
        input.stick_y = input.stick_y.saturating_add(rand_y);
    }
}

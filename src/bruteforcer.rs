use crate::bounds::{Bounds, IsInBounds, Weights, adjust_weights};
use crate::{ANGLE_VEL_WEIGHTS, DES_ANGLE_VEL, DES_FACE_ANGLE, DES_HSPD, DES_POS, FACE_ANGLE_WEIGHTS, GAME_CREATION_LOCK, HSPD_WEIGHT, INF, M64File, POS_WEIGHTS, OUT_NAME};
use crate::{BOUND_CORRECTION, END_FRAME, PERM_FREQ, PERM_SIZE, START_FRAME, WAFEL_PATH};
use crate::{NUM_THREADS, VERSION};
use rand::random_range;
use std::fs::copy;
use std::path::Path;
use wafel_api::{save_m64, Game};
use wafel_api::Input;
use wafel_api::Value;
// Spawn more dlls

pub fn spawn_dlls() {
    println!("spawning DLLs...");
    let path = &*format!("{WAFEL_PATH}libsm64\\sm64_{VERSION}.dll");
    //let m64_path = &*format!("{INP_NAME}.m64");
    for i in 0..NUM_THREADS - 1 {
        let path_copy = &*format!("{WAFEL_PATH}libsm64\\sm64_{VERSION}{i}.dll");
        // let m64_copy = &*format!("{INP_NAME}.{i}.m64");
        if !Path::new(path_copy).is_file() {
            println!("{i}");
            copy(path, path_copy).unwrap();
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

// Checking if mario falls within the limits set
pub fn calculate_score(game: &Game, weights: &Weights) -> f64 {
    let mut result: f64 = 0.0;
    result = 0.0;
    for i in 0..3 {
        result += (DES_POS[i] - game.read("gMarioState.pos").as_f32_3()[i]).abs() as f64
            * weights.pos_weights[i];
        result += (DES_ANGLE_VEL[i] - (game.read("gMarioState.angleVel").as_i16_3()[i])).abs()
            as f64
            * weights.angle_vel_weights[i];
    }
    result += (DES_HSPD - game.read("gMarioState.forwardVel").as_f32()).abs() as f64
        * weights.hspd_weight;
    result += (DES_FACE_ANGLE[0] as f64
        - (game.read("gMarioState.faceAngle").as_i16_3()[0] as f64))
        .abs()
        * weights.face_angle_weights[0];
    result += (DES_FACE_ANGLE[1] - (game.read("gMarioState.faceAngle").as_i16_3()[1] as u16) as i32)
        .abs() as f64
        * weights.face_angle_weights[1];
    result += (DES_FACE_ANGLE[2] as f64
        - (game.read("gMarioState.faceAngle").as_i16_3()[2] as f64))
        .abs()
        * weights.face_angle_weights[2];
    result
}

// Adjust weights for fitness calculations and run
pub fn bruteforce_loop(m64: &mut M64File, thread_num: u16) {
    // LOCK: Only protect this block
    // Create a new game instance with the DLL for this thread
    let mut game = {
        let _lock = GAME_CREATION_LOCK.lock().unwrap();
        unsafe {
            Game::new(&format!(
                "{WAFEL_PATH}libsm64\\sm64_{VERSION}_{thread_num}.dll"
            ))
        }
    };
    println!("Thread {}: Created game instance", thread_num);
    let mut start_st = game.save_state();
    for frame in 0..END_FRAME {
        set_inputs(&mut game, &m64.1[frame as usize]);
        game.advance();
        if frame == START_FRAME {
            // Save the state at the start frame for bruteforcing
            start_st = game.save_state();
        }
    }
    let pos = game.read("gMarioState.pos").as_f32_3();
    let angle = game.read("gMarioState.faceAngle").as_i16_3();

    let mut weights = Weights {
        pos_weights: POS_WEIGHTS,
        face_angle_weights: FACE_ANGLE_WEIGHTS,
        angle_vel_weights: ANGLE_VEL_WEIGHTS,
        hspd_weight: HSPD_WEIGHT,
    };
    if BOUND_CORRECTION {
        let bounds = Bounds::new();
        let in_bounds = IsInBounds::check_bounds_from_game(&game, &bounds);
        adjust_weights(&game, &in_bounds, &mut weights);
    }

    let mut result: f64 = calculate_score(&game, &weights);
    println!(
        "Thread {}: Initial score: {}, at frame {}",
        thread_num, result, END_FRAME
    );
    let mut count = 0;
    loop {
        game.load_state(&start_st);

        // Perturb the inputs for the current frame
        let mut m64_perturb: M64File = m64.clone();
        perturb_inputs_simple(
            &mut m64_perturb.1,
            START_FRAME,
            END_FRAME,
            PERM_SIZE,
            PERM_FREQ,
        );
        // Set the perturbed inputs and advance the game
        for frame in START_FRAME..END_FRAME {
            set_inputs(&mut game, &m64_perturb.1[frame as usize]);
            game.advance();
        }
        if BOUND_CORRECTION {
            weights = Weights::new();
            let bounds = Bounds::new();
            let in_bounds = IsInBounds::check_bounds_from_game(&game, &bounds);
            adjust_weights(&game, &in_bounds, &mut weights);
        }
        let new_score = calculate_score(&game, &weights);
        if new_score < result {
            // If the new score is better, update the inputs and result
            result = new_score;
            println!(
                "Thread {}: New best score: {}, at frame {}",
                thread_num, result, END_FRAME
            );
            println!(
                "Position: {:?}, Face Angle: {:?}, Angle Vel: {:?}, Forward Vel: {}",
                game.read("gMarioState.pos").as_f32_3(),
                game.read("gMarioState.faceAngle").as_i16_3(),
                game.read("gMarioState.angleVel").as_i16_3(),
                game.read("gMarioState.forwardVel").as_f32()
            );
            *m64 = m64_perturb;
        }
        if count % 100000 == 0 {
            save_m64(OUT_NAME, &m64.0, &m64.1)
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

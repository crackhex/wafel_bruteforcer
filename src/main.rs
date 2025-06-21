mod bounds;
mod bruteforcer;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use wafel_api::Input;
use wafel_api::M64Metadata;
use wafel_api::load_m64;
// Number of cores used
pub const INF: f64 = 1.0 / 0.0;
pub const INF32: f32 = 1.0 / 0.0;
type M64File = (M64Metadata, Vec<Input>);
const NUM_THREADS: u16 = 4;

// Wafel path and game version
// Make sure double backslashes are useds
const WAFEL_PATH: &str = "D:\\PATH\\TO\\WAFEL\\";
const VERSION: &str = "us";

// Info of target m64
const ZERO_INDEX_FRAMES: bool = false;
const START_FRAME: u32 = 4801;
const END_FRAME: u32 = 4831;
const MOVIE_LENGTH: u32 = END_FRAME - START_FRAME;
const INP_NAME: &str = "C:\\Users\\PATH\\TO\\INP.m64";
const INP_NAME2: &str = "C:\\Users\\PATH\\TO\\INP2.m64";
const OUT_NAME: &str = "C:\\Users\\PATH\\TO\\INP.m64";
// Basis m64 info (for state loaded conversion)
// will do later
const STATE_LOADED: bool = false;
const OFFSET: i32 = 0;
const BASIS_NAME: &str = ".m64";

// Bound correction toggle
// Makes bruteforcer try matching bounds when not originally within it
const BOUND_CORRECTION: bool = true;

// Permutation settings and such
// Will probably implement a better algorithm later but this is fine for now
const PERM_FREQ: f32 = 0.1; // 0 to 1
const PERM_SIZE: u8 = 10; // 0 to 255

// Desired targets for each variable
const DES_POS: [f32; 3] = [100.0, 200.0, 300.0]; // x, y, z
const DES_FACE_ANGLE: [i32; 3] = [32, 32, 32]; // Pitch, Yaw, Roll
const DES_ANGLE_VEL: [i16; 3] = [32, 32, 32]; // Pitchvel, Yawvel, Rollvel
const DES_HSPD: f32 = 48.0; // Forward Speed

// Limits to variables
const POS_LIMITS: [(f32, f32); 3] = [(-INF32, INF32), (-INF32, INF32), (-INF32, INF32)]; // Set to None
const FACE_ANGLE_LIMITS: [(i32, i32); 3] = [(-32768, 32767), (0, 65535), (-32768, 32767)];
const ANGLE_VEL_LIMITS: [(i16, i16); 3] = [(-32768, 32767), (-32768, 32767), (-32768, 32767)];
const HSPD_LIMITS: (f32, f32) = (34.0, 1000.0);
const COIN_LIMIT: i32 = 100;

// Weights of variables for fitness function
const POS_WEIGHTS: [f64; 3] = [10.0, 10.0, 10.0]; // Weights for x, y, z for result
const FACE_ANGLE_WEIGHTS: [f64; 3] = [0.0, 10.0, 0.0];
const ANGLE_VEL_WEIGHTS: [f64; 3] = [0.0, 0.0, 0.0];
const HSPD_WEIGHT: f64 = 10.0;

const OBJID_MEMORY_ADDR: i64 = 0;
const OBJID_WAFEL_SLOT: i64 = 1;
const OBJID_STROOP_SLOT: i64 = 2;
static GAME_CREATION_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub struct AtomicF64 {
    storage: AtomicU64,
}
impl AtomicF64 {
    pub fn new(value: f64) -> Self {
        let as_u64 = value.to_bits();
        Self {
            storage: AtomicU64::new(as_u64),
        }
    }
    pub fn store(&self, value: f64, ordering: Ordering) {
        let as_u64 = value.to_bits();
        self.storage.store(as_u64, ordering)
    }
    pub fn load(&self, ordering: Ordering) -> f64 {
        let as_u64 = self.storage.load(ordering);
        f64::from_bits(as_u64)
    }
}
pub fn main() {
    //env::set_var("RUST_BACKTRACE", "full");

    // Number of threads for multiprocessing

    let mut m64: M64File = load_m64(INP_NAME);
    //let mut handles = vec![];
    //spawn_dlls();

    let mut global_score = AtomicF32::new(0.0);
    for i in 0..NUM_THREADS {
        let mut m64_clones = m64.clone();
        /*let handle = std::thread::spawn(move || {});
        handles.push(handle);*/
    }
    bruteforcer::bruteforce_loop(&mut m64, 1);

    /*for handle in handles {
        handle.join().unwrap();
    }*/
    println!("{NUM_THREADS}");
    println!("{MOVIE_LENGTH}");
    println!("{WAFEL_PATH}");
}

use rand::{distr::uniform::SampleUniform, prelude::*};
use std::sync::{Mutex, OnceLock};

pub fn random_range<T: PartialOrd + SampleUniform>(range: std::ops::Range<T>) -> T {
    static RNG: OnceLock<Mutex<StdRng>> = OnceLock::new();
    if RNG.get().is_none() {
        RNG.set(Mutex::new(StdRng::from_os_rng())).unwrap();
    }
    RNG.get().unwrap().lock().unwrap().random_range(range)
}

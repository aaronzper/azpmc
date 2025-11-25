use std::hash::{DefaultHasher, Hash, Hasher};
use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng, rngs::StdRng};
use crate::{settings::SEED, world::Coordinate};

const FREQ: f64 = 0.01;

pub fn sample_elevation(x: Coordinate, y: Coordinate) -> usize {
    let sampler = Perlin::new(SEED);

    let f_x = x as f64 * FREQ;
    let f_y = y as f64 * FREQ;

    let elev = 70.0 + 
        sampler.get([f_x * 0.5, f_y * 0.5, 2.0]) * 30.0 +
        sampler.get([f_x, f_y, 0.0]) * 20.0 +
        sampler.get([f_x * 5.0, f_y * 5.0, 1.0]) * 5.0;

    elev.round() as usize
}

pub fn sample_tree(x: Coordinate, y: Coordinate) -> bool {
    let mut hasher = DefaultHasher::new();
    SEED.hash(&mut hasher);
    x.hash(&mut hasher);
    y.hash(&mut hasher);

    let mut rng = StdRng::seed_from_u64(hasher.finish());
    rng.random::<f32>() < 0.008
}

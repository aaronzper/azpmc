use noise::{NoiseFn, Perlin};
use crate::{settings::SEED, world::Coordinate};

const FREQ: f64 = 0.01;

pub fn sample_elevation(x: Coordinate, y: Coordinate) -> usize {
    let sampler = Perlin::new(SEED);

    let f_x = x as f64 * FREQ;
    let f_y = y as f64 * FREQ;

    let mut elev = 70.0 + 
        sampler.get([f_x, f_y, 0.0]) * 20.0 +
        sampler.get([f_x * 5.0, f_y * 5.0, 1.0]) * 5.0;

    if x % 5 == 0 && y % 11 == 0 {
        elev += 4.0;
    }

    elev.round() as usize
}

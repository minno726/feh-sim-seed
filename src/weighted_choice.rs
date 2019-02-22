use rand::{distributions::Distribution, Rng};

// Optimized version of rand::WeightedIndex for a fixed-size collection of floats.
#[derive(Copy, Clone, Debug, Default)]
pub struct WeightedIndex4 {
    // Cumulative weights stored for faster lookup
    values: [f32; 4],
}

impl WeightedIndex4 {
    pub fn new<T: Into<f32> + Copy>(values: [T; 4]) -> Self {
        let total = values[0].into() + values[1].into() + values[2].into() + values[3].into();
        Self {
            values: [
                values[0].into() / total,
                (values[0].into() + values[1].into()) / total,
                (values[0].into() + values[1].into() + values[2].into()) / total,
                1.0,
            ],
        }
    }
}

impl Distribution<usize> for WeightedIndex4 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let choice = rng.gen::<f32>();
        if choice > self.values[1] {
            if choice > self.values[2] {
                3
            } else {
                2
            }
        } else {
            if choice > self.values[0] {
                1
            } else {
                0
            }
        }
    }
}

use rand::{distributions::Distribution, Rng};

/// Optimized version of rand::WeightedIndex for a fixed-size collection of four floats.
#[derive(Copy, Clone, Debug, Default)]
pub struct WeightedIndex4 {
    // Cumulative weights stored for faster lookup
    values: [f32; 4],
}

impl WeightedIndex4 {
    /// Constructs a sampler from the given weights. Weights do not need to sum to 1.
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

/// Optimized version of rand::WeightedIndex for a fixed-size collection of five floats.
#[derive(Copy, Clone, Debug, Default)]
pub struct WeightedIndex5 {
    // Cumulative weights stored for faster lookup
    values: [f32; 5],
}

impl WeightedIndex5 {
    /// Constructs a sampler from the given weights. Weights do not need to sum to 1.
    pub fn new<T: Into<f32> + Copy>(values: [T; 5]) -> Self {
        let total = values[0].into()
            + values[1].into()
            + values[2].into()
            + values[3].into()
            + values[4].into();
        Self {
            values: [
                values[0].into() / total,
                (values[0].into() + values[1].into()) / total,
                (values[0].into() + values[1].into() + values[2].into()) / total,
                (values[0].into() + values[1].into() + values[2].into() + values[3].into()) / total,
                1.0,
            ],
        }
    }
}

impl Distribution<usize> for WeightedIndex5 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let choice = rng.gen::<f32>();
        if choice > self.values[1] {
            if choice > self.values[2] {
                if choice > self.values[3] {
                    4
                } else {
                    3
                }
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

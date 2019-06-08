use std::ops::{Deref, DerefMut, Index, IndexMut};

/// Associative array of u32 -> u32 with the interface and implementation optimized
/// for use as a counter for small numbers with a dense distribution.
#[derive(Default, Debug, Clone)]
pub struct Counter {
    data: Vec<u32>,
}

impl Index<u32> for Counter {
    type Output = u32;

    /// Infallible. Returns 0 if index is out of range.
    fn index(&self, index: u32) -> &Self::Output {
        self.data.get(index as usize).unwrap_or(&0)
    }
}

impl IndexMut<u32> for Counter {
    /// Infallible. Resizes container if index is out of range.
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let index = index as usize;
        if index >= self.data.len() {
            self.data.resize(index + 1, 0);
        }
        &mut self.data[index]
    }
}

impl Deref for Counter {
    type Target = Vec<u32>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Counter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

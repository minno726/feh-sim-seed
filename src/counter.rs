use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Default, Debug, Clone)]
pub struct Counter {
    data: Vec<u32>,
}

impl Index<u32> for Counter {
    type Output = u32;

    fn index(&self, index: u32) -> &Self::Output {
        self.data.get(index as usize).unwrap_or(&0)
    }
}

impl IndexMut<u32> for Counter {
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

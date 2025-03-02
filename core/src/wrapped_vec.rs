use std::usize;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct WrappingVec<T, const N: usize> {
    vec: Vec<T>,
    top: usize,
}

impl<T, const N: usize> Default for WrappingVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> WrappingVec<T, N> {
    pub fn new() -> Self {
        Self {
            vec: Vec::with_capacity(N),
            top: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.vec.len() < N {
            self.vec.push(value);
        } else {
            self.vec[self.top] = value;
        }

        self.top = if self.top + 1 >= N { 0 } else { self.top + 1 };
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
        let start_index = if self.vec.len() == N { self.top } else { 0 };

        (0..self.vec.len()).map(move |i| {
            let index = (start_index + i) % N;
            &self.vec[index]
        })
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
        self.top = 0;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_push() {
        let mut vec = WrappingVec::<i32, 3>::new();

        // Push below capacity
        vec.push(0);
        vec.push(1);
        assert_eq!(vec.iter().collect::<Vec<_>>(), vec![&0, &1]);

        // Push to capacity
        vec.push(2);
        assert_eq!(vec.iter().collect::<Vec<_>>(), vec![&0, &1, &2]);

        // Push beyond capacity
        vec.push(3);
        assert_eq!(vec.iter().collect::<Vec<_>>(), vec![&1, &2, &3]);

        // Push beyond capacity again
        vec.push(4);
        assert_eq!(vec.iter().collect::<Vec<_>>(), vec![&2, &3, &4]);
    }
}

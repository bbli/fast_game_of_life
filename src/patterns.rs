use super::*;
use std::ops::{Deref, DerefMut};
// no need since we "inherit" parent's uses
//use super::b_matrix_vector::BMatrixVector;
use rand::prelude::*;

pub struct PatternBuilder {
    vec: BMatrixVector,
}

impl Deref for PatternBuilder {
    type Target = BMatrixVector;
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl DerefMut for PatternBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl PatternBuilder {
    pub fn new() -> Self {
        let vec = BMatrixVector::default();
        PatternBuilder { vec }
    }
    pub fn build(self) -> BMatrixVector {
        self.vec
    }
    pub fn make_square(mut self, init_x: i32, init_y: i32) -> Self {
        *self.at_mut(init_x, init_y).unwrap() = true;
        *self.at_mut(init_x + 1, init_y).unwrap() = true;
        *self.at_mut(init_x + 1, init_y + 1).unwrap() = true;
        *self.at_mut(init_x, init_y + 1).unwrap() = true;

        self
    }
    pub fn make_blinker(mut self, init_x: i32, init_y: i32) -> Self {
        *self.at_mut(init_x, init_y).unwrap() = true;
        *self.at_mut(init_x, init_y + 1).unwrap() = true;
        *self.at_mut(init_x, init_y + 2).unwrap() = true;

        self
    }
    pub fn make_t(mut self, init_x: i32, init_y: i32) -> Self {
        // First Row
        *self.at_mut(init_x, init_y).unwrap() = false;
        *self.at_mut(init_x + 1, init_y).unwrap() = true;
        *self.at_mut(init_x + 2, init_y).unwrap() = false;
        //Second Row
        *self.at_mut(init_x, init_y + 1).unwrap() = true;
        *self.at_mut(init_x + 1, init_y + 1).unwrap() = true;
        *self.at_mut(init_x + 2, init_y + 1).unwrap() = true;

        self
    }
    pub fn make_r_pentomino(mut self, init_x: i32, init_y: i32) -> Self {
        // First column
        *self.at_mut(init_x, init_y).unwrap() = false;
        *self.at_mut(init_x, init_y + 1).unwrap() = true;
        *self.at_mut(init_x, init_y + 2).unwrap() = false;
        // Second column
        *self.at_mut(init_x + 1, init_y).unwrap() = true;
        *self.at_mut(init_x + 1, init_y + 1).unwrap() = true;
        *self.at_mut(init_x + 1, init_y + 2).unwrap() = true;
        //Third Column
        *self.at_mut(init_x + 2, init_y).unwrap() = true;
        *self.at_mut(init_x + 2, init_y + 1).unwrap() = false;
        *self.at_mut(init_x + 2, init_y + 2).unwrap() = false;

        self
    }
    pub fn make_glider(mut self, init_x: i32, init_y: i32) -> Self {
        // First Row
        *self.at_mut(init_x, init_y).unwrap() = true;
        *self.at_mut(init_x + 1, init_y).unwrap() = true;
        *self.at_mut(init_x + 2, init_y).unwrap() = true;
        // Second Row
        *self.at_mut(init_x, init_y + 1).unwrap() = true;
        *self.at_mut(init_x + 1, init_y + 1).unwrap() = false;
        *self.at_mut(init_x + 2, init_y + 1).unwrap() = false;
        //Third Row
        *self.at_mut(init_x, init_y + 2).unwrap() = false;
        *self.at_mut(init_x + 1, init_y + 2).unwrap() = true;
        *self.at_mut(init_x + 2, init_y + 2).unwrap() = false;
        self
    }

    pub fn make_random(mut self, start_point: (i32, i32), width: i32, height: i32) -> Self {
        let mut rng = rand::thread_rng();
        for j in 0..height {
            for i in 0..width {
                if rand::random() {
                    *self.at_mut(start_point.0 + i, start_point.1 + j).unwrap() = true;
                }
            }
        }

        self
    }
}

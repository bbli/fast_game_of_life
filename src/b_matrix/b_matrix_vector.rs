// for globals
use super::*;
use std::ops::{Deref, DerefMut};

#[cfg(test)]
use mocktopus::macros::*;

// Since we need to copy the init_seed twice into vec & new_vec
#[derive(Clone)]
// has to be on heap otherwise stack overflow
pub struct BMatrixVector(pub Vec<bool>);

// NOTE: For array indexing
impl Deref for BMatrixVector {
    type Target = Vec<bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for BMatrixVector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for BMatrixVector {
    fn default() -> Self {
        BMatrixVector(vec![false; (GRID_SIZE * GRID_SIZE) as usize])
    }
}

//#[mockable]
impl BMatrixVector {
    pub fn new_for_test(vec: Vec<bool>) -> Self {
        BMatrixVector(vec)
    }
}

impl MatrixView for BMatrixVector {
    type Item = bool;
    fn at(&self, i: i32, j: i32) -> GameResult<Self::Item> {
        if i >= 0 && j >= 0 && i < GRID_SIZE && j < GRID_SIZE {
            //bool is copy type, so moving is fine
            Ok(self.0[(j * GRID_SIZE + i) as usize])
        } else if i >= GRID_SIZE || j >= GRID_SIZE {
            //if i< GRID_SIZE && j<GRID_SIZE && i>=0 && j>=0{
            Err(GameError::EventLoopError(format!(
                "IndexError: b_matrix_vector's i must be less than {} and j must be less than {}",
                GRID_SIZE, GRID_SIZE
            )))
        } else {
            Err(GameError::EventLoopError(
                "IndexError(b_matrix.at): i and j must be nonnegative".to_string(),
            ))
        }
    }
    fn at_mut(&mut self, i: i32, j: i32) -> GameResult<&mut Self::Item> {
        if i >= 0 && j >= 0 && i < GRID_SIZE && j < GRID_SIZE {
            Ok(&mut self.0[(j * GRID_SIZE + i) as usize])
        } else if i >= GRID_SIZE || j >= GRID_SIZE {
            Err(GameError::EventLoopError(format!(
                "IndexError: b_matrix_vector's i must be less than {} and j must be less than {}",
                GRID_SIZE, GRID_SIZE
            )))
        } else {
            Err(GameError::EventLoopError(
                "IndexError(b_matrix.at): i and j must be nonnegative".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    #[test]
    fn test_BMatrixVector_index_on_subview() {
        let b_matrix_vector = BMatrixVector::default();
        // Check that a point close to origin
        let value = b_matrix_vector.at(1, 1).unwrap();
        assert_eq!(value, false);
        // Check last point:
        let value = b_matrix_vector
            .at((GRID_SIZE - 1) as i32, (GRID_SIZE - 1) as i32)
            .unwrap();
        assert_eq!(value, false);
    }

    #[should_panic]
    #[test]
    fn test_BMatrixVector_at_outOfBounds() {
        println!("HI!!!!!!");
        let b_matrix_vector = BMatrixVector::default();

        let value = b_matrix_vector.at((2 * GRID_SIZE) as i32, 0).unwrap();
    }
}

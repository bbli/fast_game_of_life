use super::*;
use rayon::prelude::*;
use std::ops::{Deref, DerefMut};

#[cfg(test)]
use mocktopus::macros::*;

// has to be on heap otherwise stack overflow
pub struct BMatrixVector(Vec<bool>);
//unsafe impl Send for *const BMatrixVector{}

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

// ************  Helper Functions  ************
fn get_location_from_idx(idx: usize) -> (i32, i32) {
    let idx = idx as i32;
    let i = idx % GRID_SIZE;
    let j = (idx - i) / GRID_SIZE;
    (i, j)
}

mod life {
    use super::*;
    pub fn convert_bool(i: i32, j: i32, b_matrix_vector: &BMatrixVector) -> u32 {
        match b_matrix_vector.at(i, j) {
            Ok(value) => {
                if value {
                    1
                } else {
                    0
                }
            }
            //EC: off screen
            Err(_) => 0,
        }
    }

    // since we are using this to survey around, x and y can now be negative
    // but "at" method covers this error handling
    // TODO: time how fast w/o local variables + refactor into 3 by 3 permutation
    pub fn get_count(i: i32, j: i32, b_matrix_vector: &BMatrixVector) -> u32 {
        let mut total = 0;
        total += convert_bool(i + 1, j, b_matrix_vector);
        total += convert_bool(i + 1, j + 1, b_matrix_vector);
        total += convert_bool(i, j + 1, b_matrix_vector);
        total += convert_bool(i - 1, j + 1, b_matrix_vector);
        total += convert_bool(i - 1, j, b_matrix_vector);
        total += convert_bool(i - 1, j - 1, b_matrix_vector);
        total += convert_bool(i, j - 1, b_matrix_vector);
        total + convert_bool(i + 1, j - 1, b_matrix_vector)
        //let total= 0;
        //for delta_y in -1..2{
        //for delta_x in -1..2{
        //total += convert_bool(i+delta_x,j+delta_y);
        //}
        //}
        //return total - convert_bool(i,j,b_matrix_vector)
    }

    pub fn new_cell_value(state: bool, count: u32, b_matrix_vector: &BMatrixVector) -> bool {
        match state {
            //dead transition
            false => {
                if count == 3 {
                    true
                } else {
                    false
                }
            }
            //alive transition
            true => {
                if count == 2 || count == 3 {
                    true
                } else {
                    false
                }
            }
        }
    }
}

//#[mockable]
impl BMatrixVector {
    pub fn new_for_test(vec: Vec<bool>) -> Self {
        BMatrixVector(vec)
    }
    pub fn next_b_matrix(&self) -> BMatrixVector {
        let mut new_results = BMatrixVector::default();

        for j in 0..GRID_SIZE {
            for i in 0..GRID_SIZE {
                let count = life::get_count(i, j, &self);
                let state = self.at(i, j).unwrap();
                let mut cell_ptr = new_results.at_mut(i, j).unwrap();
                *cell_ptr = life::new_cell_value(state, count, &self);
            }
        }

        new_results
    }
    // self has to be immutable for multi-thread reads
    pub fn next_b_matrix_rayon(&self) -> BMatrixVector {
        let mut new_results = BMatrixVector::default();

        new_results
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, cell_ptr)| {
                let (i, j) = get_location_from_idx(idx);
                let count = life::get_count(i, j, &self);
                let state = self.at(i, j).unwrap();
                *cell_ptr = life::new_cell_value(state, count, &self);
            });

        new_results
    }

    //fn do_job(start_y:i32, end_y: i32, )
    pub fn next_b_matrix_threadpool(&self, region_pool: &mut RegionPool) -> BMatrixVector {
        // ************  MULTITHREADED THREADPOOL  ************
        let mut new_results = BMatrixVector::default();
        // 0. allocate threadpool during Grid::new() DONE
        // 1. code to partition grid evenly into num_of_threads(also in setup) DONE
        // 2. start up each thread -> since they have a predefined job
        // 3. join to wait
        //
        // need local variable since closures require unique acess to its borrows
        let region_iterator = region_pool.create_iter_mut(&mut new_results);
        region_pool.scoped(|scope| {
            for (slice, iter_offset) in region_iterator {
                scope.execute(move || {
                    for (rel_i, cell_ptr) in slice.iter_mut().enumerate() {
                        let idx = rel_i + iter_offset as usize;
                        let (i, j) = get_location_from_idx(idx);
                        let count = life::get_count(i, j, &self);
                        let state = self.at(i, j).unwrap();
                        *cell_ptr = life::new_cell_value(state, count, &self);
                    }
                });
            }
            scope.join_all();
        });

        new_results
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
    fn at_mut<'a>(&'a mut self, i: i32, j: i32) -> GameResult<&'a mut Self::Item> {
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
        let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        );
        let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();
        // initialize a Grid object
        let update_method = BackendEngine::Skip;
        let grid = Grid::new(ctx, update_method).unwrap();
        // Check that a point close to origin
        let value = grid.b_matrix.at(1, 1).unwrap();
        assert_eq!(value, false);
        // Check last point:
        let value = grid
            .b_matrix
            .at((GRID_SIZE - 1) as i32, (GRID_SIZE - 1) as i32)
            .unwrap();
        assert_eq!(value, false);
    }

    #[should_panic]
    #[test]
    fn test_BMatrixVector_at_outOfBounds() {
        println!("HI!!!!!!");
        let mut globals = setup().unwrap();
        let update_method = BackendEngine::Skip;
        let grid = Grid::new(&mut globals.ctx, update_method).unwrap();

        let value = grid.b_matrix.at((2 * GRID_SIZE) as i32, 0).unwrap();
    }

    #[test]
    fn test_update_b_matrix_single_cell_become_dead() {
        let mut b_matrix_vector = BMatrixVector::default();
        let i = GRID_SIZE - 1;
        let j = 40;
        let i = i as i32;
        let j = j as i32;

        *b_matrix_vector.at_mut(i, j).unwrap() = true;
        assert_eq!(b_matrix_vector.at(i, j).unwrap(), true);
        let next_b_matrix_vector = b_matrix_vector.next_b_matrix();

        assert_eq!(next_b_matrix_vector.at(i, j).unwrap(), false);
    }

    #[test]
    fn test_update_b_matrix_edge_cell_become_alive() {
        let mut b_matrix_vector = BMatrixVector::default();
        let i = GRID_SIZE - 1;
        let j = 40;
        let i = i as i32;
        let j = j as i32;
        *b_matrix_vector.at_mut(i, j + 1).unwrap() = true;
        *b_matrix_vector.at_mut(i - 1, j).unwrap() = true;
        *b_matrix_vector.at_mut(i, j - 1).unwrap() = true;

        let next_b_matrix_vector = b_matrix_vector.next_b_matrix();

        assert_eq!(next_b_matrix_vector.at(i, j + 1).unwrap(), false);
        assert_eq!(next_b_matrix_vector.at(i - 1, j).unwrap(), true);
        assert_eq!(next_b_matrix_vector.at(i, j - 1).unwrap(), false);
        assert_eq!(next_b_matrix_vector.at(i, j).unwrap(), true);
    }

    #[test]
    fn test_update_b_matrix_corner_cell_stays_alive() {
        let mut b_matrix_vector = BMatrixVector::default();
        let i = GRID_SIZE - 1;
        let j = GRID_SIZE - 1;

        *b_matrix_vector.at_mut(i, j - 1).unwrap() = true;
        *b_matrix_vector.at_mut(i - 1, j).unwrap() = true;
        *b_matrix_vector.at_mut(i, j).unwrap() = true;

        let next_b_matrix_vector = b_matrix_vector.next_b_matrix();

        assert_eq!(next_b_matrix_vector.at(i, j).unwrap(), true);
        assert_eq!(next_b_matrix_vector.at(i, j - 1).unwrap(), true);
        assert_eq!(next_b_matrix_vector.at(i - 1, j).unwrap(), true);
    }

    #[test]
    fn test_get_location_from_idx() {
        let i: i32 = 3;
        let j: i32 = 2;
        let idx = j * GRID_SIZE + i;
        let (new_i, new_j) = get_location_from_idx(idx as usize);
        assert_eq!(i, new_i);
        assert_eq!(j, new_j);
    }

    #[test]
    #[ignore]
    fn test_next_b_matrix_threadpool_first_thread() {
        // NOTE: B/c of closures, hard to abstract over so we will just plain out
        // override the method we are testing -> So if implementation changes,
        // make sure to change this too
        BMatrixVector::next_b_matrix_threadpool.mock_safe(
            |my_self: &BMatrixVector, region_pool: &mut RegionPool| {
                let mut new_results = BMatrixVector::default();
                //unlike in original code, we are not going to move region_iterator
                let mut region_iterator = region_pool.create_iter_mut(&mut new_results);
                region_pool.scoped(|scope| {
                    if let Some((first_slice, first_offset)) = region_iterator.next() {
                        scope.execute(move || {
                            for (rel_i, cell_ptr) in first_slice.iter_mut().enumerate() {
                                let idx = rel_i + first_offset as usize;
                                let (i, j) = get_location_from_idx(idx);
                                let count = life::get_count(i, j, &my_self);
                                let state = my_self.at(i, j).unwrap();
                                *cell_ptr = life::new_cell_value(state, count, &my_self);
                            }
                        })
                    } else {
                        panic!("iterator should still have elements");
                    }
                });

                MockResult::Return(new_results)
            },
        );

        let mut globals = setup().unwrap();
        let init_b_matrix_vector = patterns::PatternBuilder::new()
            .make_random((0, 0), GRID_SIZE, GRID_SIZE)
            .build();
        let update_method = BackendEngine::MultiThreaded(100);
        //let update_method = BackendEngine::Rayon;
        let mut grid = Grid::new(&mut globals.ctx, update_method)
            .unwrap()
            .init_seed(init_b_matrix_vector);

        // should only update the first section of rows
        event::run(&mut globals.ctx, &mut globals.event_loop, &mut grid);
    }
}

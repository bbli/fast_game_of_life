use rayon::prelude::*;
use scoped_threadpool::Pool;
use std::cmp::Ordering;
use std::mem;
use std::ops::{Deref, DerefMut};

use super::b_matrix_vector::*;
// for globals
use super::*;

pub enum Backend {
    Single,
    MultiThreaded(i32),
    Rayon,
    Skip,
}


pub trait Engine{
    fn next_b_matrix(&mut self,old_vec:&BMatrixVector, new_vec: &mut BMatrixVector);
}

pub fn create_engine(update_method: Backend) -> Box<dyn Engine>{
    use Backend::*;
    match update_method{
        Single => Box::new(SingleThreadEngine::new()),
        MultiThreaded(worker_count) => Box::new(MultiThreadedEngine::new(worker_count)),
        Rayon => Box::new(RayonEngine::new()),
        Skip => Box::new(SkipEngine::new())
    }
}

// ************  Engine Implementations  ************   
fn get_location_from_idx(idx: usize) -> (i32, i32) {
    let idx = idx as i32;
    let i = idx % GRID_SIZE;
    //let j = (idx - i) / GRID_SIZE;
    let j = idx /GRID_SIZE;
    (i, j)
}

struct SingleThreadEngine;
impl Engine for SingleThreadEngine{
    fn next_b_matrix(&mut self,old_vec:&BMatrixVector, new_vec: &mut BMatrixVector){
        for j in 0..GRID_SIZE {
            for i in 0..GRID_SIZE {
                let count = life::get_count(i, j,old_vec);
                let state = old_vec.at(i, j).unwrap();
                let mut cell_ptr = new_vec.at_mut(i, j).unwrap();
                *cell_ptr = life::new_cell_value(state, count, old_vec);
            }
        }
    }
}
impl SingleThreadEngine{
    fn new()->Self{
        SingleThreadEngine{}
    }
}


struct SkipEngine;
impl Engine for SkipEngine{
    fn next_b_matrix(&mut self,old_vec:&BMatrixVector, new_vec: &mut BMatrixVector){
    }
}
impl SkipEngine{
    fn new()->Self{
        SkipEngine{}
    }
}


struct RayonEngine;
impl Engine for RayonEngine{
    fn next_b_matrix(&mut self,old_vec:&BMatrixVector, new_vec: &mut BMatrixVector){
        new_vec
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, cell_ptr)| {
                let (i, j) = get_location_from_idx(idx);
                let count = life::get_count(i, j,old_vec);
                let state = old_vec.at(i, j).unwrap();
                *cell_ptr = life::new_cell_value(state, count,old_vec);
            });
    }
}
impl RayonEngine{
    fn new()->Self{
        RayonEngine{}
    }
}

// ************  Mutli Threading Code  ************
fn get_num_elems_each_time(vector: &BMatrixVector, worker_count: i32) -> i32 {
    vector.len() as i32 / worker_count
}

struct MultiThreadedEngine{
    threadpool: Pool,
    worker_count: i32
}
impl Engine for MultiThreadedEngine{
    fn next_b_matrix(&mut self,old_vec:&BMatrixVector, new_vec: &mut BMatrixVector){
        // ************  MULTITHREADED THREADPOOL  ************
        // 0. allocate threadpool during Grid::new() DONE
        // 1. code to partition grid evenly into num_of_threads(also in setup) DONE
        // 2. start up each thread -> since they have a predefined job
        // 3. join to wait
        //
        // need local variable since closures require unique acess to its borrows
        let region_iterator = self.create_iter_mut(new_vec);
        self.threadpool.scoped(|scope| {
            for (slice, iter_offset) in region_iterator {
                scope.execute(move || {
                    for (rel_i, cell_ptr) in slice.iter_mut().enumerate() {
                        let idx = rel_i + iter_offset as usize;
                        let (i, j) = get_location_from_idx(idx);
                        let count = life::get_count(i, j,old_vec);
                        let state = old_vec.at(i, j).unwrap();
                        *cell_ptr = life::new_cell_value(state, count,old_vec);
                    }
                });
            }
            scope.join_all();
        });
    }
}
impl MultiThreadedEngine {
    fn new(worker_count: i32) -> Self {
        //let threadpool = Arc::new(ThreadPool::new(worker_count as usize));
        let threadpool = Pool::new(worker_count as u32);
        MultiThreadedEngine {
            threadpool,
            worker_count,
        }
    }
    // EC: worker_count is 1 -> max_offset should be 0, so edge case is fine too
    fn create_iter_mut<'a>(&mut self, vector: &'a mut BMatrixVector) -> RegionPoolIterMut<'a> {
        let num_elems_each_time = get_num_elems_each_time(vector, self.worker_count);

        let max_offset = num_elems_each_time * (self.worker_count - 1);
        RegionPoolIterMut {
            ptr: &mut vector[..],
            offset: 0,
            num_elems_each_time,
            max_offset,
        }
    }
}


struct RegionPoolIterMut<'a> {
    ptr: &'a mut [bool],
    offset: i32,
    num_elems_each_time: i32,
    max_offset: i32,
}
impl<'a> Iterator for RegionPoolIterMut<'a> {
    type Item = (&'a mut [bool], i32);

    // EC: at end when we need to take a bit more
    fn next(&mut self) -> Option<Self::Item> {
        // old offset "points" to offset we are about to return
        let old_offset = self.offset;
        self.offset += self.num_elems_each_time;

        match old_offset.cmp(&self.max_offset) {
            // after last case
            Ordering::Greater => None,
            // last case -> just return ptr
            Ordering::Equal => {
                // Since this works, self.ptr must have 'a lifetime
                // even though self has local
                //let l = self.ptr;
                //self.ptr = &mut [];
                // But that said, we can't explicitly take, since self.ptr doesn't own its values
                let slice = mem::replace(&mut self.ptr, &mut []);
                let l = slice;

                Some((l, old_offset))
            }
            // Recursive case
            Ordering::Less => {
                // self.ptr.split_at_mut will pass in the local lifetime rather than  'a
                //let (l,r) = self.ptr.split_at_mut(self.num_elems_each_time as usize);

                // Rustonomicon uses mem::replace to resolve(slice doesn't do anything in Drop though?) -> though the below works too
                //let slice = self.ptr;
                let slice = mem::replace(&mut self.ptr, &mut []);

                let (l, r) = slice.split_at_mut(self.num_elems_each_time as usize);
                self.ptr = r;

                Some((l, old_offset))
            }
        }
    }
}

// ************  GAME OF LIFE RULES  ************   
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    #[test]
    fn test_RegionPoolIterMut_get_num_elems_each_time_workerCount1() {
        let worker_count = 1;
        let vec = BMatrixVector::default();
        let num_elems = get_num_elems_each_time(&vec, worker_count);
        assert_eq!(num_elems, GRID_SIZE * GRID_SIZE);
    }
    #[test]
    fn test_RegionPoolIterMut_next_edge_case() {
        let worker_count = 1;
        let mut region_pool = MultiThreadedEngine::new(worker_count);
        let bool_vec = vec![true, true, true, false, false, false, false];

        let test_vec = bool_vec.clone();
        let mut b_matrix_vector = BMatrixVector::new_for_test(bool_vec);
        let mut region_iterator = region_pool.create_iter_mut(&mut b_matrix_vector);

        if let Some((whole_slice, offset)) = region_iterator.next() {
            assert_eq!(whole_slice, test_vec);
            assert_eq!(offset, 0);
        } else {
            panic!("iterator should still have elements");
        }
    }
    #[test]
    fn test_RegionPoolIterMut_step_through_next() {
        let worker_count = 3;
        let mut region_pool = MultiThreadedEngine::new(worker_count);
        let mut vec =
            BMatrixVector::new_for_test(vec![true, true, true, false, false, false, false]);
        let mut region_iterator = region_pool.create_iter_mut(&mut vec);

        if let Some((slice1, offset1)) = region_iterator.next() {
            assert_eq!(slice1, vec![true, true]);
            assert_eq!(offset1, 0);
        } else {
            panic!("iterator should still have elements");
        }

        if let Some((slice2, offset2)) = region_iterator.next() {
            assert_eq!(slice2, vec![true, false]);
            assert_eq!(offset2, 2);
        } else {
            panic!("iterator should still have elements");
        }

        if let Some((slice3, offset3)) = region_iterator.next() {
            assert_eq!(slice3, vec![false, false, false]);
            assert_eq!(offset3, 4);
        } else {
            panic!("iterator should still have elements");
        }

        if let Some(_) = region_iterator.next() {
            panic!("iterator should be empty now");
        }
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
        let mut next_b_matrix_vector = BMatrixVector::default();
        SingleThreadEngine::new().next_b_matrix(&b_matrix_vector,&mut next_b_matrix_vector);

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

        let mut next_b_matrix_vector = BMatrixVector::default();
        SingleThreadEngine::new().next_b_matrix(&b_matrix_vector,&mut next_b_matrix_vector);

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

        let mut next_b_matrix_vector = BMatrixVector::default();
        SingleThreadEngine::new().next_b_matrix(&b_matrix_vector,&mut next_b_matrix_vector);

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
    // This test can't work anymore as mocktopus mocks only work on the main thread
    //#[test]
    //#[ignore]
    //fn test_next_b_matrix_threadpool_first_thread() {
        //// NOTE: B/c of closures, hard to abstract over so we will just plain out
        //// override the method we are testing -> So if implementation changes,
        //// make sure to change this too
        //engine::next_b_matrix_threadpool.mock_safe(
            //|vec: &BMatrixVector, new_vec: &mut BMatrixVector, region_pool: &mut RegionPool| {
                ////unlike in original code, we are not going to move region_iterator
                //let mut region_iterator = region_pool.create_iter_mut(new_vec);
                //region_pool.scoped(|scope| {
                    //if let Some((first_slice, first_offset)) = region_iterator.next() {
                        ////println!("Num elements")
                        //panic!("got here");
                        //scope.execute(move || {
                            //for (rel_i, cell_ptr) in first_slice.iter_mut().enumerate() {
                                //let idx = rel_i + first_offset as usize;
                                //let (i, j) = get_location_from_idx(idx);
                                //let count = life::get_count(i, j, &vec);
                                //let state = vec.at(i, j).unwrap();
                                //*cell_ptr = life::new_cell_value(state, count,vec);
                            //}
                        //});
                        //scope.join_all();
                    //} else {
                        //panic!("iterator should still have elements");
                    //}
                //});

                //MockResult::Return(())
            //},
        //);

        //let mut globals = setup().unwrap();
        //let init_b_matrix_vector = patterns::PatternBuilder::new()
            //.make_random((0, 0), GRID_SIZE, GRID_SIZE)
            //.build();
        //let update_method = Backend::MultiThreaded(500);
        ////let update_method = Backend::Rayon;
        //let mut grid = Grid::new(&mut globals.ctx, update_method)
            //.unwrap()
            //.init_seed(init_b_matrix_vector);

        //// should only update the first section of rows
        //event::run(&mut globals.ctx, &mut globals.event_loop, &mut grid);
    //}
}

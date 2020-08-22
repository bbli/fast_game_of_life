use std::ops::{Deref,DerefMut};
use rayon::prelude::*;

// has to be on heap otherwise stack overflow
pub struct BMatrixVector(Vec<bool>);
unsafe impl Send for *const BMatrixVector{}

use super::*;

// NOTE: For array indexing
impl Deref for BMatrixVector{
    type Target = Vec<bool>;
    fn deref(&self) -> &Self::Target{
        &self.0
    }
}
impl DerefMut for BMatrixVector{
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.0
    }
}

impl BMatrixVector{
    pub fn new()-> Self{
        BMatrixVector(vec![false;(GRID_SIZE*GRID_SIZE) as usize])
    }
}

fn get_location_from_idx(idx:usize)->(i32,i32){
    let idx = idx as i32;
    let i = idx % GRID_SIZE;
    let j = (idx - i) / GRID_SIZE;
    (i,j)
}
fn par_convert_bool(i:i32,j:i32,b_matrix:&BMatrixVector)-> u32{
        match b_matrix.at(i,j){
            Ok(value) => if value {1}else {0},
            //EC: off screen
            Err(_) => 0
        }
}

fn par_get_count(i:i32,j:i32,b_matrix: &BMatrixVector)->u32{
        let right = par_convert_bool(i+1,j,b_matrix);
        let down = par_convert_bool(i,j+1,b_matrix);
        let left = par_convert_bool(i-1,j,b_matrix);
        let up = par_convert_bool(i,j-1,b_matrix);
        right + down + left + up
}

fn par_new_cell_value(i:i32,j:i32,count:u32,b_matrix:&BMatrixVector) -> bool{
    let state = b_matrix.at(i,j).unwrap();
    match state{
        //dead transition
        false => {if count== 3 {true} else {false}}
        //alive transition
        true => {if count == 2 || count == 3 {true} else {false}}
    }
}
impl BMatrixVector{
    fn convert_bool(&self,x:i32,y:i32)-> u32{
        match self.at(x,y){
            Ok(value) => if value {1}else {0},
            //EC: off screen
            Err(_) => 0
        }
    }
    // since we are using this to survey around, x and y can now be negative
    // but "at" method covers this error handling
    fn get_count(&self, i:i32,j:i32)-> u32{
        let right = self.convert_bool(i+1,j);
        let down = self.convert_bool(i,j+1);
        let left = self.convert_bool(i-1,j);
        let up = self.convert_bool(i,j-1);
        right + down + left + up
    }

    pub fn new_cell_value(&self,i:i32, j:i32, count:u32)-> bool{
        let state = self.at(i,j).unwrap();
        match state{
            //dead transition
            false => {if count== 3 {true} else {false}}
            //alive transition
            true => {if count == 2 || count == 3 {true} else {false}}
        }
    }

    pub fn next_bmatrix(&self)-> BMatrixVector{
        let mut new_results = BMatrixVector::new();

        for j in 0..GRID_SIZE{
            for i in 0..GRID_SIZE{
                let count = self.get_count(i,j);
                let mut new_value_ref = new_results.at_mut(i,j).unwrap();
                *new_value_ref = self.new_cell_value(i,j,count);
            }
        }

        new_results
    }
    pub fn next_bmatrix_rayon(&self) -> BMatrixVector{
        let mut new_results = BMatrixVector::new();

        new_results.par_iter_mut().enumerate().for_each(|(idx,value)| {
            let (i,j) = get_location_from_idx(idx);
            let count = par_get_count(i,j,&self);
            *value = par_new_cell_value(i,j,count,&self);
        });

        new_results
    }
    pub fn next_bmatrix_threadpool(&self, region_pool: &RegionPool)-> BMatrixVector{
        // ************  MULTITHREADED THREADPOOL  ************   
        let new_results = BMatrixVector::new();
        // 0. allocate threadpool during Grid::new() DONE
        // 1. code to partition grid evenly into num_of_threads(also in setup) DONE
        // 2. start up each thread -> since they have a predefined job
        for (start_y,end_y) in region_pool.work_region_list.iter(){
            let start_y = start_y.clone();
            let end_y = end_y.clone();
            let vec_ptr = self as *const BMatrixVector;

            region_pool.execute(move ||{
                for j in start_y..end_y+1{
                    for i in 0..GRID_SIZE{
                        let count = (*vec_ptr).get_count(i,j);
                        let mut new_value_ref = new_results.at_mut(i,j).unwrap();
                        *new_value_ref = (*vec_ptr).new_cell_value(i,j,count);
                    }
                }
            })
        }

        // Checking that values were not moved
        for (start_y, end_y) in region_pool.work_region_list.iter(){
            let x = start_y +1 ;
        }
        
        // 3. join to wait
        region_pool.join();

        new_results
    }
}

impl MatrixView for BMatrixVector{
    type Item = bool;
    fn at(&self, i:i32, j:i32) -> GameResult<Self::Item>{
        if i < 0 || j < 0 {
            Err(GameError::EventLoopError("IndexError(bmatrix.at): i and j must be nonnegative".to_string()))
        }
        else if i >= GRID_SIZE || j>= GRID_SIZE {
        //if i< GRID_SIZE && j<GRID_SIZE && i>=0 && j>=0{
            Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        }
        else{
            //bool is copy type, so moving is fine
            Ok(self.0[(j*GRID_SIZE +i) as usize])
        }
    }
    fn at_mut<'a>(&'a mut self, i:i32, j:i32) -> GameResult<&'a mut Self::Item>{
        if i < 0 || j < 0 {
            Err(GameError::EventLoopError("IndexError(bmatrix.at): i and j must be nonnegative".to_string()))
        }
        else if i >= GRID_SIZE || j>= GRID_SIZE {
        //if i< GRID_SIZE && j<GRID_SIZE && i>=0 && j>=0{
            Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        }
        else{
            Ok(&mut self.0[(j*GRID_SIZE +i) as usize])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use super::*;
    #[test]
    fn test_BMatrixVector_index_on_subview(){
        let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        );
        let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();
        // initialize a Grid object
        let grid = Grid::new(ctx).unwrap();
        // Check that a point close to origin
        let value = grid.b_matrix.at(1,1).unwrap();
        assert_eq!(value,false);
        // Check last point: 
        let value = grid.b_matrix.at((GRID_SIZE-1) as i32,(GRID_SIZE-1) as i32).unwrap();
        assert_eq!(value,false);
    }

    #[should_panic]
    #[test]
    fn test_BMatrixVector_at_outOfBounds(){
        println!("HI!!!!!!");
        let globals = setup().unwrap();

        let value = globals.grid.b_matrix.at((2*GRID_SIZE) as i32,0).unwrap();
    }

    #[test]
    fn test_update_bmatrix_single_cell_become_dead(){
        let mut b_matrix = BMatrixVector::new();
        let i = GRID_SIZE-1;
        let j = 40;
        let i = i as i32;
        let j = j as i32;

        *b_matrix.at_mut(i,j).unwrap() = true;
        assert_eq!(b_matrix.at(i,j).unwrap(),true);
        let new_matrix = b_matrix.next_bmatrix();

        assert_eq!(new_matrix.at(i,j).unwrap(),false);
    }

    #[test]
    fn test_update_bmatrix_single_cell_become_alive(){
        let mut b_matrix = BMatrixVector::new();
        let i = 40;
        let j = 40;
        *b_matrix.at_mut(i+1,j).unwrap() = true;
        *b_matrix.at_mut(i,j+1).unwrap() = true;
        *b_matrix.at_mut(i-1,j).unwrap() = true;

        let next_bmatrix = b_matrix.next_bmatrix();

        assert_eq!(next_bmatrix.at(i,j).unwrap(),true);
        assert_eq!(next_bmatrix.at(i+1,j).unwrap(),false);
        assert_eq!(next_bmatrix.at(i,j+1).unwrap(),false);
        assert_eq!(next_bmatrix.at(i-1,j).unwrap(),false);
    }

    #[test]
    fn test_update_bmatrix_edge_cell_become_alive(){
        let mut b_matrix = BMatrixVector::new();
        let i = GRID_SIZE-1;
        let j = 40;
        let i = i as i32;
        let j = j as i32;
        *b_matrix.at_mut(i,j+1).unwrap() = true;
        *b_matrix.at_mut(i-1,j).unwrap() = true;
        *b_matrix.at_mut(i,j-1).unwrap() = true;

        let next_bmatrix = b_matrix.next_bmatrix();

        assert_eq!(next_bmatrix.at(i,j+1).unwrap(),false);
        assert_eq!(next_bmatrix.at(i-1,j).unwrap(),false);
        assert_eq!(next_bmatrix.at(i,j-1).unwrap(),false);
        assert_eq!(next_bmatrix.at(i,j).unwrap(),true);
    }

    #[test]
    fn test_update_bmatrix_corner_cell_stays_alive(){
        let mut b_matrix = BMatrixVector::new();
        let i = GRID_SIZE-1;
        let j = GRID_SIZE-1;
        let i = i as i32;
        let j = j as i32;

        *b_matrix.at_mut(i,j-1).unwrap() = true;
        *b_matrix.at_mut(i-1,j).unwrap() = true;
        *b_matrix.at_mut(i,j).unwrap() = true;

        let next_bmatrix = b_matrix.next_bmatrix();

        assert_eq!(next_bmatrix.at(i,j).unwrap(),true);
        assert_eq!(next_bmatrix.at(i,j-1).unwrap(),false);
        assert_eq!(next_bmatrix.at(i-1,j).unwrap(),false);
    }

    #[test]
    fn test_get_location_from_idx(){
        let i:i32 = 3;
        let j:i32 = 2;
        let idx = j*GRID_SIZE+i;
        let (new_i,new_j) = get_location_from_idx(idx as usize);
        assert_eq!(i,new_i);
        assert_eq!(j,new_j);
    }
}

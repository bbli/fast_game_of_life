use std::ops::{Deref,DerefMut};
// has to be on heap otherwise stack overflow
pub struct BMatrix(Vec<bool>);
use super::*;

// NOTE: These two may or may not be nesscary
impl Deref for BMatrix{
    type Target = Vec<bool>;
    fn deref(&self) -> &Self::Target{
        &self.0
    }
}
impl DerefMut for BMatrix{
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.0
    }
}

impl BMatrix{
    pub fn new()-> Self{
        BMatrix(vec![false;(GRID_SIZE*GRID_SIZE) as usize])
    }
}

impl BMatrix{
    fn convert_bool(&self,x:i32,y:i32)-> u32{
        match self.at(x,y){
            Ok(value) => if value {1}else {0},
            //EC: off screen
            Err(_) => 0
        }
    }
    // since we are using this to survey around, x and y can now be negative
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

    pub fn next_bmatrix(&self)-> BMatrix{
        let mut new_results = BMatrix::new();
        for j in 0..GRID_SIZE{
            for i in 0..GRID_SIZE{
                let i = i as i32;
                let j = j as i32;

                let count = self.get_count(i,j);
                let mut new_value_ref = new_results.at_mut(i,j).unwrap();
                *new_value_ref = self.new_cell_value(i,j,count);
            }
        }
        new_results
    }
}

impl MatrixView for BMatrix{
    type Item = bool;
    fn at(&self, i:i32, j:i32) -> GameResult<Self::Item>{
        if i< GRID_SIZE && j<GRID_SIZE && i>=0 && j>=0{
            //bool is copy type, so moving is fine
            Ok(self.0[(j*GRID_SIZE +i) as usize])
        }
        else{
            Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        }
    }
    fn at_mut<'a>(&'a mut self, i:i32, j:i32) -> GameResult<&'a mut Self::Item>{
        if i< GRID_SIZE && j<GRID_SIZE && i>=0 && j>=0{
            Ok(&mut self.0[(j*GRID_SIZE +i) as usize])
        }
        else{
            Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        }
    }
}

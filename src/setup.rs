use super::*;
use rand::prelude::*;

pub fn make_blinker(i:i32,j:i32,init_bmatrix:&mut BMatrix){
    let init_location:(i32,i32) = (80,50);
    if i==init_location.0 && j == init_location.1{
        *init_bmatrix.at_mut(i,j).unwrap() = true;
        *init_bmatrix.at_mut(i,j+1).unwrap() = true;
        *init_bmatrix.at_mut(i,j+2).unwrap() = true;
    }
}

pub fn make_square(i:i32,j:i32,init_bmatrix:&mut BMatrix){
    let init_location:(i32,i32) = (50,50);
    if i==init_location.0 && j == init_location.1{
        *init_bmatrix.at_mut(i,j).unwrap() = true;
        *init_bmatrix.at_mut(i+1,j).unwrap() = true;
        *init_bmatrix.at_mut(i+1,j+1).unwrap() = true;
        *init_bmatrix.at_mut(i,j+1).unwrap() = true;
    }
}

pub fn make_random(b_matrix: &mut BMatrix){
    let mut rng = rand::thread_rng();
    for j in 0..GRID_SIZE{
        for i in 0..GRID_SIZE{
            if rand::random(){
                *b_matrix.at_mut(i as i32,j as i32).unwrap() =true;
            }
        }
    }
}

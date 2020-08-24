use super::*;
use rand::prelude::*;

pub fn make_blinker(i:i32,j:i32,init_bmatrix_vector:&mut BMatrixVector){
    let init_location:(i32,i32) = (80,50);
    if i==init_location.0 && j == init_location.1{
        *init_bmatrix_vector.at_mut(i,j).unwrap() = true;
        *init_bmatrix_vector.at_mut(i,j+1).unwrap() = true;
        *init_bmatrix_vector.at_mut(i,j+2).unwrap() = true;
    }
}

pub fn make_square(i:i32,j:i32,init_bmatrix_vector:&mut BMatrixVector){
    let init_location:(i32,i32) = (50,50);
    if i==init_location.0 && j == init_location.1{
        *init_bmatrix_vector.at_mut(i,j).unwrap() = true;
        *init_bmatrix_vector.at_mut(i+1,j).unwrap() = true;
        *init_bmatrix_vector.at_mut(i+1,j+1).unwrap() = true;
        *init_bmatrix_vector.at_mut(i,j+1).unwrap() = true;
    }
}

pub fn make_random(b_matrix_vector: &mut BMatrixVector){
    let mut rng = rand::thread_rng();
    for j in 0..GRID_SIZE{
        for i in 0..GRID_SIZE{
            if rand::random(){
                *b_matrix_vector.at_mut(i as i32,j as i32).unwrap() =true;
            }
        }
    }
}

pub fn make_r_pentomino(b_matrix_vector: &mut BMatrixVector){
}

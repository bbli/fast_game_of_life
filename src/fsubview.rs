use ggez::graphics::spritebatch;
use ggez::graphics;
use ggez::{Context, GameResult};
//use ggez::nalgebra as na;
use nalgebra::geometry::Point2;
use super::*;

pub struct SpriteBatchHandler{
    spritebatch: spritebatch::SpriteBatch,
    // we needs this vec b/c SpriteIdx wraps around a private field
    // so we can't dynamically construct the SpriteIdx ourselves when
    // we want to use the spritebatch.set method
    handle_list: Vec<spritebatch::SpriteIdx>
}

//impl SpriteBatchHandler{
    //fn set_correct(&mut self,i:i32,j:i32){
        //let sprite_handle = self.handle_list.at(i,j).unwrap();
        //self.spritebatch.set(sprite_handle,new_cell(i,j));
    //}
    //fn set_invalid(&mut self,i:i32,j:i32){
        //let sprite_handle = self.handle_list.at(i,j).unwrap();
        //self.spritebatch.set(sprite_handle,new_cell(INVALID_X,INVALID_Y));
    //}
//}
pub struct FSubview{
    pub black_sb_handler: SpriteBatchHandler,
    pub white_sb_handler: SpriteBatchHandler,
    pub relative_offset: Point
}

impl FSubview{
    pub fn clear(&mut self){
        self.black_sb_handler.spritebatch.clear();
        self.white_sb_handler.spritebatch.clear();
    }
    pub fn add_to_white(&mut self,relative_i:i32,relative_j:i32){
        self.white_sb_handler.spritebatch.add(new_cell(relative_i,relative_j));
    }
    pub fn add_to_black(&mut self,relative_i:i32,relative_j:i32){
        self.black_sb_handler.spritebatch.add(new_cell(relative_i,relative_j));
    }
    pub fn update_relative_offset(&mut self,x:f32,y:f32){
        self.relative_offset.x = x;
        self.relative_offset.y = y;
    }
    // NOTE: Is setting costly? If so, should only set if location actually changes
    //pub fn change_to_black(&mut self,i: i32, j: i32){
        ////set corresponding black sprite to correct location given i,j
        //self.black_sb_handler.set_correct(i,j);
        //// set corresponding white sprite to invalid location
        //self.white_sb_handler.set_invalid(i,j);
    //}
    //pub fn change_to_white(&mut self, i: i32, j: i32){
        //self.white_sb_handler.set_correct(i,j);
        //self.black_sb_handler.set_invalid(i,j);
    //}
    pub fn draw(&self,ctx: &mut Context)-> GameResult{
        //println!("Relative offset x: {}",self.relative_offset.x);
        //println!("Relative offset y: {}",self.relative_offset.y);
        let offset_draw_param = DrawParam::new()
                            .dest(Point2::new(-self.relative_offset.x,-self.relative_offset.y));
        match graphics::draw(ctx, &self.black_sb_handler.spritebatch ,offset_draw_param){
            Ok(value) => graphics::draw(ctx, &self.white_sb_handler.spritebatch ,offset_draw_param),
            Err(x) => Err(x)
        }
    }
}
//pub enum CellState{
    //BLACK,
    //WHITE
//}
pub fn new_cell(i:i32, j:i32) -> DrawParam{
    DrawParam::default()
        .dest(Point2::new(
                i as f32 * (CELL_SIZE as f32 + CELL_GAP),
                j as f32 * (CELL_SIZE as f32 + CELL_GAP)
                          )
              )
}

pub fn create_init_SpriteBatchHandler(image:Image, ctx:&mut Context) -> SpriteBatchHandler{

        let mut spritebatch = spritebatch::SpriteBatch::new(image);

        let mut handle_list = Vec::new();
        //handle_list.reserve((GRID_SIZE*GRID_SIZE)as usize);
        //// Create x axis first since in graphics first index corresponds with 
        //// the column, not row
        //for j in 0..GRID_SIZE {
            //for i in 0..GRID_SIZE{
                ////let sprite_idx = if black { spritebatch.add(new_cell(i,j))}
                                ////else {spritebatch.add(new_cell(INVALID_X,INVALID_Y))};
                //let sprite_idx = spritebatch.add(new_cell(i,j));
                ////if j < 2{
                    ////println!("{:#?}",sprite_idx);
                ////}
                //handle_list.push(sprite_idx);
                ////handle_list.0.push(spritebatch.add(new_cell(i,j)));
            //}
        //}
        SpriteBatchHandler{
            spritebatch,
            handle_list
        }
}

//Not Nesscary anymore since we just create new batch everytime now
//impl MatrixView for Vec<spritebatch::SpriteIdx>{
    //type Item= spritebatch::SpriteIdx;
    //fn at(&self,i:i32, j:i32)-> GameResult<Self::Item>{
        //if i < 0 || j < 0 {
            //Err(GameError::EventLoopError("IndexError(bmatrix.at): i and j must be nonnegative".to_string()))
        //}
        //else if i >= GRID_SIZE || j>= GRID_SIZE {
        ////if i< GRID_SIZE && j<GRID_SIZE && i>=0 && j>=0{
            //Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        //}
        //else{
            //Ok(self[(j*GRID_SIZE+ i) as usize].clone())
        //}
    //}
    //fn at_mut<'a>(&'a mut self,i:i32, j:i32)-> GameResult<&'a mut Self::Item>{
        //if i < 0 || j < 0 {
            //Err(GameError::EventLoopError("IndexError(bmatrix.at): i and j must be nonnegative".to_string()))
        //}
        //else if i >= GRID_SIZE || j>= GRID_SIZE {
        ////if i< GRID_SIZE && j<GRID_SIZE && i>=0 && j>=0{
            //Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        //}
        //else{
            //Ok(&mut self[(j*GRID_SIZE + i) as usize])
        //}
    //}
//}

pub fn get_base_index_right(x:f32)->i32{
    empty_moves_forward(x)
}

pub fn get_base_index_bottom(y:f32)->i32{
    empty_moves_forward(y)
}

pub fn get_base_index_top(y:f32)->i32{
    empty_moves_back(y)
}

pub fn get_base_index_left(x:f32)->i32{
    empty_moves_back(x)
}


pub fn empty_moves_back(z:f32)->i32{
    // num_sections gives the number of complete CELL_SIZE+CELL_GAP sections
    // -1 since it starts from 1(instead of 0 like the matrices)
    let num_sections = (z/(CELL_SIZE+CELL_GAP)).ceil();
    if num_sections == 0.0{
        num_sections as i32
    }
    else{
        num_sections as i32 -1
    }
}

// EC: 0,0
// PRECONDITON: z is positive
//width>0 so we don't need to account for 0 edge case like in empty_moves_back
pub fn empty_moves_forward(z:f32) -> i32{
    // ************  Float Land  ************   
    let num_sections = (z/(CELL_SIZE+CELL_GAP)).ceil();
    let rightmost_point = num_sections*(CELL_SIZE+CELL_GAP);
    let threshold = rightmost_point - CELL_GAP;

    // ************  Adjusting back to Index Land  ************   
     //means we are currently inside a box
     //so num_sections is the "correct" idx
    if threshold >= z{
        (num_sections - 1.0) as i32
    }
    // we need to extend down to next cell
    else{
        num_sections as i32
    }
}

pub fn get_top_of_cell(j:i32)->f32{
    j as f32*(CELL_SIZE+CELL_GAP)
}
pub fn get_distance_to_top(offset_y:f32,top_idx:i32)->GameResult<f32>{
    let top_of_upper_bound_cell = get_top_of_cell(top_idx);
    println!("offset_y: {}",offset_y);
    //println!("top_idx: {}",top_idx);
    println!("top_of_upper_bound_cell: {}",top_of_upper_bound_cell);

    if offset_y >= top_of_upper_bound_cell{
        Ok(offset_y-top_of_upper_bound_cell)
    }
    else{
        Err(GameError::EventLoopError("Top bounding cell should be above current offset".to_string()))
    }
}
pub fn get_left_of_cell(i:i32)->f32{
    i as f32*(CELL_SIZE+CELL_GAP)
}
pub fn get_distance_to_left(offset_x:f32,left_idx:i32)->GameResult<f32>{
    let left_of_upper_bound_cell = get_left_of_cell(left_idx);
    if offset_x >= left_of_upper_bound_cell{
        Ok(offset_x-left_of_upper_bound_cell)
    }
    else{
        Err(GameError::EventLoopError("Left bounding cell should be to left of current offset".to_string()))
    }
}


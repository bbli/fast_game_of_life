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

impl SpriteBatchHandler{
    fn set_correct(&mut self,i:i32,j:i32){
        let sprite_handle = self.handle_list.at(i,j).unwrap();
        self.spritebatch.set(sprite_handle,new_cell(i,j));
    }
    fn set_invalid(&mut self,i:i32,j:i32){
        let sprite_handle = self.handle_list.at(i,j).unwrap();
        self.spritebatch.set(sprite_handle,new_cell(INVALID_X,INVALID_Y));
    }
}
pub struct FSubview{
    pub black_sb_handler: SpriteBatchHandler,
    pub white_sb_handler: SpriteBatchHandler,
}

impl FSubview{
    // NOTE: Is setting costly? If so, should only set if location actually changes
    pub fn change_to_black(&mut self,i: i32, j: i32){
        //set corresponding black sprite to correct location given i,j
        self.black_sb_handler.set_correct(i,j);
        // set corresponding white sprite to invalid location
        self.white_sb_handler.set_invalid(i,j);
    }
    pub fn change_to_white(&mut self, i: i32, j: i32){
        self.white_sb_handler.set_correct(i,j);
        self.black_sb_handler.set_invalid(i,j);
    }
    pub fn draw(&self,ctx: &mut Context,f_offset:(f32,f32))-> GameResult{
        let offset_draw_param = DrawParam::new()
                            .dest(Point2::new(f_offset.0,f_offset.1));
        match graphics::draw(ctx, &self.black_sb_handler.spritebatch ,offset_draw_param){
            Ok(value) => graphics::draw(ctx, &self.white_sb_handler.spritebatch ,offset_draw_param),
            Err(x) => Err(x)
        }
    }
}
pub enum CellState{
    BLACK,
    WHITE
}
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
        handle_list.reserve((GRID_SIZE*GRID_SIZE)as usize);
        // Create x axis first since in graphics first index corresponds with 
        // the column, not row
        for j in 0..GRID_SIZE {
            for i in 0..GRID_SIZE{
                //let sprite_idx = if black { spritebatch.add(new_cell(i,j))}
                                //else {spritebatch.add(new_cell(INVALID_X,INVALID_Y))};
                let sprite_idx = spritebatch.add(new_cell(i,j));
                //if j < 2{
                    //println!("{:#?}",sprite_idx);
                //}
                handle_list.push(sprite_idx);
                //handle_list.0.push(spritebatch.add(new_cell(i,j)));
            }
        }
        SpriteBatchHandler{
            spritebatch,
            handle_list
        }
}

impl MatrixView for Vec<spritebatch::SpriteIdx>{
    type Item= spritebatch::SpriteIdx;
    fn at(&self,i:i32, j:i32)-> GameResult<Self::Item>{
        if i< GRID_SIZE && j < GRID_SIZE && i>=0 &&j>=0{
            Ok(self[(j*GRID_SIZE+ i) as usize].clone())
        }
        else{
            Err(GameError::EventLoopError(format!("IndexError: f_subview's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        }
    }
    fn at_mut<'a>(&'a mut self,i:i32, j:i32)-> GameResult<&'a mut Self::Item>{
        if i< GRID_SIZE && j < GRID_SIZE && i>=0 && j>=0{
            Ok(&mut self[(j*GRID_SIZE + i) as usize])
        }
        else{
            Err(GameError::EventLoopError(format!("IndexError: f_subview's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        }
    }
}

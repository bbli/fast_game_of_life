use ggez::graphics::spritebatch;
use ggez::graphics;
use ggez::{Context, GameResult};
use ggez::nalgebra as na;
use super::*;

pub struct SpriteBatchHandler{
    spritebatch: spritebatch::SpriteBatch,
    // we needs this vec b/c SpriteIdx wraps around a private field
    // so we can't dynamically construct the SpriteIdx ourselves when
    // we want to use the spritebatch.set method
    handle_list: Vec<spritebatch::SpriteIdx>
}
pub struct FSubview{
    pub black_sb_handler: SpriteBatchHandler,
    pub white_sb_handler: SpriteBatchHandler
}

impl FSubview{
    pub fn draw(&self,ctx: &mut Context)-> GameResult{
        match graphics::draw(ctx, &self.black_sb_handler.spritebatch , (na::Point2::new(0.0, 0.0),)){
            Ok(value) => graphics::draw(ctx, &self.white_sb_handler.spritebatch , (na::Point2::new(0.0, 0.0),)),
            Err(x) => Err(x)
        }
    }
}
pub enum CellState{
    BLACK,
    WHITE
}
pub fn new_cell(i:usize, j:usize) -> DrawParam{
    DrawParam::default()
        .dest(Point2::new(
                i as f32 * (CELL_SIZE as f32 + CELL_GAP),
                j as f32 * (CELL_SIZE as f32 + CELL_GAP)
                          )
              )
}

pub fn create_init_SpriteBatchHandler(state: CellState, ctx:&mut Context) -> SpriteBatchHandler{
        let black = match state{
            CellState::BLACK => true,
            CellState::WHITE => false
        };

        let mut spritebatch;
        if black{
            let black_image = Image::solid(ctx,CELL_SIZE,BLACK!()).unwrap();
            spritebatch = spritebatch::SpriteBatch::new(black_image);
        }
        else{
            let white_image = Image::solid(ctx,CELL_SIZE,WHITE!()).unwrap();
            spritebatch = spritebatch::SpriteBatch::new(white_image);
        }

        let mut handle_list = Vec::new();
        handle_list.reserve(NUM_BLOCKS_HEIGHT*NUM_BLOCKS_WIDTH);
        // Create x axis first since in graphics first index corresponds with 
        // the column, not row
        for j in 0..NUM_BLOCKS_HEIGHT {
            for i in 0..NUM_BLOCKS_WIDTH{
                let sprite_idx = if black { spritebatch.add(new_cell(i,j))}
                                else {spritebatch.add(new_cell(INVALID_X,INVALID_Y))};
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

//impl MatrixView for FSubview{
    //type Item= spritebatch::SpriteIdx;
    //fn at(&self,i:usize, j:usize)-> GameResult<Self::Item>{
        //if i< NUM_BLOCKS_WIDTH && j < NUM_BLOCKS_HEIGHT{
            //Ok(self.0[j*NUM_BLOCKS_WIDTH as usize + i])
        //}
        //else{
            //Err(GameError::EventLoopError(format!("IndexError: f_subview's i must be less than {} and j must be less than {}",NUM_BLOCKS_WIDTH,NUM_BLOCKS_HEIGHT)))
        //}
    //}
//}

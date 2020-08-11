#![allow(non_snake_case)]
#![warn(clippy::all)]
//! The simplest possible example that does something.

use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::error::GameError;
use ggez::graphics::{DrawMode, Image, Rect,DrawParam};
use ggez::graphics::spritebatch;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};

use nalgebra::geometry::Point2;
use std::ops::{Deref,DerefMut};

// ************  Frontend Globals  ************
const WINDOW_WIDTH: usize = 1920;
const WINDOW_HEIGHT: usize = 1080;

const CELL_SIZE: u16 = 20;
//any smaller and may not print out correctly
const CELL_GAP: f32 = CELL_SIZE as f32 / 6.0;

const NUM_BLOCKS_WIDTH: usize = ((WINDOW_WIDTH as f32 / (CELL_SIZE as f32 + CELL_GAP)) + 1.0) as usize;
const NUM_BLOCKS_HEIGHT: usize = ((WINDOW_HEIGHT as f32 / (CELL_SIZE as f32 + CELL_GAP)) + 1.0) as usize;
const INVALID_X: usize = 2*NUM_BLOCKS_WIDTH;
const INVALID_Y: usize = 2*NUM_BLOCKS_HEIGHT;

// ************  Backend Globals  ************
// Some code may be reliant on this number being bigger than max(NUM_BLOCKS_WIDTH,NUM_BLOCKS_HEIGHT) 
// Unfortunately, Rust currently does not support compile time if
//const GRID_SIZE: usize = 20 + std::cmp::max(NUM_BLOCKS_HEIGHT, NUM_BLOCKS_WIDTH);
const GRID_SIZE: usize = 200;

// ************  Macros  ************
macro_rules! BLACK {
    () => {
        [0.0, 0.0, 0.0, 1.0].into()
    };
}
macro_rules! WHITE {
    () => {
        [1.0, 1.0, 1.0, 1.0].into()
    };
}

fn BLACK() -> [u8; (4*CELL_SIZE*CELL_SIZE) as usize]{
    let mut array = [0; (4*CELL_SIZE*CELL_SIZE) as usize];
    for (i,x) in array.iter_mut().enumerate(){
        if i%4 == 3{
            *x = 255;
        }
    }
    array
}

macro_rules! GREY {
    () => {
        [0.5, 0.5, 0.5, 1.0].into()
    };
}

// has to be on heap otherwise stack overflow
struct BMatrix(Vec<bool>);

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

struct SpriteBatchHandler{
    spritebatch: spritebatch::SpriteBatch,
    // we needs this vec b/c SpriteIdx wraps around a private field
    // so we can't dynamically construct the SpriteIdx ourselves when
    // we want to use the spritebatch.set method
    handle_list: Vec<spritebatch::SpriteIdx>
}
struct FSubview{
    black_sb_handler: SpriteBatchHandler,
    white_sb_handler: SpriteBatchHandler
}

struct Grid {
    b_matrix: BMatrix,
    //mesh: graphics::Mesh,
    f_subview: FSubview,
}

fn new_rect(i: usize, j: usize) -> Rect {
    let i = i as f32;
    let j = j as f32;
    Rect::new(
        i * (CELL_SIZE as f32 + CELL_GAP),
        j * (CELL_SIZE as f32 + CELL_GAP),
        CELL_SIZE as f32,
        CELL_SIZE as f32,
    )
}

fn new_cell(i:usize, j:usize) -> DrawParam{
    DrawParam::default()
        .dest(Point2::new(
                i as f32 * (CELL_SIZE as f32 + CELL_GAP),
                j as f32 * (CELL_SIZE as f32 + CELL_GAP)
                          )
              )
}

enum CellState{
    BLACK,
    WHITE
}

fn create_init_SpriteBatchHandler(state: CellState, ctx:&mut Context) -> SpriteBatchHandler{
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

impl BMatrix{
    fn new()-> Self{
        BMatrix(vec![false;GRID_SIZE*GRID_SIZE])
    }
}
impl Grid {
    // returns a Result object rather than Self b/c creating the image may fail
    fn new(ctx: &mut Context) -> GameResult<Grid> {
        // ************  MESH BUILDER METHOD  ************   
        //let mut mesh = graphics::MeshBuilder::new();

        //for i in 0..NUM_BLOCKS_WIDTH {
            //for j in 0..NUM_BLOCKS_HEIGHT {
                //mesh.rectangle(DrawMode::fill(), new_rect(i, j), BLACK!());
            //}
        //}
        //let mesh = mesh.build(ctx)?;
        //Ok(Grid {
            //matrix: Box::new([[false; GRID_SIZE]; GRID_SIZE]),
            //mesh,
        //})

        // ************  SPRITE METHOD  ************   
        let b_matrix = BMatrix::new();

        //let image = Image::from_rgba8(_ctx, CELL_SIZE as u16, CELL_SIZE as u16,&BLACK());
        let black_sb_handler = create_init_SpriteBatchHandler(CellState::BLACK,ctx);

        let white_sb_handler = create_init_SpriteBatchHandler(CellState::WHITE,ctx);
        
        Ok(Grid{b_matrix, f_subview: FSubview{black_sb_handler,white_sb_handler}})
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

    fn new_cell_value(&self,i:i32, j:i32, count:u32)-> bool{
        let state = self.at(i,j).unwrap();
        match state{
            //dead transition
            false => {if count== 3 {true} else {false}}
            //alive transition
            true => {if count == 2 || count == 3 {true} else {false}}
        }
    }

    fn next_bmatrix(&self)-> BMatrix{
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

trait MatrixView{
    /// i and j are with respect to computer graphics convention
    type Item;
    fn at(&self,i:i32, j:i32)-> GameResult<Self::Item>;
    fn at_mut<'a>(&'a mut self,i:i32, j:i32)-> GameResult<&'a mut Self::Item>;
}
impl MatrixView for BMatrix{
    type Item = bool;
    fn at(&self, i:i32, j:i32) -> GameResult<Self::Item>{
        if i< GRID_SIZE as i32 && j<GRID_SIZE as i32 && i>=0 && j>=0{
            //bool is copy type, so moving is fine
            Ok(self.0[(j*GRID_SIZE as i32 +i) as usize])
        }
        else{
            Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        }
    }
    fn at_mut<'a>(&'a mut self, i:i32, j:i32) -> GameResult<&'a mut Self::Item>{
        if i< GRID_SIZE as i32 && j<GRID_SIZE as i32 && i>=0 && j>=0{
            Ok(&mut self.0[(j*GRID_SIZE as i32 +i) as usize])
        }
        else{
            Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        }
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


impl event::EventHandler for Grid {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.b_matrix.next_bmatrix();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, GREY!());

        graphics::draw(ctx, &self.f_subview.black_sb_handler.spritebatch , (na::Point2::new(0.0, 0.0),))?;

        graphics::draw(ctx, &self.f_subview.white_sb_handler.spritebatch , (na::Point2::new(0.0, 0.0),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
        conf::WindowMode::default()
            .resizable(true)
            .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
    );
    let (ref mut ctx, ref mut event_loop) = cb.build()?;
    let state = &mut Grid::new(ctx)?;
    event::run(ctx, event_loop, state)
}
#[cfg(test)]
mod tests {
    // ************  SETUP  ************   
    use ggez::conf;
    use ggez::{Context, GameResult};
    use ggez::event::EventsLoop;
    use super::*;
    struct Globals{
        ctx: Context,
        event_loop: EventsLoop,
        grid: Grid
    }
    fn setup() -> GameResult<Globals>{
        let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        );
        let (mut ctx ,mut event_loop) = cb.build()?;
        // initialize a Grid object
        let grid = Grid::new(&mut ctx)?;
        Ok(Globals{ctx,event_loop,grid})
    }
    // ************  ACTUAL TESTING  ************   
    #[test]
    fn test_image_black_macro(){
        let array = crate::BLACK();
        for (i,x) in array.iter().enumerate(){
            if i% 4 == 3{
                assert_eq!(x,&255);
            }
            else{
                assert_eq!(x,&0);
            }

            if i > 20{
                break;
            }
        }
    }

    #[test]
    fn test_BMatrix_index_on_subview(){
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
    fn test_BMatrix_at_outOfBounds(){
        println!("HI!!!!!!");
        let globals = setup().unwrap();

        let value = globals.grid.b_matrix.at((2*GRID_SIZE) as i32,0).unwrap();
    }

    #[test]
    fn test_update_bmatrix_single_cell_become_dead(){
        let mut b_matrix = BMatrix::new();
        let i = NUM_BLOCKS_WIDTH-1;
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
        let mut b_matrix = BMatrix::new();
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
        let mut b_matrix = BMatrix::new();
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
        let mut b_matrix = BMatrix::new();
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


    //#[test]
    //fn test_FSubview_at(){
        //let globals = setup().unwrap();

        //// This test is contigent on Grid::new initalizing 
        //// columns first
        //let value = globals.grid.f_subview.at(3,1).unwrap();
        //let expected_value = 1*NUM_BLOCKS_WIDTH + 3;
        //println!("Expected value should be: {}. Actual value is: {:?}",expected_value,value);
        ////assert_eq!(value,1*NUM_BLOCKS_WIDTH+3);
    //}

    //#[test]
    //fn test_draw_off_grid_doesnt_panic(){
        //let mut globals = setup().unwrap();

        //// create modified spritebatch. Note the grid's f_subview will be wrong
        //let image = Image::solid(&mut globals.ctx,CELL_SIZE,BLACK!()).unwrap();
        //let mut f_spritebatch = spritebatch::SpriteBatch::new(image);
        //f_spritebatch.add(new_cell(10,10));
        
        //// 
        //f_spritebatch.add(new_cell(400,100));

        //globals.grid.f_spritebatch = f_spritebatch;
        //println!("Value of globals are:");
        //println!("WINDOW_WIDTH: {}",WINDOW_WIDTH);
        //println!("WINDOW_HEIGHT: {}",WINDOW_HEIGHT);
        //println!("NUM_BLOCKS_WIDTH: {}",NUM_BLOCKS_WIDTH);
        //println!("NUM_BLOCKS_HEIGHT: {}",NUM_BLOCKS_HEIGHT);
        //event::run(&mut globals.ctx, &mut globals.event_loop, &mut globals.grid).unwrap();
    //}
}

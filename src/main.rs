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

// ************  Frontend Globals  ************
const WINDOW_WIDTH: usize = 1920;
const WINDOW_HEIGHT: usize = 1080;

const CELL_SIZE: u16 = 20;
//any smaller and may not print out correctly
const CELL_GAP: f32 = CELL_SIZE as f32 / 6.0;

const NUM_BLOCKS_WIDTH: usize = ((WINDOW_WIDTH as f32 / (CELL_SIZE as f32 + CELL_GAP)) + 1.0) as usize;
const NUM_BLOCKS_HEIGHT: usize = ((WINDOW_HEIGHT as f32 / (CELL_SIZE as f32 + CELL_GAP)) + 1.0) as usize;

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
struct FSubview(Vec<spritebatch::SpriteIdx>);

struct Grid {
    b_matrix: BMatrix,
    //mesh: graphics::Mesh,
    f_spritebatch: spritebatch::SpriteBatch,
    // we needs this vec b/c SpriteIdx wraps around a private field
    // so we can't dynamically construct the SpriteIdx ourselves when
    // we want to use the spritebatch.set method
    f_subview: FSubview
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
        .color(BLACK!())
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
        let b_matrix = BMatrix(vec![false;GRID_SIZE*GRID_SIZE]);

        //let image = Image::from_rgba8(_ctx, CELL_SIZE as u16, CELL_SIZE as u16,&BLACK());
        let image = Image::solid(ctx,CELL_SIZE,BLACK!())?;
        let mut f_spritebatch = spritebatch::SpriteBatch::new(image);

        let mut f_subview = FSubview(Vec::new());
        f_subview.0.reserve(NUM_BLOCKS_HEIGHT*NUM_BLOCKS_WIDTH);
        // Create x axis first since in graphics first index corresponds with 
        // the column, not row
        for j in 0..NUM_BLOCKS_HEIGHT {
            for i in 0..NUM_BLOCKS_WIDTH{
                let sprite_idx = f_spritebatch.add(new_cell(i,j));
                //if j < 2{
                    //println!("{:#?}",sprite_idx);
                //}
                f_subview.0.push(sprite_idx);
                //f_subview.0.push(f_spritebatch.add(new_cell(i,j)));
            }
        }
        Ok(Grid{b_matrix, f_spritebatch,f_subview})
    }
}

trait MatrixView{
    /// i and j are with respect to computer graphics convention
    type Item;
    fn at(&self,i:usize, j:usize)-> GameResult<Self::Item>;
}
impl MatrixView for BMatrix{
    type Item = bool;
    fn at(&self, i:usize, j:usize) -> GameResult<Self::Item>{
        if i< GRID_SIZE && j<GRID_SIZE{
            Ok(self.0[j*GRID_SIZE+i])
        }
        else{
            Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",GRID_SIZE,GRID_SIZE)))
        }
    }
}

impl MatrixView for FSubview{
    type Item= spritebatch::SpriteIdx;
    fn at(&self,i:usize, j:usize)-> GameResult<Self::Item>{
        if i< NUM_BLOCKS_WIDTH && j < NUM_BLOCKS_HEIGHT{
            Ok(self.0[j*NUM_BLOCKS_WIDTH as usize + i])
        }
        else{
            Err(GameError::EventLoopError(format!("IndexError: f_subview's i must be less than {} and j must be less than {}",NUM_BLOCKS_WIDTH,NUM_BLOCKS_HEIGHT)))
        }
    }
}

impl event::EventHandler for Grid {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, GREY!());

        graphics::draw(ctx, &self.f_spritebatch , (na::Point2::new(0.0, 0.0),))?;

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
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

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
        let value = grid.b_matrix.at(GRID_SIZE-1,GRID_SIZE-1).unwrap();
        assert_eq!(value,false);
    }

    #[should_panic]
    #[test]
    fn test_BMatrix_at_outOfBounds(){
        let globals = setup().unwrap();

        let value = globals.grid.b_matrix.at(2*GRID_SIZE,0).unwrap();
    }

    #[test]
    fn test_FSubview_at(){
        let globals = setup().unwrap();

        // This test is contigent on Grid::new initalizing 
        // columns first
        let value = globals.grid.f_subview.at(3,1).unwrap();
        let expected_value = 1*NUM_BLOCKS_WIDTH + 3;
        println!("Expected value should be: {}. Actual value is: {:?}",expected_value,value);
        //assert_eq!(value,1*NUM_BLOCKS_WIDTH+3);
    }

    #[test]
    #[ignore]
    fn test_new_cell_first_arguments_pushes_black_cell_to_the_right(){
        let white = 
        let black = 
    }
}

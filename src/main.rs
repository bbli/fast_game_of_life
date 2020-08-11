#![allow(non_snake_case)]
#![warn(clippy::all)]
//! The simplest possible example that does something.

use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::error::GameError;
use ggez::graphics::{DrawMode, Image, Rect,DrawParam,BlendMode};
use ggez::{Context, GameResult};
use rand::prelude::*;


mod bmatrix;
use bmatrix::*;

mod fsubview;
use fsubview::*;
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
const GRID_SIZE: i32 = 120;
const INVALID_X: i32 = 2*GRID_SIZE;
const INVALID_Y: i32 = 2*GRID_SIZE;

// ************  Macros  ************
#[macro_export]
macro_rules! BLACK {
    () => {
        [0.0, 0.0, 0.0, 1.0].into()
    };
}
#[macro_export]
macro_rules! WHITE {
    () => {
        [1.0, 1.0, 1.0, 1.0].into()
    };
}

//fn BLACK() -> [u8; (4*CELL_SIZE*CELL_SIZE) as usize]{
    //let mut array = [0; (4*CELL_SIZE*CELL_SIZE) as usize];
    //for (i,x) in array.iter_mut().enumerate(){
        //if i%4 == 3{
            //*x = 255;
        //}
    //}
    //array
//}

#[macro_export]
macro_rules! GREY {
    () => {
        [0.5, 0.5, 0.5, 1.0].into()
    };
}

// ************  MAIN CODE  ************   

struct Grid {
    b_matrix: BMatrix,
    //mesh: graphics::Mesh,
    f_subview: FSubview,
    f_offset: (f32,f32)
}

//fn new_rect(i: usize, j: usize) -> Rect {
    //let i = i as f32;
    //let j = j as f32;
    //Rect::new(
        //i * (CELL_SIZE as f32 + CELL_GAP),
        //j * (CELL_SIZE as f32 + CELL_GAP),
        //CELL_SIZE as f32,
        //CELL_SIZE as f32,
    //)
//}


impl Grid {
    // returns a Result object rather than Self b/c creating the image may fail
    fn new(ctx: &mut Context) -> GameResult<Grid> {
        // ************  MESH BUILDER METHOD  ************   
        //let mut mesh = graphics::MeshBuilder::new();

        //for i in 0..GRID_SIZE {
            //for j in 0..GRID_SIZE {
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

        let white_image = Image::solid(ctx,CELL_SIZE,WHITE!()).unwrap();
        let white_sb_handler = create_init_SpriteBatchHandler(white_image,ctx);

        // this should override the white color set above
        let black_image = Image::solid(ctx,CELL_SIZE,BLACK!()).unwrap();
        let black_sb_handler = create_init_SpriteBatchHandler(black_image,ctx);
        
        Ok(Grid{
            b_matrix, 
            f_subview: FSubview{black_sb_handler,white_sb_handler},
            f_offset: (0.0,0.0)
            }
        )
    }

    fn init_seed(mut self, init_bmatrix: BMatrix) -> Self{
        self.b_matrix = init_bmatrix;
        self
    }

    fn set_offset(mut self, x:f32, y:f32) -> Self{
        self.f_offset = (x,y);
        self
    }

    fn update_view(&mut self){
        for j in 0..GRID_SIZE{
            for i in 0..GRID_SIZE{
                if self.b_matrix.at(i,j).unwrap(){
                    self.f_subview.change_to_black(i,j);
                }
                else{
                    self.f_subview.change_to_white(i,j);
                }
            }
        }
    }
}


trait MatrixView{
    /// i and j are with respect to computer graphics convention
    type Item;
    fn at(&self,i:i32, j:i32)-> GameResult<Self::Item>;
    fn at_mut<'a>(&'a mut self,i:i32, j:i32)-> GameResult<&'a mut Self::Item>;
}


impl event::EventHandler for Grid {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.b_matrix = self.b_matrix.next_bmatrix();
        // use update b_matrix to update view
        self.update_view();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, GREY!());

        self.f_subview.draw(ctx,self.f_offset)?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    // ************  GRID  ************   
    let mut init_bmatrix = BMatrix::new();
    // Blinker pattern
    let mut rng = rand::thread_rng();
    for j in 0..GRID_SIZE{
        for i in 0..GRID_SIZE{
            if rand::random(){
                *init_bmatrix.at_mut(i as i32,j as i32).unwrap() =true;
            }
        }
    }
    // ************  GGEZ  ************   
    let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
        conf::WindowMode::default()
            .resizable(true)
            .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
    );

    // ************  RUNNING  ************   
    let (ref mut ctx, ref mut event_loop) = cb.build()?;
    graphics::set_blend_mode(ctx,BlendMode::Replace);
    //let origin_point = (GRID_SIZE/2) as f32;
    let origin_point = 0.0 as f32;
    let ref mut state = Grid::new(ctx)?.init_seed(init_bmatrix).set_offset(-origin_point,-origin_point);
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
    //#[test]
    //fn test_image_black_macro(){
        //let array = crate::BLACK();
        //for (i,x) in array.iter().enumerate(){
            //if i% 4 == 3{
                //assert_eq!(x,&255);
            //}
            //else{
                //assert_eq!(x,&0);
            //}

            //if i > 20{
                //break;
            //}
        //}
    //}

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
        //let expected_value = 1*GRID_SIZE + 3;
        //println!("Expected value should be: {}. Actual value is: {:?}",expected_value,value);
        ////assert_eq!(value,1*GRID_SIZE+3);
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
        //println!("GRID_SIZE: {}",GRID_SIZE);
        //event::run(&mut globals.ctx, &mut globals.event_loop, &mut globals.grid).unwrap();
    //}
}

#![allow(non_snake_case)]
#![warn(clippy::all)]
//! The simplest possible example that does something.

use ggez::conf;
use ggez::event;
use ggez::event::KeyCode;
use ggez::graphics;
use ggez::input::keyboard;
use ggez::error::GameError;
use ggez::graphics::{DrawMode, Image, Rect,DrawParam,BlendMode};
use ggez::{Context, GameResult};

//use std::{thread,time};
use std::time::{SystemTime,UNIX_EPOCH};

mod bmatrix;
use bmatrix::*;

mod fsubview;
use fsubview::FSubview;

mod user;
use user::OffsetState;

mod setup;
//use fsubview;
// ************  Frontend Globals  ************
const WINDOW_WIDTH: usize = 1920;
const WINDOW_HEIGHT: usize = 1080;

const CELL_SIZE: f32 = 20.0;
//any smaller and may not print out correctly
const CELL_GAP: f32 = CELL_SIZE / 6.0;
const EPSILON: f32 = 1e-2f32;

// ************  Backend Globals  ************
// 1. Some code may be reliant(update_view) on this number being bigger than max(NUM_BLOCKS_WIDTH,NUM_BLOCKS_HEIGHT), which is an approximation to worst case scenario, whose function is also provided below
// ```
// fn get_worst_case_num_of_blocks(side: usize) -> usize{
    //let distance_left_to_pack_on_one_side: f32 = side as f32/2.0 - CELL_SIZE as f32/2.0;

    //let mut num_of_blocks_that_fit = (distance_left_to_pack_on_one_side/(CELL_SIZE as f32+CELL_GAP)) as i32;
    //num_of_blocks_that_fit += 1;

    //(1+ 2*num_of_blocks_that_fit) as usize
    //}
// ```

// 2. Base off exact formula above, +3/4 will be safe
//const NUM_BLOCKS_WIDTH: usize = ((WINDOW_WIDTH as f32 / (CELL_SIZE as f32 + CELL_GAP)) + 3.0) as usize;
//const NUM_BLOCKS_HEIGHT: usize = ((WINDOW_HEIGHT as f32 / (CELL_SIZE as f32 + CELL_GAP)) + 3.0) as usize;

// 3. Unfortunately, Rust currently does not support compile time if,
// so just hardcode a number for the grid size
//const GRID_SIZE: usize >= std::cmp::max(NUM_BLOCKS_HEIGHT, NUM_BLOCKS_WIDTH);
static GRID_SIZE: i32 = 200;


//const INVALID_X: i32 = 2*GRID_SIZE;
//const INVALID_Y: i32 = 2*GRID_SIZE;

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

#[derive(Clone,Copy)]
pub struct Point{
    x: f32,
    y: f32
}
impl Point{
    pub fn new(x:f32,y:f32)->Point{
        Point{x,y}
    }
}

struct Grid {
    b_matrix: BMatrix,
    sys_time: Option<SystemTime>,
    //mesh: graphics::Mesh,
    f_subview: FSubview,
    f_offset: OffsetState
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
        let sys_time = None;
        //let image = Image::from_rgba8(_ctx, CELL_SIZE as u16, CELL_SIZE as u16,&BLACK());

        let white_image = Image::solid(ctx,CELL_SIZE as u16,WHITE!()).unwrap();
        let white_sb_handler = fsubview::create_init_SpriteBatchHandler(white_image,ctx);

        // this should override the white color set above
        let black_image = Image::solid(ctx,CELL_SIZE as u16,BLACK!()).unwrap();
        let black_sb_handler = fsubview::create_init_SpriteBatchHandler(black_image,ctx);
        
        Ok(Grid{
            b_matrix, 
            sys_time,
            f_subview: FSubview{black_sb_handler,white_sb_handler,relative_offset: Point{x:0.0,y:0.0}},
            f_offset: OffsetState::default()
            }
        )
    }

    fn init_seed(mut self, init_bmatrix: BMatrix) -> Self{
        self.b_matrix = init_bmatrix;
        self
    }

    // NOTE: Please initialize to a region inside
    fn init_offset(mut self, x:f32, y:f32) -> Self{
        self.f_offset = OffsetState::Inside(Point::new(x,y));
        self
    }

    fn update_view(&mut self)->GameResult{
        // 1. get bounding boxes
        // TODO: remove divide by 2 once testing of offset/user moving is complete
        let offset_point = self.f_offset.get_point();
        let top_idx = fsubview::get_base_index_top(offset_point.y);
        //let bottom_idx = get_base_index_bottom(self.f_offset.y+WINDOW_HEIGHT as f32/2.0);
        let bottom_idx = fsubview::get_base_index_bottom(offset_point.y+WINDOW_HEIGHT as f32);

        let left_idx = fsubview::get_base_index_left(offset_point.x);
        let right_idx = fsubview::get_base_index_right(offset_point.x+WINDOW_WIDTH as f32);

        //println!("Top idx: {}",top_idx);
        //println!("Bottom idx: {}",bottom_idx);
        //println!("Left idx: {}",left_idx);
        //println!("right idx: {}",right_idx);

        // 2. now draw from base_index_top -> base_index_bottom, inclusive
        self.f_subview.clear();
        for j in top_idx..bottom_idx+1{
            let relative_j = j - top_idx;
            for i in left_idx..right_idx+1{
                let relative_i = i - left_idx;

                if self.b_matrix.at(i,j)?{
                    //self.f_subview.change_to_white(i,j);
                    self.f_subview.add_to_white(relative_i,relative_j);
                }
                else{
                    //self.f_subview.change_to_black(i,j);
                    self.f_subview.add_to_black(relative_i,relative_j);
                }
            }
        }
        // 3. finally define new relative offset
        // aka relative to the box at (left_idx,top_idx)
        let rel_offset_y = fsubview::get_distance_to_top(offset_point.y,top_idx)?;
        let rel_offset_x = fsubview::get_distance_to_left(offset_point.x,left_idx)?;
        self.f_subview.update_relative_offset(rel_offset_x,rel_offset_y);
        Ok(())
    }

    fn update_offset(&mut self, ctx: &mut Context){
        if keyboard::is_key_pressed(ctx,KeyCode::Right){
            self.f_offset = user::transition_offset_state_right(self.f_offset);
        }
        if keyboard::is_key_pressed(ctx,KeyCode::Left){
            self.f_offset = user::transition_offset_state_left(self.f_offset);
        }
        if keyboard::is_key_pressed(ctx,KeyCode::Up){
            self.f_offset = user::transition_offset_state_up(self.f_offset);
        }
        if keyboard::is_key_pressed(ctx,KeyCode::Down){
            self.f_offset = user::transition_offset_state_down(self.f_offset);
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
        self.sys_time = Some(SystemTime::now());
        //let ten_seconds = time::Duration::from_secs(10);
        //thread::sleep(ten_seconds);

        self.b_matrix = self.b_matrix.next_bmatrix();
        self.update_offset(_ctx);
        // use update b_matrix to update view
        self.update_view()?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, GREY!());
        self.f_subview.draw(ctx)?;
        graphics::present(ctx)?;

        let time_lapse = self.sys_time.unwrap().elapsed().expect("System clock did something funny");
        println!("Time elapsed: {}ms",time_lapse.as_millis());
        Ok(())
    }
}

pub fn main() -> GameResult {
    //let NUM_BLOCKS_WIDTH:usize = get_worst_case_num_of_blocks(WINDOW_WIDTH);
    //let NUM_BLOCKS_HEIGHT:usize = get_worst_case_num_of_blocks(WINDOW_HEIGHT);
    //println!("NUM_BLOCKS_WIDTH: {}",NUM_BLOCKS_WIDTH);
    //println!("NUM_BLOCKS_HEIGHT: {}",NUM_BLOCKS_HEIGHT);
    //println!("NUM_BLOCKS_WIDTH2: {}",NUM_BLOCKS_WIDTH2);
    //println!("NUM_BLOCKS_HEIGHT2: {}",NUM_BLOCKS_HEIGHT2);

    // ************  GRID  ************   
    let mut init_bmatrix = BMatrix::new();
    setup::make_random(&mut init_bmatrix);
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
    let ref mut state = Grid::new(ctx)?.init_seed(init_bmatrix).init_offset(origin_point,origin_point);
    event::run(ctx, event_loop, state)
}
#[cfg(test)]
mod tests {
    // ************  SETUP  ************   
    use ggez::conf;
    use ggez::{Context, GameResult};
    use ggez::event::EventsLoop;
    use super::*;
    use assert_approx_eq::assert_approx_eq;

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

    #[test]
    fn test_bounding_space_vertical(){
        let height = CELL_SIZE/2.0+CELL_GAP/2.0;
        let offset_y = (2.0*(CELL_SIZE+CELL_GAP)) - CELL_GAP/2.0;
        // so from variables above, we know ending should be on empty space
        let top_idx = fsubview::get_base_index_top(offset_y);
        let bottom_idx = fsubview::get_base_index_bottom(offset_y+height);
        assert_eq!(top_idx,1);
        assert_eq!(bottom_idx,2);
    }

    #[test]
    fn test_bounding_space_horizontal(){
        let width = CELL_SIZE+CELL_GAP+CELL_SIZE/2.0+CELL_GAP/2.0;
        let offset_x = (CELL_SIZE+CELL_GAP) + CELL_SIZE/2.0;
        // so from variables above, we know ending should be on empty space
        let left_idx = fsubview::get_base_index_left(offset_x);
        let right_idx = fsubview::get_base_index_right(offset_x+width);
        assert_eq!(left_idx,1);
        assert_eq!(right_idx,3);
    }

    #[test]
    fn test_bounding_space_edge_case_at_origin_x(){
        let width = 2.0*(CELL_SIZE+CELL_GAP)+CELL_SIZE/2.0;
        // so from above, we know ending should be on empty space
        let left_idx = fsubview::get_base_index_left(0.0);
        let right_idx = fsubview::get_base_index_right(0.0+width);
        assert_eq!(left_idx,0);
        assert_eq!(right_idx,2);
    }

    #[test]
    fn test_bounding_space_edge_case_at_origin_y(){
        let height = 2.0*(CELL_SIZE+CELL_GAP)+CELL_SIZE/2.0;
        // so from above, we know ending should be on empty space
        let top_idx = fsubview::get_base_index_top(0.0);
        let bottom_idx = fsubview::get_base_index_bottom(0.0+height);
        assert_eq!(top_idx,0);
        assert_eq!(bottom_idx,2);
    }
    #[test]
    fn test_bounding_space_edge_case_at_max_offset_x(){
        let width = WINDOW_WIDTH;
        let offset_x = user::get_max_offset_x();

        let right_idx = fsubview::get_base_index_right(offset_x+width as f32);
        assert_eq!(right_idx,(GRID_SIZE-1) as i32);
    }

    #[test]
    fn test_bounding_space_edge_case_at_max_offset_y(){
        let height = WINDOW_HEIGHT;
        let offset_y = user::get_max_offset_y();

        let bottom_idx = fsubview::get_base_index_right(offset_y+height as f32);
        assert_eq!(bottom_idx,(GRID_SIZE-1) as i32);
    }

    #[test]
    fn test_get_distance_to_top_inside(){
        let offset_y = CELL_SIZE+CELL_GAP+CELL_SIZE/3.0;
        // based off variable above
        let top_idx = 1;
        assert_approx_eq!(fsubview::get_distance_to_top(offset_y,top_idx).unwrap(), CELL_SIZE/3.0,1e-3f32);
    }

    #[test]
    fn test_get_distance_to_top_empty(){
        let offset_y = CELL_SIZE+CELL_GAP/3.0;
        // based off variable above
        let top_idx = 0;
        assert_approx_eq!(fsubview::get_distance_to_top(offset_y,top_idx).unwrap(), CELL_SIZE+CELL_GAP/3.0,1e-3f32);
    }
    #[test]
    fn test_get_distance_to_left_inside(){
        let offset_x = CELL_SIZE+CELL_GAP+CELL_SIZE/2.0;
        // based off variable above
        let left_idx = 1;
        assert_approx_eq!(fsubview::get_distance_to_left(offset_x,left_idx).unwrap(), CELL_SIZE/2.0,1e-3f32);
    }
    #[test]
    fn test_get_distance_to_left_empty(){
        let offset_x = CELL_SIZE+CELL_GAP/2.5;
        // based off variable above
        let left_idx = 0;
        assert_approx_eq!(fsubview::get_distance_to_left(offset_x,left_idx).unwrap(), CELL_SIZE+CELL_GAP/2.5,1e-3f32);
    }

    #[test]
    #[ignore]
    fn test_update_view_before_shift(){
        // NOTE: turn off next_bmatrix() before executing this
        let mut init_bmatrix = BMatrix::new();
        for j in 0..GRID_SIZE{
            for i in 0..GRID_SIZE{
                //make_blinker(i,j,&mut init_bmatrix);
                //make_square(i,j,&mut init_bmatrix);
                if i>GRID_SIZE/2{
                    *init_bmatrix.at_mut(i,j).unwrap() = true;
                }
            }
        }

        let ref mut globals = setup().unwrap();
        globals.grid.b_matrix = init_bmatrix;
        event::run(&mut globals.ctx,&mut globals.event_loop,&mut globals.grid);

    }

    #[test]
    #[ignore]
    fn test_update_view_after_shift(){
        // NOTE: turn off next_bmatrix() before executing this
        println!("GRID_SIZE: {}",GRID_SIZE);
        let mut init_bmatrix = BMatrix::new();
        // just make part of the screen white
        for j in 0..GRID_SIZE{
            for i in 0..GRID_SIZE{
                //make_blinker(i,j,&mut init_bmatrix);
                //make_square(i,j,&mut init_bmatrix);
                if i>GRID_SIZE/2{
                    *init_bmatrix.at_mut(i,j).unwrap() = true;
                }
            }
        }

        let mut globals = setup().unwrap();
        globals.grid = globals.grid.init_offset(user::get_max_offset_x(),0.0);
        globals.grid.b_matrix = init_bmatrix;
        event::run(&mut globals.ctx,&mut globals.event_loop,&mut globals.grid);
    }

    #[test]
    #[ignore]
    fn test_transition_bottom_right_corner(){
        // ************  GRID  ************   
        let mut init_bmatrix = BMatrix::new();
        setup::make_random(&mut init_bmatrix);
        // ************  GGEZ  ************   
        let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        );

        // ************  RUNNING  ************   
        let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();
        graphics::set_blend_mode(ctx,BlendMode::Replace);
        //let origin_point = (GRID_SIZE/2) as f32;
        let ref mut state = Grid::new(ctx).unwrap().init_seed(init_bmatrix).init_offset(user::get_max_offset_x()-5.0,user::get_max_offset_y()-5.0);
        event::run(ctx, event_loop, state);
    }

    #[test]
    #[ignore]
    fn test_transition_top_left_corner(){
        // ************  GRID  ************   
        let mut init_bmatrix = BMatrix::new();
        setup::make_random(&mut init_bmatrix);
        // ************  GGEZ  ************   
        let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        );

        // ************  RUNNING  ************   
        let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();
        graphics::set_blend_mode(ctx,BlendMode::Replace);
        //let origin_point = (GRID_SIZE/2) as f32;
        let ref mut state = Grid::new(ctx).unwrap().init_seed(init_bmatrix).init_offset(0.0,0.0);
        event::run(ctx, event_loop, state);
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

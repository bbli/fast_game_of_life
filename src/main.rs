#![cfg_attr(test, feature(proc_macro_hygiene))]

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
use std::ops::Deref;
use threadpool::ThreadPool;

mod bmatrix;
use bmatrix::*;

mod fsubview;
use fsubview::FSubview;

mod user;
use user::OffsetState;

mod setup;
//use fsubview;
#[cfg(test)]
use mocktopus::macros::*;
// ************  Frontend Globals  ************
const WINDOW_WIDTH: usize = 1920;
const WINDOW_HEIGHT: usize = 1080;

const CELL_SIZE: f32 = 20.0;
//any smaller and may not print out correctly
const CELL_GAP: f32 = CELL_SIZE / 6.0;
const EPSILON: f32 = 1e-2f32;

//const SW_HORIZONTAL_SECTIONS:i32 = ((CELL_SIZE+CELL_GAP+WINDOW_WIDTH as f32)/(CELL_SIZE+CELL_GAP)).ceil() as i32;

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
//const NUM_BLOCKS_WIDTH: i32 = ((WINDOW_WIDTH as f32 / (CELL_SIZE as f32 + CELL_GAP)) + 3.0) as i32;
//const NUM_BLOCKS_HEIGHT: i32 = ((WINDOW_HEIGHT as f32 / (CELL_SIZE as f32 + CELL_GAP)) + 3.0) as i32;

// 3. Unfortunately, Rust currently does not support compile time if,
// so just hardcode a number for the grid size
//const GRID_SIZE: usize >= std::cmp::max(NUM_BLOCKS_HEIGHT, NUM_BLOCKS_WIDTH);
const GRID_SIZE: i32 = 2000;


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

#[derive(Clone,Copy)]
pub struct Point{
    x: f32,
    y: f32
}
impl Point{
    pub fn new(x:f32,y:f32)->Point{
        if x < 0.0 || y < 0.0{
            panic!("Point needs to be positive in both dimensions");
        }
        Point{x,y}
    }
}

struct RegionPool{
    threadpool: ThreadPool,
    work_region_list: Vec<(i32,i32)>
}

pub struct Grid {
    b_matrix: BMatrixVector,
    sys_time: Option<SystemTime>,
    region_pool: RegionPool,
    f_subview: FSubview,
    // TODO: change name to user
    f_user_offset: OffsetState,
}

fn new_rect(i: i32, j: i32) -> Rect {
    let i = i as f32;
    let j = j as f32;
    Rect::new(
        i * (CELL_SIZE as f32 + CELL_GAP),
        j * (CELL_SIZE as f32 + CELL_GAP),
        CELL_SIZE as f32,
        CELL_SIZE as f32,
    )
}
// Split the grid horizontally so that each thread can index like
// for j in (start_y,end_y)
fn partition_grid(threads: i32)->Vec<(i32,i32)>{
    let mut work_region_list = Vec::new();
    let num_rows = GRID_SIZE/threads;
    for i in 0..threads{
        let start_y;
        let end_y;
        if i == threads-1{
            start_y = i*num_rows;
            end_y = GRID_SIZE-1;
        }
        else{
            start_y = i*num_rows;
            end_y = (i+1)*num_rows -1;
        }
        work_region_list.push((start_y,end_y))
    }
    work_region_list
}

impl Default for RegionPool{
    fn default()-> Self{
        let worker_count:i32 = 8;
        //let threadpool = Arc::new(ThreadPool::new(worker_count as usize));
        let threadpool = ThreadPool::new(worker_count as usize);
        let work_region_list = partition_grid(worker_count);
        RegionPool{
            threadpool,
            work_region_list
        }
    }
}

impl Deref for RegionPool{
    type Target = ThreadPool;
    fn deref(&self) -> &Self::Target{
        &self.threadpool
    }
}


//#[mockable]
impl Grid {
    // returns a Result object rather than Self b/c creating the image may fail
    fn new(ctx: &mut Context) -> GameResult<Grid> {
        
        let b_matrix = BMatrixVector::new();
        let sys_time = None;
        let f_subview = FSubview::new(ctx)?;
        let f_user_offset = OffsetState::default();
        let region_pool = RegionPool::default();
    
        Ok(Grid{
            b_matrix, 
            sys_time,
            region_pool,
            f_subview,
            f_user_offset
            }
        )
    }

    fn init_seed(mut self, init_bmatrix: BMatrixVector) -> Self{
        self.b_matrix = init_bmatrix;
        self
    }

    // NOTE: Please initialize to a region inside
    fn init_offset(mut self, x:f32, y:f32) -> Self{
        self.f_user_offset = OffsetState::Inside(Point::new(x,y));
        self
    }

    fn updateBackend(&mut self){
        // 1. Single Threaded
        //self.b_matrix = self.b_matrix.next_bmatrix();
        // 2. Multi-Threaded with Rayon
        //self.b_matrix = self.b_matrix.next_bmatrix_rayon();
        // 3. Multi-Threaded with ThreadPool
        self.b_matrix = self.b_matrix.next_bmatrix_threadpool(&self.region_pool)
    }

    // Invariant Sliding Window Version
    fn update_view(&mut self, ctx: &mut Context) -> GameResult{
        // 1. get bounding boxes
        let offset_point = self.f_user_offset.get_point();
        let (left_idx,right_idx) = self.f_subview.get_horizontal_window_range(offset_point.x,offset_point.x+WINDOW_WIDTH as f32);
        let (top_idx,bottom_idx) = self.f_subview.get_vertical_window_range(offset_point.y,offset_point.y+WINDOW_HEIGHT as f32);

        // 2. now draw from base_index_top -> base_index_bottom, inclusive
        self.f_subview.startView();
        for j in top_idx..bottom_idx+1{
            let relative_j = j - top_idx;
            for i in left_idx..right_idx+1{
                let relative_i = i - left_idx;

                if self.b_matrix.at(i,j)?{
                    //self.f_subview.change_to_white(i,j);
                    self.f_subview.addWhiteToView(relative_i,relative_j);
                }
                else{
                    //self.f_subview.change_to_black(i,j);
                    self.f_subview.addBlackToView(relative_i,relative_j);
                }
            }
        }
        self.f_subview.endView(ctx);

        // 3. finally define new relative offset
        // aka relative to the box at (left_idx,top_idx)
        let rel_offset_y = fsubview::get_distance_to_top(offset_point.y,top_idx)?;
        let rel_offset_x = fsubview::get_distance_to_left(offset_point.x,left_idx)?;
        self.f_subview.update_relative_offset(rel_offset_x,rel_offset_y);
        Ok(())
    }

    // Smallest Bounding Sliding Window Version
    //fn update_view(&mut self,ctx:&mut Context)->GameResult{
        //// 1. get bounding boxes
        //let offset_point = self.f_user_offset.get_point();
        //let (top_idx,bottom_idx) = fsubview::get_vertical_range_of_view(offset_point.y);
        //let (left_idx,right_idx) = fsubview::get_horizontal_range_of_view(offset_point.x);

        //println!("Vertical Range: {}",bottom_idx-top_idx);
        //println!("Horizontal Range: {}",right_idx-left_idx);
        ////println!("Top idx: {}",top_idx);
        ////println!("Bottom idx: {}",bottom_idx);
        ////println!("Left idx: {}",left_idx);
        ////println!("right idx: {}",right_idx);

        //// 2. now draw from base_index_top -> base_index_bottom, inclusive
        //self.f_subview.startView();
        //for j in top_idx..bottom_idx+1{
            //let relative_j = j - top_idx;
            //for i in left_idx..right_idx+1{
                //let relative_i = i - left_idx;

                //if self.b_matrix.at(i,j)?{
                    ////self.f_subview.change_to_white(i,j);
                    //self.f_subview.addWhiteToView(relative_i,relative_j);
                //}
                //else{
                    ////self.f_subview.change_to_black(i,j);
                    //self.f_subview.addBlackToView(relative_i,relative_j);
                //}
            //}
        //}
        //self.f_subview.endView(ctx);
        //// 3. finally define new relative offset
        //// aka relative to the box at (left_idx,top_idx)
        //let rel_offset_y = fsubview::get_distance_to_top(offset_point.y,top_idx)?;
        //let rel_offset_x = fsubview::get_distance_to_left(offset_point.x,left_idx)?;
        //self.f_subview.update_relative_offset(rel_offset_x,rel_offset_y);
        //Ok(())
    //}

    fn update_offset(&mut self, ctx: &mut Context){
        if keyboard::is_key_pressed(ctx,KeyCode::Right){
            self.f_user_offset = user::transition_offset_state_right(self.f_user_offset);
        }
        if keyboard::is_key_pressed(ctx,KeyCode::Left){
            self.f_user_offset = user::transition_offset_state_left(self.f_user_offset);
        }
        if keyboard::is_key_pressed(ctx,KeyCode::Up){
            self.f_user_offset = user::transition_offset_state_up(self.f_user_offset);
        }
        if keyboard::is_key_pressed(ctx,KeyCode::Down){
            self.f_user_offset = user::transition_offset_state_down(self.f_user_offset);
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
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.sys_time = Some(SystemTime::now());
        //let ten_seconds = time::Duration::from_secs(10);
        //thread::sleep(ten_seconds);

        self.updateBackend();

        self.update_offset(ctx);
        // use update b_matrix to update view
        self.update_view(ctx)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, GREY!());
        self.f_subview.drawView(ctx)?;
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
    let mut init_bmatrix = BMatrixVector::new();
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
    pub use ggez::conf;
    pub use ggez::{Context, GameResult};
    pub use ggez::event::EventsLoop;
    pub use super::*;
    pub use assert_approx_eq::assert_approx_eq;

    pub struct Globals{
        pub ctx: Context,
        pub event_loop: EventsLoop,
        pub grid: Grid
    }
    pub fn setup() -> GameResult<Globals>{
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
    #[ignore]
    fn test_update_view_before_offset(){
        // NOTE: turn off next_bmatrix() before executing this
        let mut init_bmatrix = BMatrixVector::new();
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
    fn test_update_view_after_offset(){
        // NOTE: turn off next_bmatrix() before executing this
        println!("GRID_SIZE: {}",GRID_SIZE);
        let mut init_bmatrix = BMatrixVector::new();
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

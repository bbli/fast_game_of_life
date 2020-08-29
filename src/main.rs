#![cfg_attr(test, feature(proc_macro_hygiene))]
#![allow(non_snake_case)]
#![warn(clippy::all)]
//#![feature(sync)]

use ggez::error::GameError;
use ggez::event::KeyCode;
use ggez::graphics::{BlendMode, DrawParam, Image};
use ggez::input::keyboard;
use ggez::{conf, event, graphics};
use ggez::{Context, GameResult};

use std::{thread,time};
use std::ops::{Deref, DerefMut};

mod b_matrix;
use b_matrix::*;

mod fsubview;
use fsubview::FSubview;

mod user;
use user::{OffsetState, Point};

mod patterns;

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
// 1. Some code may be reliant(sync_update_view) on this number being bigger than max(NUM_BLOCKS_WIDTH,NUM_BLOCKS_HEIGHT), which is an approximation to worst case scenario, whose function is also provided below
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
const GRID_SIZE: i32 = 2000; // Size is probably bigger than you computer screen. For reference, on my 1920 wide laptop, it will fit about 82 cells across

const INVALID_X: i32 = 2 * GRID_SIZE;
const INVALID_Y: i32 = 2 * GRID_SIZE;

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

#[macro_export]
macro_rules! GREY {
    () => {
        [0.5, 0.5, 0.5, 1.0].into()
    };
}

// ************  MAIN CODE  ************

pub struct Grid {
    b_matrix: BMatrix,
    f_subview: FSubview,
    f_user_offset: OffsetState,
}
//#[mockable]
impl Grid {
    // returns a Result object rather than Self b/c creating the image may fail
    fn new(ctx: &mut Context, update_method: Backend) -> GameResult<Grid> {
        let b_matrix = BMatrix::new(update_method);
        let f_subview = FSubview::new(ctx)?;
        let f_user_offset = OffsetState::default();

        Ok(Grid {
            b_matrix,
            f_subview,
            f_user_offset,
        })
    }

    fn init_seed(mut self, init_b_matrix_vector: BMatrixVector) -> Self {

        let mut new_vec_lock = self.b_matrix.new_vec.grab_lock();
        let new_vec_raw: &mut BMatrixVector = new_vec_lock.deref_mut();

        let mut vec_lock = self.b_matrix.vec.grab_writer_lock();
        let vec_raw: &mut BMatrixVector = vec_lock.deref_mut();
        *new_vec_raw = init_b_matrix_vector.clone();
        *vec_raw = init_b_matrix_vector;

        // Have to explicitly drop b/c self can't return until there are no borrows
        std::mem::drop(new_vec_lock);
        std::mem::drop(vec_lock);
        self
    }

    // NOTE: Please initialize to a region inside
    fn init_offset(mut self, x: f32, y: f32) -> Self {
        if x > 0.0 && y > 0.0 {
            self.f_user_offset = OffsetState::Inside(Point::new(x, y));
            self
        } else {
            panic!("Please set initial offset to be positive. Default is at (0.0,0.0)");
        }
    }

    // Invariant Sliding Window Version
    fn sync_update_view(&mut self, ctx: &mut Context) -> GameResult {
        // 0. Extracting updated b_matrix_vector
        let vec_lock = self.b_matrix.vec.grab_reader_lock();
        let vec_raw = vec_lock.deref();
        // 1. get bounding boxes
        let offset_point = self.f_user_offset.get_point();
        let (left_idx, right_idx) = self
            .f_subview
            .get_horizontal_window_range(offset_point.x, offset_point.x + WINDOW_WIDTH as f32);
        let (top_idx, bottom_idx) = self
            .f_subview
            .get_vertical_window_range(offset_point.y, offset_point.y + WINDOW_HEIGHT as f32);

        // 2. now draw from base_index_top -> base_index_bottom, inclusive
        self.f_subview.startView();
        for j in top_idx..bottom_idx + 1 {
            let relative_j = j - top_idx;
            for i in left_idx..right_idx + 1 {
                let relative_i = i - left_idx;

                if vec_raw.at(i, j)? {
                    //self.f_subview.change_to_white(i,j);
                    self.f_subview.addWhiteToView(relative_i, relative_j);
                } else {
                    //self.f_subview.change_to_black(i,j);
                    self.f_subview.addBlackToView(relative_i, relative_j);
                }
            }
        }
        self.f_subview.endView(ctx);

        // 3. finally define new relative offset
        // aka relative to the box at (left_idx,top_idx)
        let rel_offset_y = fsubview::get_distance_to_top(offset_point.y, top_idx)?;
        let rel_offset_x = fsubview::get_distance_to_left(offset_point.x, left_idx)?;
        self.f_subview
            .update_relative_offset(rel_offset_x, rel_offset_y);
        Ok(())
    }
}

trait MatrixView {
    /// i and j are with respect to computer graphics convention
    type Item;
    fn at(&self, i: i32, j: i32) -> GameResult<Self::Item>;
    fn at_mut(&mut self, i: i32, j: i32) -> GameResult<&mut Self::Item>;
}

impl event::EventHandler for Grid {
    fn update(&mut self, ctx: &mut Context) -> GameResult {

        self.b_matrix.sync_main_update_backend();

        self.f_user_offset.update(ctx);
        // use updated b_matrix and offset to update view
        self.sync_update_view(ctx)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        //we should sleep as otherwise we spend too much time redrawing
        let time = time::Duration::from_millis(10);
        thread::sleep(time);

        graphics::clear(ctx, GREY!());
        self.f_subview.drawView(ctx)?;
        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    // ************  GRID  ************
    // NOTE: all patterns start drawing from the top leftmost corner of the
    // "smallest bounding rectangle" of the pattern
    let start_point = (0, 150);
    let mut init_b_matrix_vector = patterns::PatternBuilder::new()
        //.make_square(0,0)
        //.make_blinker(5,5)
        //.make_t(12,12)
        //.make_r_pentomino(30,30)
        //.make_glider(60,60)
        //.make_random(start_point,400,500)
        .make_random((0, 0), GRID_SIZE, GRID_SIZE)
        .build();
    // ************  GGEZ  ************
    let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
        conf::WindowMode::default()
            .resizable(true)
            .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
    );

    // ************  RUNNING  ************
    let (ref mut ctx, ref mut event_loop) = cb.build()?;
    graphics::set_blend_mode(ctx, BlendMode::Replace)?;

    // default start at (0,0), but can change if you want
    // Note these numbers must be positive or will panic
    let origin_point = 0.1 as f32;
    //let update_method = Backend::MultiThreaded(8);
    //let update_method = Backend::Single;
    let update_method = Backend::Rayon;
    let ref mut state = Grid::new(ctx, update_method)?
        .init_seed(init_b_matrix_vector)
        .init_offset(origin_point, origin_point);
    event::run(ctx, event_loop, state)
}
#[cfg(test)]
mod tests {
    // ************  SETUP  ************
    pub use super::*;
    pub use assert_approx_eq::assert_approx_eq;
    pub use ggez::event::EventsLoop;
    pub use ggez::{conf, event, graphics};
    pub use ggez::{Context, GameResult};
    pub use mocktopus::mocking::*;

    pub struct Globals {
        pub ctx: Context,
        pub event_loop: EventsLoop,
    }
    pub fn setup() -> GameResult<Globals> {
        let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        );
        let (mut ctx, mut event_loop) = cb.build()?;
        // initialize a Grid object
        graphics::set_blend_mode(&mut ctx, BlendMode::Replace);
        Ok(Globals { ctx, event_loop })
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
    // NOTE: turn off next_b_matrix() before executing this
    fn test_update_view_before_offset() {
        let mut init_b_matrix_vector = BMatrixVector::default();
        for j in 0..GRID_SIZE {
            for i in 0..GRID_SIZE {
                //make_blinker(i,j,&mut init_b_matrix_vector);
                //make_square(i,j,&mut init_b_matrix_vector);
                if i > GRID_SIZE / 2 {
                    *init_b_matrix_vector.at_mut(i, j).unwrap() = true;
                }
            }
        }

        let mut globals = setup().unwrap();

        let update_method = Backend::Skip;
        let mut grid = Grid::new(&mut globals.ctx, update_method)
            .unwrap()
            .init_seed(init_b_matrix_vector);
        event::run(&mut globals.ctx, &mut globals.event_loop, &mut grid);
    }

    #[test]
    #[ignore]
    // NOTE: turn off next_b_matrix() before executing this
    fn test_update_view_after_offset() {
        println!("GRID_SIZE: {}", GRID_SIZE);
        let mut init_b_matrix_vector = BMatrixVector::default();
        // just make part of the screen white
        for j in 0..GRID_SIZE {
            for i in 0..GRID_SIZE {
                //make_blinker(i,j,&mut init_b_matrix_vector);
                //make_square(i,j,&mut init_b_matrix_vector);
                if i > GRID_SIZE / 2 {
                    *init_b_matrix_vector.at_mut(i, j).unwrap() = true;
                }
            }
        }

        let mut globals = setup().unwrap();

        let update_method = Backend::Skip;
        let mut grid = Grid::new(&mut globals.ctx, update_method)
            .unwrap()
            .init_offset(user::get_max_offset_x(), 0.1)
            .init_seed(init_b_matrix_vector);
        event::run(&mut globals.ctx, &mut globals.event_loop, &mut grid);
    }

    //#[test]
    //fn test_draw_off_grid_doesnt_panic(){
    //let mut globals = setup(Backend::Skip).unwrap();

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

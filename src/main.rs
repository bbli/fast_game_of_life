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
use std::sync::{Arc,Mutex,MutexGuard,RwLock,RwLockWriteGuard,RwLockReadGuard};
use std::thread::JoinHandle;
use std::cmp::Ordering;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::time::{SystemTime, UNIX_EPOCH};
//use threadpool::ThreadPool;
use scoped_threadpool::Pool;
//use tokio::sync::Semaphore;
//use semaphore::Semaphore;

mod b_matrix_vector;
use b_matrix_vector::*;

mod fsubview;
use fsubview::FSubview;

mod user;
use user::{OffsetState, Point};

mod patterns;
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
const GRID_SIZE: i32 = 2000;

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

// ************  Threading Code  ************
//(TODO: should probably refactor this and bmatrix_vector.rs into its own folder)

pub struct RegionPool {
    threadpool: Pool,
    worker_count: i32,
}
impl RegionPool {
    fn new(worker_count: i32) -> Self {
        //let threadpool = Arc::new(ThreadPool::new(worker_count as usize));
        let threadpool = Pool::new(worker_count as u32);
        RegionPool {
            threadpool,
            worker_count,
        }
    }
}

fn get_num_elems_each_time(vector: &BMatrixVector, worker_count: i32) -> i32 {
    vector.len() as i32 / worker_count
}

impl RegionPool {
    // EC: worker_count is 1 -> max_offset should be 0, so edge case is fine too
    fn create_iter_mut<'a>(&mut self, vector: &'a mut BMatrixVector) -> RegionPoolIterMut<'a> {
        let num_elems_each_time = get_num_elems_each_time(vector, self.worker_count);

        let max_offset = num_elems_each_time * (self.worker_count - 1);
        RegionPoolIterMut {
            ptr: &mut vector[..],
            offset: 0,
            num_elems_each_time,
            max_offset,
        }
    }
}

struct RegionPoolIterMut<'a> {
    ptr: &'a mut [bool],
    offset: i32,
    num_elems_each_time: i32,
    max_offset: i32,
}

impl<'a> Iterator for RegionPoolIterMut<'a> {
    type Item = (&'a mut [bool], i32);

    // EC: at end when we need to take a bit more
    fn next(&mut self) -> Option<Self::Item> {
        // old offset "points" to offset we are about to return
        let old_offset = self.offset;
        self.offset += self.num_elems_each_time;

        match old_offset.cmp(&self.max_offset) {
            // after last case
            Ordering::Greater => None,
            // last case -> just return ptr
            Ordering::Equal => {
                // Since this works, self.ptr must have 'a lifetime
                // even though self has local
                //let l = self.ptr;
                //self.ptr = &mut [];
                // But that said, we can't explicitly take, since self.ptr doesn't own its values
                let slice = mem::replace(&mut self.ptr, &mut []);
                let l = slice;

                Some((l, old_offset))
            }
            // Recursive case
            Ordering::Less => {
                // self.ptr.split_at_mut will pass in the local lifetime rather than  'a
                //let (l,r) = self.ptr.split_at_mut(self.num_elems_each_time as usize);

                // Rustonomicon uses mem::replace to resolve(slice doesn't do anything in Drop though?) -> though the below works too
                //let slice = self.ptr;
                let slice = mem::replace(&mut self.ptr, &mut []);

                let (l, r) = slice.split_at_mut(self.num_elems_each_time as usize);
                self.ptr = r;

                Some((l, old_offset))
            }
        }
    }
}

impl Deref for RegionPool {
    type Target = Pool;
    fn deref(&self) -> &Self::Target {
        &self.threadpool
    }
}

impl DerefMut for RegionPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.threadpool
    }
}

// Split the grid horizontally so that each thread can index like
// for j in (start_y,end_y)
//fn partition_grid(threads: i32)->Vec<(i32,i32)>{
//let mut work_region_list = Vec::new();
//let num_rows = GRID_SIZE/threads;
//for i in 0..threads{
//let start_y;
//let end_y;
//if i == threads-1{
//start_y = i*num_rows;
//end_y = GRID_SIZE-1;
//}
//else{
//start_y = i*num_rows;
//end_y = (i+1)*num_rows -1;
//}
//work_region_list.push((start_y,end_y))
//}
//work_region_list
//}

struct MainWorkerHandle(JoinHandle<()>);
impl MainWorkerHandle{
    fn signal(&self){
        self.0.thread().unpark();
    }
}

struct BMatrix {
    vec: MyArcRwLock<BMatrixVector>,
    new_vec: MyArcMut<BMatrixVector>,
    main_worker_thread: MainWorkerHandle,
    status: MyArcMut<WorkFlag>,
}

//impl Deref for BMatrix {
    //type Target = BMatrixVector;
    //fn deref(&self) -> &Self::Target {
        //&self.vec
    //}
//}

//impl DerefMut for BMatrix {
    //fn deref_mut(&mut self) -> &mut Self::Target {
        //&mut self.vec
    //}
//}


impl BMatrix {
    fn new(update_method: BackendEngine) -> Self {
        let vec = MyArcRwLock::new(BMatrixVector::default());
        let vec2 = vec.clone();
        let new_vec = MyArcMut::new(BMatrixVector::default());
        let new_vec2 = new_vec.clone();

        let status = MyArcMut::new(WorkFlag::Done);
        //let status = Arc::new(Mutex::new(WorkFlag::Done));
        let status2 = status.clone();

        // Spin up new thread and have it sleep until event loop starts and BMatrix calls signal
        let main_worker_thread = thread::spawn(
            move ||{
                let mut main_worker = MainWorker::new(update_method,status2,new_vec2,vec2);
                main_worker.sync_worker_do_work();
            });
        BMatrix {
            vec,
            new_vec,
            main_worker_thread: MainWorkerHandle(main_worker_thread),
            status,
        }
    }
    fn update_vector(&mut self){
        // utilizing low level nature of swap function to do shallow swap
        let mut new_vec_lock = self.new_vec.grab_lock();
        let new_vec_raw: &mut BMatrixVector = new_vec_lock.deref_mut();

        let mut vec_lock = self.vec.grab_writer_lock();
        let vec_raw: &mut BMatrixVector = vec_lock.deref_mut();
        mem::swap(vec_raw,new_vec_raw);
    }

    fn sync_main_update_backend(&mut self){
        if let WorkFlag::Done = self.status.get() {
            // no need to lock since MainWorker can't modify
            // until we call signal anyways
            self.update_vector();

            self.status.set(WorkFlag::InProgress);
            
            self.main_worker_thread.signal();
        }
    }
}

fn print_time_lapse(sys_time: SystemTime){
    let time_lapse = sys_time
        .elapsed()
        .expect("System clock did something funny");
    println!("Time elapsed: {}ms", time_lapse.as_millis());
}

struct MyArcMut<T>(Arc<Mutex<T>>);
impl<T> MyArcMut<T>{
    fn new(value: T)->Self{
        MyArcMut(Arc::new(Mutex::new(value)))
    }
    // use this only if set and get are too limiting
    fn grab_lock(&self) -> MutexGuard<T>{
        self.0.lock().unwrap()
    }
}

impl<T:Clone> MyArcMut<T>{
    fn set(&mut self, value : T){
        *self.0.lock().unwrap().deref_mut() = value;
    }
    fn get(&self) -> T{
        self.0.lock().unwrap().deref().clone()
    }
}

impl<T> Clone for MyArcMut<T>{
    fn clone(&self) -> MyArcMut<T>{
        MyArcMut(self.0.clone())
    }
}

struct MyArcRwLock<T>(Arc<RwLock<T>>);
impl<T> MyArcRwLock<T>{
    fn new(value: T)->Self{
        MyArcRwLock(Arc::new(RwLock::new(value)))
    }
    fn grab_writer_lock(&self) -> RwLockWriteGuard<T>{
        self.0.write().unwrap()
    }
    fn grab_reader_lock(&self) -> RwLockReadGuard<T>{
        self.0.read().unwrap()
    }
}
impl<T> Clone for MyArcRwLock<T>{
    fn clone(&self) -> MyArcRwLock<T>{
        MyArcRwLock(self.0.clone())
    }
}

// This wrapper struct is solely for strong typing purposes
//struct MyArc<T>(MyArcMut<T>);
//impl<T> Deref for MyArc<T> {
    //type Target = MyArcMut<T>;
    //fn deref(&self) -> &Self::Target {
        ////&self.0.lock().unwrap().deref()
        //&self.0
    //}
//}

//impl<T> DerefMut for MyArc<T> {
    //fn deref_mut(&mut self) -> &mut Self::Target {
        ////self.0.lock().unwrap().deref_mut()
        //&mut self.0
    //}
//}
//impl<T:Clone> MyArc<T>{
    //fn new(value: T)-> MyArc<T>{
        //MyArc(MyArcMut::new(value))
    //}
//}

struct MainWorker{
    new_vec: MyArcMut<BMatrixVector>,
    vec: MyArcRwLock<BMatrixVector>,
    region_pool: RegionPool,
    update_method: BackendEngine,
    status: MyArcMut<WorkFlag>
}
impl MainWorker{
    fn new(update_method: BackendEngine, status: MyArcMut<WorkFlag>, new_vec: MyArcMut<BMatrixVector>, vec: MyArcRwLock<BMatrixVector>)->Self{
        use BackendEngine::*;
        // NOTE: using new match ergonomics
        let region_pool = match &update_method {
            Single | Rayon | Skip => RegionPool::new(1),
            MultiThreaded(x) => RegionPool::new(*x),
        };
        MainWorker{
            new_vec,
            vec,
            region_pool,
            update_method,
            status
        }
    }

    //fn get_new_results(&self) -> BMatrixVector{
        ////BMatrixVector(self.new_vec.clone())
    //}
    //fn get_new_status(&self) -> WorkFlag{
        //self.status
    //}

    fn wait(&self){
        thread::park();
    }
    fn sync_worker_do_work(&mut self){
        loop{
            self.wait();

            let sys_time = SystemTime::now();
            self.backendMethodDispatch();
            print_time_lapse(sys_time);

            self.status.set(WorkFlag::Done);
        }
    }

    fn backendMethodDispatch(&mut self) {
        use BackendEngine::*;
        // so I can temporarily bypass arc + mutex restrictions
        // for multi-threading purposes
        let mut new_vec_lock = self.new_vec.grab_lock();
        let new_vec_raw = new_vec_lock.deref_mut();

        let vec_lock = self.vec.grab_reader_lock();
        let vec_raw = vec_lock.deref();
        match &self.update_method {
            Single => {
                //new_vec = new_vec.next_b_matrix();
                engine::next_b_matrix(vec_raw,new_vec_raw);
            }
            Rayon => {
                engine::next_bmatrix_rayon(vec_raw,new_vec_raw);
            }
            MultiThreaded(_worker_count) => {
                let region_pool = &mut self.region_pool;
                engine::next_b_matrix_threadpool(vec_raw,new_vec_raw,region_pool);
            }
            Skip => {}
        }
    }
}

#[derive(Copy,Clone)]
enum WorkFlag{
    InProgress,
    Done
}

enum BackendEngine {
    Single,
    MultiThreaded(i32),
    Rayon,
    Skip,
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
    fn new(ctx: &mut Context, update_method: BackendEngine) -> GameResult<Grid> {
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
    //let update_method = BackendEngine::MultiThreaded(8);
    //let update_method = BackendEngine::Single;
    let update_method = BackendEngine::Rayon;
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

        let update_method = BackendEngine::Skip;
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

        let update_method = BackendEngine::Skip;
        let mut grid = Grid::new(&mut globals.ctx, update_method)
            .unwrap()
            .init_offset(user::get_max_offset_x(), 0.1)
            .init_seed(init_b_matrix_vector);
        event::run(&mut globals.ctx, &mut globals.event_loop, &mut grid);
    }

    #[test]
    fn test_RegionPoolIterMut_get_num_elems_each_time_workerCount1() {
        let worker_count = 1;
        let vec = BMatrixVector::default();
        let num_elems = get_num_elems_each_time(&vec, worker_count);
        assert_eq!(num_elems, GRID_SIZE * GRID_SIZE);
    }

    #[test]
    fn test_RegionPoolIterMut_next_edge_case() {
        let worker_count = 1;
        let mut region_pool = RegionPool::new(worker_count);
        let bool_vec = vec![true, true, true, false, false, false, false];

        let test_vec = bool_vec.clone();
        let mut b_matrix_vector = BMatrixVector::new_for_test(bool_vec);
        let mut region_iterator = region_pool.create_iter_mut(&mut b_matrix_vector);

        if let Some((whole_slice, offset)) = region_iterator.next() {
            assert_eq!(whole_slice, test_vec);
            assert_eq!(offset, 0);
        } else {
            panic!("iterator should still have elements");
        }
    }

    #[test]
    fn test_RegionPoolIterMut_step_through_next() {
        let worker_count = 3;
        let mut region_pool = RegionPool::new(worker_count);
        let mut vec =
            BMatrixVector::new_for_test(vec![true, true, true, false, false, false, false]);
        let mut region_iterator = region_pool.create_iter_mut(&mut vec);

        if let Some((slice1, offset1)) = region_iterator.next() {
            assert_eq!(slice1, vec![true, true]);
            assert_eq!(offset1, 0);
        } else {
            panic!("iterator should still have elements");
        }

        if let Some((slice2, offset2)) = region_iterator.next() {
            assert_eq!(slice2, vec![true, false]);
            assert_eq!(offset2, 2);
        } else {
            panic!("iterator should still have elements");
        }

        if let Some((slice3, offset3)) = region_iterator.next() {
            assert_eq!(slice3, vec![false, false, false]);
            assert_eq!(offset3, 4);
        } else {
            panic!("iterator should still have elements");
        }

        if let Some(_) = region_iterator.next() {
            panic!("iterator should be empty now");
        }
    }

    //#[test]
    //fn test_draw_off_grid_doesnt_panic(){
    //let mut globals = setup(BackendEngine::Skip).unwrap();

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

#![allow(non_snake_case)]
#![warn(clippy::all)]
//! The simplest possible example that does something.

use ggez::event;
use ggez::conf;
use ggez::graphics;
use ggez::graphics::{DrawMode,Rect};
use ggez::nalgebra as na;
use ggez::{Context, GameResult};

// ************  Frontend Globals  ************   
const WINDOW_WIDTH: usize = 1920;
const WINDOW_HEIGHT: usize = 1080;

const CELL_SIZE: f32 = 20.0;
//any smaller and may not print out correctly
const CELL_GAP: f32 = CELL_SIZE/6 as f32;

const NUM_BLOCKS_WIDTH: usize = ((WINDOW_WIDTH as f32/ (CELL_SIZE + CELL_GAP)) + 1.0) as usize;
const NUM_BLOCKS_HEIGHT: usize = ((WINDOW_HEIGHT as f32/ (CELL_SIZE + CELL_GAP)) + 1.0) as usize;

// ************  Backend Globals  ************   
const GRID_SIZE: usize = 400;

// ************  Macros  ************   
macro_rules! BLACK {
    () => {
        [0.0, 0.0 ,0.0, 1.0].into()
    }
}

macro_rules! GREY {
    () => {
        [0.5, 0.5 ,0.5, 1.0].into()
    }
}

struct Grid {
    // otherwise stack overflow
    matrix: Box<[[bool; GRID_SIZE] ;GRID_SIZE]>,
    mesh: graphics::Mesh
}

fn new_rect(i: usize, j:usize) -> Rect{
    let i = i as f32;
    let j = j as f32;
    Rect::new(i*(CELL_SIZE+CELL_GAP),j*(CELL_SIZE+CELL_GAP),CELL_SIZE,CELL_SIZE)
}
impl Grid {
    fn new(ctx: &mut Context) -> GameResult<Grid> {
        let mut mesh = graphics::MeshBuilder::new();

        for i in 0..NUM_BLOCKS_WIDTH{
            for j in 0..NUM_BLOCKS_HEIGHT{
                mesh.rectangle(DrawMode::fill(),new_rect(i,j),BLACK!());
            }
        }
            //.rectangle(DrawMode::fill(),Rect::new(0.0,0.0,100.0,100.0),BLACK!())
        let mesh = mesh.build(ctx)?;
        Ok(Grid { matrix: Box::new([[false;GRID_SIZE];GRID_SIZE]) , mesh})
    }
}

impl event::EventHandler for Grid {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx,GREY!());

        graphics::draw(ctx, &self.mesh , (na::Point2::new(0.0,0.0),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez")
        .window_mode(conf::WindowMode::default()
                     .resizable(true)
                     .dimensions(WINDOW_WIDTH as f32,WINDOW_HEIGHT as f32));
    let (ref mut ctx, ref mut event_loop) = cb.build()?;
    let state = &mut Grid::new(ctx)?;
    event::run(ctx, event_loop, state)
}

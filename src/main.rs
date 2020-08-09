#![allow(non_snake_case)]
#![warn(clippy::all)]
//! The simplest possible example that does something.

use ggez::conf;
use ggez::event;
use ggez::graphics;
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
const GRID_SIZE: usize = 400;

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

struct Grid {
    // otherwise stack overflow
    matrix: Box<[[bool; GRID_SIZE]; GRID_SIZE]>,
    //mesh: graphics::Mesh,
    view: spritebatch::SpriteBatch
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
        //let image = Image::from_rgba8(_ctx, CELL_SIZE as u16, CELL_SIZE as u16,&BLACK());
        let image = Image::solid(ctx,CELL_SIZE,BLACK!())?;
        let mut spritebatch = spritebatch::SpriteBatch::new(image);
        for i in 0..NUM_BLOCKS_WIDTH{
            for j in 0..NUM_BLOCKS_HEIGHT {
                spritebatch.add(new_cell(i,j));
            }
        }
        Ok(Grid{matrix: Box::new([[false; GRID_SIZE]; GRID_SIZE]),view:spritebatch})
    }
}

impl event::EventHandler for Grid {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, GREY!());

        graphics::draw(ctx, &self.view , (na::Point2::new(0.0, 0.0),))?;

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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_image_macro(){
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
}

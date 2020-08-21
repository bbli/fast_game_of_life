use ggez::graphics::spritebatch;
use ggez::graphics;
use ggez::{Context, GameResult};
//use ggez::nalgebra as na;
use nalgebra::geometry::Point2;
use super::*;

#[cfg(test)]
use mocktopus::macros::*;

struct SpriteBatchHandler{
    spritebatch: spritebatch::SpriteBatch,
    // we needs this vec b/c SpriteIdx wraps around a private field
    // so we can't dynamically construct the SpriteIdx ourselves when
    // we want to use the spritebatch.set method
    handle_list: Vec<spritebatch::SpriteIdx>,
    sw_horizontal_sections: i32,
    sw_vertical_sections: i32,
}

/// responsible for drawing onto the canvas
pub struct FSubview{
    black_sb_handler: SpriteBatchHandler,
    white_sb_handler: SpriteBatchHandler,
    // Note: relative_offset should be positive -> draw will take care of negative
    relative_offset: Point,
    sw_horizontal_sections: i32,
    sw_vertical_sections: i32,

    mesh_builder: graphics::MeshBuilder,
    mesh: graphics::Mesh
}

 //Sliding Window Setup
fn get_section_given_offset(offset:f32)-> i32{
    (offset/(CELL_SIZE+CELL_GAP)).ceil() as i32
}
fn get_1d_section(length: usize)->i32{
    let one_shift_offset = CELL_SIZE+ CELL_GAP + length as f32;
    get_section_given_offset(one_shift_offset)
}


impl FSubview{
    pub fn new(ctx: &mut Context)-> GameResult<FSubview>{
        let sw_horizontal_sections = get_1d_section(WINDOW_WIDTH);
        let sw_vertical_sections = get_1d_section(WINDOW_HEIGHT);
        // ************  MESH BUILDER METHOD  ************   
        let mut f_mesh = graphics::MeshBuilder::new();

        for j in 0..sw_vertical_sections {
            for i in 0..sw_horizontal_sections {
                f_mesh.rectangle(DrawMode::fill(), new_rect(i, j), BLACK!());
            }
        }
        let f_mesh = f_mesh.build(ctx)?;

        // ************  SPRITE METHOD  ************   
        //let image = Image::from_rgba8(ctx, CELL_SIZE as u16, CELL_SIZE as u16,&BLACK());
        // create both handles with invalid locations for all sprites
        let white_image = Image::solid(ctx,CELL_SIZE as u16,WHITE!()).unwrap();
        let white_sb_handler = fsubview::SpriteBatchHandler::new(white_image,sw_horizontal_sections,sw_vertical_sections);

        let black_image = Image::solid(ctx,CELL_SIZE as u16,BLACK!()).unwrap();
        let black_sb_handler = fsubview::SpriteBatchHandler::new(black_image,sw_horizontal_sections,sw_vertical_sections);

        
        Ok(FSubview{
            black_sb_handler,
            white_sb_handler,
            relative_offset: Point{x:0.0, y:0.0},
            sw_horizontal_sections,
            sw_vertical_sections,

            mesh: f_mesh,
            mesh_builder: graphics::MeshBuilder::new()
        })
    }
    pub fn startView(&mut self){
        //self.black_sb_handler.spritebatch.clear();
        //self.white_sb_handler.spritebatch.clear();
        //self.mesh_builder = graphics::MeshBuilder::new();
    }
    pub fn addWhiteToView(&mut self,relative_i:i32,relative_j:i32){
        //self.white_sb_handler.spritebatch.add(new_cell(relative_i,relative_j));
        //self.mesh_builder.rectangle(DrawMode::fill(),new_rect(relative_i,relative_j),WHITE!());
        self.change_to_white(relative_i,relative_j);
    }
    pub fn addBlackToView(&mut self,relative_i:i32,relative_j:i32){
        //self.black_sb_handler.spritebatch.add(new_cell(relative_i,relative_j));
        //self.mesh_builder.rectangle(DrawMode::fill(),new_rect(relative_i,relative_j),BLACK!());
        self.change_to_black(relative_i,relative_j);
    }
    pub fn endView(&mut self,ctx:&mut Context){
        //self.mesh = self.mesh_builder.build(ctx).expect("Something went wrong during Mesh Building");
    }
    pub fn update_relative_offset(&mut self,x:f32,y:f32){
        self.relative_offset.x = x;
        self.relative_offset.y = y;
    }
    pub fn drawView(&self,ctx: &mut Context)-> GameResult{
        //println!("Relative offset x: {}",self.relative_offset.x);
        //println!("Relative offset y: {}",self.relative_offset.y);
        let offset_draw_param = DrawParam::new()
                            .dest(Point2::new(-self.relative_offset.x,-self.relative_offset.y));
        match graphics::draw(ctx, &self.black_sb_handler.spritebatch ,offset_draw_param){
            Ok(value) => graphics::draw(ctx, &self.white_sb_handler.spritebatch ,offset_draw_param),
            Err(x) => Err(x)
        }
        //graphics::draw(ctx,&self.mesh,offset_draw_param)
    }
}

//#[mockable]
impl FSubview{
    // NOTE: Is setting costly? If so, should only set if location actually changes
    fn change_to_black(&mut self,relative_i: i32, relative_j: i32){
        //set corresponding black sprite to correct location given i,j
        self.black_sb_handler.set_correct(relative_i,relative_j);
        // set corresponding white sprite to invalid location
        self.white_sb_handler.set_invalid(relative_i,relative_j);
    }
    fn change_to_white(&mut self, relative_i: i32, relative_j: i32){
        self.white_sb_handler.set_correct(relative_i,relative_j);
        self.black_sb_handler.set_invalid(relative_i,relative_j);
    }
    pub fn get_horizontal_window_range(&self,x_left: f32, x_right: f32)-> (i32,i32){
        let num_sections_crossed = (x_right/ (CELL_SIZE+CELL_GAP)).ceil() as i32;

        let left_idx:i32;
        let right_idx:i32;
        if num_sections_crossed < self.sw_horizontal_sections{
            left_idx = 0;
            right_idx = convert_section_to_idx(self.sw_horizontal_sections)
        }
        else{
            left_idx = convert_section_to_idx(num_sections_crossed-self.sw_horizontal_sections+1);
            right_idx = convert_section_to_idx(num_sections_crossed)
        }

        (left_idx,right_idx)
    }

    pub fn get_vertical_window_range(&self, y_top: f32, y_bottom: f32)->(i32,i32){
        let num_sections_crossed = (y_bottom/ (CELL_SIZE+CELL_GAP)).ceil() as i32;

        let top_idx:i32;
        let bottom_idx:i32;
        if num_sections_crossed < self.sw_vertical_sections {
            top_idx = 0;
            bottom_idx = convert_section_to_idx(self.sw_vertical_sections)
        }
        else{
            top_idx = convert_section_to_idx(num_sections_crossed-self.sw_vertical_sections+1);
            bottom_idx = convert_section_to_idx(num_sections_crossed)
        }

        (top_idx,bottom_idx)
    }
}

//#[mockable]
impl SpriteBatchHandler{
    pub fn new(image: Image, sw_horizontal_sections: i32, sw_vertical_sections: i32 ) -> SpriteBatchHandler{

            // 1. create the spritebatch
            let mut spritebatch = spritebatch::SpriteBatch::new(image);

            // 2. Create the vector of handles then wrap
            let mut handle_list = Vec::new();
            handle_list.reserve((sw_horizontal_sections*sw_vertical_sections)as usize);
            //// Create x axis first since in graphics first index corresponds with 
            //// the column, not row
            for j in 0..sw_vertical_sections {
                for i in 0..sw_horizontal_sections{
                    let sprite_idx = spritebatch.add(new_cell(INVALID_X,INVALID_Y));
                    handle_list.push(sprite_idx);
                    //handle_list.0.push(spritebatch.add(new_cell(i,j)));
                }
            }
            SpriteBatchHandler{
                spritebatch,
                handle_list,
                sw_horizontal_sections,
                sw_vertical_sections
            }
    }
        fn set_correct(&mut self,relative_i:i32,relative_j:i32){
            let sprite_handle = self.at(relative_i,relative_j).unwrap();
            self.spritebatch.set(sprite_handle,new_cell(relative_i,relative_j)).unwrap();
        }
        fn set_invalid(&mut self,relative_i:i32,relative_j:i32){
            let sprite_handle = self.at(relative_i,relative_j).unwrap();
            self.spritebatch.set(sprite_handle,new_cell(INVALID_X,INVALID_Y)).unwrap();
        }
    }

impl MatrixView for SpriteBatchHandler{
    type Item= spritebatch::SpriteIdx;
    fn at(&self,i:i32, j:i32)-> GameResult<Self::Item>{
        if i < 0 || j < 0 {
            Err(GameError::EventLoopError("IndexError(bmatrix.at): i and j must be nonnegative".to_string()))
        }
        else if i >= self.sw_horizontal_sections || j>= self.sw_vertical_sections {
            Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",self.sw_horizontal_sections,self.sw_vertical_sections)))
        }
        else{
            Ok(self.handle_list[(j*self.sw_horizontal_sections+ i) as usize])
        }
    }
    fn at_mut<'a>(&'a mut self,i:i32, j:i32)-> GameResult<&'a mut Self::Item>{
        if i < 0 || j < 0 {
            Err(GameError::EventLoopError("IndexError(bmatrix.at): i and j must be nonnegative".to_string()))
        }
        else if i >= self.sw_horizontal_sections || j>= self.sw_vertical_sections {
            Err(GameError::EventLoopError(format!("IndexError: b_matrix's i must be less than {} and j must be less than {}",self.sw_horizontal_sections,self.sw_vertical_sections)))
        }
        else{
            Ok(&mut self.handle_list[(j*self.sw_horizontal_sections + i) as usize])
        }
    }
}
//pub enum CellState{
    //BLACK,
    //WHITE
//}
pub fn new_cell(i:i32, j:i32) -> DrawParam{
    DrawParam::default()
        .dest(Point2::new(
                i as f32 * (CELL_SIZE as f32 + CELL_GAP),
                j as f32 * (CELL_SIZE as f32 + CELL_GAP)
                          )
              )
}

// ************  Smallest Bounding Sliding Window Helper Functions  ************   
//pub fn get_vertical_range_of_view(y:f32)->(i32,i32){
    //(get_base_index_top(y),get_base_index_bottom(y+WINDOW_HEIGHT as f32))
//}

//pub fn get_horizontal_range_of_view(x:f32) -> (i32,i32){
    //(get_base_index_left(x),get_base_index_right(x+WINDOW_WIDTH as f32))
//}

//fn get_base_index_right(x:f32)->i32{
    //empty_moves_forward(x)
//}

//fn get_base_index_bottom(y:f32)->i32{
    //empty_moves_forward(y)
//}

//fn get_base_index_top(y:f32)->i32{
    //empty_moves_back(y)
//}

//fn get_base_index_left(x:f32)->i32{
    //empty_moves_back(x)
//}


//pub fn empty_moves_back(z:f32)->i32{
    //// num_sections gives the number of complete CELL_SIZE+CELL_GAP sections
    //// -1 since it starts from 1(instead of 0 like the matrices)
    //let num_sections = (z/(CELL_SIZE+CELL_GAP)).ceil();
    //if num_sections == 0.0{
        //num_sections as i32
    //}
    //else{
        //num_sections as i32 -1
    //}
//}

// EC: 0,0
// PRECONDITON: z is positive
//width>0 so we don't need to account for 0 edge case like in empty_moves_back
//pub fn empty_moves_forward(z:f32) -> i32{
    //// ************  Float Land  ************   
    //let num_sections = (z/(CELL_SIZE+CELL_GAP)).ceil();
    //let rightmost_point = num_sections*(CELL_SIZE+CELL_GAP);
    //let threshold = rightmost_point - CELL_GAP;

    //// ************  Adjusting back to Index Land  ************   
     ////means we are currently inside a box
     ////so num_sections is the "correct" idx
    //if threshold + EPSILON >= z{
        //(num_sections - 1.0) as i32
    //}
    //// we need to extend down to next cell
    //else{
        //num_sections as i32
    //}
//}

// ************  Relative Offset Helper Functions  ************   

fn get_top_of_cell(j:i32)->f32{
    j as f32*(CELL_SIZE+CELL_GAP)
}
pub fn get_distance_to_top(offset_y:f32,top_idx:i32)->GameResult<f32>{
    let top_of_upper_bound_cell = get_top_of_cell(top_idx);
    //println!("offset_y: {}",offset_y);
    //println!("top_of_upper_bound_cell: {}",top_of_upper_bound_cell);

    if offset_y >= top_of_upper_bound_cell{
        Ok(offset_y-top_of_upper_bound_cell)
    }
    else{
        Err(GameError::EventLoopError("Top bounding cell should be above current offset".to_string()))
    }
}
fn get_left_of_cell(i:i32)->f32{
    i as f32*(CELL_SIZE+CELL_GAP)
}
pub fn get_distance_to_left(offset_x:f32,left_idx:i32)->GameResult<f32>{
    let left_of_upper_bound_cell = get_left_of_cell(left_idx);
    if offset_x >= left_of_upper_bound_cell{
        Ok(offset_x-left_of_upper_bound_cell)
    }
    else{
        Err(GameError::EventLoopError("Left bounding cell should be to left of current offset".to_string()))
    }
}

// ************  Invariant Sliding Window Helper Functions  ************   
// During Grid update view
fn convert_section_to_idx(i:i32)->i32{
    i-1
}

#[cfg(test)]
mod tests {
    use mocktopus::mocking::*;
    use crate::tests::*;
    use super::*;

    //#[test]
    //fn test_bounding_space_vertical(){
        //let height = CELL_SIZE/2.0+CELL_GAP/2.0;
        //let offset_y = (2.0*(CELL_SIZE+CELL_GAP)) - CELL_GAP/2.0;
        //// so from variables above, we know ending should be on empty space
        //let top_idx = fsubview::get_base_index_top(offset_y);
        //let bottom_idx = fsubview::get_base_index_bottom(offset_y+height);
        //assert_eq!(top_idx,1);
        //assert_eq!(bottom_idx,2);
    //}
    //#[test]
    //fn test_bounding_space_horizontal(){
        //let width = CELL_SIZE+CELL_GAP+CELL_SIZE/2.0+CELL_GAP/2.0;
        //let offset_x = (CELL_SIZE+CELL_GAP) + CELL_SIZE/2.0;
        //// so from variables above, we know ending should be on empty space
        //let left_idx = fsubview::get_base_index_left(offset_x);
        //let right_idx = fsubview::get_base_index_right(offset_x+width);
        //assert_eq!(left_idx,1);
        //assert_eq!(right_idx,3);
    //}
    //#[test]
    //fn test_bounding_space_edge_case_at_origin_x(){
        //let width = 2.0*(CELL_SIZE+CELL_GAP)+CELL_SIZE/2.0;
        //// so from above, we know ending should be on empty space
        //let left_idx = fsubview::get_base_index_left(0.0);
        //let right_idx = fsubview::get_base_index_right(0.0+width);
        //assert_eq!(left_idx,0);
        //assert_eq!(right_idx,2);
    //}
    //#[test]
    //fn test_bounding_space_edge_case_at_origin_y(){
        //let height = 2.0*(CELL_SIZE+CELL_GAP)+CELL_SIZE/2.0;
        //// so from above, we know ending should be on empty space
        //let top_idx = fsubview::get_base_index_top(0.0);
        //let bottom_idx = fsubview::get_base_index_bottom(0.0+height);
        //assert_eq!(top_idx,0);
        //assert_eq!(bottom_idx,2);
    //}
    //#[test]
    //fn test_bounding_space_edge_case_at_max_offset_x(){
        //let width = WINDOW_WIDTH;
        //let offset_x = user::get_max_offset_x();

        //let right_idx = fsubview::get_base_index_right(offset_x+width as f32);
        //assert_eq!(right_idx,(GRID_SIZE-1) as i32);
    //}
    //#[test]
    //fn test_bounding_space_edge_case_at_max_offset_y(){
        //let height = WINDOW_HEIGHT;
        //let offset_y = user::get_max_offset_y();

        //let bottom_idx = fsubview::get_base_index_right(offset_y+height as f32);
        //assert_eq!(bottom_idx,(GRID_SIZE-1) as i32);
    //}


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
    fn test_mock_safe_overrides_nested_method(){
        SpriteBatchHandler::new.mock_safe(|image,sw_horizontal_sections, sw_vertical_sections|{
            let mut spritebatch = spritebatch::SpriteBatch::new(image);
            let mut handle_list = Vec::new();
            let sprite_idx = spritebatch.add(new_cell(INVALID_X,INVALID_Y));
            handle_list.push(sprite_idx);
            MockResult::Return(SpriteBatchHandler{
                spritebatch,
                handle_list,
                sw_horizontal_sections,
                sw_vertical_sections
            })
        });

        let mut globals = setup().unwrap();
        let black_image = Image::solid(&mut globals.ctx,CELL_SIZE as u16,BLACK!()).unwrap();
        let dummy = 125;
        let sprite_handler = SpriteBatchHandler::new(black_image,dummy,dummy);
        assert_eq!(sprite_handler.handle_list.len(),1);
    }

    #[test]
    #[ignore]
    fn test_change_to_white(){
        Grid::update_view.mock_safe(|my_self: &mut Grid, ctx: &mut Context|{
            // change one of the blocks to white now -> 
            // Assumes SpriteBatchHandler::new sets everything to invalid at first
            my_self.f_subview.change_to_white(20,20);
            MockResult::Return(Ok(()))
        });

        let mut globals = setup().unwrap();
        // 2. then change back to white
        event::run(&mut globals.ctx,&mut globals.event_loop,&mut globals.grid);
    }
    #[test]
    #[ignore]
    fn test_change_to_black(){
        Grid::update_view.mock_safe(|my_self: &mut Grid, ctx: &mut Context|{
            // change one of the blocks to black now -> 
            // Assumes SpriteBatchHandler::new sets everything to invalid at first
            my_self.f_subview.change_to_black(20,20);
            MockResult::Return(Ok(()))
        });

        let mut globals = setup().unwrap();
        // 2. then change back to white
        event::run(&mut globals.ctx,&mut globals.event_loop,&mut globals.grid);
    }

    #[should_panic]
    #[test]
    fn test_SpriteBatchHandler_at_outOfBounds(){
        let globals = setup().unwrap();

        // This test is contigent on Grid::new initalizing 
        // columns first
        let sw_horizontal_sections = globals.grid.f_subview.sw_horizontal_sections;
        globals.grid.f_subview.white_sb_handler.at(sw_horizontal_sections,1).unwrap();
    }
    #[test]
    fn test_SpriteBatchHandler_at_rightAtEdge(){
        let globals = setup().unwrap();

        // This test is contigent on Grid::new initalizing 
        // columns first
        let sw_horizontal_sections = globals.grid.f_subview.sw_horizontal_sections;
        let sw_vertical_sections = globals.grid.f_subview.sw_vertical_sections;
        globals.grid.f_subview.white_sb_handler.at(sw_horizontal_sections-1,sw_vertical_sections-1).unwrap();
    }

    // NOTE: not the best way to test as I am replacing the entire functions just for the sake of
    // manipulating a global variable. But it's either this 2) or make the global variable
    // static/mutable 3)or pass global variable in as a function argument
    #[test]
    fn test_get_horizontal_window_range_small_window(){
        let globals = setup().unwrap();

        let (left_idx,right_idx) = globals.grid.f_subview.get_horizontal_window_range(0.0, WINDOW_WIDTH as f32/2.0);
        assert_eq!(left_idx,0);
        assert_eq!(right_idx,globals.grid.f_subview.sw_horizontal_sections-1);
    }

    #[test]
    fn test_get_horizontal_window_range_large_window(){
        let globals = setup().unwrap();

        let right_edge_of_view = (GRID_SIZE-1) as f32 * (CELL_SIZE+CELL_GAP) + CELL_SIZE/2.0;
        let (left_idx,right_idx) = globals.grid.f_subview.get_horizontal_window_range(0.0, right_edge_of_view);
        assert_eq!(right_idx,GRID_SIZE-1);
    }
    #[test]
    fn test_get_vertical_window_range_small_window(){
        let globals = setup().unwrap();

        let (left_idx,right_idx) = globals.grid.f_subview.get_vertical_window_range(0.0, WINDOW_HEIGHT as f32/2.0);
        assert_eq!(left_idx,0);
        assert_eq!(right_idx,globals.grid.f_subview.sw_vertical_sections-1);
    }

    #[test]
    fn test_get_vertical_window_range_large_window(){
        let globals = setup().unwrap();

        let bottom_edge_of_view = (GRID_SIZE-1) as f32 * (CELL_SIZE+CELL_GAP) + CELL_SIZE/2.0;
        let (top_idx,bottom_idx) = globals.grid.f_subview.get_vertical_window_range(0.0, bottom_edge_of_view);
        assert_eq!(bottom_idx,GRID_SIZE-1);
    }
}

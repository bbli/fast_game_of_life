use super::*;

const EPSILON: f32 = 2.5*CELL_SIZE;

pub fn get_max_offset_x()-> f32{
    (GRID_SIZE-1) as f32*(CELL_SIZE+CELL_GAP)+ CELL_SIZE - WINDOW_WIDTH as f32
}

pub fn get_max_offset_y()-> f32{
    (GRID_SIZE-1) as f32*(CELL_SIZE+CELL_GAP)+ CELL_SIZE - WINDOW_HEIGHT as f32
}

// NOTE: states do not represent when the offset reaches the corners
// but rather when it reaches the max offset in the x and y direction
pub enum OffsetState {
    Inside(Point),
    TopEdge(Point),
    RightEdge(Point),
    BottomEdge(Point),
    LeftEdge(Point),
    TopLeftCorner(Point),
    TopRightCorner(Point),
    BottomRightCorner(Point),
    BottomLeftCorner(Point)
}
impl Default for OffsetState{
    fn default() -> Self{
        OffsetState::TopRightCorner(Point::new(0.0,0.0))
    }
}


pub fn transition_offset_state_right(state: OffsetState) -> OffsetState{
    use OffsetState::*;
    match state{
        Inside(point) =>
        {
            let new_x = point.x +2.0*CELL_SIZE;
            let new_point = Point::new(new_x,point.y);
            if get_max_offset_x() - new_x < EPSILON{
                RightEdge(new_point)
            }
            else{
                Inside(new_point)
            }
        }
        TopEdge(point) =>
        {
            let new_x = point.x + 2.0*CELL_SIZE;
            let new_point = Point::new(new_x,point.y);
            if get_max_offset_x() - new_x < EPSILON{
                TopRightCorner(new_point)
            }
            else{
                TopEdge(new_point)
            }
        },
        BottomEdge(point) =>
        {
            let new_x = point.x + 2.0*CELL_SIZE;
            let new_point = Point::new(new_x,point.y);
            if get_max_offset_x() - new_x < EPSILON{
                BottomRightCorner(new_point)
            }
            else{
                BottomEdge(new_point)
            }
        },
        LeftEdge(point) =>
        {
            // Assume window width is large than twice the cell size
            let new_x = point.x +2.0*CELL_SIZE;
            let new_point = Point::new(new_x,point.y);
            Inside(new_point)
        },
        RightEdge(point) => RightEdge(point),
        TopLeftCorner(point) =>
        {
            // Assume window width is large than twice the cell size
            let new_x = point.x +2.0*CELL_SIZE;
            let new_point = Point::new(new_x,point.y);
            TopEdge(new_point)
        }
        TopRightCorner(point) => TopRightCorner(point),
        BottomRightCorner(point) => BottomRightCorner(point),
        BottomLeftCorner(point) => 
        {
            let new_x = point.x +2.0*CELL_SIZE;
            let new_point = Point::new(new_x,point.y);
            BottomEdge(new_point)
        }
    }
}

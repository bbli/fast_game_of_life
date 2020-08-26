use super::*;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    //NOTE: we will not check the maximum
    pub fn new(x: f32, y: f32) -> Point {
        if x < 0.0 || y < 0.0 {
            panic!(format!(
                "Point needs to be positive in both dimensions.\n current x: {}\n current y: {}",
                x, y
            ));
        }
        Point { x, y }
    }
}

pub fn get_max_offset_x() -> f32 {
    (GRID_SIZE - 1) as f32 * (CELL_SIZE + CELL_GAP) + CELL_SIZE - WINDOW_WIDTH as f32
}

pub fn get_max_offset_y() -> f32 {
    (GRID_SIZE - 1) as f32 * (CELL_SIZE + CELL_GAP) + CELL_SIZE - WINDOW_HEIGHT as f32
}

// NOTE: states do not represent when the offset reaches the corners
// but rather when it reaches the max offset in the x and y direction
#[derive(Clone, Copy)]
pub enum OffsetState {
    Inside(Point),
    TopEdge(Point),
    RightEdge(Point),
    BottomEdge(Point),
    LeftEdge(Point),
    TopLeftCorner(Point),
    TopRightCorner(Point),
    BottomRightCorner(Point),
    BottomLeftCorner(Point),
}
impl Default for OffsetState {
    fn default() -> Self {
        OffsetState::TopLeftCorner(Point::new(0.0, 0.0))
    }
}

impl OffsetState {
    pub fn get_point(&self) -> Point {
        use OffsetState::*;
        match self {
            &Inside(ref point) => point.clone(),
            &TopEdge(ref point) => point.clone(),
            &RightEdge(ref point) => point.clone(),
            &BottomEdge(ref point) => point.clone(),
            &LeftEdge(ref point) => point.clone(),
            &TopLeftCorner(ref point) => point.clone(),
            &TopRightCorner(ref point) => point.clone(),
            &BottomRightCorner(ref point) => point.clone(),
            &BottomLeftCorner(ref point) => point.clone(),
        }
    }
    pub fn update(&mut self, ctx: &mut Context) {
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            *self = transition_offset_state_right(*self);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            *self = transition_offset_state_left(*self);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up) {
            *self = transition_offset_state_up(*self);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            *self = transition_offset_state_down(*self);
        }

        //println!("Point: {:?}",self.get_point());
    }
}

fn transition_offset_state_right(state: OffsetState) -> OffsetState {
    use OffsetState::*;
    match state {
        Inside(point) => {
            let new_x = point.x + 2.0 * CELL_SIZE;
            if get_max_offset_x() <= new_x {
                let right_edge_point = Point::new(get_max_offset_x(), point.y);
                RightEdge(right_edge_point)
            } else {
                let new_point = Point::new(new_x, point.y);
                Inside(new_point)
            }
        }
        TopEdge(point) => {
            let new_x = point.x + 2.0 * CELL_SIZE;
            if get_max_offset_x() <= new_x {
                let right_edge_point = Point::new(get_max_offset_x(), point.y);
                TopRightCorner(right_edge_point)
            } else {
                let new_point = Point::new(new_x, point.y);
                TopEdge(new_point)
            }
        }
        BottomEdge(point) => {
            let new_x = point.x + 2.0 * CELL_SIZE;
            if get_max_offset_x() <= new_x {
                let right_edge_point = Point::new(get_max_offset_x(), point.y);
                BottomRightCorner(right_edge_point)
            } else {
                let new_point = Point::new(new_x, point.y);
                BottomEdge(new_point)
            }
        }
        LeftEdge(point) => {
            // Assume window width is large than twice the cell size
            let new_x = point.x + 2.0 * CELL_SIZE;
            let new_point = Point::new(new_x, point.y);
            Inside(new_point)
        }
        RightEdge(point) => RightEdge(point),
        TopLeftCorner(point) => {
            // Assume window width is large than twice the cell size
            let new_x = point.x + 2.0 * CELL_SIZE;
            let new_point = Point::new(new_x, point.y);
            TopEdge(new_point)
        }
        TopRightCorner(point) => TopRightCorner(point),
        BottomRightCorner(point) => BottomRightCorner(point),
        BottomLeftCorner(point) => {
            let new_x = point.x + 2.0 * CELL_SIZE;
            let new_point = Point::new(new_x, point.y);
            BottomEdge(new_point)
        }
    }
}
fn transition_offset_state_left(state: OffsetState) -> OffsetState {
    use OffsetState::*;
    match state {
        Inside(point) => {
            let new_x = point.x - 2.0 * CELL_SIZE;
            if new_x < EPSILON {
                let left_edge_point = Point::new(0.0, point.y);
                LeftEdge(left_edge_point)
            } else {
                let new_point = Point::new(new_x, point.y);
                Inside(new_point)
            }
        }
        TopEdge(point) => {
            let new_x = point.x - 2.0 * CELL_SIZE;
            if new_x < EPSILON {
                let left_edge_point = Point::new(0.0, point.y);
                TopLeftCorner(left_edge_point)
            } else {
                let new_point = Point::new(new_x, point.y);
                TopEdge(new_point)
            }
        }
        BottomEdge(point) => {
            let new_x = point.x - 2.0 * CELL_SIZE;
            if new_x < EPSILON {
                let left_edge_point = Point::new(0.0, point.y);
                BottomLeftCorner(left_edge_point)
            } else {
                let new_point = Point::new(new_x, point.y);
                BottomEdge(new_point)
            }
        }
        LeftEdge(point) => LeftEdge(point),
        RightEdge(point) => {
            // Assume window width is large than twice the cell size
            let new_x = point.x - 2.0 * CELL_SIZE;
            let new_point = Point::new(new_x, point.y);
            Inside(new_point)
        }
        TopLeftCorner(point) => TopLeftCorner(point),
        TopRightCorner(point) => {
            // Assume window width is large than twice the cell size
            let new_x = point.x - 2.0 * CELL_SIZE;
            let new_point = Point::new(new_x, point.y);
            TopEdge(new_point)
        }
        BottomRightCorner(point) => {
            let new_x = point.x - 2.0 * CELL_SIZE;
            let new_point = Point::new(new_x, point.y);
            BottomEdge(new_point)
        }
        BottomLeftCorner(point) => BottomLeftCorner(point),
    }
}
fn transition_offset_state_up(state: OffsetState) -> OffsetState {
    use OffsetState::*;
    match state {
        Inside(point) => {
            let new_y = point.y - 2.0 * CELL_SIZE;
            if new_y < EPSILON {
                let top_edge_point = Point::new(point.x, 0.0);
                TopEdge(top_edge_point)
            } else {
                let new_point = Point::new(point.x, new_y);
                Inside(new_point)
            }
        }
        TopEdge(point) => TopEdge(point),
        BottomEdge(point) => {
            let new_y = point.y - 2.0 * CELL_SIZE;
            let new_point = Point::new(point.x, new_y);
            Inside(new_point)
        }
        LeftEdge(point) => {
            let new_y = point.y - 2.0 * CELL_SIZE;
            if new_y < EPSILON {
                let top_edge_point = Point::new(point.x, 0.0);
                TopLeftCorner(top_edge_point)
            } else {
                let new_point = Point::new(point.x, new_y);
                LeftEdge(new_point)
            }
        }
        RightEdge(point) => {
            let new_y = point.y - 2.0 * CELL_SIZE;
            if new_y < EPSILON {
                let top_edge_point = Point::new(point.x, 0.0);
                TopRightCorner(top_edge_point)
            } else {
                let new_point = Point::new(point.x, new_y);
                RightEdge(new_point)
            }
        }
        TopLeftCorner(point) => TopLeftCorner(point),
        TopRightCorner(point) => TopRightCorner(point),
        BottomRightCorner(point) => {
            let new_y = point.y - 2.0 * CELL_SIZE;
            let new_point = Point::new(point.x, new_y);
            RightEdge(new_point)
        }
        BottomLeftCorner(point) => {
            let new_y = point.y - 2.0 * CELL_SIZE;
            let new_point = Point::new(point.x, new_y);
            LeftEdge(new_point)
        }
    }
}
fn transition_offset_state_down(state: OffsetState) -> OffsetState {
    use OffsetState::*;
    match state {
        Inside(point) => {
            let new_y = point.y + 2.0 * CELL_SIZE;
            if get_max_offset_y() < new_y {
                let bottom_edge_point = Point::new(point.x, get_max_offset_y());
                BottomEdge(bottom_edge_point)
            } else {
                let new_point = Point::new(point.x, new_y);
                Inside(new_point)
            }
        }
        TopEdge(point) => {
            let new_y = point.y + 2.0 * CELL_SIZE;
            let new_point = Point::new(point.x, new_y);
            Inside(new_point)
        }
        BottomEdge(point) => BottomEdge(point),
        LeftEdge(point) => {
            let new_y = point.y + 2.0 * CELL_SIZE;
            if get_max_offset_y() < new_y {
                let bottom_edge_point = Point::new(point.x, get_max_offset_y());
                BottomLeftCorner(bottom_edge_point)
            } else {
                let new_point = Point::new(point.x, new_y);
                LeftEdge(new_point)
            }
        }
        RightEdge(point) => {
            let new_y = point.y + 2.0 * CELL_SIZE;
            if get_max_offset_y() < new_y {
                let bottom_edge_point = Point::new(point.x, get_max_offset_y());
                BottomRightCorner(bottom_edge_point)
            } else {
                let new_point = Point::new(point.x, new_y);
                RightEdge(new_point)
            }
        }
        TopLeftCorner(point) => {
            let new_y = point.y + 2.0 * CELL_SIZE;
            let new_point = Point::new(point.x, new_y);
            LeftEdge(new_point)
        }
        TopRightCorner(point) => {
            let new_y = point.y + 2.0 * CELL_SIZE;
            let new_point = Point::new(point.x, new_y);
            RightEdge(new_point)
        }
        BottomRightCorner(point) => BottomRightCorner(point),
        BottomLeftCorner(point) => BottomLeftCorner(point),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    #[test]
    #[ignore]
    fn test_transition_bottom_right_corner() {
        // ************  GRID  ************
        let mut init_b_matrix_vector = patterns::PatternBuilder::new()
            .make_random((0, 0), GRID_SIZE, GRID_SIZE)
            .build();
        // ************  GGEZ  ************
        let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        );

        // ************  RUNNING  ************
        let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();
        graphics::set_blend_mode(ctx, BlendMode::Replace);
        let update_method = BackendEngine::Rayon;
        let ref mut state = Grid::new(ctx, update_method)
            .unwrap()
            .init_seed(init_b_matrix_vector)
            .init_offset(
                user::get_max_offset_x() - 5.0,
                user::get_max_offset_y() - 5.0,
            );
        event::run(ctx, event_loop, state);
    }

    #[test]
    #[ignore]
    fn test_transition_top_left_corner() {
        // ************  GRID  ************
        let mut init_b_matrix_vector = patterns::PatternBuilder::new()
            .make_random((0, 0), GRID_SIZE, GRID_SIZE)
            .build();
        // ************  GGEZ  ************
        let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        );

        // ************  RUNNING  ************
        let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();
        graphics::set_blend_mode(ctx, BlendMode::Replace);
        let update_method = BackendEngine::Rayon;
        let ref mut state = Grid::new(ctx, update_method)
            .unwrap()
            .init_seed(init_b_matrix_vector);
        event::run(ctx, event_loop, state);
    }
}

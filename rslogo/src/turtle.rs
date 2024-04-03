use crate::errors::TurtleError;

/// The turtle is a construct in Logo which is responsible for generating the resulting image.
/// A Logo program is effectively a set of instructions on manipulating the turtle to produce the
/// desired image.
pub struct Turtle {
    x: f32,
    y: f32,
    heading: f32,
    pen_state: PenState,
    pen_color: f32,
}

impl Turtle {
    /// Returns a new instance of Turtle with default values
    pub fn new() -> Self {
        Turtle {
            x: 0_f32,
            y: 0_f32,
            heading: 0_f32,
            pen_state: PenState::Up,
            pen_color: 0_f32,
        }
    }

    /// Validates and sets the value for the pen colour for the turtle
    /// Returns the current pen colour when successful, and a ColourOutOfRange
    /// error otherwise.
    pub fn set_pen_colour(&mut self, value: f32) -> Result<f32, TurtleError> {
        match value {
            0_f32..=15_f32 => {
                self.pen_color = value;
                Ok(self.pen_color)
            }
            _ => Err(TurtleError::ColourOutOfRange(value)),
        }
    }

    /// Set the heading of the turtle to the given value. Returns the
    /// current heading of the turtle when successful, and an error otherwise
    pub fn set_heading(&mut self, angle: f32) -> Result<f32, TurtleError> {
        match angle {
            0_f32..=360_f32 => {
                self.heading = angle;
                Ok(self.heading)
            }
            _ => Err(TurtleError::AngleOutOfRange(angle)),
        }
    }

    /// Increments the heading of the turtle with the given value. Returns the
    /// current heading of the turtle when successful, and an error otherwise
    pub fn turn(&mut self, angle: f32) -> Result<f32, TurtleError> {
        match angle {
            0_f32..=360_f32 => {
                self.heading += angle;
                Ok(self.heading)
            }
            _ => Err(TurtleError::AngleOutOfRange(angle)),
        }
    }

    pub fn set_coordinates(
        &mut self,
        x: Option<f32>,
        y: Option<f32>,
    ) -> Result<(f32, f32), TurtleError> {
        match (x, y) {
            (None, None) => Ok((self.x, self.y)),
            (None, Some(y)) => {
                self.y = y;
                Ok((self.x, self.y))
            }
            (Some(x), None) => {
                self.x = x;
                Ok((self.x, self.y))
            }
            (Some(x), Some(y)) => {
                self.x = x;
                self.y = y;
                Ok((self.x, self.y))
            }
        }
    }

    pub fn move_turtle(
        &mut self,
        x: Option<f32>,
        y: Option<f32>,
    ) -> Result<(f32, f32), TurtleError> {
        match (x, y) {
            (None, None) => Ok((self.x, self.y)),
            (None, Some(y)) => {
                self.y += y;
                Ok((self.x, self.y))
            }
            (Some(x), None) => {
                self.x += x;
                Ok((self.x, self.y))
            }
            (Some(x), Some(y)) => {
                self.x += x;
                self.y += y;
                Ok((self.x, self.y))
            }
        }
    }

    pub fn set_pen_state(&mut self, state: PenState) -> &PenState {
        self.pen_state = state;
        &self.pen_state
    }

    pub fn get_pen_state(&self) -> &PenState {
        &self.pen_state
    }

    pub(crate) fn get_turtle_coords(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn get_pen_colour(&self) -> f32 {
        self.pen_color
    }

    pub fn get_heading(&self) -> f32 {
        self.heading
    }
}

#[derive(Debug, PartialEq)]
pub enum PenState {
    Up,
    Down,
}

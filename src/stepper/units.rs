#![allow(dead_code)]

use micromath::F32Ext;

#[derive(Clone, Copy)]
pub struct Position{
    value: f64
}
impl Position{
    pub fn from_mm(value: f64) -> Position{
        Position { value }
    }

    pub fn to_mm(self) -> f64 {
        self.value
    }

    pub fn subtract(&self, position: Position) -> Position{
        let value = position.to_mm() - self.to_mm();
        Position::from_mm(value)
    }

    pub fn add(&self, position: Position) -> Position{
        let value = position.to_mm() + self.to_mm();
        Position::from_mm(value)
    }
}

#[derive(Clone, Copy)]
pub struct Position2D{
    x: Position,
    y: Position,
}
impl Position2D{
    pub fn new(x: Position, y: Position) -> Position2D{
        Position2D { x, y }
    }
    pub fn get_x(&self) -> Position{
        self.x
    }

    pub fn get_y(&self) -> Position{
        self.y
    }

    pub fn get_angle(&self) -> f32 {
        (self.get_y().to_mm() as f32).atan2(self.get_x().to_mm() as f32)
    }

    pub fn angle(&self, position: Position2D) -> f32{
        let delta = self.subtract(position);
        (delta.get_y().to_mm() as f32).atan2(delta.get_x().to_mm() as f32)
    }

    pub fn subtract(&self, position: Position2D) -> Position2D{
        let delta_x = position.get_x().to_mm() - self.get_x().to_mm();
        let delta_y = position.get_y().to_mm() - self.get_y().to_mm();
        Position2D::new(Position::from_mm(delta_x), Position::from_mm(delta_y))
    }

    pub fn add(&self, position: Position2D) -> Position2D{
        let delta_x = position.get_x().to_mm() + self.get_x().to_mm();
        let delta_y = position.get_y().to_mm() + self.get_y().to_mm();
        Position2D::new(Position::from_mm(delta_x), Position::from_mm(delta_y))
    }
}


#[derive(Clone, Copy)]
pub struct Position3D {
    x: Position,
    y: Position,
    z: Position,
}
impl Position3D{
    pub fn new(x: Position, y: Position, z: Position) -> Position3D{
        Position3D { x, y, z }
    }
    pub fn get_x(&self) -> Position{
        self.x
    }

    pub fn get_y(&self) -> Position{
        self.y
    }

    pub fn get_z(&self) -> Position{
        self.z
    }

    pub fn subtract(&self, position: Position3D) -> Position3D{
        let x = position.get_x().to_mm() - self.get_x().to_mm();
        let y = position.get_y().to_mm() - self.get_y().to_mm();
        let z = position.get_z().to_mm() - self.get_z().to_mm();
        Position3D::new(Position::from_mm(x), Position::from_mm(y), Position::from_mm(z))
    }

    pub fn add(&self, position: Position3D) -> Position3D{
        let x = position.get_x().to_mm() + self.get_x().to_mm();
        let y = position.get_y().to_mm() + self.get_y().to_mm();
        let z = position.get_z().to_mm() + self.get_z().to_mm();
        Position3D::new(Position::from_mm(x), Position::from_mm(y), Position::from_mm(z))
    }
}

#[derive(Clone, Copy)]
pub struct Speed {
    // mm per second
    value: f64
}

impl Speed {
    pub fn from_mmps(value: f64) -> Result<Speed, ()>{
        if value.is_sign_negative(){
            return Result::Err(());
        }
        Result::Ok(Speed{
            value
        })
    }

    pub fn to_mmps(&self) -> f64{
        self.value
    }
}

#[derive(Clone, Copy)]
pub struct Length{
    // mm
    value: f64
}

impl Length{
    pub fn from_mm(value: f64) -> Result<Length, ()>{
        if value.is_sign_negative(){
            return Result::Err(());
        }
        Result::Ok(Length{
            value
        })
    }

    pub fn to_mm(self) -> f64{
        self.value
    }
}
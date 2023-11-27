#![allow(dead_code)]

use micromath::F32Ext;

#[derive(Clone, Copy)]
pub enum Unit {
    Millimeter,
    Inch,
}

#[derive(Clone, Copy)]
pub struct Position {
    // in mm
    value: f64,
}
impl Position {
    pub fn from_mm(mm: f64) -> Position {
        Position { value: mm }
    }

    pub fn from_inches(inches: f64) -> Position {
        Position {
            value: inches * 25.4,
        }
    }

    pub fn from_unit(value: f64, unit: Unit) -> Position {
        match unit {
            Unit::Millimeter => Position::from_mm(value),
            Unit::Inch => Position::from_inches(value),
        }
    }

    pub fn to_mm(self) -> f64 {
        self.value
    }

    pub fn subtract(&self, position: Position) -> Position {
        let value = position.to_mm() - self.to_mm();
        Position::from_mm(value)
    }

    pub fn add(&self, position: Position) -> Position {
        let value = position.to_mm() + self.to_mm();
        Position::from_mm(value)
    }
}

#[derive(Clone, Copy)]
pub struct Position2D {
    x: Position,
    y: Position,
}
impl Position2D {
    pub fn new(x: Position, y: Position) -> Position2D {
        Position2D { x, y }
    }
    pub fn get_x(&self) -> Position {
        self.x
    }

    pub fn get_y(&self) -> Position {
        self.y
    }

    pub fn get_angle(&self) -> f32 {
        (self.get_y().to_mm() as f32).atan2(self.get_x().to_mm() as f32)
    }

    pub fn get_magnitude(&self) -> Length {
        let magnitude =
            ((self.x.to_mm() * self.x.to_mm() + self.y.to_mm() * self.y.to_mm()) as f32).sqrt();
        Length::from_mm(magnitude as f64).unwrap()
    }

    pub fn angle(&self, position: Position2D) -> f32 {
        let delta = self.subtract(position);
        (delta.get_y().to_mm() as f32).atan2(delta.get_x().to_mm() as f32)
    }

    pub fn subtract(&self, position: Position2D) -> Position2D {
        let x = position.get_x().subtract(self.get_x());
        let y = position.get_y().subtract(self.get_y());
        Position2D::new(x, y)
    }

    pub fn add(&self, position: Position2D) -> Position2D {
        let x = position.get_x().add(self.get_x());
        let y = position.get_y().add(self.get_y());
        Position2D::new(x, y)
    }
}

#[derive(Clone, Copy)]
pub struct Position3D {
    x: Position,
    y: Position,
    z: Position,
}
impl Position3D {
    pub fn new(x: Position, y: Position, z: Position) -> Position3D {
        Position3D { x, y, z }
    }
    pub fn get_x(&self) -> Position {
        self.x
    }

    pub fn get_y(&self) -> Position {
        self.y
    }

    pub fn get_z(&self) -> Position {
        self.z
    }

    pub fn subtract(&self, position: Position3D) -> Position3D {
        let x = position.get_x().subtract(self.get_x());
        let y = position.get_y().subtract(self.get_y());
        let z = position.get_z().subtract(self.get_z());
        Position3D::new(x, y, z)
    }

    pub fn add(&self, position: Position3D) -> Position3D {
        let x = position.get_x().add(self.get_x());
        let y = position.get_y().add(self.get_y());
        let z = position.get_z().add(self.get_z());
        Position3D::new(x, y, z)
    }
}

#[derive(Clone, Copy)]
pub struct Speed {
    // mm per second
    value: f64,
}

impl Speed {
    pub fn from_mmps(value: f64) -> Result<Speed, ()> {
        if value.is_sign_negative() {
            return Result::Err(());
        }
        Result::Ok(Speed { value })
    }

    pub fn to_mmps(&self) -> f64 {
        self.value
    }
}

#[derive(Clone, Copy)]
pub struct Length {
    // mm
    value: f64,
}

impl Length {
    pub fn from_mm(value: f64) -> Result<Length, ()> {
        if value.is_sign_negative() {
            return Result::Err(());
        }
        Result::Ok(Length { value })
    }

    pub fn to_mm(self) -> f64 {
        self.value
    }
}

#[derive(Clone, Copy)]
pub struct Temperature{
    // unit: C (celsius)
    value: f64
}

impl Temperature{
    pub fn from_celsius(value: f64) -> Temperature{
        Temperature { value }
    }

    pub fn from_kelvin(value: f64) -> Temperature{
        Temperature { value: value - 273.15 } 
    }

    pub fn to_kelvin(&self) -> f64 {
        return self.value + 273.15
    }

    pub fn to_celsius(&self) -> f64 {
        return self.value
    }
}
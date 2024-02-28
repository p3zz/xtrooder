use super::{
    angle::{acos, atan2, Angle},
    common::sqrt,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unit {
    Millimeter,
    Inch,
    MillimeterPerSecond,
    InchPerSecond
}

pub struct Measurement{
    unit: Unit,
    value: f64
}

impl Measurement{

    pub fn new(value: f64, unit: Unit) -> Measurement{
        Measurement{value, unit}
    }

    pub fn from_mm(value: f64) -> Measurement {
        Measurement{value, unit: Unit::Millimeter}
    }

    pub fn from_inches(value: f64) -> Measurement {
        Measurement{value, unit: Unit::Inch}
    }

    pub fn from_mm_per_second(value: f64) -> Measurement {
        Measurement{value, unit: Unit::MillimeterPerSecond}
    }

    pub fn from_inches_per_second(value: f64) -> Measurement {
        Measurement{value, unit: Unit::InchPerSecond}
    }

    pub fn to_mm(&self) -> Option<f64> {
        match self.unit{
            Unit::Millimeter => Some(self.value),
            Unit::Inch => Some(self.value * 25.4),
            _ => None
        }
    }

    pub fn to_inches(&self) -> Option<f64> {
        match self.unit{
            Unit::Millimeter => Some(self.value / 25.4),
            Unit::Inch => Some(self.value),
            _ => None
        }
    }

    pub fn to_mm_per_second(&self) -> Option<f64> {
        match self.unit{
            Unit::MillimeterPerSecond => Some(self.value),
            Unit::InchPerSecond => Some(self.value * 25.4),
            _ => None
        }
    }

    pub fn to_inches_per_second(&self) -> Option<f64> {
        match self.unit{
            Unit::MillimeterPerSecond => Some(self.value / 25.4),
            Unit::InchPerSecond => Some(self.value),
            _ => None
        }
    }
}

pub struct Vector {
    x: f64,
    y: Option<f64>,
    z: Option<f64>,
    unit: Unit
}

impl Vector {
    // Constructor for 1D vector
    pub fn new_1d(x: f64, unit: Unit) -> Vector {
        Vector { x, y: None, z: None, unit }
    }

    // Constructor for 2D vector
    pub fn new_2d(x: f64, y: f64, unit: Unit) -> Vector {
        Vector { x, y: Some(y), z: None, unit }
    }

    // Constructor for 3D vector
    pub fn new_3d(x: f64, y: f64, z: f64, unit: Unit) -> Vector {
        Vector { x, y: Some(y), z: Some(z), unit }
    }

    pub fn add(&self, other: &Vector) -> Option<Vector> {
        if self.unit != other.unit{
            return None;
        }
        match (self, other){
            (
                Vector{x: x_a, y: None, z: None, unit: unit_a},
                Vector{x: x_b, y: None, z: None,unit: unit_b}
            ) => Some(Vector::new_1d(x_a + x_b, self.unit)),
            (
                Vector{x: x_a,y: Some(y_a),z: None, unit: unit_a},
                Vector{x: x_b,y: Some(y_b),z: None, unit: unit_b}
            ) => Some(Vector::new_2d(x_a + x_b, y_a + y_b, self.unit)),
            (
                Vector{x: x_a,y: Some(y_a),z: Some(z_a),unit: unit_a},
                Vector{x: x_b,y: Some(y_b),z: Some(z_b),unit: unit_b}
            ) => Some(Vector::new_3d(x_a + x_b, y_a + y_b, z_a + z_b, self.unit)),
            _ => None
        }
    }

    pub fn sub(&self, other: &Vector) -> Option<Vector> {
        if self.unit != other.unit{
            return None;
        }
        match (self, other){
            (
                Vector{x: x_a, y: None, z: None, unit: unit_a},
                Vector{x: x_b, y: None, z: None, unit: unit_b}
            ) => Some(Vector::new_1d(x_a - x_b, self.unit)),
            (
                Vector{x: x_a,y: Some(y_a),z: None, unit: unit_a},
                Vector{x: x_b,y: Some(y_b),z: None, unit: unit_b}
            ) => Some(Vector::new_2d(x_a - x_b, y_a - y_b, self.unit)),
            (
                Vector{x: x_a,y: Some(y_a),z: Some(z_a),unit: unit_a},
                Vector{x: x_b,y: Some(y_b),z: Some(z_b),unit: unit_b}
            ) => Some(Vector::new_3d(x_a - x_b, y_a - y_b, z_a - z_b, self.unit)),
            _ => None
        }
    }

    pub fn get_angle(&self) -> Option<Angle> {
        match self.y{
            Some(y) => Some(atan2(y, self.x)),
            None => None,
        }
    }

    pub fn get_magnitude(&self) -> Option<Measurement> {
        match (self.x, self.y, self.z){
            (x, None, None) => Some(Measurement::new(x, self.unit)),
            (x, Some(y), None) => Some(Measurement::new(sqrt(x * x + y * y), self.unit)) ,
            (x, Some(y), Some(z)) => Some(Measurement::new(sqrt(x * x + y * y + z * z), self.unit)),
            _ => None
        }
    }

    pub fn angle(&self, other: &Vector) -> Option<Angle> {
        match (self, other){
            (
                Vector{x: x_a, y: Some(y_a), z: None, unit: unit_a},
                Vector{x: x_b, y: Some(y_b), z: None, unit: unit_b}
            ) => {
                let n = self.dot(other).unwrap();
                let d = self.get_magnitude().unwrap().to_mm().unwrap() * other.get_magnitude().unwrap().to_mm().unwrap();
                Some(acos(n / d))
            },
            _ => None
        }
    }

    pub fn dot(&self, other: &Vector) -> Option<f64> {
        match (self, other){
            (
                Vector{x: x_a, y: Some(y_a), z: None, unit: unit_a},
                Vector{x: x_b, y: Some(y_b), z: None, unit: unit_b}
            ) => Some(x_a * x_b + y_a * y_b),
            _ => None
        }
    }

    pub fn normalize(&self) -> Option<Vector> {
        let mag = self.get_magnitude();
        if mag.is_none(){
            return None;
        }
        let m = mag.unwrap().to_mm().unwrap();
        match (self.y, self.z){
            (Some(y), None) => Some(Vector::new_2d(self.x / m, y / m, self.unit)),
            (Some(y), Some(z)) => Some(Vector::new_3d(self.x / m, y / m, z / m, self.unit)),
            _ => None
        }
    }

    pub fn get_x(&self) -> Measurement {
        Measurement::new(self.x, self.unit)
    }

    pub fn get_y(&self) -> Option<Measurement> {
        match self.y{
            Some(y) => Some(Measurement::new(y, self.unit)),
            None => None,
        }
    }

    pub fn get_z(&self) -> Option<Measurement> {
        match self.z{
            Some(z) => Some(Measurement::new(z, self.unit)),
            None => None,
        }
    }

    pub fn get_unit(&self) -> Unit{
        self.unit
    }

}
    
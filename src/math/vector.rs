use super::{angle::Angle, computable::Computable};
use micromath::F32Ext;

#[derive(Clone, Copy)]
pub enum Unit {
    Millimeter,
    Inch,
}

#[derive(Clone, Copy)]
pub struct Vector {
    // in mm
    value: f64,
}
impl Vector {
    pub fn from_mm(value: f64) -> Vector {
        Vector { value }
    }

    pub fn from_inches(inches: f64) -> Vector {
        Vector {
            value: inches * 25.4,
        }
    }

    pub fn from_unit(value: f64, unit: Unit) -> Vector {
        match unit {
            Unit::Millimeter => Vector::from_mm(value),
            Unit::Inch => Vector::from_inches(value),
        }
    }

    pub fn to_mm(self) -> f64 {
        self.value
    }

    pub fn mul(&self, other: Vector) -> Vector {
        Vector::from_mm(self.to_mm() * other.to_mm())
    }

    pub fn div(&self, other: Vector) -> Vector {
        Vector::from_mm(self.to_mm() / other.to_mm())
    }
}

impl Computable<Vector> for Vector {
    fn add(&self, other: Vector) -> Vector {
        Vector::from_mm(self.to_mm() + other.to_mm())
    }

    fn sub(&self, other: Vector) -> Vector {
        Vector::from_mm(self.to_mm() - other.to_mm())
    }
}

#[derive(Clone, Copy)]
pub struct Vector2D {
    x: Vector,
    y: Vector,
}
impl Vector2D {
    pub fn new(x: Vector, y: Vector) -> Vector2D {
        Vector2D { x, y }
    }
    pub fn get_x(&self) -> Vector {
        self.x
    }

    pub fn get_y(&self) -> Vector {
        self.y
    }

    pub fn get_angle(&self) -> Angle {
        let value = (self.get_y().to_mm() as f32).atan2(self.get_x().to_mm() as f32) as f64;
        Angle::from_radians(value)
    }

    pub fn get_magnitude(&self) -> Vector {
        let x = self.get_x().mul(self.x);
        let y = self.get_y().mul(self.y);
        let mag_sq = x.add(y);
        let mag = (mag_sq.to_mm() as f32).sqrt() as f64;
        Vector::from_mm(mag)
    }

    // θ = cos-1 [ (a · b) / (|a| |b|) ]
    pub fn angle(&self, vector: Vector2D) -> Angle {
        let n = self.dot(vector);
        let d = self.get_magnitude().mul(vector.get_magnitude());
        let res = (n.div(d).to_mm() as f32).asin();
        let value = res.asin() as f64;
        Angle::from_radians(value)
    }

    pub fn dot(&self, vector: Vector2D) -> Vector {
        let x = self.get_x().mul(vector.get_x());
        let y = self.get_y().mul(vector.get_y());
        x.mul(y)
    }
}

impl Computable<Vector2D> for Vector2D {
    fn add(&self, other: Vector2D) -> Vector2D {
        let x = other.get_x().add(self.get_x());
        let y = other.get_y().add(self.get_y());
        Vector2D::new(x, y)
    }

    fn sub(&self, other: Vector2D) -> Vector2D {
        let x = other.get_x().sub(self.get_x());
        let y = other.get_y().sub(self.get_y());
        Vector2D::new(x, y)
    }
}

#[derive(Clone, Copy)]
pub struct Vector3D {
    x: Vector,
    y: Vector,
    z: Vector,
}
impl Vector3D {
    pub fn new(x: Vector, y: Vector, z: Vector) -> Vector3D {
        Vector3D { x, y, z }
    }
    pub fn get_x(&self) -> Vector {
        self.x
    }

    pub fn get_y(&self) -> Vector {
        self.y
    }

    pub fn get_z(&self) -> Vector {
        self.z
    }
}

impl Computable<Vector3D> for Vector3D {
    fn add(&self, other: Vector3D) -> Vector3D {
        let x = other.get_x().add(self.get_x());
        let y = other.get_y().add(self.get_y());
        let z = other.get_z().add(self.get_z());
        Vector3D::new(x, y, z)
    }

    fn sub(&self, other: Vector3D) -> Vector3D {
        let x = other.get_x().sub(self.get_x());
        let y = other.get_y().sub(self.get_y());
        let z = other.get_z().sub(self.get_z());
        Vector3D::new(x, y, z)
    }
}
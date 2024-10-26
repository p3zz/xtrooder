use measurements::Measurement;
use core::ops::{Add, Sub, Mul, Div};

use super::{
    angle::{acos, atan2, Angle},
    common::sqrt,
};

#[derive(Clone, Copy)]
pub struct Vector2D<M> {
    x: M,
    y: M,
}

impl <M> Vector2D<M>
where M: Clone + Copy {
    pub fn new(x: M, y: M) -> Self {
        Vector2D { x, y }
    }

    pub fn get_x(&self) -> M {
        self.x
    }

    pub fn get_y(&self) -> M {
        self.y
    }
}

impl <M> Vector2D<M>
where M: Clone + Copy + Measurement{
    pub fn get_angle(&self) -> Angle {
        atan2(self.y.as_base_units(), self.x.as_base_units())
    }
}

impl<M> Vector2D<M>
where
    M: Clone + Copy + Measurement + Add<Output = M> + Sub<Output = M> + Mul<Output = M> + Div<Output = M>,
{
    pub fn get_magnitude(&self) -> M {
        let x = self.x * self.x;
        let y = self.y * self.y;
        let v = sqrt((x + y).as_base_units());
        M::from_base_units(v)
    }

    pub fn angle(&self, other: &Self) -> Result<Angle, ()> {
        let n = self.dot(other);
        let mag = self.get_magnitude();
        let d = mag * mag;
        let res = n / d;
        Ok(acos(res.as_base_units()))
    }

    pub fn dot(&self, other: &Self) -> M {
        self.x * other.x + self.y * other.y
    }

    pub fn normalize(&self) -> Vector2D<f64> {
        let mag = self.get_magnitude();
        let x = self.x / mag;
        let y = self.y / mag;
        Vector2D::new(x.as_base_units(), y.as_base_units())
    }

}

impl<M> Add for Vector2D<M>
where M: Add<Output = M> + Clone + Copy{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let x = self.x + rhs.x;
        let y = self.y + rhs.y;
        Self::new(x, y)
    }
}

impl<M> Sub for Vector2D<M>
where M: Sub<Output = M> + Clone + Copy{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let x = self.x - rhs.x;
        let y = self.y - rhs.y;
        Self::new(x, y)
    }
}

#[derive(Clone, Copy)]
pub struct Vector3D<M> {
    x: M,
    y: M,
    z: M,
}

impl <M> Vector3D<M>
where M: Clone + Copy {
    pub fn new(x: M, y: M, z: M) -> Vector3D<M> {
        Vector3D { x, y, z }
    }

    pub fn get_x(&self) -> M {
        self.x
    }

    pub fn get_y(&self) -> M {
        self.y
    }

    pub fn get_z(&self) -> M {
        self.z
    }   
}

impl<M> Vector3D<M>
where
    M: Measurement + Add<Output = M> + Sub<Output = M> + Mul<Output = M> + Div<Output = M> + Clone + Copy,
{
    pub fn get_magnitude(&self) -> M {
        let x = self.x * self.x;
        let y = self.y * self.y;
        let z = self.z * self.z;
        let v = sqrt((x + y + z).as_base_units());
        M::from_base_units(v)
    }

    pub fn normalize(&self) -> Self {
        let mag = self.get_magnitude();
        let x = self.x / mag;
        let y = self.y / mag;
        let z = self.z / mag;
        Self::new(x, y, z)
    }
}

impl<M> Add for Vector3D<M>
where M: Add<Output = M> + Clone + Copy{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let x = self.x + rhs.x;
        let y = self.y + rhs.y;
        let z = self.z + rhs.z;
        Self::new(x, y, z)
    }
}

impl<M> Sub for Vector3D<M>
where M: Sub<Output = M> + Clone + Copy{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let x = self.x - rhs.x;
        let y = self.y - rhs.y;
        let z = self.z - rhs.z;
        Self::new(x, y, z)
    }
}

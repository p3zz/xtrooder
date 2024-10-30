use measurements::Measurement;
use core::ops::{Add, Sub};

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
    M: Clone + Copy + Measurement,
{
    pub fn get_magnitude(&self) -> M {
        let x = self.x.as_base_units() * self.x.as_base_units();
        let y = self.y.as_base_units() * self.y.as_base_units();
        let v = sqrt(x + y);
        M::from_base_units(v)
    }

    pub fn angle(&self, other: &Self) -> Angle {
        let n = self.dot(other);
        let mag = self.get_magnitude();
        let d = mag.as_base_units() * mag.as_base_units();
        let res = n / d;
        acos(res)
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x.as_base_units() * other.x.as_base_units() + self.y.as_base_units() * other.y.as_base_units()
    }

    pub fn normalize(&self) -> Vector2D<f64> {
        let mag = self.get_magnitude();
        let x = self.x.as_base_units() / mag.as_base_units();
        let y = self.y.as_base_units() / mag.as_base_units();
        Vector2D::new(x, y)
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
    M: Clone + Copy + Measurement,
{
    pub fn get_magnitude(&self) -> M {
        let x = self.x.as_base_units() * self.x.as_base_units();
        let y = self.y.as_base_units() * self.y.as_base_units();
        let z = self.z.as_base_units() * self.z.as_base_units();
        let v = sqrt(x + y + z);
        M::from_base_units(v)
    }

    pub fn normalize(&self) -> Vector3D<f64> {
        let mag = self.get_magnitude();
        let x = self.x.as_base_units() / mag.as_base_units();
        let y = self.y.as_base_units() / mag.as_base_units();
        let z = self.z.as_base_units() / mag.as_base_units();
        Vector3D::new(x, y, z)
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

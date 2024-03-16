use super::{
    angle::{acos, atan2, Angle},
    common::sqrt, measurable::Measurable,
};

impl Measurable for f64{
    fn add(&self, other: &Self) -> Self {
        self + other
    }
    
    fn sub(&self, other: &Self) -> Self {
        self - other
    }
    
    fn mul(&self, other: &Self) -> Self {
        self * other
    }
    
    fn div(&self, other: &Self) -> Result<f64, ()> {
        Ok(self / other)
    }
    
    fn to_raw(&self) -> Self {
        *self
    }
    
    fn from_raw(value: f64) -> Self {
        value
    }
}

#[derive(Clone, Copy)]
pub struct Vector2D<M> {
    x: M,
    y: M,
}

impl <M> Vector2D<M>
where M: Measurable + Clone + Copy {

    pub fn new(x: M, y: M) -> Vector2D<M> {
        Vector2D { x, y }
    }

    pub fn add(&self, other: &Vector2D<M>) -> Vector2D<M> {
        Vector2D::new(self.x.add(&other.x), self.y.add(&other.y))
    }

    pub fn sub(&self, other: &Vector2D<M>) -> Vector2D<M> {
        Vector2D::new(self.x.sub(&other.x), self.y.sub(&other.y))
    }

    pub fn get_magnitude(&self) -> M {
        let x = self.x.mul(&self.x);
        let y = self.y.mul(&self.y);
        let v = sqrt(x.add(&y).to_raw());
        M::from_raw(v)
    }
    
    pub fn get_angle(&self) -> Angle{
        atan2(self.y.to_raw(), self.x.to_raw())
    }

    pub fn angle(&self, other: &Vector2D<M>) -> Result<Angle, ()> {
        let n = self.dot(other);
        let mag = self.get_magnitude();
        let d = mag.mul(&mag);
        let res = n.div(&d)?;
        Ok(acos(res))
    }

    pub fn dot(&self, other: &Vector2D<M>) -> M {
        self.x.mul(&other.x).add(&self.y.mul(&other.y))
    }

    pub fn normalize(&self) -> Result<Vector2D<f64>, ()> {
        let mag = self.get_magnitude();
        let x = self.x.div(&mag)?;
        let y = self.y.div(&mag)?;
        Ok(Vector2D::new(x, y))
    }

    pub fn get_x(&self) -> M {
        self.x
    }

    pub fn get_y(&self) -> M {
        self.y
    }

}

#[derive(Clone, Copy)]
pub struct Vector3D<M> {
    x: M,
    y: M,
    z: M,
}

impl <M> Vector3D<M>
where M: Measurable + Clone + Copy {

    pub fn new(x: M, y: M, z: M) -> Vector3D<M> {
        Vector3D { x, y, z }
    }

    pub fn add(&self, other: &Vector3D<M>) -> Vector3D<M> {
        Vector3D::new(self.x.add(&other.x), self.y.add(&other.y), self.z.add(&other.z))
    }

    pub fn sub(&self, other: &Vector3D<M>) -> Vector3D<M> {
        Vector3D::new(self.x.sub(&other.x), self.y.sub(&other.y), self.z.sub(&other.z))
    }

    pub fn get_magnitude(&self) -> M {
        let x = self.x.mul(&self.x);
        let y = self.y.mul(&self.y);
        let z = self.z.mul(&self.z);
        let value = sqrt(x.add(&y).add(&z).to_raw());
        M::from_raw(value)
    }
    
    pub fn normalize(&self) -> Result<Vector3D<f64>, ()> {
        let mag = self.get_magnitude();
        let x = self.x.div(&mag)?;
        let y = self.y.div(&mag)?;
        let z = self.z.div(&mag)?;
        Ok(Vector3D::new(x, y, z))
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
    
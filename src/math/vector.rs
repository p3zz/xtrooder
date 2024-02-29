use super::{
    angle::{acos, atan2, Angle},
    common::sqrt, measurable::Measurable,
};

#[derive(Clone, Copy, Debug, PartialEq)]

pub struct Vector2D<M> {
    x: M,
    y: M,
}

impl <M> Vector2D<M>
where M: Measurable<M>{

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
        let value = sqrt(self.x.sqr().add(&self.y.sqr()).get_value());
        M::from_value(value)
    }
    
    pub fn get_angle(&self) -> Angle{
        atan2(self.y.get_value(), self.x.get_value())
    }

    pub fn angle(&self, other: &Vector2D<M>) -> Result<Angle, ()> {
        let n = self.dot(other);
        let d = self.get_magnitude().sqr();
        let res = n.div(&d)?;
        Ok(acos(res))
    }

    pub fn dot(&self, other: &Vector2D<M>) -> M {
        self.x.mul(&other.x).add(&self.y.mul(&other.y))
    }

    pub fn normalize(&self) -> Result<Vector2D<M>, ()> {
        let mag = self.get_magnitude();
        let x = self.x.div(&mag)?;
        let y = self.y.div(&mag)?;
        Ok(Vector2D::new(M::from_value(x), M::from_value(y)))
    }

    pub fn get_x(&self) -> &M {
        &self.x
    }

    pub fn get_y(&self) -> &M {
        &self.y
    }

}

pub struct Vector3D<M> {
    x: M,
    y: M,
    z: M,
}

impl <M> Vector3D<M>
where M: Measurable<M>{

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
        let value = sqrt(self.x.sqr().add(&self.y.sqr()).add(&self.z.sqr()).get_value());
        M::from_value(value)
    }
    
    pub fn normalize(&self) -> Result<Vector3D<M>, ()> {
        let mag = self.get_magnitude();
        let x = self.x.div(&mag)?;
        let y = self.y.div(&mag)?;
        let z = self.z.div(&mag)?;
        Ok(Vector3D::new(M::from_value(x), M::from_value(y), M::from_value(z)))
    }

    pub fn get_x(&self) -> &M {
        &self.x
    }

    pub fn get_y(&self) -> &M {
        &self.y
    }

    pub fn get_z(&self) -> &M {
        &self.z
    }
}
    
pub trait Computable{
    fn from_raw(value: f64) -> Self;
    fn to_raw(&self) -> f64;
    fn add(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
    fn mul(&self, other: &Self) -> Self;
    fn div(&self, other: &Self) -> Result<f64, ()>;
}

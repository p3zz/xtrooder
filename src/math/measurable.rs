pub trait Measurable<T>{
    fn from_value(value: f64) -> T;
    fn get_value(&self) -> f64;
    fn add(&self, other: &T) -> T;
    fn sub(&self, other: &T) -> T;
    fn mul(&self, other: &T) -> T;
    fn div(&self, other: &T) -> Result<f64, ()>;
    fn sqr(&self) -> T;
}

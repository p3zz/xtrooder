pub trait Computable<T>{
    fn add(&self, other: T) -> T;
    fn sub(&self, other: T) -> T;
}

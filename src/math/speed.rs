use super::{distance::{Distance, DistanceUnit}, computable::Computable};

#[derive(Clone, Copy)]
pub struct Speed{
    // mm per second
    value: f64
}

impl Speed{

    pub fn from_unit(value: f64, unit: DistanceUnit) -> Self {
        match unit{
            DistanceUnit::Millimeter => Self::from_mm_per_second(value),
            DistanceUnit::Inch => Self::from_inches_per_second(value),
        }
    }

    pub fn from_mm_per_second(value: f64) -> Self {
        Self{value}
    }

    pub fn from_inches_per_second(value: f64) -> Self {
        Self{value: value * 25.4}
    }

    pub fn from_revolutions_per_second(value: f64, steps_per_revolution: u64, distance_per_step: Distance) -> Self {
        let distance_per_revolution = Distance::from_mm(steps_per_revolution as f64 * distance_per_step.to_mm());
        Self{value: distance_per_revolution.to_mm() * value}
    }

    pub fn to_mm_per_second(&self) -> f64 {
        self.value
    }

    pub fn to_inches_per_second(&self) -> f64 {
        self.value / 25.4
    }

    pub fn to_revolutions_per_second(&self, steps_per_revolution: u64, distance_per_step: Distance) -> f64 {
        let distance_per_revolution = Distance::from_mm(steps_per_revolution as f64 * distance_per_step.to_mm());
        if distance_per_revolution.to_mm() == 0f64{
            return 0f64;
        }
        self.value / distance_per_revolution.to_mm()
    }
}

impl Computable for Speed{
    
    fn add(&self, other: &Self) -> Self {
        let value = self.to_mm_per_second() + other.to_mm_per_second();
        Self::from_mm_per_second(value)
    }
    
    fn sub(&self, other: &Self) -> Self {
        let value = self.to_mm_per_second() - other.to_mm_per_second();
        Self::from_mm_per_second(value)
    }

    fn mul(&self, other: &Self) -> Self {
        let value = self.to_mm_per_second() * other.to_mm_per_second();
        Self::from_mm_per_second(value)
    }

    fn div(&self, other: &Self) -> Result<f64, ()> {
        if other.to_mm_per_second() == 0f64{
            return Err(());
        }
        let value = self.to_mm_per_second() / other.to_mm_per_second();
        Ok(value)
    }

    fn to_raw(&self) -> f64 {
        self.to_mm_per_second()
    }
    
    fn from_raw(value: f64) -> Self {
        Self::from_mm_per_second(value)
    }
}

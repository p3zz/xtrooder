use super::{distance::{Distance, DistanceUnit}, measurable::Measurable};

pub struct Speed{
    // mm per second
    value: f64
}

impl Speed{

    pub fn from_unit(value: f64, unit: DistanceUnit) -> Speed {
        match unit{
            DistanceUnit::Millimeter => Speed::from_mm_per_second(value),
            DistanceUnit::Inch => Speed::from_inches_per_second(value),
        }
    }

    pub fn from_mm_per_second(value: f64) -> Speed {
        Speed{value}
    }

    pub fn from_inches_per_second(value: f64) -> Speed {
        Speed{value: value * 25.4}
    }

    pub fn from_revolutions_per_second(value: f64, steps_per_revolution: u64, distance_per_step: Distance) -> Speed {
        let distance_per_revolution = Distance::from_mm(steps_per_revolution as f64 * distance_per_step.to_mm());
        Speed{value: distance_per_revolution.to_mm() * value}
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

impl Measurable<Speed> for Speed{
    
    fn from_value(value: f64) -> Speed {
        Speed{value}
    }

    fn get_value(&self) -> f64 {
        self.value
    }
    
    fn add(&self, other: &Speed) -> Speed {
        let value = self.to_mm_per_second() + other.to_mm_per_second();
        Speed::from_mm_per_second(value)
    }
    
    fn sub(&self, other: &Speed) -> Speed {
        let value = self.to_mm_per_second() - other.to_mm_per_second();
        Speed::from_mm_per_second(value)
    }

    fn mul(&self, other: &Speed) -> Speed {
        let value = self.to_mm_per_second() * other.to_mm_per_second();
        Speed::from_mm_per_second(value)
    }

    fn div(&self, other: &Speed) -> Result<f64, ()> {
        if other.to_mm_per_second() == 0f64{
            return Err(());
        }
        let value = self.to_mm_per_second() / other.to_mm_per_second();
        Ok(value)
    }

    fn sqr(&self) -> Speed {
        let value = self.to_mm_per_second() * self.to_mm_per_second();
        Speed::from_inches_per_second(value)
    }
}

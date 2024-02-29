use super::measurable::Measurable;

pub enum DistanceUnit{
    Millimeter,
    Inch
}

pub struct Distance{
    // mm
    value: f64
}

impl Distance{
    pub fn from_mm(value: f64) -> Distance {
        Distance{value}
    }

    pub fn from_inches(value: f64) -> Distance {
        Distance{value: value * 25.4}
    }

    pub fn to_mm(&self) -> f64 {
        self.value
    }

    pub fn to_inches(&self) -> f64 {
        self.value / 25.4
    }
}

impl Measurable<Distance> for Distance{
    
    fn from_value(value: f64) -> Distance {
        Distance{value}
    }

    fn get_value(&self) -> f64 {
        self.value
    }
    
    fn add(&self, other: &Distance) -> Distance {
        let value = self.to_mm() + other.to_mm();
        Distance::from_mm(value)
    }
    
    fn sub(&self, other: &Distance) -> Distance {
        let value = self.to_mm() - other.to_mm();
        Distance::from_mm(value)
    }

    fn mul(&self, other: &Distance) -> Distance {
        let value = self.to_mm() * other.to_mm();
        Distance::from_mm(value)
    }

    fn div(&self, other: &Distance) -> Result<f64, ()> {
        if other.to_mm() == 0f64{
            Err(())
        }
        else{
            Ok(self.to_mm() / other.to_mm())
        }
    }

    fn sqr(&self) -> Distance {
        let value = self.to_mm() * self.to_mm();
        Distance::from_mm(value)
    }
}

#[derive(Clone, Copy)]
pub struct Temperature {
    // unit: C (celsius)
    value: f64,
}

impl Temperature {
    pub fn from_celsius(value: f64) -> Temperature {
        Temperature { value }
    }

    pub fn from_kelvin(value: f64) -> Temperature {
        Temperature {
            value: value - 273.15,
        }
    }

    pub fn to_kelvin(&self) -> f64 {
        return self.value + 273.15;
    }

    pub fn to_celsius(&self) -> f64 {
        return self.value;
    }
}

#[derive(Clone, Copy)]
pub enum TemperatureUnit {
    Celsius,
    Kelvin,
    Farhenheit,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Temperature {
    // unit: C (celsius)
    value: f64,
}

impl Temperature {
    pub fn from_unit(value: f64, unit: TemperatureUnit) -> Self {
        match unit {
            TemperatureUnit::Celsius => Self::from_celsius(value),
            TemperatureUnit::Kelvin => Self::from_kelvin(value),
            TemperatureUnit::Farhenheit => Self::from_fahrenheit(value),
        }
    }

    pub fn from_celsius(value: f64) -> Self {
        Self { value }
    }

    pub fn from_kelvin(value: f64) -> Self {
        Self {
            value: value - 273.15,
        }
    }

    pub fn from_fahrenheit(value: f64) -> Self {
        Self {
            value: (value - 32.0) * (5.0 / 9.0),
        }
    }

    pub fn to_kelvin(&self) -> f64 {
        self.value + 273.15
    }

    pub fn to_fahrenheit(&self) -> f64 {
        (self.value * 9.0 / 5.0) + 32.0
    }

    pub fn to_celsius(&self) -> f64 {
        self.value
    }
}

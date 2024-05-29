use super::computable::Computable;

#[derive(Clone, Copy)]
pub enum DistanceUnit {
    Millimeter,
    Inch,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Distance {
    // mm
    value: f64,
}

impl Distance {
    pub fn from_unit(value: f64, unit: DistanceUnit) -> Self {
        match unit {
            DistanceUnit::Millimeter => Self::from_mm(value),
            DistanceUnit::Inch => Self::from_inches(value),
        }
    }

    pub fn from_mm(value: f64) -> Self {
        Self { value }
    }

    pub fn from_inches(value: f64) -> Self {
        Self {
            value: value * 25.4,
        }
    }

    pub fn to_mm(&self) -> f64 {
        self.value
    }

    pub fn to_inches(&self) -> f64 {
        self.value / 25.4
    }
}

impl Computable for Distance {
    fn add(&self, other: &Self) -> Self {
        let value = self.to_mm() + other.to_mm();
        Self::from_mm(value)
    }

    fn sub(&self, other: &Self) -> Self {
        let value = self.to_mm() - other.to_mm();
        Self::from_mm(value)
    }

    fn mul(&self, other: &Self) -> Self {
        let value = self.to_mm() * other.to_mm();
        Self::from_mm(value)
    }

    fn div(&self, other: &Self) -> Result<f64, ()> {
        if other.to_mm() == 0f64 {
            Err(())
        } else {
            Ok(self.to_mm() / other.to_mm())
        }
    }

    fn to_raw(&self) -> f64 {
        self.to_mm()
    }

    fn from_raw(value: f64) -> Self {
        Self::from_mm(value)
    }
}

#[cfg(feature = "defmt-log")]
impl defmt::Format for Distance{
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{} mm", self.to_mm())
    }
}

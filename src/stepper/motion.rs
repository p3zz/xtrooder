#[no_std]

#[derive(Clone, Copy)]
pub struct Position1D{
    value: f64
}
impl Position1D{
    pub fn from_mm(value: f64) -> Position1D{
        Position1D { value }
    }

    pub fn to_mm(self) -> f64 {
        self.value
    }
}

#[derive(Clone, Copy)]
pub struct Position3D {
    x: Position1D,
    y: Position1D,
    z: Position1D,
}
impl Position3D{
    pub fn new(x: Position1D, y: Position1D, z: Position1D) -> Position3D{
        Position3D { x, y, z }
    }
    pub fn get_x(&self) -> Position1D{
        self.x
    }

    pub fn get_y(&self) -> Position1D{
        self.y
    }

    pub fn get_z(&self) -> Position1D{
        self.z
    }
}

#[derive(Clone, Copy)]
pub struct Speed {
    // mm per second
    value: f64
}

impl Speed {
    pub fn from_mmps(value: f64) -> Result<Speed, ()>{
        if value.is_sign_negative(){
            return Result::Err(());
        }
        Result::Ok(Speed{
            value
        })
    }

    pub fn to_mmps(&self) -> f64{
        self.value
    }
}

#[derive(Clone, Copy)]
pub struct Length{
    // mm
    value: f64
}

impl Length{
    pub fn from_mm(value: f64) -> Result<Length, ()>{
        if value.is_sign_negative(){
            return Result::Err(());
        }
        Result::Ok(Length{
            value
        })
    }

    pub fn to_mm(self) -> f64{
        self.value
    }
}

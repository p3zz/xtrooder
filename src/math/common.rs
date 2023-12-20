use micromath::F32Ext;

pub fn abs(value: f64) -> f64 {
    (value as f32).abs() as f64
}

pub fn sqrt(value: f64) -> f64 {
    (value as f32).sqrt() as f64
}
use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::nalgebra::{Point3, UnitQuaternion, Vector3};
use kiss3d::window::Window;

pub struct AxisColor {
    red: Point3<f32>,
    green: Point3<f32>,
    blue: Point3<f32>,
}

impl AxisColor {
    pub fn new() -> AxisColor {
        AxisColor {
            red: Point3::new(1.0, 0.0, 0.0),
            green: Point3::new(0.0, 1.0, 0.0),
            blue: Point3::new(0.0, 0.0, 1.0),

        }
    }
}

struct GraphicLine {
    origin: Point3<f32>,
    destination: Point3<f32>,
    color: Point3<f32>,
}

impl GraphicLine {
    pub fn new(origin: Point3<f32>, destination: Point3<f32>, color: Point3<f32>) -> GraphicLine {
        GraphicLine {
            origin,
            destination,
            color,
        }
    }
}

struct GraphicAxis {
    pub x: GraphicLine,
    pub y: GraphicLine,
    pub z: GraphicLine,
}

impl GraphicAxis {
    pub fn new() -> GraphicAxis {
        let color = AxisColor::new();
        let origin: Point3<f32> = Point3::new(0.0, 0.0, 0.0);
        let length: f32 = 10.0;
        GraphicAxis {
            x: GraphicLine::new(origin, Point3::new(length, 0.0, 0.0), color.red),
            y: GraphicLine::new(origin, Point3::new(0.0, length, 0.0), color.blue),
            z: GraphicLine::new(origin, Point3::new(0.0, 0.0, length), color.green),
        }
    }
}

pub struct GraphicEnvironment {
    pub window: Window,
    axis: GraphicAxis,
}

impl GraphicEnvironment {
    pub fn new(window_name: &str) -> GraphicEnvironment {
        GraphicEnvironment {
            window: Window::new(window_name),
            axis: GraphicAxis::new(),
        }
    }

    pub fn init(&mut self) -> () {
        self.window.set_light(Light::StickToCamera);
    }

    pub fn draw_axis(&mut self) {
        self.window.draw_line(&self.axis.x.origin, &self.axis.x.destination, &self.axis.x.color);
        self.window.draw_line(&self.axis.y.origin, &self.axis.y.destination, &self.axis.y.color);
        self.window.draw_line(&self.axis.z.origin, &self.axis.z.destination, &self.axis.z.color);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let serialport = args.get(1).expect("Serial port not specified");
    let file_path = args.get(2).expect("File not specified");

    let file = fs::File::open(file_path).expect("File not found");
    let mut bufreader = BufReader::new(file);

    let mut port = serialport::new(serialport, 19200)
        .open()
        .expect("Failed to open port");

    let mut buf = [0u8; 32];
    let mut line = String::new();
    let mut v: Vec<u8> = Vec::new();

    // FIXME ugly as shit pls refactor
    loop {
        if let Ok(n) = port.read(&mut buf) {
            for i in 0..n {
                let elem = *buf.get(i).unwrap();
                if elem == b'#' {
                    let s = String::from_utf8(v.clone()).unwrap();
                    if s.as_str() == "next" {
                        match bufreader.read_line(&mut line) {
                            Ok(_) => {
                                println!("{}", line);
                                port.write_all(line.as_bytes()).unwrap();
                                line.clear();
                            }
                            Err(_) => print!("error while reading file"),
                        }
                    }
                    v.clear();
                } else {
                    v.push(elem);
                }
            }
        }
    }
}

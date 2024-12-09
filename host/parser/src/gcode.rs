use core::{
    fmt::{Display, Write},
    str::FromStr,
    time::Duration,
};

use heapless::{LinearMap, String, Vec};
use math::{
    measurements::{Distance, Speed, Temperature},
    DistanceUnit, DurationUnit, TemperatureUnit,
};

pub enum GCommandType {
    G,
    M,
    D,
}

#[derive(PartialEq, Debug, Clone)]
pub enum GCommand {
    // https://marlinfw.org/docs/gcode/G000-G001.html
    G0 {
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        f: Option<Speed>,
    },
    G1 {
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
        f: Option<Speed>,
    },
    G2 {
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
        f: Option<Speed>,
        i: Option<Distance>,
        j: Option<Distance>,
        r: Option<Distance>,
    },
    G3 {
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
        f: Option<Speed>,
        i: Option<Distance>,
        j: Option<Distance>,
        r: Option<Distance>,
    },
    // dwell
    G4 {
        p: Option<Duration>,
        s: Option<Duration>,
    },
    // retract
    G10,
    // recover
    G11,
    // set distance unit to inches
    G20,
    // set distance unit to millimeters
    G21,
    // auto home
    G28 {
        x: bool,
        y: bool,
        z: bool,
    },
    // set positioning as absolute
    G90,
    // set positioning as relative
    G91,
    // TODO set steppers position
    // G92,
    // List SD Card files
    M20,
    // Init SD card (mount fs)
    M21,
    // Simulate ejection of the SD card
    M22,
    // Select SD file
    M23 {
        filename: String<12>,
    },
    // Start or Resume SD print
    M24 {
        s: u64,
        t: Duration,
    },
    // Pause SD print
    M25,
    // [future] Report SD print status
    M27,
    // report print time
    M31,
    // set hotend temperature
    M104 {
        s: Temperature,
    },
    // report temperatures
    M105,
    // set fan speed
    // 0 to 255, 255 -> max speed
    M106 {
        s: u8,
    },
    // [future] wait for hotend temperature
    M109 {
        r: Temperature,
        s: Temperature,
    },
    // report position
    M114,
    // report fan speed
    // optional duration s
    M123 {
        s: Option<Duration>,
    },
    // set bed temperature
    M140 {
        s: Temperature,
    },
    // set temperature unit
    M149 {
        u: TemperatureUnit,
    },
    // position auto-report
    M154 {
        s: Duration,
    },
    // temperature auto-report
    M155 {
        s: Duration,
    },
    // [future] wait for bed temperature
    M190 {
        r: Temperature,
        s: Temperature,
    },
    // [future] wait for probe temperature
    M192 {
        r: Temperature,
        s: Temperature,
    },
    // [future] set max feedrate
    M203 {
        x: Speed,
        y: Speed,
        z: Speed,
        e: Speed,
    },
    // firmware retraction settings
    M207 {
        f: Speed,
        s: Distance,
        z: Distance,
    },
    // firmware recover settings
    M208 {
        f: Speed,
        s: Distance,
    },
    // set feedrate multiplier
    M220 {
        s: f64,
    },
    // set flow multiplier
    M221 {
        s: f64,
    },
    // abort sd print
    // TODO
    M524,
    // debug command - linear movement xyz
    D0 {
        x: Distance,
        y: Distance,
        z: Distance,
        t: Duration,
    },
    // debug command - linear movement xyze
    D1 {
        x: Distance,
        y: Distance,
        z: Distance,
        e: Distance,
        t: Duration,
    },
    // debug command - enable debug report
    D114,
    // debug command - disable debug report
    D115,
}

// #[cfg(feature = "defmt-log")]
// impl defmt::Format for GCommand {
//     fn format(&self, fmt: defmt::Formatter) {
//         match self {
//             GCommand::G0 { x, y, z, f } => {
//                 defmt::write!(fmt, "G0 [x: {}] [y: {}] [z: {}] [f: {}]", x, y, z, f)
//             }
//             GCommand::G1 { x, y, z, e, f } => defmt::write!(
//                 fmt,
//                 "G1 [x: {}] [y: {}] [z: {}] [e: {}] [f: {}]",
//                 x,
//                 y,
//                 z,
//                 e,
//                 f
//             ),
//             GCommand::G2 {
//                 x,
//                 y,
//                 z,
//                 e,
//                 f,
//                 i,
//                 j,
//                 r,
//             } => defmt::write!(
//                 fmt,
//                 "G2 [x: {}] [y: {}] [z: {}] [e: {}] [f: {}] [i: {}] [j: {}] [r: {}]",
//                 x,
//                 y,
//                 z,
//                 e,
//                 f,
//                 i,
//                 j,
//                 r
//             ),
//             GCommand::G3 {
//                 x,
//                 y,
//                 z,
//                 e,
//                 f,
//                 i,
//                 j,
//                 r,
//             } => defmt::write!(
//                 fmt,
//                 "G3 [x: {}] [y: {}] [z: {}] [e: {}] [f: {}] [i: {}] [j: {}] [r: {}]",
//                 x,
//                 y,
//                 z,
//                 e,
//                 f,
//                 i,
//                 j,
//                 r
//             ),
//             GCommand::G4 { p, s } => defmt::write!(fmt, "G4 [p: {}] [s: {}]", p, s),
//             GCommand::G20 => defmt::write!(fmt, "G20"),
//             GCommand::G21 => defmt::write!(fmt, "G21"),
//             GCommand::G90 => todo!(),
//             GCommand::G91 => todo!(),
//             GCommand::M104 { s } => defmt::write!(fmt, "M104 [S: {}]", s),
//             GCommand::M140 { s } => defmt::write!(fmt, "M140 [S: {}]", s),
//             GCommand::M149 { u } => defmt::write!(fmt, "M149 [U: {}]", u),
//             GCommand::M20 => defmt::write!(fmt, "M20"),
//             GCommand::M21 => defmt::write!(fmt, "M21"),
//             GCommand::M22 => defmt::write!(fmt, "M22"),
//             GCommand::M23 { filename } => defmt::write!(fmt, "M23 [{}]", filename.as_str()),
//             GCommand::M24 { s, t } => defmt::write!(fmt, "M24 [s: {}] [t: {}]", s, t.as_millis()),
//             GCommand::M25 => defmt::write!(fmt, "M25"),
//             GCommand::M27 => defmt::write!(fmt, "M27"),
//             GCommand::D0 { x, y, z, t } => {
//                 defmt::write!(fmt, "D0 [x: {}] [y: {}] [z: {}] [t: {}]", x, y, z, t)
//             }
//             GCommand::D1 { x, y, z, e, t } => defmt::write!(
//                 fmt,
//                 "D1 [x: {}] [y: {}] [z: {}] [e: {}] [t: {}]",
//                 x,
//                 y,
//                 z,
//                 e,
//                 t
//             ),
//             _ => panic!("Format not implemented"),
//         }
//     }
// }

impl Display for GCommand {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GCommand::G0 { x, y, z, f } => {
                let mut x_t: String<16> = String::new();
                let mut y_t: String<16> = String::new();
                let mut z_t: String<16> = String::new();
                let mut f_t: String<16> = String::new();
                if let Some(x) = x {
                    core::write!(&mut x_t, " X{}", x.as_millimeters())?;
                }
                if let Some(y) = y {
                    core::write!(&mut y_t, " Y{}", y.as_millimeters())?;
                }
                if let Some(z) = z {
                    core::write!(&mut z_t, " Z{}", z.as_millimeters())?;
                }
                if let Some(f) = f {
                    core::write!(&mut f_t, " F{}", f.as_meters_per_second() * 1000.0)?;
                }
                core::write!(fmt, "G0{}{}{}{}", x_t, y_t, z_t, f_t)
            }
            GCommand::G1 { x, y, z, e, f } => {
                let mut x_t: String<16> = String::new();
                let mut y_t: String<16> = String::new();
                let mut z_t: String<16> = String::new();
                let mut e_t: String<16> = String::new();
                let mut f_t: String<16> = String::new();
                if let Some(x) = x {
                    core::write!(&mut x_t, " X{}", x.as_millimeters())?;
                }
                if let Some(y) = y {
                    core::write!(&mut y_t, " Y{}", y.as_millimeters())?;
                }
                if let Some(z) = z {
                    core::write!(&mut z_t, " Z{}", z.as_millimeters())?;
                }
                if let Some(e) = e {
                    core::write!(&mut e_t, " E{}", e.as_millimeters())?;
                }
                if let Some(f) = f {
                    core::write!(&mut f_t, " F{}", f.as_meters_per_second() * 1000.0)?;
                }
                core::write!(fmt, "G1{}{}{}{}{}", x_t, y_t, z_t, e_t, f_t)
            }
            GCommand::G2 {
                x,
                y,
                z,
                e,
                f,
                i,
                j,
                r,
            } => {
                let mut x_t: String<16> = String::new();
                let mut y_t: String<16> = String::new();
                let mut z_t: String<16> = String::new();
                let mut e_t: String<16> = String::new();
                let mut f_t: String<16> = String::new();
                let mut i_t: String<16> = String::new();
                let mut j_t: String<16> = String::new();
                let mut r_t: String<16> = String::new();
                if let Some(x) = x {
                    core::write!(&mut x_t, " X{}", x.as_millimeters())?;
                }
                if let Some(y) = y {
                    core::write!(&mut y_t, " Y{}", y.as_millimeters())?;
                }
                if let Some(z) = z {
                    core::write!(&mut z_t, " Z{}", z.as_millimeters())?;
                }
                if let Some(e) = e {
                    core::write!(&mut e_t, " E{}", e.as_millimeters())?;
                }
                if let Some(f) = f {
                    core::write!(&mut f_t, " F{}", f.as_meters_per_second() * 1000.0)?;
                }
                if let Some(i) = i {
                    core::write!(&mut i_t, " I{}", i.as_millimeters())?;
                }
                if let Some(j) = j {
                    core::write!(&mut j_t, " J{}", j.as_millimeters())?;
                }
                if let Some(r) = r {
                    core::write!(&mut r_t, " R{}", r.as_millimeters())?;
                }
                core::write!(
                    fmt,
                    "G2{}{}{}{}{}{}{}{}",
                    x_t,
                    y_t,
                    z_t,
                    e_t,
                    f_t,
                    i_t,
                    j_t,
                    r_t
                )
            }
            GCommand::G3 {
                x,
                y,
                z,
                e,
                f,
                i,
                j,
                r,
            } => {
                let mut x_t: String<16> = String::new();
                let mut y_t: String<16> = String::new();
                let mut z_t: String<16> = String::new();
                let mut e_t: String<16> = String::new();
                let mut f_t: String<16> = String::new();
                let mut i_t: String<16> = String::new();
                let mut j_t: String<16> = String::new();
                let mut r_t: String<16> = String::new();
                if let Some(x) = x {
                    core::write!(&mut x_t, " X{}", x.as_millimeters())?;
                }
                if let Some(y) = y {
                    core::write!(&mut y_t, " Y{}", y.as_millimeters())?;
                }
                if let Some(z) = z {
                    core::write!(&mut z_t, " Z{}", z.as_millimeters())?;
                }
                if let Some(e) = e {
                    core::write!(&mut e_t, " E{}", e.as_millimeters())?;
                }
                if let Some(f) = f {
                    core::write!(&mut f_t, " F{}", f.as_meters_per_second() * 1000.0)?;
                }
                if let Some(i) = i {
                    core::write!(&mut i_t, " I{}", i.as_millimeters())?;
                }
                if let Some(j) = j {
                    core::write!(&mut j_t, " J{}", j.as_millimeters())?;
                }
                if let Some(r) = r {
                    core::write!(&mut r_t, " R{}", r.as_millimeters())?;
                }
                core::write!(
                    fmt,
                    "G3{}{}{}{}{}{}{}{}",
                    x_t,
                    y_t,
                    z_t,
                    e_t,
                    f_t,
                    i_t,
                    j_t,
                    r_t
                )
            }
            GCommand::G4 { p, s } => {
                let mut p_t: String<16> = String::new();
                let mut s_t: String<16> = String::new();
                if let Some(p) = p {
                    core::write!(&mut p_t, " X{}", p.as_millis())?;
                }
                if let Some(s) = s {
                    core::write!(&mut s_t, " Y{}", s.as_secs())?;
                }
                core::write!(fmt, "G4{}{}", p_t, s_t)
            }
            GCommand::G10 => {
                core::write!(fmt, "G10")
            }
            GCommand::G11 => {
                core::write!(fmt, "G11")
            }
            GCommand::G20 => {
                core::write!(fmt, "G22")
            }
            GCommand::G21 => {
                core::write!(fmt, "G21")
            }
            GCommand::G28 { x, y, z } => {
                let mut x_t = "";
                let mut y_t = "";
                let mut z_t = "";
                if *x {
                    x_t = " X";
                }
                if *y {
                    y_t = " Y";
                }
                if *z {
                    z_t = " Z";
                }
                core::write!(fmt, "G28{}{}{}", x_t, y_t, z_t)
            }
            GCommand::G90 => {
                core::write!(fmt, "G90")
            }
            GCommand::G91 => {
                core::write!(fmt, "G91")
            }
            GCommand::M20 => {
                core::write!(fmt, "M20")
            }
            GCommand::M21 => {
                core::write!(fmt, "M21")
            }
            GCommand::M22 => {
                core::write!(fmt, "M22")
            }
            GCommand::M23 { filename } => {
                core::write!(fmt, "M23 {}", filename)
            }
            // GCommand::M24 { s, t } => todo!(),
            GCommand::M25 => {
                core::write!(fmt, "M25")
            }
            GCommand::M27 => {
                core::write!(fmt, "M27")
            }
            GCommand::M31 => {
                core::write!(fmt, "M31")
            }
            GCommand::M104 { s } => {
                core::write!(fmt, "M104 {}", s.as_celsius())
            }
            GCommand::M105 => {
                core::write!(fmt, "105")
            }
            GCommand::M106 { s } => {
                core::write!(fmt, "M106 {}", s)
            }
            // GCommand::M109 { r, s } => {
            // core::write!(fmt, "M104 {}", s.as_celsius())
            // },
            GCommand::M114 => {
                core::write!(fmt, "114")
            }
            GCommand::M123 { s } => todo!(),
            GCommand::M140 { s } => todo!(),
            GCommand::M149 { u } => todo!(),
            GCommand::M154 { s } => todo!(),
            GCommand::M155 { s } => todo!(),
            GCommand::M190 { r, s } => todo!(),
            GCommand::M192 { r, s } => todo!(),
            GCommand::M203 { x, y, z, e } => todo!(),
            GCommand::M207 { f, s, z } => todo!(),
            GCommand::M208 { f, s } => todo!(),
            GCommand::M220 { s } => todo!(),
            GCommand::M221 { s } => todo!(),
            GCommand::M524 => todo!(),
            GCommand::D0 { x, y, z, t } => todo!(),
            GCommand::D1 { x, y, z, e, t } => todo!(),
            GCommand::D114 => todo!(),
            GCommand::D115 => todo!(),
            _ => todo!(),
        }
    }
}

fn extract_speed(cmd: &LinearMap<&str, &str, 16>, key: &str, unit: DistanceUnit) -> Option<Speed> {
    let distance = extract_distance(cmd, key, unit)?;
    Some(Speed::from_meters_per_second(distance.as_meters() / 60.0))
}

fn extract_distance(
    cmd: &LinearMap<&str, &str, 16>,
    key: &str,
    unit: DistanceUnit,
) -> Option<Distance> {
    let val = extract_token_as_number(cmd, key)?;
    match unit {
        DistanceUnit::Millimeter => Some(Distance::from_millimeters(val)),
        DistanceUnit::Inch => Some(Distance::from_inches(val)),
    }
}

fn extract_duration(
    cmd: &LinearMap<&str, &str, 16>,
    key: &str,
    unit: DurationUnit,
) -> Option<Duration> {
    let value = extract_token_as_number(cmd, key)?;
    match unit {
        DurationUnit::Second => Some(Duration::from_secs_f64(value)),
        DurationUnit::Millisecond => Some(Duration::from_secs_f64(value / 1000f64)),
    }
}

fn extract_temperature(
    cmd: &LinearMap<&str, &str, 16>,
    key: &str,
    unit: TemperatureUnit,
) -> Option<Temperature> {
    let value = extract_token_as_number(cmd, key)?;
    match unit {
        TemperatureUnit::Celsius => Some(Temperature::from_celsius(value)),
        TemperatureUnit::Farhenheit => Some(Temperature::from_fahrenheit(value)),
        TemperatureUnit::Kelvin => Some(Temperature::from_kelvin(value)),
    }
}

fn extract_token_as_number(cmd: &LinearMap<&str, &str, 16>, key: &str) -> Option<f64> {
    match extract_token_as_string(cmd, key) {
        Some(t) => t.parse::<f64>().ok(),
        None => None,
    }
}

fn extract_token_as_string<'a>(cmd: &'a LinearMap<&str, &str, 16>, key: &str) -> Option<&'a str> {
    match cmd.get(key) {
        Some(t) => Some(t),
        None => None,
    }
}

#[derive(Clone, Copy)]
enum ParserState {
    ReadingCommand,
    ReadingComment(char),
}
pub struct GCodeParser {
    distance_unit: DistanceUnit,
    temperature_unit: TemperatureUnit,
}

impl Default for GCodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl GCodeParser {
    pub const fn new() -> Self {
        Self {
            distance_unit: DistanceUnit::Millimeter,
            temperature_unit: TemperatureUnit::Celsius,
        }
    }

    pub fn parse(&mut self, data: &str) -> Option<GCommand> {
        let mut state = ParserState::ReadingCommand;
        let mut command_start = None;
        let mut command_end = None;

        for (i, b) in data.chars().enumerate() {
            match state {
                ParserState::ReadingCommand => match b {
                    ';' | '(' => {
                        state = ParserState::ReadingComment(b);
                        if command_start.is_some() && command_end.is_none() {
                            command_end = Some(i);
                        }
                    }
                    '\n' => {
                        let start = command_start?;
                        let end = command_end.unwrap_or(i);
                        return self.parse_line(data[start..end].trim());
                    }
                    _ => {
                        if command_start.is_none() {
                            command_start = Some(i);
                        }
                    }
                },
                ParserState::ReadingComment(start) => {
                    if start == '(' && b == ')' {
                        state = ParserState::ReadingCommand;
                    } else if start == ';' && b == ';' {
                        state = ParserState::ReadingCommand;
                    }
                }
            }
        }

        if let Some(start) = command_start {
            let end = command_end.unwrap_or(data.len());
            return self.parse_line(data[start..end].trim());
        }
        None
    }

    pub fn set_distance_unit(&mut self, unit: DistanceUnit) {
        self.distance_unit = unit;
    }

    pub fn set_temperature_unit(&mut self, unit: TemperatureUnit) {
        self.temperature_unit = unit;
    }

    pub fn parse_line(&self, line: &str) -> Option<GCommand> {
        let mut tokens: Vec<&str, 16> = line.split(' ').collect();
        if tokens.is_empty() {
            return None;
        }
        // we can safely unwrap because we already checked the size of the tokens
        let cmd_type = tokens.remove(0);
        // the command type (G2, M21, etc) must have at least 2 characters (prefix + code)
        if cmd_type.len() < 2 {
            return None;
        }
        let (prefix, code) = {
            let key = cmd_type.get(0..1)?;
            let value = cmd_type.get(1..)?.parse::<u64>().ok()?;
            match key {
                "G" => (GCommandType::G, value),
                "M" => (GCommandType::M, value),
                _ => return None,
            }
        };

        let mut args: LinearMap<&str, &str, 16> = LinearMap::new();

        for t in &tokens {
            let key = t.get(0..1)?;
            let v = t.get(1..).unwrap_or("");
            args.insert(key, v).ok()?;
        }

        match (prefix, code) {
            (GCommandType::G, 0) => {
                let x = extract_distance(&args, "X", self.distance_unit);
                let y = extract_distance(&args, "Y", self.distance_unit);
                let z = extract_distance(&args, "Z", self.distance_unit);
                let f = extract_speed(&args, "F", self.distance_unit);
                Some(GCommand::G0 { x, y, z, f })
            }
            (GCommandType::G, 1) => {
                let x = extract_distance(&args, "X", self.distance_unit);
                let y = extract_distance(&args, "Y", self.distance_unit);
                let z = extract_distance(&args, "Z", self.distance_unit);
                let e = extract_distance(&args, "E", self.distance_unit);
                let f = extract_speed(&args, "F", self.distance_unit);
                Some(GCommand::G1 { x, y, z, e, f })
            }
            (GCommandType::G, 2) => {
                let x = extract_distance(&args, "X", self.distance_unit);
                let y = extract_distance(&args, "Y", self.distance_unit);
                let z = extract_distance(&args, "Z", self.distance_unit);
                let e = extract_distance(&args, "E", self.distance_unit);
                let f = extract_speed(&args, "F", self.distance_unit);
                let i = extract_distance(&args, "I", self.distance_unit);
                let j = extract_distance(&args, "J", self.distance_unit);
                let r = extract_distance(&args, "R", self.distance_unit);
                Some(GCommand::G2 {
                    x,
                    y,
                    z,
                    e,
                    f,
                    i,
                    j,
                    r,
                })
            }
            (GCommandType::G, 3) => {
                let x = extract_distance(&args, "X", self.distance_unit);
                let y = extract_distance(&args, "Y", self.distance_unit);
                let z = extract_distance(&args, "Z", self.distance_unit);
                let e = extract_distance(&args, "E", self.distance_unit);
                let f = extract_speed(&args, "F", self.distance_unit);
                let i = extract_distance(&args, "I", self.distance_unit);
                let j = extract_distance(&args, "J", self.distance_unit);
                let r = extract_distance(&args, "R", self.distance_unit);
                Some(GCommand::G3 {
                    x,
                    y,
                    z,
                    e,
                    f,
                    i,
                    j,
                    r,
                })
            }
            (GCommandType::G, 4) => {
                let p = extract_duration(&args, "P", DurationUnit::Millisecond);
                let s = extract_duration(&args, "S", DurationUnit::Second);
                Some(GCommand::G4 { p, s })
            }
            (GCommandType::G, 10) => Some(GCommand::G10),
            (GCommandType::G, 11) => Some(GCommand::G11),
            (GCommandType::G, 20) => Some(GCommand::G20),
            (GCommandType::G, 21) => Some(GCommand::G21),
            (GCommandType::G, 28) => {
                let (mut x, mut y, mut z) = (false, false, false);
                if tokens.is_empty() {
                    (x, y, z) = (true, true, true)
                } else {
                    for t in &tokens {
                        match *t {
                            "X" => x = true,
                            "Y" => y = true,
                            "Z" => z = true,
                            _ => (),
                        };
                    }
                }
                Some(GCommand::G28 { x, y, z })
            }
            (GCommandType::G, 90) => Some(GCommand::G90),
            (GCommandType::G, 91) => Some(GCommand::G91),
            (GCommandType::M, 20) => Some(GCommand::M20),
            (GCommandType::M, 21) => Some(GCommand::M21),
            (GCommandType::M, 22) => Some(GCommand::M22),
            (GCommandType::M, 23) => {
                if tokens.is_empty() {
                    return None;
                }
                let filename = tokens.first()?;
                let filename: String<12> = String::from_str(filename).ok()?;
                Some(GCommand::M23 { filename })
            }
            // FIXME use real params for M24
            (GCommandType::M, 24) => Some(GCommand::M24 {
                s: 0,
                t: Duration::from_secs(0),
            }),
            (GCommandType::M, 25) => Some(GCommand::M25),
            (GCommandType::M, 31) => Some(GCommand::M31),
            (GCommandType::M, 104) => {
                let s = extract_temperature(&args, "S", self.temperature_unit)?;
                Some(GCommand::M104 { s })
            }
            (GCommandType::M, 105) => Some(GCommand::M105),
            (GCommandType::M, 106) => {
                let s = extract_token_as_number(&args, "S")?;
                if (0f64..255f64).contains(&s) {
                    Some(GCommand::M106 { s: s as u8 })
                } else {
                    None
                }
            }
            (GCommandType::M, 114) => Some(GCommand::M114),
            (GCommandType::M, 123) => {
                let s = extract_duration(&args, "S", DurationUnit::Second);
                Some(GCommand::M123 { s })
            }
            (GCommandType::M, 140) => {
                let s = extract_temperature(&args, "S", self.temperature_unit)?;
                Some(GCommand::M140 { s })
            }
            (GCommandType::M, 149) => {
                let filename = tokens.first()?;
                let u = match *filename {
                    "C" => Some(TemperatureUnit::Celsius),
                    "F" => Some(TemperatureUnit::Farhenheit),
                    "K" => Some(TemperatureUnit::Kelvin),
                    _ => None,
                }?;
                Some(GCommand::M149 { u })
            }
            (GCommandType::M, 154) => {
                let s = extract_duration(&args, "S", DurationUnit::Second)?;
                Some(GCommand::M154 { s })
            }
            (GCommandType::M, 155) => {
                let s = extract_duration(&args, "S", DurationUnit::Second)?;
                Some(GCommand::M155 { s })
            }
            (GCommandType::M, 207) => {
                let f = extract_speed(&args, "F", self.distance_unit)?;
                let s = extract_distance(&args, "S", self.distance_unit)?;
                let z = extract_distance(&args, "Z", self.distance_unit)?;
                Some(GCommand::M207 { f, s, z })
            }
            (GCommandType::M, 208) => {
                let f = extract_speed(&args, "F", self.distance_unit)?;
                let s = extract_distance(&args, "S", self.distance_unit)?;
                Some(GCommand::M208 { f, s })
            }
            // set feedrate multiplier
            (GCommandType::M, 220) => {
                let s = extract_token_as_number(&args, "S")?;
                Some(GCommand::M220 { s })
            }
            (GCommandType::M, 221) => {
                let s = extract_token_as_number(&args, "S")?;
                Some(GCommand::M221 { s })
            }
            (GCommandType::M, 524) => Some(GCommand::M524),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use math::measurements::Length;

    use super::*;

    #[test]
    fn test_parse_line_g0_complete() {
        let parser = GCodeParser::new();
        let line = "G0 X10.1 Y9.0 Z1.0 E2.0 F1200";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G0 {
                    x: Some(Distance::from_millimeters(10.1)),
                    y: Some(Distance::from_millimeters(9.0)),
                    z: Some(Distance::from_millimeters(1.0)),
                    f: Some(Speed::from_meters_per_second(1.2))
                }
        );
    }

    #[test]
    fn test_parse_line_g0_incomplete() {
        let parser = GCodeParser::new();
        let line = "G0 X10.1 F1200";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G0 {
                    x: Some(Distance::from_millimeters(10.1)),
                    y: None,
                    z: None,
                    f: Some(Speed::from_meters_per_second(1.20))
                }
        );
    }

    #[test]
    fn test_parse_line_g0_invalid() {
        let parser = GCodeParser::new();
        let line = "hello";
        let command = parser.parse_line(line);
        assert!(command.is_none());
    }

    #[test]
    fn test_parse_line_m104_valid() {
        let parser = GCodeParser::new();
        let line = "M104 S10";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::M104 {
                    s: Temperature::from_celsius(10.0)
                }
        );
    }

    #[test]
    fn test_parse_line_g1_complete() {
        let parser = GCodeParser::new();
        let line = "G1 X10.1 Y9.0 Z1.0 E2.0 F1200";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G1 {
                    x: Some(Distance::from_millimeters(10.1)),
                    y: Some(Distance::from_millimeters(9.0)),
                    z: Some(Distance::from_millimeters(1.0)),
                    e: Some(Distance::from_millimeters(2.0)),
                    f: Some(Speed::from_meters_per_second(1.20))
                }
        );
    }

    #[test]
    fn test_parse_line_g28_y() {
        let parser = GCodeParser::new();
        let line = "G28 Y";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G28 {
                    x: false,
                    y: true,
                    z: false
                }
        );
    }

    #[test]
    fn test_parse_line_g28_all() {
        let parser = GCodeParser::new();
        let line = "G28";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G28 {
                    x: true,
                    y: true,
                    z: true
                }
        );
    }

    #[test]
    fn test_parse_line_g1_incomplete() {
        let parser = GCodeParser::new();
        let line = "G1 X10.1 F1200";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G1 {
                    x: Some(Distance::from_millimeters(10.1)),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(Speed::from_meters_per_second(1.20))
                }
        );
    }

    #[test]
    fn test_parse_line_g1_invalid() {
        let parser = GCodeParser::new();
        let line = "G1 ciao lala";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G1 {
                    x: None,
                    y: None,
                    z: None,
                    e: None,
                    f: None
                }
        );
    }

    #[test]
    fn test_parse_line_m149_celsius() {
        let parser = GCodeParser::new();
        let line = "M149 C";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::M149 {
                    u: TemperatureUnit::Celsius,
                }
        );
    }

    #[test]
    fn test_parse_line_m149_farhenheit() {
        let parser = GCodeParser::new();
        let line = "M149 F";
        let command = parser.parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::M149 {
                    u: TemperatureUnit::Farhenheit,
                }
        );
    }

    #[test]
    fn test_parser_incomplete() {
        let data = "hellohellohellohello";
        let mut parser = GCodeParser::new();
        let res = parser.parse(data);
        assert!(res.is_none());
    }

    #[test]
    fn test_parser_valid() {
        let data = "G1 X10.1 F1200\n";
        let mut parser = GCodeParser::new();
        let res = parser.parse(data);
        assert!(res.is_some());
        assert!(
            res.unwrap()
                == GCommand::G1 {
                    x: Some(Distance::from_millimeters(10.1)),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(Speed::from_meters_per_second(1.20))
                }
        );
    }

    #[test]
    fn test_parser_valid_with_comment_semicolon() {
        let data = "G1 X10.1 F1200;comment";
        let mut parser = GCodeParser::new();
        let res = parser.parse(data);
        assert!(res.is_some());
        assert!(
            res.unwrap()
                == GCommand::G1 {
                    x: Some(Distance::from_millimeters(10.1)),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(Speed::from_meters_per_second(1.20))
                }
        );
    }

    #[test]
    fn test_parser_invalid_with_comment_semicolon() {
        let data = ";G1 X10.1 F1200;comment";
        let mut parser = GCodeParser::new();
        let res = parser.parse(data);
        assert!(res.is_none());
    }

    #[test]
    fn test_parser_invalid_with_comment_double_semicolon() {
        let data = ";comment;G1 X10.1 F1200";
        let mut parser = GCodeParser::new();
        let res = parser.parse(data);
        assert!(res.is_some());
        assert!(
            res.unwrap()
                == GCommand::G1 {
                    x: Some(Distance::from_millimeters(10.1)),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(Speed::from_meters_per_second(1.20))
                }
        );
    }

    #[test]
    fn test_parser_invalid_with_comment_parenthesis() {
        let data = "G1 X10.1 F1200(some comment)";
        let mut parser = GCodeParser::new();
        let res = parser.parse(data);
        assert!(res.is_some());
        assert!(
            res.unwrap()
                == GCommand::G1 {
                    x: Some(Distance::from_millimeters(10.1)),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(Speed::from_meters_per_second(1.20))
                }
        );
    }

    #[test]
    fn test_parser_valid_2_commands() {
        let data1 = "G20\n";
        let data2 = "G21\n";
        let mut parser = GCodeParser::new();
        let res1 = parser.parse(data1);
        assert!(res1.is_some());
        assert!(res1.unwrap() == GCommand::G20);
        let res2 = parser.parse(data2);
        assert!(res2.is_some());
        assert!(res2.unwrap() == GCommand::G21);
    }

    #[test]
    fn test_display_g0() {
        let cmd = GCommand::G0 {
            x: Some(Length::from_millimeters(3.1)),
            y: None,
            z: None,
            f: Some(Speed::from_meters_per_second(1.0)),
        };
        let res = cmd.to_string();
        assert_eq!("G0 X3.1 F1000", res.as_str());
    }

    #[test]
    fn test_display_g1() {
        let cmd = GCommand::G1 {
            x: Some(Length::from_millimeters(3.1)),
            y: None,
            z: None,
            e: Some(Length::from_millimeters(31.45)),
            f: Some(Speed::from_meters_per_second(1.0)),
        };
        let res = cmd.to_string();
        assert_eq!("G1 X3.1 E31.45 F1000", res.as_str());
    }
}

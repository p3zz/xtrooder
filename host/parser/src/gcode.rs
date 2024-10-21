use core::{str::FromStr, time::Duration};

use heapless::{spsc::Queue, LinearMap, String, Vec};
use math::{
    distance::{Distance, DistanceUnit},
    duration::DurationUnit,
    speed::Speed,
    temperature::{Temperature, TemperatureUnit},
};

pub enum GCommandType {
    G,
    M,
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
    // set distance unit to inches
    G20,
    // set distance unit to millimeters
    G21,
    // set positioning as absolute
    G90,
    // set positioning as relative
    G91,
    // List SD Card files
    M20,
    // Init SD card (mount fs)
    M21,
    // Simulate ejection of the SD card
    M22,
    // Select SD file
    M23 { filename: String<12>},
    // Start or Resume SD print
    M24 {s: u64, t: Duration},
    // Pause SD print
    M25,
    // Report SD print status
    M26,
    // set hotend temperature
    M104 {
        s: Temperature,
    },
    // set bed temperature
    M140 {
        s: Temperature,
    },
    // set temperature unit
    M149,
}

#[cfg(feature="defmt-log")]
impl defmt::Format for GCommand{
    fn format(&self, fmt: defmt::Formatter) {
        match self{
            GCommand::G0 { x, y, z, f } => defmt::write!(fmt, "G0 [x: {}] [y: {}] [z: {}] [f: {}]", x, y, z, f),
            GCommand::G1 { x, y, z, e, f } => defmt::write!(fmt, "G1 [x: {}] [y: {}] [z: {}] [e: {}] [f: {}]", x, y, z, e, f),
            GCommand::G2 { x, y, z, e, f, i, j, r } => defmt::write!(fmt, "G2 [x: {}] [y: {}] [z: {}] [e: {}] [f: {}] [i: {}] [j: {}] [r: {}]", x, y, z, e, f, i, j, r),
            GCommand::G3 { x, y, z, e, f, i, j, r } => defmt::write!(fmt, "G3 [x: {}] [y: {}] [z: {}] [e: {}] [f: {}] [i: {}] [j: {}] [r: {}]", x, y, z, e, f, i, j, r),
            GCommand::G4 { p, s } => defmt::write!(fmt, "G4 [p: {}] [s: {}]", p, s),
            GCommand::G20 => defmt::write!(fmt, "G20"),
            GCommand::G21 => defmt::write!(fmt, "G21"),
            GCommand::G90 => todo!(),
            GCommand::G91 => todo!(),
            GCommand::M104 { s } => defmt::write!(fmt, "M104 [S: {}]", s),
            GCommand::M140 { s } => defmt::write!(fmt, "M140 [S: {}]", s),
            GCommand::M149 => todo!(),
            GCommand::M20 => defmt::write!(fmt, "M20"),
            GCommand::M21 => defmt::write!(fmt, "M21"),
            GCommand::M22 => defmt::write!(fmt, "M22"),
            GCommand::M23 { filename } => defmt::write!(fmt, "M23 [{}]", filename.as_str()),
            GCommand::M24 { s, t } => defmt::write!(fmt, "M24 [s: {}] [t: {}]", s, t.as_millis()),
            GCommand::M25 => defmt::write!(fmt, "M25"),
            GCommand::M26 => defmt::write!(fmt, "M26"),
            _ => todo!()
        }
    }
}

fn extract_speed(cmd: &LinearMap<&str, &str, 16>, key: &str, unit: DistanceUnit) -> Option<Speed> {
    extract_token_as_number(cmd, key).map(|value| Speed::from_unit(value, unit))
}

fn extract_distance(
    cmd: &LinearMap<&str, &str, 16>,
    key: &str,
    unit: DistanceUnit,
) -> Option<Distance> {
    extract_token_as_number(cmd, key).map(|value| Distance::from_unit(value, unit))
}

fn extract_duration(
    cmd: &LinearMap<&str, &str, 16>,
    key: &str,
    unit: DurationUnit,
) -> Option<Duration> {
    let value = extract_token_as_number(cmd, key)? as u64;
    match unit {
        DurationUnit::Second => Some(Duration::from_secs(value)),
        DurationUnit::Millisecond => Some(Duration::from_millis(value)),
    }
}

fn extract_temperature(
    cmd: &LinearMap<&str, &str, 16>,
    key: &str,
    unit: TemperatureUnit,
) -> Option<Temperature> {
    extract_token_as_number(cmd, key).map(|value| Temperature::from_unit(value, unit))
}

fn extract_token_as_number(cmd: &LinearMap<&str, &str, 16>, key: &str) -> Option<f64> {
    match extract_token_as_string(cmd, key) {
        Some(t) => t.parse::<f64>().ok(),
        None => None,
    }
}

fn extract_token_as_string<'a>(cmd: &'a LinearMap<&str, &str, 16>, key: &str) -> Option<&'a str> {
    match cmd.get(key).copied() {
        Some(t) => Some(t),
        None => None,
    }
}

enum ParserState {
    ReadingCommand,
    ReadingComment,
}
pub struct GCodeParser {
    distance_unit: DistanceUnit,
    temperature_unit: TemperatureUnit,
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
        let mut data_buffer: String<32> = String::new();
        for b in data.chars() {
            if b == '\n'{
                break;
            }
            match state {
                ParserState::ReadingCommand => match b {
                    ';' | '(' => {
                        state = ParserState::ReadingComment
                    },
                    // todo check buffer overflow
                    _ => data_buffer.push(b).unwrap(),
                },
                ParserState::ReadingComment => match b {
                    ')' => state = ParserState::ReadingCommand,
                    _ => (),
                },
            }
        }
        self.parse_line(&data_buffer.as_str())
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
        if cmd_type.len() < 2{
            return None;
        }
        let (prefix, code) = {
            let key = cmd_type.get(0..1)?;
            let value = cmd_type.get(1..)?.parse::<u64>().ok()?;
            match key{
                "G" => (GCommandType::G, value),
                "M" => (GCommandType::M, value),
                _ => return None
            }
        };
        
        let mut args: LinearMap<&str, &str, 16> = LinearMap::new();

        for t in &tokens {
            let key = t.get(0..1)?;
            let v = t.get(1..)?;
            args.insert(key, v).unwrap();
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
            (GCommandType::G, 20) => Some(GCommand::G20),
            (GCommandType::G, 21) => Some(GCommand::G21),
            (GCommandType::G, 90) => Some(GCommand::G90),
            (GCommandType::G, 91) => Some(GCommand::G91),
            (GCommandType::M, 21) => Some(GCommand::M21),
            (GCommandType::M, 22) => Some(GCommand::M22),
            (GCommandType::M, 23) => {
                if tokens.len() == 0{
                    return None;
                }
                let filename = tokens.get(0)?;
                let filename = String::from_str(filename).unwrap();
                // let filename: String<12> = String::from_str(&filename).unwrap();
                Some(GCommand::M23{filename})
            },
            // FIXME use real params for M24
            (GCommandType::M, 24) => Some(GCommand::M24{s: 0, t: Duration::from_secs(0)}),
            (GCommandType::M, 25) => Some(GCommand::M25),
            (GCommandType::M, 104) => {
                let s = extract_temperature(&args, "S", self.temperature_unit);
                if s.is_some(){
                    Some(GCommand::M104 { s: s.unwrap() })
                }
                else{
                    None
                }
            },
            (GCommandType::M, 140) => {
                let s = extract_temperature(&args, "S", self.temperature_unit);
                if s.is_some(){
                    Some(GCommand::M140 { s: s.unwrap() })
                }
                else{
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
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
                    x: Some(Distance::from_mm(10.1)),
                    y: Some(Distance::from_mm(9.0)),
                    z: Some(Distance::from_mm(1.0)),
                    f: Some(Speed::from_mm_per_second(1200.0))
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
                    x: Some(Distance::from_mm(10.1)),
                    y: None,
                    z: None,
                    f: Some(Speed::from_mm_per_second(1200.0))
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
        assert!(command.unwrap() == GCommand::M104 { s: Temperature::from_celsius(10.0) });
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
                    x: Some(Distance::from_mm(10.1)),
                    y: Some(Distance::from_mm(9.0)),
                    z: Some(Distance::from_mm(1.0)),
                    e: Some(Distance::from_mm(2.0)),
                    f: Some(Speed::from_mm_per_second(1200.0))
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
                    x: Some(Distance::from_mm(10.1)),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(Speed::from_mm_per_second(1200.0))
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
                    x: Some(Distance::from_mm(10.1)),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(Speed::from_mm_per_second(1200.0))
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
                    x: Some(Distance::from_mm(10.1)),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(Speed::from_mm_per_second(1200.0))
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
}

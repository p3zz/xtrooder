use core::{
    fmt::Write,
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
    // set steppers position
    G92{
        x: Option<Distance>,
        y: Option<Distance>,
        z: Option<Distance>,
        e: Option<Distance>,
    },
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
    // E absolute
    M82,
    // E relative
    M83,
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
    // fan off
    M107,
    // wait for hotend temperature
    M109 {
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
    // wait for bed temperature
    M190 {
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
    M524,
}

fn extract_speed(cmd: &LinearMap<char, Option<&str>, 16>, key: char, unit: DistanceUnit) -> Option<Speed> {
    let distance = extract_distance(cmd, key, unit)?;
    Some(Speed::from_meters_per_second(distance.as_meters() / 60.0))
}

fn extract_distance(
    cmd: &LinearMap<char, Option<&str>, 16>,
    key: char,
    unit: DistanceUnit,
) -> Option<Distance> {
    let val = extract_token_as_number(cmd, key)?;
    match unit {
        DistanceUnit::Millimeter => Some(Distance::from_millimeters(val)),
        DistanceUnit::Inch => Some(Distance::from_inches(val)),
    }
}

fn extract_duration(
    cmd: &LinearMap<char, Option<&str>, 16>,
    key: char,
    unit: DurationUnit,
) -> Option<Duration> {
    let value = extract_token_as_number(cmd, key)?;
    match unit {
        DurationUnit::Second => Some(Duration::from_secs_f64(value)),
        DurationUnit::Millisecond => Some(Duration::from_secs_f64(value / 1000f64)),
    }
}

fn extract_temperature(
    cmd: &LinearMap<char, Option<&str>, 16>,
    key: char,
    unit: TemperatureUnit,
) -> Option<Temperature> {
    let value = extract_token_as_number(cmd, key)?;
    match unit {
        TemperatureUnit::Celsius => Some(Temperature::from_celsius(value)),
        TemperatureUnit::Farhenheit => Some(Temperature::from_fahrenheit(value)),
        TemperatureUnit::Kelvin => Some(Temperature::from_kelvin(value)),
    }
}

fn extract_token_as_number(cmd: &LinearMap<char, Option<&str>, 16>, key: char) -> Option<f64> {
    match extract_token_as_string(cmd, key) {
        Some(t) => t.parse::<f64>().ok(),
        None => None,
    }
}

fn extract_token_as_string<'a>(cmd: &'a LinearMap<char, Option<&str>, 16>, key: char) -> Option<&'a str> {
    match cmd.get(&key) {
        Some(t) => *t,
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
                    if (start == '(' && b == ')') || (start == ';' && b == ';') {
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
        // SAFETY - we can safely remove because we already checked the size of the tokens
        let cmd_type = tokens.remove(0);
        // the command type (G2, M21, etc) must have at least 2 characters (prefix + code)
        if cmd_type.len() < 2 {
            return None;
        }
        let (prefix, code) = {
            let key = cmd_type.get(0..1)?.chars().next()?.to_ascii_uppercase();
            let value = cmd_type.get(1..)?.parse::<u64>().ok()?;
            match key {
                'G' => (GCommandType::G, value),
                'M' => (GCommandType::M, value),
                _ => return None,
            }
        };

        let mut args: LinearMap<char, Option<&str>, 16> = LinearMap::new();

        for t in tokens {
            let key = t.get(0..1)?.chars().next()?.to_ascii_uppercase();
            let v = t.get(1..);
            args.insert(key, v).ok()?;
        }

        match (prefix, code) {
            (GCommandType::G, 0) => {
                let x = extract_distance(&args, 'X', self.distance_unit);
                let y = extract_distance(&args, 'Y', self.distance_unit);
                let z = extract_distance(&args, 'Z', self.distance_unit);
                let f = extract_speed(&args, 'F', self.distance_unit);
                Some(GCommand::G0 { x, y, z, f })
            }
            (GCommandType::G, 1) => {
                let x = extract_distance(&args, 'X', self.distance_unit);
                let y = extract_distance(&args, 'Y', self.distance_unit);
                let z = extract_distance(&args, 'Z', self.distance_unit);
                let e = extract_distance(&args, 'E', self.distance_unit);
                let f = extract_speed(&args, 'F', self.distance_unit);
                Some(GCommand::G1 { x, y, z, e, f })
            }
            (GCommandType::G, 2) => {
                let x = extract_distance(&args, 'X', self.distance_unit);
                let y = extract_distance(&args, 'Y', self.distance_unit);
                let z = extract_distance(&args, 'Z', self.distance_unit);
                let e = extract_distance(&args, 'E', self.distance_unit);
                let f = extract_speed(&args, 'F', self.distance_unit);
                let i = extract_distance(&args, 'I', self.distance_unit);
                let j = extract_distance(&args, 'J', self.distance_unit);
                let r = extract_distance(&args, 'R', self.distance_unit);
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
                let x = extract_distance(&args, 'X', self.distance_unit);
                let y = extract_distance(&args, 'Y', self.distance_unit);
                let z = extract_distance(&args, 'Z', self.distance_unit);
                let e = extract_distance(&args, 'E', self.distance_unit);
                let f = extract_speed(&args, 'F', self.distance_unit);
                let i = extract_distance(&args, 'I', self.distance_unit);
                let j = extract_distance(&args, 'J', self.distance_unit);
                let r = extract_distance(&args, 'R', self.distance_unit);
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
                let p = extract_duration(&args, 'P', DurationUnit::Millisecond);
                let s = extract_duration(&args, 'S', DurationUnit::Second);
                Some(GCommand::G4 { p, s })
            }
            (GCommandType::G, 10) => Some(GCommand::G10),
            (GCommandType::G, 11) => Some(GCommand::G11),
            (GCommandType::G, 20) => Some(GCommand::G20),
            (GCommandType::G, 21) => Some(GCommand::G21),
            (GCommandType::G, 28) => {
                let (mut x, mut y, mut z) = (false, false, false);
                if args.is_empty() {
                    (x, y, z) = (true, true, true)
                } else {
                    for t in args.iter() {
                        match t.0 {
                            'X' => x = true,
                            'Y' => y = true,
                            'Z' => z = true,
                            _ => (),
                        };
                    }
                }
                Some(GCommand::G28 { x, y, z })
            }
            (GCommandType::G, 90) => Some(GCommand::G90),
            (GCommandType::G, 91) => Some(GCommand::G91),
            (GCommandType::G, 92) => {
                let x = extract_distance(&args, 'X', self.distance_unit);
                let y = extract_distance(&args, 'Y', self.distance_unit);
                let z = extract_distance(&args, 'Z', self.distance_unit);
                let e = extract_distance(&args, 'E', self.distance_unit);
                Some(GCommand::G92 { x, y, z, e })
            }
            (GCommandType::M, 20) => Some(GCommand::M20),
            (GCommandType::M, 21) => Some(GCommand::M21),
            (GCommandType::M, 22) => Some(GCommand::M22),
            (GCommandType::M, 23) => {
                let filename = extract_token_as_string(&args, 'F')?;
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
            (GCommandType::M, 82) => Some(GCommand::M82),
            (GCommandType::M, 83) => Some(GCommand::M83),
            (GCommandType::M, 104) => {
                let s = extract_temperature(&args, 'S', self.temperature_unit)?;
                Some(GCommand::M104 { s })
            }
            (GCommandType::M, 105) => Some(GCommand::M105),
            (GCommandType::M, 106) => {
                let s = extract_token_as_number(&args, 'S')?;
                if (0f64..255f64).contains(&s) {
                    Some(GCommand::M106 { s: s as u8 })
                } else {
                    None
                }
            }
            (GCommandType::M, 107) => {
                Some(GCommand::M107)
            }
            (GCommandType::M, 109) => {
                let s = extract_temperature(&args, 'S', self.temperature_unit)?;
                Some(GCommand::M109 { s })
            }
            (GCommandType::M, 114) => Some(GCommand::M114),
            (GCommandType::M, 123) => {
                let s = extract_duration(&args, 'S', DurationUnit::Second);
                Some(GCommand::M123 { s })
            }
            (GCommandType::M, 140) => {
                let s = extract_temperature(&args, 'S', self.temperature_unit)?;
                Some(GCommand::M140 { s })
            }
            (GCommandType::M, 149) => {
                let unit = args.iter().next()?;
                let u = match unit.0 {
                    'C' => Some(TemperatureUnit::Celsius),
                    'F' => Some(TemperatureUnit::Farhenheit),
                    'K' => Some(TemperatureUnit::Kelvin),
                    _ => None,
                }?;
                Some(GCommand::M149 { u })
            }
            (GCommandType::M, 154) => {
                let s = extract_duration(&args, 'S', DurationUnit::Second)?;
                Some(GCommand::M154 { s })
            }
            (GCommandType::M, 155) => {
                let s = extract_duration(&args, 'S', DurationUnit::Second)?;
                Some(GCommand::M155 { s })
            }
            (GCommandType::M, 190) => {
                let s = extract_temperature(&args, 'S', self.temperature_unit)?;
                Some(GCommand::M190 { s })
            }
            (GCommandType::M, 207) => {
                let f = extract_speed(&args, 'F', self.distance_unit)?;
                let s = extract_distance(&args, 'S', self.distance_unit)?;
                let z = extract_distance(&args, 'Z', self.distance_unit)?;
                Some(GCommand::M207 { f, s, z })
            }
            (GCommandType::M, 208) => {
                let f = extract_speed(&args, 'F', self.distance_unit)?;
                let s = extract_distance(&args, 'S', self.distance_unit)?;
                Some(GCommand::M208 { f, s })
            }
            // set feedrate multiplier
            (GCommandType::M, 220) => {
                let s = extract_token_as_number(&args, 'S')?;
                Some(GCommand::M220 { s })
            }
            (GCommandType::M, 221) => {
                let s = extract_token_as_number(&args, 'S')?;
                Some(GCommand::M221 { s })
            }
            (GCommandType::M, 524) => Some(GCommand::M524),
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
                    x: Some(Distance::from_millimeters(10.1)),
                    y: Some(Distance::from_millimeters(9.0)),
                    z: Some(Distance::from_millimeters(1.0)),
                    f: Some(Speed::from_meters_per_second(0.02))
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
                    f: Some(Speed::from_meters_per_second(0.02))
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
                    f: Some(Speed::from_meters_per_second(0.02))
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
                    f: Some(Speed::from_meters_per_second(0.02))
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
                    f: Some(Speed::from_meters_per_second(0.02))
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
                    f: Some(Speed::from_meters_per_second(0.02))
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
                    f: Some(Speed::from_meters_per_second(0.02))
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
                    f: Some(Speed::from_meters_per_second(0.02))
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

}

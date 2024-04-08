use core::result;

use heapless::{spsc::Queue, LinearMap, String, Vec};

#[derive(PartialEq, Debug)]
pub enum GCommand {
    // https://marlinfw.org/docs/gcode/G000-G001.html
    G0 {
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        f: Option<f64>,
    },
    G1 {
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        e: Option<f64>,
        f: Option<f64>,
    },
    G2 {
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        e: Option<f64>,
        f: Option<f64>,
        i: Option<f64>,
        j: Option<f64>,
        r: Option<f64>,
    },
    G3 {
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        e: Option<f64>,
        f: Option<f64>,
        i: Option<f64>,
        j: Option<f64>,
        r: Option<f64>,
    },
    G4 {
        p: Option<f64>,
        s: Option<f64>,
    },
    G20,
    G21,
    G90,
    G91,
    M104 {
        s: Option<f64>,
    },
}

enum GCommandType {
    G,
    M,
}

pub fn parse_line(line: &str) -> Option<GCommand> {
    let tokens: Vec<&str, 16> = line.split(' ').collect();
    // cmd is a command
    let mut cmd: LinearMap<&str, f64, 16> = LinearMap::new();
    if tokens.is_empty() {
        return None;
    }
    for t in tokens {
        let key = t.get(0..1)?;
        let v = t.get(1..)?;
        let value = match v.parse::<f64>() {
            Ok(n) => n,
            Err(_) => return None,
        };
        cmd.insert(key, value).unwrap();
    }

    let (t, code) = get_command_type(&cmd)?;
    match (t, code) {
        (GCommandType::G, 0) => {
            let x = retrieve_map_value(&cmd, "X");
            let y = retrieve_map_value(&cmd, "Y");
            let z = retrieve_map_value(&cmd, "Z");
            let f = retrieve_map_value(&cmd, "F");
            Some(GCommand::G0 { x, y, z, f })
        }
        (GCommandType::G, 1) => {
            let x = retrieve_map_value(&cmd, "X");
            let y = retrieve_map_value(&cmd, "Y");
            let z = retrieve_map_value(&cmd, "Z");
            let e = retrieve_map_value(&cmd, "E");
            let f = retrieve_map_value(&cmd, "F");
            Some(GCommand::G1 { x, y, z, e, f })
        }
        (GCommandType::G, 2) | (GCommandType::G, 3) => {
            let x = retrieve_map_value(&cmd, "X");
            let y = retrieve_map_value(&cmd, "Y");
            let z = retrieve_map_value(&cmd, "Z");
            let e = retrieve_map_value(&cmd, "E");
            let f = retrieve_map_value(&cmd, "F");
            let i = retrieve_map_value(&cmd, "I");
            let j = retrieve_map_value(&cmd, "J");
            let r = retrieve_map_value(&cmd, "R");
            if code == 2 {
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
            } else {
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
        }
        (GCommandType::G, 4) => {
            let p = retrieve_map_value(&cmd, "P");
            let s = retrieve_map_value(&cmd, "S");
            Some(GCommand::G4 { p, s })
        }
        (GCommandType::G, 20) => Some(GCommand::G20),
        (GCommandType::G, 21) => Some(GCommand::G21),
        (GCommandType::G, 90) => Some(GCommand::G90),
        (GCommandType::G, 91) => Some(GCommand::G91),
        (GCommandType::M, 104) => {
            let s = retrieve_map_value(&cmd, "S");
            Some(GCommand::M104 { s })
        }
        _ => None,
    }
}

fn retrieve_map_value(cmd: &LinearMap<&str, f64, 16>, key: &str) -> Option<f64> {
    match cmd.get(key) {
        Some(value) => Some(*value),
        None => None,
    }
}

fn get_command_type(cmd: &LinearMap<&str, f64, 16>) -> Option<(GCommandType, u64)> {
    match retrieve_map_value(&cmd, "G") {
        Some(code) => return Some((GCommandType::G, code as u64)),
        None => match retrieve_map_value(&cmd, "M") {
            Some(code) => return Some((GCommandType::M, code as u64)),
            None => None,
        },
    }
}

enum ParserState {
    ReadingCommand,
    ReadingComment,
}
pub struct GCodeParser {
    data_buffer: Vec<u8, 32>,
    command_queue: Queue<GCommand, 32>,
    state: ParserState,
}

impl GCodeParser {
    pub fn new() -> Self {
        Self {
            data_buffer: Vec::new(),
            command_queue: Queue::new(),
            state: ParserState::ReadingCommand,
        }
    }

    pub fn parse(&mut self, data: &[u8]) -> Result<(), ()> {
        for b in data {
            match self.state {
                ParserState::ReadingCommand => match b {
                    b';' | b'(' | b'\n' => {
                        if !self.data_buffer.is_empty() {
                            let c = String::from_utf8(self.data_buffer.clone()).unwrap();
                            if let Some(cmd) = parse_line(c.as_str()) {
                                if self.command_queue.enqueue(cmd).is_err() {
                                    return Err(());
                                }
                                self.data_buffer.clear();
                            }
                        }
                        self.state = if *b == b'\n' {
                            ParserState::ReadingCommand
                        } else {
                            ParserState::ReadingComment
                        }
                    }
                    0 => (),
                    _ => self.data_buffer.push(*b).unwrap_or_else(|_| ()),
                },
                ParserState::ReadingComment => match b {
                    b'\n' | b')' => self.state = ParserState::ReadingCommand,
                    _ => (),
                },
            }
        }
        Ok(())
    }

    pub fn pick_from_queue(&mut self) -> Option<GCommand> {
        self.command_queue.dequeue()
    }

    pub fn queue_len(&self) -> usize {
        self.command_queue.len()
    }

    pub fn is_queue_full(&self) -> bool {
        self.command_queue.is_full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line_g0_complete() {
        let line = "G0 X10.1 Y9.0 Z1.0 E2.0 F1200";
        let command = parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G0 {
                    x: Some(10.1),
                    y: Some(9.0),
                    z: Some(1.0),
                    f: Some(1200_f64)
                }
        );
    }

    #[test]
    fn test_parse_line_g0_incomplete() {
        let line = "G0 X10.1 F1200";
        let command = parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G0 {
                    x: Some(10.1),
                    y: None,
                    z: None,
                    f: Some(1200_f64)
                }
        );
    }

    #[test]
    fn test_parse_line_g0_invalid() {
        let line = "hello";
        let command = parse_line(line);
        assert!(command.is_none());
    }

    #[test]
    fn test_parse_line_g1_complete() {
        let line = "G1 X10.1 Y9.0 Z1.0 E2.0 F1200";
        let command = parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G1 {
                    x: Some(10.1),
                    y: Some(9.0),
                    z: Some(1.0),
                    e: Some(2.0),
                    f: Some(1200_f64)
                }
        );
    }

    #[test]
    fn test_parse_line_g1_incomplete() {
        let line = "G1 X10.1 F1200";
        let command = parse_line(line);
        assert!(command.is_some());
        assert!(
            command.unwrap()
                == GCommand::G1 {
                    x: Some(10.1),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(1200_f64)
                }
        );
    }

    #[test]
    fn test_parse_line_g1_invalid() {
        let line = "G1 ciao lala";
        let command = parse_line(line);
        assert!(command.is_none());
    }

    #[test]
    fn test_parser_incomplete() {
        let data = "hellohellohellohello";
        let mut parser = GCodeParser::new();
        parser.parse(data.as_bytes());
        let cmd = parser.pick_from_queue();
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parser_invalid() {
        let data = "hellohellohellohello";
        let mut parser = GCodeParser::new();
        parser.parse(data.as_bytes());
        let cmd = parser.pick_from_queue();
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parser_valid() {
        let data = "G1 X10.1 F1200\n";
        let mut parser = GCodeParser::new();
        parser.parse(data.as_bytes());
        let cmd = parser.pick_from_queue();
        assert!(cmd.is_some());
        assert!(
            cmd.unwrap()
                == GCommand::G1 {
                    x: Some(10.1),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(1200_f64)
                }
        );
    }

    #[test]
    fn test_parser_valid_with_comment_semicolon() {
        let data = "G1 X10.1 F1200;comment";
        let mut parser = GCodeParser::new();
        parser.parse(data.as_bytes());
        assert_eq!(parser.data_buffer.len(), 0);
        let cmd = parser.pick_from_queue();
        assert!(cmd.is_some());
        assert!(
            cmd.unwrap()
                == GCommand::G1 {
                    x: Some(10.1),
                    y: None,
                    z: None,
                    e: None,
                    f: Some(1200_f64)
                }
        );
    }

    #[test]
    fn test_parser_invalid_with_comment_semicolon() {
        let data = ";G1 X10.1 F1200;comment";
        let mut parser = GCodeParser::new();
        parser.parse(data.as_bytes());
        let cmd = parser.pick_from_queue();
        assert_eq!(parser.data_buffer.len(), 0);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parser_valid_3_commands() {
        let data = "G20\nG20\nG21;";
        let mut parser = GCodeParser::new();
        parser.parse(data.as_bytes());
        assert_eq!(parser.data_buffer.len(), 0);
        assert_eq!(parser.command_queue.len(), 3);
        let cmd = parser.pick_from_queue();
        assert!(cmd.is_some());
        assert!(cmd.unwrap() == GCommand::G20);
        let cmd = parser.pick_from_queue();
        assert!(cmd.is_some());
        assert!(cmd.unwrap() == GCommand::G20);
        let cmd = parser.pick_from_queue();
        assert!(cmd.is_some());
        assert!(cmd.unwrap() == GCommand::G21);
    }

    #[test]
    fn test_parser_valid_2_commands_busy_data_buffer() {
        let data = "G20\nG20\nG21";
        let mut parser = GCodeParser::new();
        parser.parse(data.as_bytes());
        assert_eq!(parser.data_buffer.len(), 3);
        assert_eq!(parser.command_queue.len(), 2);
        let cmd = parser.pick_from_queue();
        assert!(cmd.unwrap() == GCommand::G20);
        let cmd = parser.pick_from_queue();
        assert!(cmd.unwrap() == GCommand::G20);
    }
}

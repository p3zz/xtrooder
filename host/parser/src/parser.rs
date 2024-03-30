use heapless::{LinearMap, Vec, String};

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

enum ParserState{
    ReadingCommand,
    ReadingComment
}
pub struct Parser{
    buffer: Vec<u8, 16>,
    state: ParserState
}

impl Parser{
    
    pub fn new() -> Self {
        Self { buffer: Vec::new(), state: ParserState::ReadingCommand }
    }

    pub fn parse(&mut self, data: &[u8]) -> Vec<GCommand, 16>{
        let mut res: Vec<GCommand, 16> = Vec::new();
        for b in data{
            match self.state{
                ParserState::ReadingCommand => {
                    match b {
                        b';' | b'(' | b'\n' => {
                            if !self.buffer.is_empty(){
                                let c = String::from_utf8(self.buffer.clone()).unwrap();
                                if let Some(cmd) = parse_line(c.as_str()){
                                    res.push(cmd).unwrap_or_else(|_|());
                                }
                                self.buffer.clear();
                            }
                            self.state = if *b == b'\n'{
                                ParserState::ReadingCommand
                            }else{
                                ParserState::ReadingComment
                            }
                        },
                        0 => (),
                        _ => self.buffer.push(*b).unwrap_or_else(|_|())
                    }
                },
                ParserState::ReadingComment => {
                    match b {
                        b'\n' | b')' => self.state = ParserState::ReadingCommand,
                        _ => ()
                    }
                },
            }
        }
        res
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
    fn test_parser_incomplete(){
        let data = "hellohellohellohello";
        let mut parser = Parser::new();
        let cmd = parser.parse(data.as_bytes());
        assert_eq!(cmd.len(), 0);
    }

    #[test]
    fn test_parser_invalid(){
        let data = "hellohellohellohello";
        let mut parser = Parser::new();
        let cmd = parser.parse(data.as_bytes());
        assert_eq!(cmd.len(), 0);
    }

    #[test]
    fn test_parser_valid(){
        let data = "G1 X10.1 F1200\n";
        let mut parser = Parser::new();
        let cmd = parser.parse(data.as_bytes());
        assert_eq!(parser.buffer.len(), 0);
        assert_eq!(cmd.len(), 1);
        assert!(*cmd.get(0).unwrap() == GCommand::G1 { x: Some(10.1), y: None, z: None, e: None, f: Some(1200_f64) });
    }

    #[test]
    fn test_parser_valid_with_comment_semicolon(){
        let data = "G1 X10.1 F1200;comment";
        let mut parser = Parser::new();
        let cmd = parser.parse(data.as_bytes());
        assert_eq!(parser.buffer.len(), 0);
        assert_eq!(cmd.len(), 1);
        assert!(*cmd.get(0).unwrap() == GCommand::G1 { x: Some(10.1), y: None, z: None, e: None, f: Some(1200_f64) });
    }

    #[test]
    fn test_parser_invalid_with_comment_semicolon(){
        let data = ";G1 X10.1 F1200;comment";
        let mut parser = Parser::new();
        let cmd = parser.parse(data.as_bytes());
        assert_eq!(parser.buffer.len(), 0);
        assert_eq!(cmd.len(), 0);
    }

    #[test]
    fn test_parser_valid_3_commands(){
        let data = "G20\nG20\nG21;";
        let mut parser = Parser::new();
        let cmd = parser.parse(data.as_bytes());
        assert_eq!(parser.buffer.len(), 0);
        assert_eq!(cmd.len(), 3);
        assert!(*cmd.get(0).unwrap() == GCommand::G20);
        assert!(*cmd.get(1).unwrap() == GCommand::G20);
        assert!(*cmd.get(2).unwrap() == GCommand::G21);
    }


    #[test]
    fn test_parser_valid_2_commands_busy_buffer(){
        let data = "G20\nG20\nG21";
        let mut parser = Parser::new();
        let cmd = parser.parse(data.as_bytes());
        assert_eq!(parser.buffer.len(), 3);
        assert_eq!(cmd.len(), 2);
        assert!(*cmd.get(0).unwrap() == GCommand::G20);
        assert!(*cmd.get(1).unwrap() == GCommand::G20);
    }

}

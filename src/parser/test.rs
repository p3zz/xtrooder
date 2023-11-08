use crate::parser::parser::GCommand;

use super::parser::parse_line;
use defmt::assert;

fn test_parse_line_g0_complete(){
    let line = "G0 X10.1 Y9.0 Z1.0 E2.0 F1200";
    let command = parse_line(line);
    assert!(command.is_some());
    assert!(command.unwrap() == GCommand::G0 { x: Some(10.1), y: Some(9.0), z: Some(1.0), e: Some(2.0), f: Some(1200_f64) });
}

fn test_parse_line_g0_incomplete(){
    let line = "G0 X10.1 F1200";
    let command = parse_line(line);
    assert!(command.is_some());
    assert!(command.unwrap() == GCommand::G0 { x: Some(10.1), y: None, z: None, e: None, f: Some(1200_f64) });
}

fn test_parse_line_g0_invalid(){
    let line = "hello";
    let command = parse_line(line);
    assert!(command.is_none());
}

fn test_parse_line_g1_complete(){
    let line = "G1 X10.1 Y9.0 Z1.0 E2.0 F1200";
    let command = parse_line(line);
    assert!(command.is_some());
    assert!(command.unwrap() == GCommand::G1 { x: Some(10.1), y: Some(9.0), z: Some(1.0), e: Some(2.0), f: Some(1200_f64) });
}

fn test_parse_line_g1_incomplete(){
    let line = "G1 X10.1 F1200";
    let command = parse_line(line);
    assert!(command.is_some());
    assert!(command.unwrap() == GCommand::G1 { x: Some(10.1), y: None, z: None, e: None, f: Some(1200_f64) });
}

fn test_parse_line_g1_invalid(){
    let line = "G1 ciao lala";
    let command = parse_line(line);
    assert!(command.is_none());
}

pub fn test(){
    test_parse_line_g0_complete();
    test_parse_line_g0_incomplete();
    test_parse_line_g0_invalid();
    test_parse_line_g1_complete();
    test_parse_line_g1_incomplete();
    test_parse_line_g1_invalid();
}

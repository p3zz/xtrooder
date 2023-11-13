#![allow(dead_code)]

use heapless::{Vec, LinearMap};

#[derive(PartialEq)]
pub enum GCommand{
    // https://marlinfw.org/docs/gcode/G000-G001.html
    G0{x: Option<f64>, y: Option<f64>, z: Option<f64>, f: Option<f64>},
    G1{x: Option<f64>, y: Option<f64>, z: Option<f64>, e: Option<f64>, f: Option<f64>},
    G20,
    G21,
    G90,
    G91,
}

pub fn parse_line(line: &str) -> Option<GCommand>{
    let tokens: Vec<&str, 16> = line.split(' ').collect();
    // cmd is a command 
    let mut cmd: LinearMap<&str, f64, 16> = LinearMap::new();
    if tokens.is_empty(){
        return None;
    }
    for t in tokens{
        let key = t.get(0..1)?;
        let v = t.get(1..)?;
        let value = match v.parse::<f64>(){
                Ok(n) => n,
                Err(_) => return None
        };
        cmd.insert(key, value).unwrap();
    }
    let retrieve_map_value = |key: &str| -> Option<f64> {
        match cmd.get(key){
            Some(value) => Some(*value),
            None => None,
        }
    };
    let code = retrieve_map_value("G")? as u64;
    match code {
        0 => {
            let x = retrieve_map_value("X");
            let y = retrieve_map_value("Y");    
            let z = retrieve_map_value("Z");
            let f = retrieve_map_value("F");
            Some(GCommand::G0{x, y, z, f})
        },
        1 => {
            let x = retrieve_map_value("X");
            let y = retrieve_map_value("Y");    
            let z = retrieve_map_value("Z");
            let e = retrieve_map_value("E");
            let f = retrieve_map_value("F");
            Some(GCommand::G1{x, y, z, e, f})
        },
        20 => {
            Some(GCommand::G20)
        },
        21 => {
            Some(GCommand::G21)
        },
        90 => {
            Some(GCommand::G90)
        },
        91 => {
            Some(GCommand::G91)
        }
        _ => None
    }
}
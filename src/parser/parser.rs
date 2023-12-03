#![allow(dead_code)]

use heapless::{LinearMap, Vec};

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
    G20,
    G21,
    G90,
    G91,
    M104{
        s: Option<f64>
    }
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

    parse(cmd)

}

fn retrieve_map_value(cmd: &LinearMap<&str, f64, 16>, key: &str) -> Option<f64> {
    match cmd.get(key) {
        Some(value) => Some(*value),
        None => None,
    }
}

fn parse(cmd: LinearMap<&str, f64, 16>) -> Option<GCommand>{
    let mut code = retrieve_map_value(&cmd, "G");
    if code.is_some(){
        return parse_g(code.unwrap() as u64, &cmd);
    }else{
        code = retrieve_map_value(&cmd, "M");
        if code.is_some(){
            return parse_m(code.unwrap() as u64, &cmd);
        }else{
            return None
        }
    }
}

fn parse_g(code: u64, map: &LinearMap<&str, f64, 16>) -> Option<GCommand>{
    match code {
        0 => {
            let x = retrieve_map_value(map, "X");
            let y = retrieve_map_value(map, "Y");
            let z = retrieve_map_value(map, "Z");
            let f = retrieve_map_value(map, "F");
            Some(GCommand::G0 { x, y, z, f })
        }
        1 => {
            let x = retrieve_map_value(map, "X");
            let y = retrieve_map_value(map, "Y");
            let z = retrieve_map_value(map, "Z");
            let e = retrieve_map_value(map, "E");
            let f = retrieve_map_value(map, "F");
            Some(GCommand::G1 { x, y, z, e, f })
        }
        20 => Some(GCommand::G20),
        21 => Some(GCommand::G21),
        90 => Some(GCommand::G90),
        91 => Some(GCommand::G91),
        _ => None,
    }
}

fn parse_m(code: u64, map: &LinearMap<&str, f64, 16>) -> Option<GCommand>{
    match code {
        104 => {
            let s = retrieve_map_value(map, "S");
            Some(GCommand::M104 {s})
        }
        _ => None,
    }
}
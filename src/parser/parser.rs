use heapless::{Vec, LinearMap};

pub enum GCommand{
    G0{x: Option<f64>, y: Option<f64>, z: Option<f64>, e: Option<f64>, f: Option<f64>},
    G1{x: Option<f64>, y: Option<f64>, z: Option<f64>, e: Option<f64>, f: Option<f64>},
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
    let code = retrieve_map_value("G")?.to_bits();
    match code {
        0 | 1 => {
            let x = retrieve_map_value("X");
            let y = retrieve_map_value("Y");    
            let z = retrieve_map_value("Z");
            let e = retrieve_map_value("E");
            let f = retrieve_map_value("F");
            if code == 0 {
                Some(GCommand::G0{x, y, z, e, f})
            }else{
                Some(GCommand::G1{x, y, z, e, f})
            }
        },
        _ => None
    }
}
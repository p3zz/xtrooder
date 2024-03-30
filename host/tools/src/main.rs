use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::env;
use std::fs;

fn main(){
    let args: Vec<String> = env::args().collect();
    let serialport = args.get(1).expect("Serial port not specified");
    let file_path = args.get(2).expect("File not specified");

    let file = fs::File::open(file_path).expect("File not found");
    let bufreader = BufReader::new(file);
    
    let mut port = serialport::new(serialport, 19200)
        .open().expect("Failed to open port");

    for l in bufreader.lines(){
        match l{
            Ok(mut line) => {
                line.push('\n');
                match port.write_all(line.as_bytes()){
                    Ok(_) => print!("{} sent", line),
                    Err(_) => (),
                };
            },
            Err(_) => (),
        }
    }

}
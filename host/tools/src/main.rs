use std::io::BufRead;
use std::io::BufReader;
use std::time::Duration;
use std::env;
use std::fs;

fn main(){
    let args: Vec<String> = env::args().collect();
    let serialport = args.get(1).expect("Serial port not specified");
    let file_path = args.get(2).expect("File not specified");

    let file = fs::File::open(file_path).expect("File not found");
    let bufreader = BufReader::new(file);
    
    let mut port = serialport::new(serialport, 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open port");
    
    for line in bufreader.lines(){
        match line{
            Ok(l) => {
                port.write(l.as_bytes()).expect("Write failed!");
            },
            Err(_) => todo!(),
        }
    }

}
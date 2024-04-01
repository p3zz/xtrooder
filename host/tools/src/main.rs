use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let serialport = args.get(1).expect("Serial port not specified");
    let file_path = args.get(2).expect("File not specified");

    let file = fs::File::open(file_path).expect("File not found");
    let mut bufreader = BufReader::new(file);

    let mut port = serialport::new(serialport, 19200)
        .open()
        .expect("Failed to open port");

    let mut buf = [0u8; 32];
    let mut line = String::new();
    let mut v: Vec<u8> = Vec::new();

    // FIXME ugly as shit pls refactor
    loop {
        match port.read(&mut buf) {
            Ok(n) => {
                for i in 0..n {
                    let elem = *buf.get(i).unwrap();
                    if elem == b'#' {
                        let s = String::from_utf8(v.clone()).unwrap();
                        if s.as_str() == "next" {
                            match bufreader.read_line(&mut line) {
                                Ok(_) => {
                                    println!("{}", line);
                                    port.write_all(line.as_bytes()).unwrap();
                                    line.clear();
                                }
                                Err(_) => print!("error while reading file"),
                            }
                        }
                        v.clear();
                    } else {
                        v.push(elem);
                    }
                }
            }
            Err(_) => (),
        }
    }
}

use core::str::FromStr;

use heapless::{String, Vec};

pub static AT_TOKEN_PING: &'static str = "AT";
pub static AT_TOKEN_RESET: &'static str = "RESET";
pub static AT_TOKEN_VERSION: &'static str = "VERSION";
pub static AT_TOKEN_ADDRESS: &'static str = "ADDR";
pub static AT_TOKEN_NAME: &'static str = "NAME";
pub static AT_TOKEN_ROLE: &'static str = "ROLE";
pub static AT_TOKEN_UART: &'static str = "UART";
pub static AT_ENDLINE: &'static str = "\r\n";

pub enum ATToken{
    Ping,
    Reset,
    Version,
    Address,
    Name,
    Role,
    Uart,
}


enum ATRole{
    Slave = 0,
    Master = 1,
    SlaveLoop = 2
}

enum ATCommand{
    Ping,
    Reset,
    Version{value: String<16>},
    Address{nap: Vec<u8, 2>, uap: u8, lap: Vec<u8, 3>},
    Name{value: String<16>},
    Role{value: ATRole},
    Uart{baudrate: u64, stop_bit: u64, parity_bit: u64}
}

pub fn serialize_command_read(token: &ATToken) -> Result<String<32>, ()>{
    let t = serialize_token(token)?;
    match token{
        ATToken::Ping => Ok(String::from_str(t).unwrap()),
        _ => {
            let mut result: String<32> = String::new();
            result.push_str(AT_TOKEN_PING).unwrap();
            result.push('+').unwrap();
            result.push_str(t).unwrap();
            result.push('?').unwrap();
            result.push_str(AT_ENDLINE).unwrap();
            Ok(result)
        }
    }
}

// fn serialize_command_set(cmd: ATCommand) -> String<32>{
//     match cmd{
//         ATCommand::Ping => ,
//         ATCommand::Reset => todo!(),
//         ATCommand::Version { value } => todo!(),
//         ATCommand::Address { nap, uap, lap } => todo!(),
//         ATCommand::Name { value } => todo!(),
//         ATCommand::Role { value } => todo!(),
//         ATCommand::Uart { baudrate, stop_bit, parity_bit } => todo!(),
//     }
//     todo!()
// }

fn serialize_token(token: &ATToken) -> Result<&'static str, ()>{
    match token{
        ATToken::Ping => Ok(AT_TOKEN_PING),
        ATToken::Reset => Ok(AT_TOKEN_RESET),
        ATToken::Version => Ok(AT_TOKEN_VERSION),
        ATToken::Address => Ok(AT_TOKEN_ADDRESS),
        ATToken::Name => Ok(AT_TOKEN_NAME),
        ATToken::Role => Ok(AT_TOKEN_ROLE),
        ATToken::Uart => Ok(AT_TOKEN_UART),
        _ => Err(())
    }
}
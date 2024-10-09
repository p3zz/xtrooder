use core::str::FromStr;
use core::fmt::Write;
use heapless::{String, Vec};

static AT_TOKEN_PING: &'static str = "AT";
static AT_TOKEN_RESET: &'static str = "RESET";
static AT_TOKEN_VERSION: &'static str = "VERSION";
static AT_TOKEN_ADDRESS: &'static str = "ADDR";
static AT_TOKEN_NAME: &'static str = "NAME";
static AT_TOKEN_ROLE: &'static str = "ROLE";
static AT_TOKEN_UART: &'static str = "UART";
static AT_ENDLINE: &'static str = "\r\n";

enum ATRole{
    Slave = 0,
    Master = 1,
    SlaveLoop = 2
}

impl Into<char> for ATRole{
    fn into(self) -> char {
        char::from(self as u8 + b'0')
    }
}

struct ATVersionParams{
    value: String<16>
}

struct ATAddressParams{
    nap: Vec<u8, 2>,
    uap: u8,
    lap: Vec<u8, 3>
}

struct ATNameParams{
    value: String<16>
}

struct ATRoleParams{
    value: ATRole
}

struct ATUartParams{
    baudrate: u64,
    stop_bit: u64,
    parity_bit: u64
}

pub enum ATCommand{
    Ping,
    Reset,
    Version(Option<ATVersionParams>),
    Address(Option<ATAddressParams>),
    Name(Option<ATNameParams>),
    Role(Option<ATRoleParams>),
    Uart(Option<ATUartParams>),
}


enum ATParams{
    Version,
    Address{nap: Vec<u8, 2>, uap: u8, lap: Vec<u8, 3>},
    Name{value: String<16>},
    Role{value: ATRole},
    Uart{}
}

pub fn serialize_command_read(token: &ATCommand) -> Result<String<32>, ()>{
    let t = serialize_token(token)?;
    let mut result: String<32> = String::from_str(AT_TOKEN_PING).unwrap();
    if !matches!(token, ATCommand::Ping){
        result.push('+').unwrap();
        result.push_str(t).unwrap();
        if !matches!(token, ATCommand::Reset){
            result.push('?').unwrap();
        }
    }
    result.push_str(&AT_ENDLINE).unwrap();
    Ok(result)
}

pub fn serialize_command_write(token: &ATCommand) -> Result<String<32>, ()>{
    let result: String<32> = String::new();
    let mut params: String<16> = String::new();
    match token{
        ATCommand::Name(p) => {
            let param = p.ok_or(())?;
            params.push_str(param.value.as_str());
        },
        ATCommand::Role(p) => {
            let param = p.ok_or(())?;
            params.push(param.value.into());
        },
        ATCommand::Uart(p) => {
            let param = p.ok_or(())?;
            write!(params, "{},{},{}", param.baudrate, param.stop_bit, param.parity_bit);
        },
        _ => {
            return Err(())}
    };
    Ok(result)
}

fn serialize_token(token: &ATCommand) -> Result<&'static str, ()>{
    match token{
        ATCommand::Ping => Ok(AT_TOKEN_PING),
        ATCommand::Reset => Ok(AT_TOKEN_RESET),
        ATCommand::Version(_) => Ok(AT_TOKEN_VERSION),
        ATCommand::Address(_) => Ok(AT_TOKEN_ADDRESS),
        ATCommand::Name(_) => Ok(AT_TOKEN_NAME),
        ATCommand::Role(_) => Ok(AT_TOKEN_ROLE),
        ATCommand::Uart(_) => Ok(AT_TOKEN_UART),
        _ => Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_token_ping(){
        let token = ATCommand::Ping;
        let result = serialize_command_read(&token);
        assert!(result.is_ok());
        assert_eq!("AT\r\n", result.unwrap().as_str());
    }

    #[test]
    fn test_serialize_token_reset(){
        let token = ATCommand::Reset;
        let result = serialize_command_read(&token);
        assert!(result.is_ok());
        assert_eq!("AT+RESET\r\n", result.unwrap().as_str());
    }

    #[test]
    fn test_serialize_token_name(){
        let token = ATCommand::Name(None);
        let result = serialize_command_read(&token);
        assert!(result.is_ok());
        assert_eq!("AT+NAME?\r\n", result.unwrap().as_str());
    }

    #[test]
    fn test_serialize_token_address(){
        let token = ATCommand::Address(None);
        let result = serialize_command_read(&token);
        assert!(result.is_ok());
        assert_eq!("AT+ADDR?\r\n", result.unwrap().as_str());
    }

    #[test]
    fn test_serialize_token_role(){
        let token = ATCommand::Role(None);
        let result = serialize_command_read(&token);
        assert!(result.is_ok());
        assert_eq!("AT+ROLE?\r\n", result.unwrap().as_str());
    }
}
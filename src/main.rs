use std::env;
use std::fs::{read, File};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Read, Write};
use flate2::read::ZlibDecoder;

const BYTES_PER_LINE: usize = 16;

#[derive(Debug)]
struct MyError(String);

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MyError {}

impl MyError {
    fn new(msg: String) -> Self {
        Self(msg)
    }
}

fn print_hexdump<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<(),Box<dyn Error>> {
    let mut buf = [0; BYTES_PER_LINE];
    let mut offset: usize = 0;
    loop {
        match reader.read(&mut buf)? {
            0 => break,
            n => {
                writer.write(gen_info(&buf[..n], offset as u32).as_bytes())?;
                offset += BYTES_PER_LINE;
            }
        }
    }
    Ok(())
}

fn gen_info(bytes: &[u8], offset: u32) -> String {
    let body = bytes.iter().map(|n| format!("{:02x}", n)).
        collect::<Vec<_>>().join(" ");
    let zero_pad = " ".repeat((BYTES_PER_LINE - bytes.len()) * 3);
    let body_ascii = bytes.iter().map(|n|
        if is_printable(*n) {
            *n as char
        } else {
            '.'
        }
    ).collect::<String>();
    format!("{:08x} {}{}  |{}|\n", offset, body, zero_pad, body_ascii)
}

fn is_printable(c: u8) -> bool {
    c >= 0x20 && c <= 0x7e
}

#[test]
fn test_gen_info() {
    let input = "abc".as_bytes();
    let out = gen_info(input, 0);
    assert_eq!(out, "00000000 61 62 63                                         |abc|".to_string());
}

fn main() -> Result<(),Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("no file path found:\n    usage unzlib <file path>");
        return Err(Box::new(MyError::new("file path required".to_string())));
    }
    let output_file = if args.len()>2 {
        let file = File::create(&args[2])?;
        Some(file)
    } else {
        None
    };

    let bytes = read(&args[1])?; //.map_err(|e| e.to_string())?;
    let reader = ZlibDecoder::new(&bytes[..]);

    match output_file {
        None => {
            let writer = io::stdout();
            let handle = writer.lock();
            print_hexdump(reader, handle)?;
        }
        Some(f) => {
            print_hexdump(reader, f)?;
        }
    }

    Ok(())
}

use std::io::{self, BufRead, BufReader};
use std::time::Duration;
use serialport::SerialPort;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use binrw::{BinRead, io::Cursor, endian::LE, binread};

#[binread]
#[derive(Debug)]
#[br(little)]
struct Frame {
    #[br(temp)]
    _discard: u8,
    status: u8,
    micros: u32,
    #[br(count = 8)]
    values: Vec<i32>,
}

fn main() -> io::Result<()> {
    let port_name = "/dev/ttyACM0";

    loop {
        // Wait for the port to become available
        let mut port = loop {
            match serialport::new(port_name, 115200)
                .timeout(Duration::from_millis(100))
                .open()
            {
                Ok(port) => break port,
                Err(_) => {
                    eprintln!("Waiting for {} to become available...", port_name);
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        };

        eprintln!("Connected to {}", port_name);

        let reader = BufReader::new(port);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if let Some(data) = line.strip_prefix("[INFO simsamadc]: db64:") {
                        match BASE64.decode(data.trim()) {
                            Ok(decoded) => {
                                match Frame::read(&mut Cursor::new(&decoded)) {
                                    Ok(frame) => {
                                        eprintln!("Status: 0x{:02x}", frame.status);
                                        eprintln!("Micros: {:?}", frame.micros);
                                        eprintln!("Values: {:?}", frame.values);
                                    }
                                    Err(e) => eprintln!("Failed to parse frame: {}", e),
                                }
                            }
                            Err(e) => eprintln!("Failed to decode base64: {}", e),
                        }
                    } else {
                        eprintln!("{}", line);
                    }
                }
                Err(e) => {
                    eprintln!("Disconnected, waiting...");
                    break; // Break the reading loop to attempt reconnection
                }
            }
        }
    }
}

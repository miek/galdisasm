mod gal20v8;
mod gal22v10;

use clap::{Arg, App, arg_enum, value_t};
use jedec::JEDECFile;
use log::{debug, error, info, warn};
use simple_logger::SimpleLogger;
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;

use gal20v8::GAL20V8;
use gal22v10::GAL22V10;

arg_enum!{
    #[derive(PartialEq, Debug)]
    pub enum Device {
        GAL20V8,
        GAL22V10,
    }
}

fn dis(device_type: Device, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    let jed = JEDECFile::from_bytes(&buf)?;


    match device_type {
        Device::GAL20V8  => GAL20V8(jed),
        Device::GAL22V10 => GAL22V10(jed),
    }

    Ok(())
}

fn main() {
    SimpleLogger::new().init().unwrap();

    let matches = App::new("galdisasm")
        .arg(Arg::with_name("device")
             .short("d")
             .takes_value(true)
             .possible_values(&Device::variants())
             .case_insensitive(true)
             .required(true))
        .arg(Arg::with_name("jed_file")
             .required(true))
        .get_matches();

    let device_type = value_t!(matches, "device", Device).unwrap_or_else(|e| e.exit());

    match dis(device_type, matches.value_of("jed_file").unwrap()) {
        Ok(_) => (),
        Err(error) => eprintln!("Error: {}", error),
    };
}

mod gal20v8;

use clap::{Arg, App};
use jedec::JEDECFile;
use log::{debug, error, info, warn};
use simple_logger::SimpleLogger;
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;

use gal20v8::GAL20V8;

fn dis(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    let jed = JEDECFile::from_bytes(&buf)?;

    println!("Device: {:?}", jed.dev_name_str);
    //println!("{:?}", jed.f);

    GAL20V8(jed);

    Ok(())
}

fn main() {
    SimpleLogger::new().init().unwrap();

    let matches = App::new("galdisasm")
        .arg(Arg::with_name("jed_file")
             .required(true))
        .get_matches();

    match dis(matches.value_of("jed_file").unwrap()) {
        Ok(_) => (),
        Err(error) => eprintln!("Error: {}", error),
    };
}

use clap::{Arg, App};
use jedec::JEDECFile;
use log::{debug, error, info, warn};
use std::fs::File;
use std::io::Read;

fn dis(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    let jed = JEDECFile::from_bytes(&buf)?;

    println!("Device: {:?}", jed.dev_name_str);
    println!("{:?}", jed.f);

    Ok(())
}

fn main() {
    let matches = App::new("galdisasm")
        .arg(Arg::with_name("jed_file")
             .required(true))
        .get_matches();

    match dis(matches.value_of("jed_file").unwrap()) {
        Ok(_) => (),
        Err(error) => eprintln!("Error: {}", error),
    };
}

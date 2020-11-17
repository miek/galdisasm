use clap::{Arg, App};
use jedec::JEDECFile;
use log::{debug, error, info, warn};
use simple_logger::SimpleLogger;
use std::fs::File;
use std::io::Read;

fn GAL20V8(jed: JEDECFile) {
    info!("Disassembling GAL20V8 fuse array");

    let f = jed.f;

    if f.len() != 2706 {
        error!("Incorrect fuse count (found {}, expected 2706)", f.len());
    }

    let syn = f[2704];
    let ac0 = f[2705];

    let mut xor = vec![];
    let mut ac1 = vec![];
    for i in 0..8 {
        xor.push(f[2560+i]);
        ac1.push(f[2632+i]);
    }

    debug!("SYN = {}, AC0 = {}", syn, ac0);

    let mode = match (syn, ac0) {
        (false, true)  => "Registered",
        (true,  true)  => "Complex",
        (true,  false) => "Simple",
                     _ => "Unknown",
    };

    info!("{} mode", mode);

    debug!("XOR = {:?}", xor);
    debug!("AC1 = {:?}", ac1);

    let row_symbols = [
        "B", "/B", "A", "/A", // 2,  1
        "C", "/C", "T", "/T", // 3,  23
        "D", "/D", "S", "/S", // 4,  22
        "E", "/E", "R", "/R", // 5,  21
        "F", "/F", "Q", "/Q", // 6,  20
        "G", "/G", "P", "/P", // 7,  17
        "H", "/H", "O", "/O", // 8,  16
        "I", "/I", "N", "/N", // 9,  15
        "J", "/J", "M", "/M", // 10, 14
        "K", "/K", "L", "/L", // 11, 13
    ];

    let olmc_count    = 8;
    let rows_per_olmc = 8;
    let row_width     = 40;
    for olmc in 0..olmc_count {
        let mut olmc_eqn = vec![];
        for i in 0..rows_per_olmc {
            let base = (olmc * rows_per_olmc + i) * row_width;
            let row = &f[base..base+row_width];

            // If all gates enabled, skip the row
            if !row.iter().fold(false, |a, b| a || *b) {
                continue;
            }

            // Build equation from symbols corresponding to cleared fuses
            let eqn: Vec<&str> = row_symbols.iter()
                .zip(row)
                .filter_map(
                    |(s, b)| match b {
                        false => Some(*s),
                        true => None
                    })
                .collect();
            olmc_eqn.push(format!("({})", eqn.join(" * ")));
        }
        debug!("OLMC {} = {:?}", olmc, olmc_eqn.join(" + "));
    }
}

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

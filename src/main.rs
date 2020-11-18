use clap::{Arg, App};
use jedec::JEDECFile;
use log::{debug, error, info, warn};
use simple_logger::SimpleLogger;
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;

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

    let column_connections = [
        2,  1,
        3,  23,
        4,  22,
        5,  21,
        6,  20,
        7,  17,
        8,  16,
        9,  15,
        10, 14,
        11, 13,
    ];

    let column_to_symbol = |col: u8| {
        let pin = column_connections[(col/2) as usize];
        let letter = pin + 0x40;
        if col & 1 == 1 {
            String::from(from_utf8(&[0x2F, letter]).unwrap())
        } else {
            String::from(from_utf8(&[letter]).unwrap())
        }
    };

    let olmc_pins = [
        22, 21, 20, 19, 18, 17, 16, 15,
    ];

    let olmc_to_symbol = |olmc: usize| {
        let letter = olmc_pins[olmc] + 0x40;
        if xor[olmc] {
            String::from(from_utf8(&[0x2F, letter]).unwrap())
        } else {
            String::from(from_utf8(&[letter]).unwrap())
        }
    };

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
            let eqn: Vec<String> = row.iter()
                .enumerate()
                .filter_map(
                    |(s, b)| match b {
                        false => Some(column_to_symbol(s as u8)),
                        true => None
                    })
                .collect();
            olmc_eqn.push(format!("{}", eqn.join(" * ")));
        }

        // If configured as output
        if !ac1[olmc] {
            println!("{} = {}", olmc_to_symbol(olmc), olmc_eqn.join("\n   + "));
        }
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

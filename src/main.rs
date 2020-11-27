use clap::{Arg, App};
use jedec::JEDECFile;
use log::{debug, error, info, warn};
use simple_logger::SimpleLogger;
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;

const OLMC_COUNT: usize = 8;
const SYN_ADDR:   usize = 2704;
const AC0_ADDR:   usize = 2705;
const XOR_ADDR:   usize = 2560;
const AC1_ADDR:   usize = 2632;


fn GAL20V8(jed: JEDECFile) {
    info!("Disassembling GAL20V8 fuse array");

    let f = jed.f;

    if f.len() != 2706 {
        error!("Incorrect fuse count (found {}, expected 2706)", f.len());
        return;
    }

    let syn = f[SYN_ADDR];
    let ac0 = f[AC0_ADDR];

    let xor = &f[XOR_ADDR..XOR_ADDR+OLMC_COUNT];
    let ac1 = &f[AC1_ADDR..AC1_ADDR+OLMC_COUNT];

    debug!("SYN = {}, AC0 = {}", syn, ac0);

    // [GAL20V8 datasheet pages 5,7,9]
    let mode = match (syn, ac0) {
        (false, true)  => "Registered",
        (true,  true)  => "Complex",
        (true,  false) => "Simple",
                     _ => "Unknown",
    };

    info!("{} mode", mode);

    if mode != "Simple" {
        error!("{} mode not supported", mode);
        return;
    }

    debug!("XOR = {:?}", xor);
    debug!("AC1 = {:?}", ac1);

    // List of pin numbers connected to each pair of columns.
    //
    // For each pin, there is one column with the non-inverted input and
    // one with the inverted input in the fuse array.
    // [GAL20V8 datasheet page 10]
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

    // List of OLMC pins (listed top to bottom).
    // [GAL20V8 datasheet page 10]
    let olmc_pins = [
        22, 21, 20, 19, 18, 17, 16, 15,
    ];

    let olmc_to_symbol = |olmc: usize| {
        let letter = olmc_pins[olmc] + 0x40;
        // XOR=0 defines Active Low Output.
        // XOR=1 defines Active High Output.
        // [GAL20V8 datasheet page 9]
        if xor[olmc] {
            String::from(from_utf8(&[letter]).unwrap())
        } else {
            String::from(from_utf8(&['/' as u8, letter]).unwrap())
        }
    };

    let rows_per_olmc = 8;
    let row_width     = 40;
    for olmc in 0..OLMC_COUNT {
        let mut olmc_eqn = vec![];
        for i in 0..rows_per_olmc {
            let base = (olmc * rows_per_olmc + i) * row_width;
            let row = &f[base..base+row_width];

            // Unused rows (usually?) have all fuses cleared.
            // That means that all columns connect to the row, but it ulimately
            // evaluates to false as it ANDs the non-inverted & inverted input
            // from every pin. So we skip the row to tidy up the output.
            if !row.iter().any(|&x| x) {
                continue;
            }

            // For each column, include the corresponding symbol if the fuse
            // is cleared.
            let eqn: Vec<String> = row.iter()
                .enumerate()
                .filter_map(
                    |(s, b)| match b {
                        false => Some(column_to_symbol(s as u8)),
                        true => None
                    })
                .collect();

            // Finally AND all the signals
            olmc_eqn.push(format!("{}", eqn.join(" * ")));
        }

        // If configured as output
        if !ac1[olmc] {
            println!("{} = {}", olmc_to_symbol(olmc), olmc_eqn.join("\n  + "));
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

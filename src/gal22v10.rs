use jedec::JEDECFile;
use log::{debug, error, info, warn};
use std::str::from_utf8;

const OLMC_COUNT:       usize = 10;
const OLMC_CONFIG_ADDR: usize = 5808;

pub fn GAL22V10(jed: JEDECFile) {
    info!("Disassembling GAL20V8 fuse array");

    let f = jed.f;

    if f.len() != 5892 {
        error!("Incorrect fuse count (found {}, expected 5892)", f.len());
        return;
    }

    let olmc_config = &f[OLMC_CONFIG_ADDR..OLMC_CONFIG_ADDR+(OLMC_COUNT*2)];

    debug!("olmc_config = {:?}", olmc_config);

    // List of pin numbers connected to each pair of columns.
    //
    // For each pin, there is one column with the non-inverted input and
    // one with the inverted input in the fuse array.
    // [GAL22V10 datasheet page 5]
    let column_connections = [
        1, 23,
        2, 22,
        3, 21,
        4, 20,
        5, 19,
        6, 18,
        7, 17,
        8, 16,
        9, 15,
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
    // [GAL22V10 datasheet page 5]
    let olmc_pins = [
        23, 22, 21, 20, 19, 18, 17, 16, 15, 14,
    ];

    let olmc_to_symbol = |olmc: usize| {
        let letter = olmc_pins[olmc] + 0x40;
        // S0=0 defines Active Low Output.
        // S0=1 defines Active High Output.
        // [GAL22V10 datasheet page 4]
        if !olmc_config[olmc*2] {
            String::from(from_utf8(&[letter]).unwrap())
        } else {
            String::from(from_utf8(&['/' as u8, letter]).unwrap())
        }
    };

    let rows_per_olmc = [
        8, 10, 12, 14, 16, 16, 14, 12, 10, 8,
    ];
    let row_width = 44;
    let mut addr  = 0;


    // Skip async reset line for now
    addr += row_width;

    for olmc in 0..OLMC_COUNT {
        let mut olmc_eqn = vec![];

        // Skip output enable line for now
        addr += row_width;

        for _ in 0..rows_per_olmc[olmc] {
            let row = &f[addr..addr+row_width];
            addr += row_width;

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

        println!("{} = {}", olmc_to_symbol(olmc), olmc_eqn.join("\n   + "));
    }
}


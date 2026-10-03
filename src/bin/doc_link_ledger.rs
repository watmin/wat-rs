//! Reads `WAT_DOC_LINK_LOG` and compares it to `wat::doc_link::KNOWN_BROKEN_DOC_LINKS`.
//! Does not spawn cargo. `scripts/floor.sh` runs this after `cargo doc`.

fn main() {
    let path = std::env::var("WAT_DOC_LINK_LOG").unwrap_or_else(|_| {
        eprintln!(
            "WAT_DOC_LINK_LOG is unset. scripts/floor.sh sets it to the cargo doc log. \
             This program does not spawn cargo."
        );
        std::process::exit(2);
    });
    let combined = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("could not read WAT_DOC_LINK_LOG {path}: {e}");
        std::process::exit(2);
    });
    if let Err(report) = wat::doc_link::judge(&combined) {
        eprintln!("{report}");
        std::process::exit(1);
    }
}

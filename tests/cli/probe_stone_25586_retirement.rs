//! Stone 255.86 — every row this stone added is refused by name, and the
//! refusal quotes the replacement the table records.
//!
//! The row set is derived from `retirement_table_pairs_for_gate`, not copied.
//! A new row under one of these retired prefixes is covered with no edit here.

use std::io::Write;
use std::process::Command;

fn stone_row(retired: &str) -> bool {
    const PREFIXES: &[&str] = &[
        ":wat::hashmap::",
        ":wat::map::",
        ":wat::vec::",
        ":wat::vector::",
        ":wat::hashset::",
        ":wat::linkedlist::",
        ":wat::core::Bytes/",
        ":wat::core::Record/field-at",
        ":wat::core::Record/same-data?",
        ":wat::core::Record/assoc",
    ];
    PREFIXES.iter().any(|p| retired.starts_with(p))
}

// rune:lint(no-inlined-wat) — the program is built from the table row at runtime,
// the same scaffold as `retirement_table_reachable`. The only variable is the row's name.
fn probe(retired_name: &str) -> String {
    let program = format!(
        "(:wat::core::defn :user::main [] -> wat.type/nil\n  (:wat::kernel::println ({retired_name})))\n"
    );
    let tag: String = retired_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let path = std::env::temp_dir().join(format!(
        "wat-25586-retire-{}-{}-{}.wat",
        std::process::id(),
        tag,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut f = std::fs::File::create(&path).expect("create temp");
    f.write_all(program.as_bytes()).expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(&path)
        .output()
        .expect("spawn wat");
    let _ = std::fs::remove_file(&path);
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    combined
}

#[test]
fn stone_rows_are_refused_naming_their_replacement() {
    let rows: Vec<_> = wat::retirement_table_pairs_for_gate()
        .into_iter()
        .filter(|(retired, _)| stone_row(retired))
        .collect();
    assert!(
        rows.len() >= 43,
        "the 255.86 prefix filter saw {} rows; the table bridge is not reaching the stone's block",
        rows.len()
    );
    let mut missed: Vec<String> = Vec::new();
    for (retired, replacement) in &rows {
        let output = probe(retired);
        let names_it = output.contains("is retired") && output.contains(replacement);
        if !names_it {
            missed.push(format!("{retired} -> {replacement}\n{output}"));
        }
    }
    assert!(
        missed.is_empty(),
        "{} stone row(s) were not refused with their table replacement:\n{}",
        missed.len(),
        missed.join("\n---\n")
    );
}

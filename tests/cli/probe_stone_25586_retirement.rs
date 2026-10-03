//! Stone 255.86 — every row this stone added is refused by the checker, and the
//! refusal quotes the replacement the table records.
//!
//! One `startup_bare` (the stdlib world), then each row is its own program through
//! `check_program`. A single program that calls every retired name reports one
//! type-check error and stops; separate programs do not. The row set is
//! `retirement_table_pairs_for_gate`, not a copy.

use wat::check::CheckErrorKind;

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

// rune:lint(no-inlined-wat) — the program is built from the table row. The scaffold is the
// expanded `def`/`fn` shape so the checker sees the call with no macro step. The only
// variable is the row's name.
fn program(retired_name: &str) -> String {
    format!("(:wat::core::def :probe::row (:wat::core::fn [] -> wat.type/nil ({retired_name})))\n")
}

#[derive(Debug)]
enum SetupFail {
    Parse(wat::ParseError),
    Register(wat::RuntimeError),
}

fn refusal(types: &wat::TypeEnv, retired: &str) -> Result<Vec<wat::CheckError>, SetupFail> {
    let forms = wat::parse_all_with_file(&program(retired), "<row>").map_err(SetupFail::Parse)?;
    let mut sym = wat::SymbolTable::new();
    let rest = wat::register_defines(forms, &mut sym).map_err(SetupFail::Register)?;
    match wat::check_program(&rest, &sym, types) {
        Ok(()) => Ok(Vec::new()),
        Err(wat::CheckErrors(errs)) => Ok(errs),
    }
}

fn names_the_replacement(errs: &[wat::CheckError], retired: &str, replacement: &str) -> bool {
    let want = format!("'{retired}' is retired; use '{replacement}' instead");
    errs.iter().any(|e| match &e.kind {
        CheckErrorKind::MalformedForm { head, reason, .. } => head == retired && reason == &want,
        _ => false,
    })
}

#[test]
fn stone_rows_are_refused_naming_their_replacement() {
    let world = wat::freeze::startup_bare().expect("stdlib");
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
        match refusal(world.types(), retired) {
            Err(SetupFail::Parse(e)) => missed.push(format!("{retired}: {e:?}")),
            Err(SetupFail::Register(e)) => missed.push(format!("{retired}: {e:?}")),
            Ok(errs) if names_the_replacement(&errs, retired, replacement) => {}
            Ok(errs) => missed.push(format!("{retired} -> {replacement}\n{errs:?}")),
        }
    }
    assert!(
        missed.is_empty(),
        "{} stone row(s) were not refused with their table replacement:\n{}",
        missed.len(),
        missed.join("\n---\n")
    );
}

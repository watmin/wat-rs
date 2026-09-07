//! Arc 296 K — the Rust-side type floor is named and walled.
//!
//! Sibling of `no_hand_written_enumdef.rs` (J). J admits NOTHING: every enum
//! could move. K admits a floor: category roots (aggregates whose name
//! `Nature::from_root_keyword` recognizes) and a short alias list whose every
//! entry carries a printed reason. A sibling file because the two walls have
//! opposite admission rules; folding K into J would make a red ambiguous
//! between "an enum that should have moved" and "an aggregate that is a root".
//!
//! ## ⛔ WHY THE OBVIOUS TEST IS VACUOUS
//!
//! Counting `TypeDef::Aggregate` in `TypeEnv::with_builtins()` cannot tell a
//! category root from a leftover hand-written record — both are Aggregates.
//! The defect is the FORM at the registration door, not the registered type.
//! Text-scan `register_builtin(TypeDef::Aggregate(AggregateDef` /
//! `register_builtin(TypeDef::Alias`.
//!
//! Generated registrations (inventory drain, runtime-error variant loop) do
//! not carry a string-literal `name: "…"`. They are codegen, not literals
//! (STOP-3). A wall that needed an exemption for them would be drawn at the
//! wrong level.

use wat::types::Nature;

const TYPES_RS: &str = include_str!("../../src/types.rs");

/// Strip `//` comments from a line.
fn code(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

/// Hand-written `register_builtin(TypeDef::Aggregate(AggregateDef {` sites
/// whose `name` is a string literal. Generated loops (`name: format!(…)`,
/// `name,`) are skipped.
fn aggregate_literal_names() -> Vec<(usize, String)> {
    let lines: Vec<&str> = TYPES_RS.lines().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let stripped = code(lines[i]);
        if !stripped.contains("register_builtin(TypeDef::Aggregate(AggregateDef") {
            i += 1;
            continue;
        }
        // Look ahead a few lines for `name: "..."`.
        let mut name: Option<String> = None;
        let mut generated = false;
        for line in lines.iter().skip(i).take(12) {
            let c = code(line);
            if c.contains("name: format!(") || c.contains("name,") && !c.contains("name: \"")
            {
                generated = true;
                break;
            }
            if let Some(rest) = c.split("name: \"").nth(1) {
                if let Some(end) = rest.find('"') {
                    name = Some(rest[..end].to_string());
                    break;
                }
            }
        }
        if generated {
            i += 1;
            continue;
        }
        if let Some(n) = name {
            out.push((i + 1, n));
        }
        i += 1;
    }
    out
}

fn alias_literal_names() -> Vec<(usize, String)> {
    let lines: Vec<&str> = TYPES_RS.lines().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let stripped = code(lines[i]);
        if !stripped.contains("register_builtin(TypeDef::Alias") {
            i += 1;
            continue;
        }
        let mut name: Option<String> = None;
        for line in lines.iter().skip(i).take(8) {
            let c = code(line);
            if let Some(rest) = c.split("name: \"").nth(1) {
                if let Some(end) = rest.find('"') {
                    name = Some(rest[..end].to_string());
                    break;
                }
            }
        }
        if let Some(n) = name {
            out.push((i + 1, n));
        }
        i += 1;
    }
    out
}

/// The alias floor: each surviving Rust AliasDef must be here, with a reason
/// the failure message prints. Not a permission list for aggregates — those
/// ask `Nature::from_root_keyword`.
fn alias_floor_reason(name: &str) -> Option<&'static str> {
    match name {
        ":wat::core::nil" => Some(
            "unit is TypeExpr::Tuple([]), not a path. A wat typealias of \
             :wat::core::nil only becomes Tuple([]) via the canonicalize \
             special-case that exists because nil is already the unit — the \
             concept declaring itself (STOP-2).",
        ),
        _ => None,
    }
}

#[test]
fn aggregate_literals_are_only_category_roots() {
    let names = aggregate_literal_names();
    assert!(
        !names.is_empty(),
        "the scan found no Aggregate string-literal names — a wall that cannot \
         see the three category roots is not this wall (row 6: must ADMIT roots)"
    );
    let mut refused = Vec::new();
    for (line, name) in &names {
        if Nature::from_root_keyword(name).is_none() {
            refused.push((line, name.clone()));
        }
    }
    assert!(
        refused.is_empty(),
        "hand-written `TypeDef::Aggregate` literal(s) whose name is NOT a \
         category root (Nature::from_root_keyword is None) at {refused:?}. \
         Move the type to wat via `wat_record_from!`, or if it is generated \
         codegen the wall should not have seen a string-literal name."
    );
}

#[test]
fn alias_literals_are_only_the_named_floor() {
    let names = alias_literal_names();
    let mut refused = Vec::new();
    for (line, name) in &names {
        if alias_floor_reason(name).is_none() {
            refused.push((line, name.clone()));
        }
    }
    assert!(
        refused.is_empty(),
        "hand-written `TypeDef::Alias` literal(s) not on the named floor at \
         {refused:?}. Movable aliases go through `wat_alias_register_from!`. \
         A survivor must carry a reason in `alias_floor_reason` distinguishing \
         impossible-in-principle from not-moved-yet."
    );
    // Non-vacuity: nil is expected to survive. A scan that finds zero aliases
    // after STOP-2 would still be a named floor of size 1 that went missing.
    let nil = names.iter().any(|(_, n)| n == ":wat::core::nil");
    assert!(
        nil,
        "the named alias floor should still contain `:wat::core::nil` (STOP-2). \
         If nil moved, update alias_floor_reason and this assertion together."
    );
}

#[test]
fn category_roots_are_present_and_admitted() {
    let names: Vec<String> = aggregate_literal_names()
        .into_iter()
        .map(|(_, n)| n)
        .collect();
    for root in [
        ":wat::core::Struct",
        ":wat::core::Record",
        ":wat::holon::Record",
    ] {
        assert!(
            names.iter().any(|n| n == root),
            "category root {root} is missing from Aggregate literals — a wall \
             that cannot see the roots it must ADMIT is not this wall"
        );
        assert!(
            Nature::from_root_keyword(root).is_some(),
            "oracle does not recognize {root} — the wall asks Nature::from_root_keyword, \
             not a hand-list"
        );
    }
}

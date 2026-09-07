//! Arc 296 J — a hand-written `TypeDef::Enum(EnumDef { … })` literal in
//! `src/types.rs` is refused. The only admitted form is `wat_enum_register_from!`.
//!
//! ## The property
//!
//! Every enum wat uses is declared in wat. H-3 and this stone moved the
//! registrations; this lint closes the door so the 26th cannot be hand-written
//! tomorrow. The substrate already enforces the hard half: a wat-sourced type is
//! re-declared at load, and `Existing::Equivalent` no-ops only if byte-equivalent.
//! This lint only has to refuse the hand-written FORM at the door.
//!
//! ## ⛔ WHY THE OBVIOUS TEST IS VACUOUS
//!
//! The natural gate is "count `TypeDef::Enum` in `TypeEnv::with_builtins()`".
//! **That test can never fail at the thing this stone cares about.** A wat-sourced
//! enum and a hand-written `EnumDef { … }` literal both produce `TypeDef::Enum`;
//! the TypeEnv cannot tell them apart. A green count would prove the types exist,
//! not that they were declared in wat. A test whose success condition is
//! indifferent to the defect is a green that proves nothing
//! ([[feedback_a_green_test_can_prove_nothing]]).
//!
//! So this gate scans the SOURCE of `register_builtin_types` for the literal
//! FORM `register_builtin(TypeDef::Enum(EnumDef {`. Comments do not count.
//! `wat_enum_register_from!` does not count — it is the admitted form.
//!
//! A wider grep for `TypeDef::Enum(EnumDef {` also hits `parse_defenum` (it
//! *constructs* an EnumDef from a wat form) and the defservice op/reply
//! synthesizer (it *emits* enums the surface declared). Those are not type
//! homes. Refusing them would force an exemption on day one (STOP-5).

/// Line numbers (1-based) of live `register_builtin(TypeDef::Enum(EnumDef {`
/// literals in `src/types.rs`.
fn hand_written_enumdef_sites() -> Vec<usize> {
    const TYPES_RS: &str = include_str!("../../src/types.rs");
    let mut sites = Vec::new();
    for (i, line) in TYPES_RS.lines().enumerate() {
        let stripped = line.split("//").next().unwrap_or("");
        if stripped.contains("register_builtin(TypeDef::Enum(EnumDef {") {
            sites.push(i + 1);
        }
    }
    sites
}

fn wat_enum_register_from_count() -> usize {
    const TYPES_RS: &str = include_str!("../../src/types.rs");
    TYPES_RS.matches("wat_enum_register_from!(").count()
}

fn enum_macro_source_paths() -> Vec<String> {
    const TYPES_RS: &str = include_str!("../../src/types.rs");
    let mut out = Vec::new();
    for (i, _) in TYPES_RS.match_indices("wat_enum_register_from!(") {
        let rest = &TYPES_RS[i..];
        let Some(open) = rest.find('"') else { continue };
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { continue };
        out.push(after[..close].to_string());
    }
    out
}

fn stdlib_paths() -> Vec<String> {
    const STDLIB_RS: &str = include_str!("../../src/load/stdlib.rs");
    let mut out = Vec::new();
    for (i, _) in STDLIB_RS.match_indices("path: \"") {
        let after = &STDLIB_RS[i + "path: \"".len()..];
        let Some(close) = after.find('"') else { continue };
        out.push(after[..close].to_string());
    }
    out
}

#[test]
fn no_hand_written_enumdef_literals_in_types_rs() {
    let sites = hand_written_enumdef_sites();
    assert!(
        sites.is_empty(),
        "hand-written `TypeDef::Enum(EnumDef {{ … }})` literal(s) in src/types.rs at line(s) \
         {sites:?}. A type wat uses is declared in wat; the admitted form is \
         `wat_enum_register_from!(env, \"<file>\", \":wat::ns::Name\")`. Transcribe the \
         enum into its family `.wat` file and register from there — do not add an \
         exemption on this lint's first day."
    );
}

#[test]
fn wat_enum_register_from_calls_are_the_sweep() {
    let n = wat_enum_register_from_count();
    // 26 moved this stone + Option/Result (H-3) + LociDiedError (H-2c) + the
    // arity-3 ServiceEvent proof in types.rs tests. A scan that finds fewer
    // is asserting over a partial sweep and would pass while literals remain.
    assert!(
        n >= 29,
        "the scan found only {n} `wat_enum_register_from!` call(s) in src/types.rs — \
         expected at least 26 (stone J) + 3 (H-3/H-2c). Either the scan broke or the \
         sweep is incomplete."
    );
}

#[test]
fn every_wat_enum_register_from_source_is_in_the_stdlib_load_set() {
    let sources = enum_macro_source_paths();
    let loaded = stdlib_paths();
    let missing: Vec<&String> = sources.iter().filter(|s| !loaded.contains(s)).collect();
    assert!(
        missing.is_empty(),
        "a `wat_enum_register_from!` source file is not in STDLIB_FILES: {missing:?}\n\
         The load-time re-declaration is what proves byte-equivalence. Drop the file \
         and that proof silently stops."
    );
}

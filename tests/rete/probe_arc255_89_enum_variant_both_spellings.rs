//! A nonexistent enum variant is refused in both spellings, in one file.
//!
//! `:evt::G::Hii` was already `UnknownEnumVariant`. Conversion wrote that same
//! mistake as `evt.G/Hii`, and `check_operand_field_ref` only entered on a
//! keyword, so the symbol spelling compiled and matched nothing.

use std::path::Path;
use std::process::{Command, Stdio};

fn run(rel: &str) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(bin)
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {rel} in {}: {e}", manifest.display()));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn a_nonexistent_variant_is_refused_in_both_spellings() {
    let (ok, out, err) = run("tests/rete/probe_arc255_89_enum_variant_both_spellings.wat");
    assert!(
        !ok,
        "both spellings must refuse\nstdout:{out}\nstderr:{err}"
    );
    assert_eq!(out, "", "main must not print\n{err}");
    wat::assert_edn_eq!(
        err.trim().to_string(),
        include_str!("probe_arc255_89_enum_variant_both_spellings__refusal.edn"),
        "two UnknownEnumVariant errors, one per spelling: rules evt::symbol-typo and \
         evt::keyword-typo, enum-path evt::G, variant Hii, available Hi and Lo"
    );
}

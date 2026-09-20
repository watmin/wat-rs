//! THE clj-ORACLE DIFFERENTIAL WARD — `clojure.edn` is the oracle; the EDN spec is the bar.
//!
//! Stone 218.7. `clojure.edn` is a **superset** of EDN (it reads quote, metadata, and
//! multi-slash symbols). The highlander split:
//!
//! | layer | job |
//! |---|---|
//! | **wat-edn** | spec-correct **EDN data**. Not clojure.edn parity. |
//! | **wat-reader** | Clojure-dialect **code** reader: EDN plus wat's reader macros. |
//!
//! Three categories (each `clj:OK / wat:ERR` that is not EDN needs a spec clause in
//! [`exemption`]; a `clj:ERR / wat:OK` needs a reason we accept invalid EDN, or it is a bug):
//!
//! | | meaning | example |
//! |---|---|---|
//! | `clj:OK / wat:OK`, `clj:ERR / wat:ERR` | parity | most rows |
//! | `clj:OK / wat:ERR` **and the construct is EDN** | **a wat bug** | i64-cap (fixed 218.7); `a:b` is now parity (218.8) |
//! | `clj:OK / wat:ERR` **and NOT EDN** | **CORRECT** — clj's superset, deliberately refused | `'x`, `^:m x` |
//! | `clj:ERR / wat:OK` | **a wat bug** unless exempted | `{:a 1 :a 2}` (bug) vs unknown tag (exempt) |
//!
//! Goldens are baked in `clj_oracle/golden.txt` (CI runs WITHOUT clojure). Regenerate:
//!
//! ```
//! python3 crates/wat-edn/tests/clj_oracle/generate_corpus.py
//! CORPUS=crates/wat-edn/tests/clj_oracle/corpus.txt \
//! GOLDEN=crates/wat-edn/tests/clj_oracle/golden.txt \
//!   clj -M crates/wat-edn/tests/clj_oracle/regen.clj
//! ```
//!
//! Grow the corpus from the grammar (`generate_corpus.py`), not a hand-list, until a
//! generation round finds zero new unexempted divergences (loop-until-dry).

const GOLDEN: &str = include_str!("clj_oracle/golden.txt");

/// The ONLY allowed divergences from the clj oracle — each with a load-bearing reason.
/// Anything not listed here must match clj exactly. A reason that does not name a spec
/// clause or a standing ruling is how a conformance suite dies.
fn exemption(input: &str) -> Option<&'static str> {
    // Do not trim: `\ ` (backslash + space) is a real corpus row; trim would
    // eat the space and hide the spec's whitespace-after-backslash rule.
    let t = input;

    // ── clj:OK / wat:ERR and NOT EDN — CORRECT (clj's superset) ──
    if t.starts_with('\'') {
        return Some(
            "quote (`'`) is a Clojure reader macro, not EDN — the spec has no quote; \
             this belongs to wat-reader",
        );
    }
    if t.starts_with('^') {
        return Some(
            "metadata (`^`) is a Clojure reader macro, not EDN; belongs to wat-reader",
        );
    }
    if t.starts_with('`') {
        return Some(
            "syntax-quote (backtick) is a Clojure reader macro, not EDN \
             (clj: Invalid leading character)",
        );
    }
    if t.starts_with('~') {
        return Some(
            "unquote (`~` / `~@`) is a Clojure reader macro, not EDN",
        );
    }
    if t.starts_with('@') {
        return Some("deref (`@`) is a Clojure reader macro, not EDN");
    }
    // Spec: "Backslash cannot be followed by whitespace." clj reads `\ `
    // as a space character anyway. Strict ⇒ we refuse.
    if t == "\\ " || t.starts_with("\\ ") {
        return Some(
            "spec: backslash cannot be followed by whitespace. clj accepts `\\ ` as \\space. \
             Strict ⇒ we refuse.",
        );
    }

    // ── clj:OK / wat:ERR — spec-strict rulings (clj is lenient) ──
    // Bare date only (`#inst "YYYY-MM-DD"`). Do not swallow `#inst "not-a-timestamp"`.
    if let Some(rest) = t.strip_prefix("#inst \"") {
        if let Some(inner) = rest.strip_suffix('"') {
            if inner.len() == 10
                && inner.as_bytes().get(4) == Some(&b'-')
                && !inner.contains('T')
            {
                return Some(
                    "spec requires RFC-3339 timestamps; a bare date is not a timestamp. \
                     clj promotes YYYY-MM-DD to midnight UTC. Strict ⇒ we refuse.",
                );
            }
        }
    }
    if slash_count_in_name(t) >= 2 {
        return Some(
            "spec: `/` can be used once only in the middle of a symbol; neither \
             prefix nor name may be empty. clj tolerates multi-slash (`a/b/c`, \
             `clojure.core//`). Strict ⇒ we refuse. wat-reader (code) may tolerate \
             them under the first-slash ruling; wat-edn (data) does not.",
        );
    }

    // ── clj:ERR / wat:OK — intentional EDN-valid superset ──
    if t.starts_with("#myapp/") {
        return Some(
            "unknown tag — wat reads it generically (spec-blessed 'read any and all edn'); \
             clj.edn's default declines. Intentional wat superset.",
        );
    }
    None
}

/// `/` separators in a symbol/keyword/tag token (not ratios `1/2`, not strings).
fn slash_count_in_name(t: &str) -> usize {
    if t.contains(' ') || t.contains('"') || t.starts_with('#') && t.contains(' ') {
        return 0;
    }
    // ratios: digits `/` digits
    if t.bytes().all(|b| b.is_ascii_digit() || b == b'/' || b == b'-' || b == b'+')
        && t.contains('/')
        && t.as_bytes().first().is_some_and(|b| b.is_ascii_digit() || *b == b'-' || *b == b'+')
    {
        return 0;
    }
    t.bytes().filter(|&b| b == b'/').count()
}

#[test]
fn wat_edn_matches_clj_oracle() {
    let mut fails = Vec::new();
    for line in GOLDEN.lines() {
        if line.is_empty() {
            continue;
        }
        let (clj, input) = line.split_once('\t').expect("golden row must be VERDICT\\tINPUT");
        let wat = match std::panic::catch_unwind(|| wat_edn::parse_owned(input)) {
            Ok(Ok(_)) => "OK",
            Ok(Err(_)) => "ERR",
            Err(_) => "PANIC",
        };
        if wat == clj {
            continue;
        }
        if exemption(input).is_some() {
            assert_ne!(
                wat, "PANIC",
                "exempted input {input:?} PANICKED — an exemption may diverge on OK/ERR, never panic"
            );
            continue;
        }
        fails.push(format!("  {input:?}\tclj:{clj}\twat:{wat}"));
    }
    assert!(
        fails.is_empty(),
        "\n\nclj-oracle parity VIOLATED — wat-edn diverges from clojure.edn on {} input(s) \
         (unexempted non-parity is an illegal state):\n{}\n\nFix wat-edn to match the EDN spec \
         (not clj's superset), or add a justified exemption that names a spec clause or standing ruling.\n",
        fails.len(),
        fails.join("\n"),
    );
}

#[test]
fn every_exemption_names_a_reason() {
    // Vacuity: at least the known not-EDN controls and the 219/strict rulings are classified.
    for input in [
        "'x",
        "^:m x",
        "`x",
        "~x",
        "@x",
        "a/b/c",
        "clojure.core//",
        "#inst \"1985-04-12\"",
        "#myapp/Foo {:x 1}",
        "\\ ",
    ] {
        assert!(
            exemption(input).is_some(),
            "expected an exemption (with a spec clause or standing ruling) for {input:?}"
        );
    }
    assert!(exemption("42").is_none());
    assert!(exemption("{:a 1 :a 2}").is_none()); // duplicate-key is a bug, not an exemption
    assert!(exemption("{:a").is_none()); // unclosed map
    assert!(exemption("#inst \"not-a-timestamp\"").is_none()); // malformed, not date-only
    assert!(exemption("::foo").is_none()); // auto-resolve
    // 218.8: `a:b` / `a#b` are parity, not exemptions. A stale exemption
    // here would be invisible to the ward (it continues at wat==clj).
    assert!(exemption("a:b").is_none());
    assert!(exemption("a#b").is_none());
}

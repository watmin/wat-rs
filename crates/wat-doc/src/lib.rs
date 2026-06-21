//! `wat-doc` — the shared prose+`@tag` doc-comment parser (arc 255.1b-iv-a).
//!
//! The doc/reflection contract (`docs/arc/2026/06/255-builtin-registry/
//! DESIGN-intrinsic-doc-reflection-contract.md`, §10) makes one load-bearing
//! decision: the prose+`@tag` parser + the mutual-checks live in **ONE shared
//! leaf crate**, depended on by BOTH `wat-macros` (the `#[wat_intrinsic]`
//! proc-macro, Rust intrinsics) AND `wat` (the runtime/checker, wat `defn`
//! forms). One implementation → the two paths **cannot drift**. An intrinsic's
//! `///` block and a wat form's docstring parse through the same code, enforce
//! the same required directives, and produce the same [`DocComment`] — parity by
//! construction, not by discipline.
//!
//! This crate is a pure leaf: no signature knowledge, no registry, no type
//! system, no codegen. It turns the joined text of a doc block into a structured
//! [`DocComment`] (enforcing the *universal* required directives), and offers the
//! one mutual-check that needs no external state — [`check_args`], the
//! `@arg`⇄signature agreement, with the caller supplying the parameter names.
//! Everything that needs the signature / the registry / the `TypeScheme`
//! (`@see` resolution, `@arg`/`@ret` type-checking, doctest generation) is the
//! consumer's job, in later stones.
//!
//! # Grammar (v1, line-based)
//!
//! The input is the joined `///` text (one string, `\n`-separated, each line
//! already stripped of its `/// ` prefix). **Prose** is every line before the
//! first recognized `@`-directive line; it is GFM, kept verbatim, trimmed of
//! surrounding blank lines, and **required**. A directive line begins (after
//! optional leading whitespace) with `@<word>`:
//!
//! | directive | req | shape |
//! |---|---|---|
//! | `@added <ver>` | yes, singleton | version string |
//! | `@arg <name> [sep] <desc>` | per-param (see [`check_args`]) | `sep` ∈ ` — `/` -- `/` - `/`: ` |
//! | `@ret [sep] <desc>` | yes, singleton | return description |
//! | `@example <expr> #=> <expected>` | ≥1 of either kind | doctested; MUST carry `#=>` |
//! | `@example-norun <expr> [#=> <expected>]` | ≥1 of either kind | illustrative; `#=>` optional |
//! | `@deprecated <ver> <use-instead>` | optional, singleton | soft-deprecation |
//! | `@see <fqdn>` | optional, repeatable | cross-reference |
//!
//! An unrecognized `@word` is a hard [`DocError::UnknownDirective`] — never a
//! silent skip. Each `@example(-norun)` is one line (multi-line examples are a
//! NAMED future extension, not built in v1).

/// One parsed `@example` / `@example-norun` directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocExample {
    /// The wat form, verbatim — the text left of `#=>` (or the whole remainder
    /// for a markerless `@example-norun`), trimmed.
    pub expr: String,
    /// The expected result, right of `#=>`, trimmed; `None` when no marker
    /// (only legal for `@example-norun`).
    pub expected: Option<String>,
    /// `true` = `@example` (doctested by the consumer); `false` =
    /// `@example-norun` (illustrative, never executed).
    pub run: bool,
}

/// One parsed `@arg` directive. Name/count agreement with the actual signature
/// is [`check_args`]'s job, not the parser's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocArg {
    /// First whitespace-delimited token after `@arg`.
    pub name: String,
    /// The remainder, with a single leading separator (` — `/` -- `/` - `/`: `)
    /// stripped, trimmed.
    pub desc: String,
}

/// A parsed `@deprecated` directive (soft deprecation — still callable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deprecation {
    /// First token after `@deprecated` — the version it was deprecated in.
    pub since: String,
    /// The remainder, trimmed — what to use instead.
    pub use_instead: String,
}

/// A fully-parsed doc comment with every *universal* required directive present
/// (prose, `@added`, `@ret`, ≥1 example). The `@arg`⇄signature match is checked
/// separately by [`check_args`], which needs the caller's parameter list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocComment {
    /// GFM body: everything before the first `@`-tag line, trimmed.
    pub prose: String,
    /// `@added <ver>`.
    pub added: String,
    /// `@arg` directives, in source order.
    pub args: Vec<DocArg>,
    /// `@ret` description.
    pub ret: String,
    /// `@example` / `@example-norun` directives, in source order (≥1).
    pub examples: Vec<DocExample>,
    /// `@deprecated`, if present.
    pub deprecated: Option<Deprecation>,
    /// `@see <fqdn>` cross-references, in source order.
    pub see: Vec<String>,
}

/// A doc-contract violation. Closed enum — diagnostic completeness by shape.
/// `parse` raises the structural variants; [`check_args`] raises the two
/// `Arg*Mismatch` variants. The consumer renders these as a `compile_error!`
/// (the proc-macro) or a runtime/check error (wat).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocError {
    /// No prose before the first directive (or only blank lines).
    MissingProse,
    /// No `@added` directive.
    MissingAdded,
    /// No `@ret` directive.
    MissingRet,
    /// No `@example` or `@example-norun` directive.
    MissingExample,
    /// A directive's payload was empty or unparseable. `why` is a short reason.
    MalformedDirective { tag: String, why: &'static str },
    /// An `@word` that is not a recognized directive.
    UnknownDirective { tag: String },
    /// An `@example` (run=true) with no `#=>` marker. (`@example-norun` may omit it.)
    ExampleMissingMarker { expr: String },
    /// A second occurrence of a singleton directive (`@added`, `@ret`, `@deprecated`).
    DuplicateSingleton { tag: String },
    /// `@arg` count ≠ signature parameter count.
    ArgCountMismatch { documented: usize, signature: usize },
    /// The `@arg` at `position` names a different parameter than the signature.
    ArgNameMismatch {
        position: usize,
        documented: String,
        signature: String,
    },
}

/// Parse a joined `///` block into a [`DocComment`], enforcing the universal
/// required directives (prose, `@added`, `@ret`, and ≥1 `@example`/`@example-norun`).
///
/// Does NOT check `@arg` against any signature — that is [`check_args`].
pub fn parse(_raw: &str) -> Result<DocComment, DocError> {
    // STRIKE 255.1b-iv-a: the executor fills this per the grammar above and the
    // unit tests below (which are the spec). RED at skeleton.
    todo!("wat-doc::parse — fill per the grammar + the #[cfg(test)] spec")
}

/// The `@arg`⇄signature mutual check: the documented args must match `params`
/// (the wat-arg names, in order) by count and name. A 0-param intrinsic must
/// document 0 args. This is what makes "`@arg` required ×params" true.
pub fn check_args(_doc: &DocComment, _params: &[&str]) -> Result<(), DocError> {
    // STRIKE 255.1b-iv-a: fill per the spec below. RED at skeleton.
    todo!("wat-doc::check_args — count+name agreement vs supplied params")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reference intrinsic doc block (`core::Bytes::to-hex`), in the exact
    /// joined form `sniff_doc` produces (`/// ` stripped, `\n`-joined). This IS
    /// the contract the parser must satisfy.
    const TO_HEX: &str = "Encode a `:wat::core::Bytes` into its lowercase-hex `:String`.\n\nMarkdown prose, GFM — flows straight to the wiki page body.\n\n@added   1.0.0\n@arg     bs — the bytes to encode\n@ret     the lowercase hex string, two chars per byte, no separators\n@example (:wat::core::Bytes::to-hex (:wat::core::Vector 255 0 16)) #=> \"ff0010\"";

    #[test]
    fn parses_the_reference_intrinsic() {
        let doc = parse(TO_HEX).expect("to-hex doc parses");
        assert_eq!(
            doc.prose,
            "Encode a `:wat::core::Bytes` into its lowercase-hex `:String`.\n\nMarkdown prose, GFM — flows straight to the wiki page body."
        );
        assert_eq!(doc.added, "1.0.0");
        assert_eq!(
            doc.args,
            vec![DocArg { name: "bs".into(), desc: "the bytes to encode".into() }]
        );
        assert_eq!(doc.ret, "the lowercase hex string, two chars per byte, no separators");
        assert_eq!(
            doc.examples,
            vec![DocExample {
                expr: "(:wat::core::Bytes::to-hex (:wat::core::Vector 255 0 16))".into(),
                expected: Some("\"ff0010\"".into()),
                run: true,
            }]
        );
        assert_eq!(doc.deprecated, None);
        assert!(doc.see.is_empty());
    }

    #[test]
    fn norun_example_may_omit_the_marker() {
        let raw = "Write bytes to a path.\n\n@added 1.0.0\n@arg p — the path\n@ret :ok on success\n@example-norun (:wat::core::File::write p data)";
        let doc = parse(raw).expect("norun parses");
        assert_eq!(
            doc.examples,
            vec![DocExample {
                expr: "(:wat::core::File::write p data)".into(),
                expected: None,
                run: false,
            }]
        );
    }

    #[test]
    fn norun_example_may_carry_an_unverified_marker() {
        let raw = "Read a uuid.\n\n@added 1.0.0\n@ret a fresh uuid\n@example-norun (:wat::core::Uuid/v4) #=> #uuid \"…\"";
        let doc = parse(raw).expect("norun-with-marker parses");
        assert_eq!(doc.examples[0].run, false);
        assert_eq!(doc.examples[0].expected.as_deref(), Some("#uuid \"…\""));
    }

    #[test]
    fn multiple_args_and_see_in_order() {
        let raw = "Blend two things.\n\n@added 1.2.0\n@arg a -- the first\n@arg b: the second\n@ret the blend\n@example (f 1 2) #=> 3\n@see :wat::core::other\n@see :wat::core::another";
        let doc = parse(raw).expect("parses");
        assert_eq!(doc.args.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(doc.args[0].desc, "the first");
        assert_eq!(doc.args[1].desc, "the second");
        assert_eq!(doc.see, vec![":wat::core::other".to_string(), ":wat::core::another".to_string()]);
    }

    #[test]
    fn deprecated_parses() {
        let raw = "Old thing.\n\n@added 1.0.0\n@ret nothing useful\n@example (g) #=> nil\n@deprecated 2.0.0 use :wat::core::new-thing instead";
        let doc = parse(raw).expect("parses");
        assert_eq!(
            doc.deprecated,
            Some(Deprecation { since: "2.0.0".into(), use_instead: "use :wat::core::new-thing instead".into() })
        );
    }

    // --- negative cases: every required-directive omission is a named error ---

    #[test]
    fn missing_prose_is_an_error() {
        let raw = "@added 1.0.0\n@ret x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingProse));
    }

    #[test]
    fn missing_added_is_an_error() {
        let raw = "Prose.\n\n@ret x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingAdded));
    }

    #[test]
    fn missing_ret_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingRet));
    }

    #[test]
    fn missing_example_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@ret x";
        assert_eq!(parse(raw), Err(DocError::MissingExample));
    }

    #[test]
    fn run_example_without_marker_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@ret x\n@example (f 1 2)";
        assert_eq!(
            parse(raw),
            Err(DocError::ExampleMissingMarker { expr: "(f 1 2)".into() })
        );
    }

    #[test]
    fn unknown_directive_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@ret x\n@example (f) #=> y\n@bogus whatever";
        assert_eq!(parse(raw), Err(DocError::UnknownDirective { tag: "@bogus".into() }));
    }

    #[test]
    fn duplicate_added_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@added 1.1.0\n@ret x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::DuplicateSingleton { tag: "@added".into() }));
    }

    // --- check_args: the @arg ⇄ signature mutual check ---

    #[test]
    fn check_args_agrees() {
        let doc = parse(TO_HEX).unwrap();
        assert_eq!(check_args(&doc, &["bs"]), Ok(()));
    }

    #[test]
    fn check_args_count_mismatch() {
        let doc = parse(TO_HEX).unwrap();
        assert_eq!(
            check_args(&doc, &[]),
            Err(DocError::ArgCountMismatch { documented: 1, signature: 0 })
        );
    }

    #[test]
    fn check_args_name_mismatch() {
        let doc = parse(TO_HEX).unwrap();
        assert_eq!(
            check_args(&doc, &["data"]),
            Err(DocError::ArgNameMismatch { position: 0, documented: "bs".into(), signature: "data".into() })
        );
    }

    #[test]
    fn check_args_zero_arity_ok() {
        let raw = "A constant.\n\n@added 1.0.0\n@ret the value\n@example (k) #=> 42";
        let doc = parse(raw).unwrap();
        assert_eq!(check_args(&doc, &[]), Ok(()));
    }
}

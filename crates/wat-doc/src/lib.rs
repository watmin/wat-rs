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
//! | `@arg <name> <type> <desc>` | per-param (see [`check_args`]) | type must start with `:` |
//! | `@ret <type> <desc>` | yes, singleton | type must start with `:` |
//! | `@example <expr> #=> <expected>` | ≥1 of either kind | doctested; MUST carry `#=>` |
//! | `@example-norun <expr> [#=> <expected>]` | ≥1 of either kind | illustrative; `#=>` optional |
//! | `@deprecated <ver> <use-instead>` | optional, singleton | soft-deprecation |
//! | `@see <fqdn>` | optional, repeatable | cross-reference |
//!
//! An unrecognized `@word` is a hard [`DocError::UnknownDirective`] — never a
//! silent skip. Old separator forms (` — `, ` -- `, ` - `, `: `) are REJECTED
//! as illegal in the type position — the grammar is firm: `@arg <name> <type> <desc>`.

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
    /// Second whitespace-delimited token after `@arg` — the type, must start with `:`.
    pub ty: String,
    /// The remainder after name and type, trimmed.
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
    /// `@ret` type token (must start with `:`).
    pub ret_type: String,
    /// `@ret` description (remainder after the type token).
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

/// The separator tokens that are now ILLEGAL in the type position.
/// If the type token equals one of these, the grammar is violated.
const SEPARATOR_TOKENS: &[&str] = &["—", "--", "-", ":"];

/// Parse a joined `///` block into a [`DocComment`], enforcing the universal
/// required directives (prose, `@added`, `@ret`, and ≥1 `@example`/`@example-norun`).
///
/// Does NOT check `@arg` against any signature — that is [`check_args`].
pub fn parse(raw: &str) -> Result<DocComment, DocError> {
    let recognized = &[
        "@added", "@arg", "@ret", "@example", "@example-norun", "@deprecated", "@see",
    ];

    // Split into prose lines and directive lines at the first recognized @-directive.
    let lines: Vec<&str> = raw.lines().collect();
    let first_directive = lines.iter().position(|l| {
        let token = l.split_whitespace().next().unwrap_or("");
        token.starts_with('@') && recognized.contains(&token)
    });

    // Prose = everything before the first directive line, trimmed of surrounding blanks.
    let prose_end = first_directive.unwrap_or(lines.len());
    let prose = trim_blank_lines(&lines[..prose_end]).join("\n");
    if prose.is_empty() {
        return Err(DocError::MissingProse);
    }

    // Walk directive lines.
    let mut added: Option<String> = None;
    let mut args: Vec<DocArg> = Vec::new();
    let mut ret_type: Option<String> = None;
    let mut ret: Option<String> = None;
    let mut examples: Vec<DocExample> = Vec::new();
    let mut deprecated: Option<Deprecation> = None;
    let mut see: Vec<String> = Vec::new();

    let directive_lines = match first_directive {
        Some(i) => &lines[i..],
        None => &[][..],
    };

    for &line in directive_lines {
        let trimmed = line.trim_start();
        let tag = trimmed.split_whitespace().next().unwrap_or("");

        if !tag.starts_with('@') {
            // Non-directive lines after the first directive (e.g. blank lines) — skip.
            continue;
        }

        if !recognized.contains(&tag) {
            return Err(DocError::UnknownDirective { tag: tag.to_string() });
        }

        // Payload = everything after the tag, leading whitespace stripped.
        let payload = trimmed[tag.len()..].trim_start();

        match tag {
            "@added" => {
                if added.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@added".into() });
                }
                if payload.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@added".into(),
                        why: "version string is empty",
                    });
                }
                added = Some(payload.to_string());
            }
            "@arg" => {
                // Firm grammar: @arg <name> <type> <desc>
                // name = first token, type = second token (must start with `:`), desc = rest
                let mut tokens = payload.splitn(3, char::is_whitespace);
                let raw_name = tokens.next().unwrap_or("");
                if raw_name.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "name is missing",
                    });
                }
                let name = raw_name.to_string();

                let ty_token = tokens.next().unwrap_or("").trim();
                if ty_token.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "type is missing; grammar is `@arg <name> <type> <desc>`",
                    });
                }
                // Reject separator tokens in the type position.
                if SEPARATOR_TOKENS.contains(&ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "separator used in type position; grammar is `@arg <name> <type> <desc>`",
                    });
                }
                // Type token must start with `:` (all wat types are keywords).
                if !ty_token.starts_with(':') {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "type token must start with `:` (e.g. `:wat::core::Bytes`); grammar is `@arg <name> <type> <desc>`",
                    });
                }
                let ty = ty_token.to_string();

                let desc = tokens.next().unwrap_or("").trim().to_string();
                if desc.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "description is empty; grammar is `@arg <name> <type> <desc>`",
                    });
                }
                args.push(DocArg { name, ty, desc });
            }
            "@ret" => {
                if ret.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@ret".into() });
                }
                // Firm grammar: @ret <type> <desc>
                let mut tokens = payload.splitn(2, char::is_whitespace);
                let ty_token = tokens.next().unwrap_or("").trim();
                if ty_token.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "type is missing; grammar is `@ret <type> <desc>`",
                    });
                }
                // Reject separator tokens in the type position.
                if SEPARATOR_TOKENS.contains(&ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "separator used in type position; grammar is `@ret <type> <desc>`",
                    });
                }
                // Type token must start with `:`.
                if !ty_token.starts_with(':') {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "type token must start with `:` (e.g. `:wat::core::String`); grammar is `@ret <type> <desc>`",
                    });
                }
                let ty = ty_token.to_string();
                let desc = tokens.next().unwrap_or("").trim().to_string();
                if desc.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "description is empty; grammar is `@ret <type> <desc>`",
                    });
                }
                ret_type = Some(ty);
                ret = Some(desc);
            }
            "@example" => {
                let rest = payload;
                match rest.split_once(" #=> ").or_else(|| rest.split_once("#=> ")) {
                    Some((left, right)) => {
                        let expr = left.trim().to_string();
                        let expected = right.trim().to_string();
                        examples.push(DocExample {
                            expr,
                            expected: Some(expected),
                            run: true,
                        });
                    }
                    None => {
                        // Check if #=> appears at end with no trailing content.
                        if let Some(left) = rest.strip_suffix("#=>") {
                            let expr = left.trim().to_string();
                            examples.push(DocExample {
                                expr,
                                expected: Some(String::new()),
                                run: true,
                            });
                        } else {
                            return Err(DocError::ExampleMissingMarker {
                                expr: rest.trim().to_string(),
                            });
                        }
                    }
                }
            }
            "@example-norun" => {
                let rest = payload;
                let (expr, expected) =
                    if let Some((left, right)) = rest.split_once(" #=> ").or_else(|| rest.split_once("#=> ")) {
                        (left.trim().to_string(), Some(right.trim().to_string()))
                    } else {
                        (rest.trim().to_string(), None)
                    };
                examples.push(DocExample { expr, expected, run: false });
            }
            "@deprecated" => {
                if deprecated.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@deprecated".into() });
                }
                let mut tokens = payload.splitn(2, char::is_whitespace);
                let since = tokens.next().unwrap_or("").to_string();
                let use_instead = tokens.next().unwrap_or("").trim_start().to_string();
                deprecated = Some(Deprecation { since, use_instead });
            }
            "@see" => {
                see.push(payload.to_string());
            }
            _ => unreachable!("recognized set is exhaustive"),
        }
    }

    // Enforce required directives.
    let added = added.ok_or(DocError::MissingAdded)?;
    let ret_type = ret_type.ok_or(DocError::MissingRet)?;
    let ret = ret.ok_or(DocError::MissingRet)?;
    if examples.is_empty() {
        return Err(DocError::MissingExample);
    }

    Ok(DocComment { prose, added, args, ret_type, ret, examples, deprecated, see })
}

/// Trim leading and trailing blank (empty/whitespace-only) lines from a slice.
fn trim_blank_lines<'a>(lines: &'a [&'a str]) -> &'a [&'a str] {
    let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(lines.len());
    let end = lines.iter().rposition(|l| !l.trim().is_empty()).map(|i| i + 1).unwrap_or(0);
    if start >= end { &[] } else { &lines[start..end] }
}

/// The `@arg`⇄signature mutual check: the documented args must match `params`
/// (the wat-arg names, in order) by count and name. A 0-param intrinsic must
/// document 0 args. This is what makes "`@arg` required ×params" true.
pub fn check_args(doc: &DocComment, params: &[&str]) -> Result<(), DocError> {
    if doc.args.len() != params.len() {
        return Err(DocError::ArgCountMismatch {
            documented: doc.args.len(),
            signature: params.len(),
        });
    }
    for (i, (arg, &param)) in doc.args.iter().zip(params.iter()).enumerate() {
        if arg.name != param {
            return Err(DocError::ArgNameMismatch {
                position: i,
                documented: arg.name.clone(),
                signature: param.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reference intrinsic doc block (`core::Bytes::to-hex`), in the exact
    /// joined form `sniff_doc` produces (`/// ` stripped, `\n`-joined). This IS
    /// the contract the parser must satisfy. Updated to firm grammar (no separator).
    const TO_HEX: &str = "Encode a `:wat::core::Bytes` into its lowercase-hex `:String`.\n\nMarkdown prose, GFM — flows straight to the wiki page body.\n\n@added   1.0.0\n@arg     bs :wat::core::Bytes the bytes to encode\n@ret     :wat::core::String the lowercase hex string, two chars per byte, no separators\n@example (:wat::core::Bytes::to-hex (:wat::core::Vector :u8 (:wat::core::u8 255) (:wat::core::u8 0) (:wat::core::u8 16))) #=> \"ff0010\"";

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
            vec![DocArg { name: "bs".into(), ty: ":wat::core::Bytes".into(), desc: "the bytes to encode".into() }]
        );
        assert_eq!(doc.ret_type, ":wat::core::String");
        assert_eq!(doc.ret, "the lowercase hex string, two chars per byte, no separators");
        assert_eq!(
            doc.examples,
            vec![DocExample {
                expr: "(:wat::core::Bytes::to-hex (:wat::core::Vector :u8 (:wat::core::u8 255) (:wat::core::u8 0) (:wat::core::u8 16)))".into(),
                expected: Some("\"ff0010\"".into()),
                run: true,
            }]
        );
        assert_eq!(doc.deprecated, None);
        assert!(doc.see.is_empty());
    }

    #[test]
    fn norun_example_may_omit_the_marker() {
        let raw = "Write bytes to a path.\n\n@added 1.0.0\n@arg p :wat::core::Path the path\n@ret :wat::core::Result ok on success\n@example-norun (:wat::core::File::write p data)";
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
        let raw = "Read a uuid.\n\n@added 1.0.0\n@ret :wat::core::String a fresh uuid\n@example-norun (:wat::core::Uuid/v4) #=> #uuid \"…\"";
        let doc = parse(raw).expect("norun-with-marker parses");
        assert_eq!(doc.examples[0].run, false);
        assert_eq!(doc.examples[0].expected.as_deref(), Some("#uuid \"…\""));
    }

    #[test]
    fn multiple_args_and_see_in_order() {
        let raw = "Blend two things.\n\n@added 1.2.0\n@arg a :wat::core::i64 the first\n@arg b :wat::core::i64 the second\n@ret :wat::core::i64 the blend\n@example (f 1 2) #=> 3\n@see :wat::core::other\n@see :wat::core::another";
        let doc = parse(raw).expect("parses");
        assert_eq!(doc.args.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(doc.args[0].desc, "the first");
        assert_eq!(doc.args[1].desc, "the second");
        assert_eq!(doc.see, vec![":wat::core::other".to_string(), ":wat::core::another".to_string()]);
    }

    #[test]
    fn deprecated_parses() {
        let raw = "Old thing.\n\n@added 1.0.0\n@ret :wat::core::i64 nothing useful\n@example (g) #=> nil\n@deprecated 2.0.0 use :wat::core::new-thing instead";
        let doc = parse(raw).expect("parses");
        assert_eq!(
            doc.deprecated,
            Some(Deprecation { since: "2.0.0".into(), use_instead: "use :wat::core::new-thing instead".into() })
        );
    }

    // --- negative cases: separator in type position is now rejected ---

    #[test]
    fn separator_in_arg_type_position_is_rejected() {
        // Old grammar: `@arg bs — the bytes` — "—" is now in the type position,
        // which is illegal (separator token rejected).
        let raw = "Prose.\n\n@added 1.0.0\n@arg bs — the bytes\n@ret :wat::core::String desc\n@example (f) #=> y";
        match parse(raw) {
            Err(DocError::MalformedDirective { tag, .. }) => assert_eq!(tag, "@arg"),
            other => panic!("expected MalformedDirective for @arg separator, got {:?}", other),
        }
    }

    #[test]
    fn non_colon_type_in_arg_is_rejected() {
        // Type token doesn't start with `:` — also illegal.
        let raw = "Prose.\n\n@added 1.0.0\n@arg bs Bytes the bytes\n@ret :wat::core::String desc\n@example (f) #=> y";
        match parse(raw) {
            Err(DocError::MalformedDirective { tag, .. }) => assert_eq!(tag, "@arg"),
            other => panic!("expected MalformedDirective for non-colon @arg type, got {:?}", other),
        }
    }

    #[test]
    fn separator_in_ret_type_position_is_rejected() {
        // Old grammar: `@ret — the desc` — "—" is now in the type position, illegal.
        let raw = "Prose.\n\n@added 1.0.0\n@arg bs :wat::core::Bytes the bytes\n@ret — the bytes\n@example (f) #=> y";
        match parse(raw) {
            Err(DocError::MalformedDirective { tag, .. }) => assert_eq!(tag, "@ret"),
            other => panic!("expected MalformedDirective for @ret separator, got {:?}", other),
        }
    }

    // --- negative cases: every required-directive omission is a named error ---

    #[test]
    fn missing_prose_is_an_error() {
        let raw = "@added 1.0.0\n@ret :wat::core::i64 x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingProse));
    }

    #[test]
    fn missing_added_is_an_error() {
        let raw = "Prose.\n\n@ret :wat::core::i64 x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingAdded));
    }

    #[test]
    fn missing_ret_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingRet));
    }

    #[test]
    fn missing_example_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@ret :wat::core::i64 x";
        assert_eq!(parse(raw), Err(DocError::MissingExample));
    }

    #[test]
    fn run_example_without_marker_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@ret :wat::core::i64 x\n@example (f 1 2)";
        assert_eq!(
            parse(raw),
            Err(DocError::ExampleMissingMarker { expr: "(f 1 2)".into() })
        );
    }

    #[test]
    fn unknown_directive_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@ret :wat::core::i64 x\n@example (f) #=> y\n@bogus whatever";
        assert_eq!(parse(raw), Err(DocError::UnknownDirective { tag: "@bogus".into() }));
    }

    #[test]
    fn duplicate_added_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@added 1.1.0\n@ret :wat::core::i64 x\n@example (f) #=> y";
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
        let raw = "A constant.\n\n@added 1.0.0\n@ret :wat::core::i64 the value\n@example (k) #=> 42";
        let doc = parse(raw).unwrap();
        assert_eq!(check_args(&doc, &[]), Ok(()));
    }
}

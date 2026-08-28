//! FM 2-bis probe for arc 233 Stone 233.2.d (substrate-symmetry).
//!
//! Asserts the substrate-symmetry invariant: every dispatch arm in
//! `dispatch_keyword_head` that delegates into the eval layer (`eval_*`
//! fns) threads `list_span` to the called fn. Inline arms that don't
//! invoke a downstream eval fn are exempt (substrate-symmetry doctrine
//! applies at the dispatch boundary, not at leaf inline blocks).
//!
//! Pre-stone state: FAILS. Approximately half of the dispatch arms
//! that call into `eval_*` drop `list_span` at the call site. The
//! probe lists the offending arm keywords on failure so sonnet can
//! sweep them mechanically (substrate-as-teacher per FM 15).
//!
//! Post-stone state: PASSES. Probe stays as permanent regression
//! guard — future dispatch arm additions that drop `list_span` will
//! fail this test.
//!
//! Composes with the precedent: Stone 233.2.c's `eval_edn_read`
//! signature plumb (commit `c0f41f6`) was the one-arm preview of the
//! canonical template this stone extends across the dispatch table.

use regex::Regex;
use std::fs;

const RUNTIME_PATH: &str = "src/runtime.rs";
// Stone 233.2.j split dispatch_keyword_head into two functions:
// - dispatch_keyword_head: handles the 3 TrackedValue producers (short match)
// - dispatch_keyword_head_value: the full dispatch table (350+ arms)
// The symmetry invariant applies to the full table, so probe the value-returning fn.
const DISPATCH_FN_SIGNATURE: &str = "fn dispatch_keyword_head_value(";
const MATCH_HEADER: &str = "match head {";

#[derive(Debug)]
struct Arm {
    keywords: String,
    body: String,
}

#[test]
fn every_dispatch_arm_calling_eval_threads_list_span() {
    let src = fs::read_to_string(RUNTIME_PATH)
        .unwrap_or_else(|e| panic!("could not read {RUNTIME_PATH}: {e}"));

    let body = extract_dispatch_match_body(&src);
    let arms = parse_arms(&body);

    // ── PARSER SANITY — a POSITIVE CONTROL, not a tuned magnitude.
    //
    // This was `arms.len() >= 300`, a bound set to the then-current count minus a
    // margin. That is a REGRESSION detector wearing a sanity check's clothes, and it
    // is at war with arc 255: the campaign's whole purpose is to move per-type verbs
    // OUT of this match and into registry homes, so the count only ever falls. Stone C
    // lowered the bound once (354 -> ~318, floor 300); Stone E-i deleted 16 more and
    // tripped it again at 289; E-ii and E-iii will trip it again. Each lowering weakens
    // the guard until it guards nothing.
    // `[[feedback_a_gate_freezes_names_never_a_count]]`
    //
    // What this check actually needs to catch is "the parser returned garbage" — so it
    // names arms the carve CANNOT take. These are polymorphic/structural heads, not
    // per-type verbs: nothing in the homes campaign relocates them, and if the parser
    // breaks they vanish together.
    const MUST_FIND: &[&str] = &[
        ":wat::core::apply",
        ":wat::core::and",
        ":wat::core::ann-form",
        ":wat::core::aggregate-new",
    ];
    for needle in MUST_FIND {
        assert!(
            arms.iter().any(|a| a.keywords.contains(needle)),
            "parser sanity: {needle} is a structural dispatch arm that no home carve \
             relocates, and the parser did not find it among {} arms. Investigate the \
             parser before trusting the symmetry verdict.",
            arms.len(),
        );
    }
    // A floor loose enough to never need nudging again: it catches "returned nothing
    // useful", which is the only failure the positive control above cannot.
    assert!(
        arms.len() >= 50,
        "parser sanity: got {} arms — the parser is broken, not the substrate.",
        arms.len(),
    );

    let mut compliant: usize = 0;
    let mut exempt: usize = 0;
    let mut violations: Vec<&Arm> = Vec::new();

    for arm in &arms {
        match classify_arm(&arm.body) {
            ArmClass::Compliant => compliant += 1,
            ArmClass::Exempt => exempt += 1,
            ArmClass::Violation => violations.push(arm),
        }
    }

    if !violations.is_empty() {
        let listed: String = violations
            .iter()
            .map(|a| format!("  {}", a.keywords))
            .collect::<Vec<_>>()
            .join("\n");
        panic!(
            "substrate-symmetry: {} of {} dispatch arms call into eval_* \
             without threading `list_span`.\n\
             \n\
             Stone 233.2.d's task: thread `list_span` uniformly across the \
             dispatch table per canonical template.\n\
             \n\
             Counts: {} compliant; {} exempt (no eval_* call); {} violations.\n\
             \n\
             Violations (arm keywords):\n{}",
            violations.len(),
            arms.len(),
            compliant,
            exempt,
            violations.len(),
            listed
        );
    }
}

enum ArmClass {
    Compliant,
    Exempt,
    Violation,
}

fn classify_arm(body: &str) -> ArmClass {
    let calls_eval = Regex::new(r"\beval_[a-zA-Z0-9_]+\b").unwrap();
    let has_list_span = Regex::new(r"\blist_span\b").unwrap();

    if !calls_eval.is_match(body) {
        return ArmClass::Exempt;
    }
    if has_list_span.is_match(body) {
        ArmClass::Compliant
    } else {
        ArmClass::Violation
    }
}

fn extract_dispatch_match_body(src: &str) -> String {
    let fn_start = src.find(DISPATCH_FN_SIGNATURE).unwrap_or_else(|| {
        panic!("could not find `{DISPATCH_FN_SIGNATURE}` in {RUNTIME_PATH}")
    });

    let match_offset = src[fn_start..].find(MATCH_HEADER).unwrap_or_else(|| {
        panic!("could not find `{MATCH_HEADER}` after dispatch_keyword_head signature")
    });
    let body_start = fn_start + match_offset + MATCH_HEADER.len();

    let bytes = src.as_bytes();
    let mut depth: i32 = 1;
    let mut i = body_start;
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while i < bytes.len() {
        let c = bytes[i];
        let next = bytes.get(i + 1).copied();

        if in_line_comment {
            if c == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if c == b'*' && next == Some(b'/') {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_string {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if c == b'/' && next == Some(b'/') {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if c == b'/' && next == Some(b'*') {
            in_block_comment = true;
            i += 2;
            continue;
        }
        if c == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return src[body_start..i].to_string();
            }
        }
        i += 1;
    }
    panic!("unterminated match body in dispatch_keyword_head");
}

fn parse_arms(body: &str) -> Vec<Arm> {
    // Arm header at start of a line (after whitespace): one or more
    // `"..."` literals joined by `|`, followed by `=>`.
    let header_re = Regex::new(
        r#"(?m)^[\t ]*("[^"]+"(?:[\t ]*\|[\t ]*"[^"]+")*)[\t ]*=>"#,
    )
    .unwrap();

    let mut arms: Vec<Arm> = Vec::new();
    let mut pending: Option<(usize, String)> = None;

    for caps in header_re.captures_iter(body) {
        let mat = caps.get(0).unwrap();
        let arm_header_start = mat.start();
        let arm_body_start = mat.end();
        let keywords = caps.get(1).unwrap().as_str().to_string();

        if let Some((prev_body_start, prev_keywords)) = pending.take() {
            arms.push(Arm {
                keywords: prev_keywords,
                body: body[prev_body_start..arm_header_start].to_string(),
            });
        }
        pending = Some((arm_body_start, keywords));
    }

    if let Some((last_body_start, last_keywords)) = pending {
        arms.push(Arm {
            keywords: last_keywords,
            body: body[last_body_start..].to_string(),
        });
    }

    arms
}

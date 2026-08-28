//! THE INLINED-EDN LINT — bans EDN-esque string literals inlined in `.rs` source; the exact
//! parallel of `no_inlined_wat_in_tests.rs`, for EDN instead of wat source.
//!
//! Builder-directed ("if a string looks like edn at the start, it must be edn — no exceptions"):
//! EDN belongs in a co-located `.edn` file, loaded via `include_str!` and compared
//! STRUCTURALLY via `wat::assert_edn_eq!` — never as an inline string literal that gets
//! string-compared (the `no_loose_string_assert` failure mode, one layer up: even an exact
//! `assert_eq!` against an inlined EDN literal is still wrong, because it hides the golden inside
//! the driver instead of beside it — see docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden').
//!
//! ## Detection: a syntactic heuristic, deliberately dumb
//!
//! A string literal's content, after trimming leading whitespace, is EDN-esque iff it starts with
//! one of `#` `{` `[` `(` — the four openers EDN's own grammar allows at top level (tagged
//! literal, map, vector, list). This is a HEURISTIC, not a parse: it does not confirm the content
//! actually parses as EDN (unlike this repo's `is_inline_wat_form`, which force-feeds the literal
//! to wat's own reader). That asymmetry is intentional and builder-directed — a string that merely
//! LOOKS EDN-esque at the front is already a smell (real prose does not open with a bare `{`), and
//! the fix for a genuine false positive is to restructure the CODE so the compared literal doesn't
//! open that way (e.g. don't inline a brace-first fragment as a bare assert target), not to carve
//! an exemption. See the rune section below for how hard that bar is.
//!
//! Covers `"..."`, `r"..."`, `r#"..."#` (any hash count), and `format!("...")` literals — the
//! extractor pulls every string literal's content regardless of which macro/call wraps it, so
//! `format!("{...}", x)` is caught the same as a bare `let s = "{...}";`.
//!
//! ## Two match-condition tightenings — not runes (first-run finding, 1653 raw hits)
//!
//! The bare 4-char heuristic above, run once over the whole tree, screamed 1653 sites. Sampling
//! showed two large classes that can NEVER be a real inlined-EDN violation, structurally, no
//! matter how loose the bar: these are tightened OUT of the detector itself (`is_lone_delimiter`,
//! `is_bare_format_scaffold`) rather than left to scream and rune'd away one by one — the "tweak
//! your match conditions" instruction applies to the LINT's own conditions here, same as it does
//! to a call site.
//!
//! - **Lone delimiter** — the ENTIRE trimmed content is one bare `#`/`{`/`[`/`(` with no matching
//!   close (`crates/wat-edn/src/json.rs` and friends push output one syntax character at a time).
//!   No EDN reader can complete such a form — it hits EOF mid-token — so it is provably not EDN,
//!   full stop, not merely "probably fine."
//! - **Format-scaffold opener** — the EDN-esque opener is really a Rust `format!` Display slot.
//!   Strip the leading whitespace-free `{…}` placeholders (`replace_format_placeholders`); if the
//!   content no longer opens EDN-esque, the `#`/`{`/`[`/`(` was scaffold: `"{}"`, `"{:?}"`,
//!   `format!("{}: expected {}", …)`, `panic!("{}\n---\n{:?}", …)`, `format!("{}/{}", …)`. The tell
//!   that separates a placeholder from a real EDN map is internal whitespace — a format spec is one
//!   unspaced token, whereas an EDN map is a spaced `:key val …` sequence — so `format!("{{:id {}
//!   :name \"item{}\"}}", …)` (the `{:id …}` map has a space, preserved) and a flat `{:a 1}` golden
//!   both survive and still scream. (Supersedes the old bare-scaffold check, which used the loose
//!   strip and would have swallowed a whitespace-free flat EDN map.)
//!
//! Both are proven exclusions (derivable from the literal's own shape, no judgment call), which is
//! why they live in the detector and not as 300+ individual runes. Everything else stays under the
//! original dumb heuristic — a `(`/`{`/`[`/`#`-opening literal with any real content past that is
//! still guilty until restructured.
//!
//! ## The wat/EDN boundary — why `(` needs a second check and `#`/`{`/`[` never do
//!
//! `no_inlined_wat_in_tests.rs` claims a `(`-led literal as ITS offender when wat's own reader
//! (`wat::parser::parse_one_with_file`) parses the literal whole to a `WatAST::List` headed by a
//! `Keyword` or `Symbol` — i.e. the literal is a real `(head ...)` call form. This lint must never
//! double-claim that same literal, so for a `(`-led literal specifically, this lint re-derives that
//! same reader verdict (`is_inline_wat_form`, copied verbatim from the sibling lint — these two
//! files intentionally do not share a module, matching every other file in `tests/lint/`) and SKIPS
//! it when true. Everything else that opens with `(` — reader-rejected, or reader-accepted to some
//! non-List/non-Keyword-or-Symbol-head shape — falls to this lint, exactly like every `#`/`{`/`[`
//! literal always does.
//!
//! `#`, `{`, and `[` need no such check: grounded in `crates/wat-reader/src/lexer.rs`, wat's own
//! reader only turns a *parenthesized* form into the `WatAST::List` shape `is_inline_wat_form`
//! looks for. `[...]` lexes to `WatAST::Vector`, `{...}` to `WatAST::Map` (`#{...}` to a set
//! variant), and bare `#` has exactly two special forms (`#holon...`, `#{...}`) — none of those
//! shapes is a `List`, so `is_inline_wat_form` structurally can never fire on them. The two lints
//! only ever compete on `(`, and only this lint's `(` branch needs the extra reader call to stay
//! complementary.
//!
//! It scans root `tests/` ONLY (the exact scan set of `no_inlined_wat_in_tests` — an inlined-EDN
//! golden is a TEST artifact; `src/`/`crates/*/src/` EDN is edn-WRITING machinery, and
//! `crates/*/tests/` inline EDN as input-under-test — none is a golden; see the scan-scope note on
//! the test fn) and FAILS listing every offending `file:line` — the campaign's progress meter, same
//! shape as `no_loose_string_assert` and `no_inlined_wat_in_tests`.
//! This is an EXPECTED-RED test until a follow-up fleet drives it to zero (`.edn`-file each real
//! offender, restructure each false positive); nextest isolates it, so a SECOND red is a real
//! regression once this settles.
//!
//! ## Rune: an EXTREMELY HARD bar (builder-directed)
//!
//! `// rune:lint(no-inlined-edn) — <reason>` exists, PER-OFFENSE (not file-wide): the marker on the
//! offending line (trailing, for a single-line literal) or the line directly above it (for a
//! multi-line raw-string literal) exempts THAT site only — mirroring `no_loose_string_assert`'s
//! per-site marker, NOT `no_inlined_wat`'s whole-file skip. File-wide would be actively wrong here:
//! a golden and a genuinely-inline parser-input literal routinely share one file, so a blanket
//! file rune would silently suppress a real golden while exempting the input — the opaque-blanket
//! failure. Per-offense keeps each exemption visible on its own line and lets excusare audit it
//! there. But the builder was explicit: "non-edn things must tweak their match conditions — any
//! attempted runes here must meet an extremely hard bar."
//! A literal that merely LOOKS EDN-esque but is genuinely not EDN is NOT a rune candidate — that is
//! a lint false positive to fix by restructuring the CODE (reshape the literal so its trimmed
//! content doesn't open with `#`/`{`/`[`/`(`), never by writing a reason and moving on. The rune is
//! reserved for a literal that IS genuinely EDN yet legitimately cannot live in a file — e.g. a
//! reader/parser unit test whose raw literal IS the input under test (the exact carve-out
//! `no_inlined_wat` already grants for the same reason). Justify hard; this bar stays stern.
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` = the project
//! lint suite, NOT a grimoire spell); excusare audits the reason so "legitimate" stays honest.

// rune:lint(no-inlined-wat) — this detector's doc + unit tests carry inline wat FORMS as the input
// under test (e.g. `is_inline_wat_form("(:wat::core::+ 2 3)")`, `is_edn_esque("(:wat::core::+ 2 3)")`)
// to exercise the wat/EDN complementarity boundary — the exact parser-test carve-out `no_inlined_wat`
// grants for a literal that IS the reader input, not a fixture world.
use std::path::{Path, PathBuf};

use wat::parser::parse_one_with_file;
use wat::WatAST;

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".claude" {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// `format!`-template placeholders (`{ns}`, `{}`, `{fire_fn}`, …) aren't wat syntax — substitute
/// each non-nested `{…}` span with a bare placeholder symbol so the template's *shape* still parses
/// as wat. Copied verbatim from `no_inlined_wat_in_tests.rs` (self-contained per-file, like every
/// other `tests/lint/` detector).
fn replace_placeholders(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        if chars[i] == '{' {
            if let Some(rel_close) = chars[i..].iter().position(|&c| c == '}') {
                let close = i + rel_close;
                let inner_has_brace = chars[i + 1..close].contains(&'{');
                if !inner_has_brace {
                    out.push_str("__ph__");
                    i = close + 1;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// The wat-domain check for a `(`-led literal, verbatim-copied contract from
/// `no_inlined_wat_in_tests.rs::is_inline_wat_form`: true iff wat's own reader parses the literal
/// whole to a `WatAST::List` headed by a `Keyword` or `Symbol`. Used ONLY to exclude `(`-led
/// literals that are `no_inlined_wat`'s offenders, not ours.
fn is_inline_wat_form(literal_content: &str) -> bool {
    let src = replace_placeholders(literal_content);
    let result = std::panic::catch_unwind(|| parse_one_with_file(&src, "<inline-edn-lint>"));
    matches!(
        result,
        Ok(Ok(WatAST::List(items, _)))
            if matches!(items.first(), Some(WatAST::Keyword(..)) | Some(WatAST::Symbol(..)))
    )
}

/// A single decoded escape from a Rust string literal, plus how many source chars it consumed.
/// Copied verbatim from `no_inlined_wat_in_tests.rs`.
fn decode_escape(chars: &[char], backslash_at: usize) -> (Option<char>, usize) {
    let n = chars.len();
    let Some(&kind) = chars.get(backslash_at + 1) else {
        return (None, 1);
    };
    match kind {
        'n' => (Some('\n'), 2),
        't' => (Some('\t'), 2),
        'r' => (Some('\r'), 2),
        '\\' => (Some('\\'), 2),
        '\'' => (Some('\''), 2),
        '"' => (Some('"'), 2),
        '0' => (Some('\0'), 2),
        'x' => {
            let start = backslash_at + 2;
            let mut end = start;
            while end < n && end < start + 2 && chars[end].is_ascii_hexdigit() {
                end += 1;
            }
            let hex: String = chars[start..end].iter().collect();
            let ch = u8::from_str_radix(&hex, 16).ok().map(|v| v as char);
            (ch, end - backslash_at)
        }
        'u' => {
            if chars.get(backslash_at + 2) != Some(&'{') {
                return (None, 2);
            }
            let start = backslash_at + 3;
            let mut end = start;
            while end < n && chars[end] != '}' {
                end += 1;
            }
            let hex: String = chars[start..end].iter().collect();
            let consumed = if end < n { end + 1 - backslash_at } else { end - backslash_at };
            let ch = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
            (ch, consumed)
        }
        '\n' => {
            let mut end = backslash_at + 2;
            while end < n && (chars[end] == ' ' || chars[end] == '\t' || chars[end] == '\n' || chars[end] == '\r') {
                end += 1;
            }
            (None, end - backslash_at)
        }
        other => (Some(other), 2),
    }
}

/// Extract every string literal's CONTENT plus the 1-based source line its opening quote starts
/// on — the `file:line` this lint reports. Same walk as `no_inlined_wat_in_tests.rs`'s extractor
/// (comment-skipping, raw-string hash-matching, char-literal-vs-lifetime disambiguation), plus line
/// tracking so violations can be reported per-site like `no_loose_string_assert`, not just per-file
/// like `no_inlined_wat`.
fn extract_string_literals(src: &str) -> Vec<(String, usize)> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 1usize;

    while i < n {
        let c = chars[i];

        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }

        // `//` line comment.
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // `/* … */` block comment — nests.
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            let mut depth = 1usize;
            while i < n && depth > 0 {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
            }
            continue;
        }

        // Raw string: `r"…"`, `r#"…"#`, `r##"…"##`, … — the head-count of `#` must match on close.
        if c == 'r' || c == 'R' {
            let mut j = i + 1;
            let mut hashes = 0usize;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                let lit_line = line;
                let content_start = j + 1;
                let mut k = content_start;
                let mut closed_at = None;
                while k < n {
                    if chars[k] == '"' {
                        let mut m = k + 1;
                        let mut h = 0usize;
                        while h < hashes && chars.get(m) == Some(&'#') {
                            h += 1;
                            m += 1;
                        }
                        if h == hashes {
                            closed_at = Some((k, m));
                            break;
                        }
                    }
                    k += 1;
                }
                match closed_at {
                    Some((close_start, resume)) => {
                        out.push((chars[content_start..close_start].iter().collect(), lit_line));
                        line += chars[i..resume].iter().filter(|&&c| c == '\n').count();
                        i = resume;
                    }
                    None => {
                        i = n;
                    }
                }
                continue;
            }
        }

        // Char literal: `'x'`, `'\n'`, `'"'`, `'\''`, `'\u{2764}'`, … Distinguished from a bare
        // lifetime by actually closing with a matching `'`.
        if c == '\'' {
            if chars.get(i + 1) == Some(&'\\') {
                let (_, consumed) = decode_escape(&chars, i + 1);
                let after = i + 1 + consumed;
                if chars.get(after) == Some(&'\'') {
                    line += chars[i..after + 1].iter().filter(|&&c| c == '\n').count();
                    i = after + 1;
                    continue;
                }
            } else if chars.get(i + 2) == Some(&'\'') {
                i += 3;
                continue;
            }
            i += 1;
            continue;
        }

        // Regular string literal.
        if c == '"' {
            let lit_line = line;
            i += 1;
            let mut content = String::new();
            while i < n {
                let cc = chars[i];
                if cc == '"' {
                    i += 1;
                    break;
                }
                if cc == '\\' {
                    let (decoded, consumed) = decode_escape(&chars, i);
                    if let Some(ch) = decoded {
                        content.push(ch);
                    }
                    line += chars[i..(i + consumed).min(n)].iter().filter(|&&c| c == '\n').count();
                    i += consumed;
                    continue;
                }
                if cc == '\n' {
                    line += 1;
                }
                content.push(cc);
                i += 1;
            }
            out.push((content, lit_line));
            continue;
        }

        i += 1;
    }

    out
}

/// Is this literal's content EDN-esque per the builder's heuristic: after trimming leading
/// whitespace, does it open with `#`, `{`, `[`, or `(`?
///
/// Tightening #5 (a match-condition fix, not a rune): a `#` immediately followed by an ASCII
/// digit is NOT EDN-esque. A tagged element is `#` followed by a SYMBOL (`#uuid …`,
/// `#wat.core/Span {…}`, `#_` discard, `#{` set) — and EDN's own grammar says a symbol may not
/// begin with a digit (a leading digit reads as the start of a NUMBER, not a symbol). So `#0`,
/// `#1`, `#2`, … can never open a valid tagged element; it is structurally a Rust-side
/// `"#{index}"` render (e.g. a `CheckErrorKind::TypeMismatch.param` string, `"#2"`), never EDN.
/// Structurally can never be a violation, so exclude it at the detector, not the site — same
/// discipline Tightening #1 already applies to the bare lone-delimiter case just below. The bare
/// `"#"` case is UNCHANGED by this: it has no second char (`chars().nth(1)` is `None`), so it
/// still opens EDN-esque here and still reaches `is_lone_delimiter`'s own exclusion downstream.
fn is_edn_esque(content: &str) -> bool {
    let trimmed = content.trim_start();
    match trimmed.chars().next() {
        Some('#') => !matches!(trimmed.chars().nth(1), Some(c) if c.is_ascii_digit()),
        Some('{') | Some('[') | Some('(') => true,
        _ => false,
    }
}

/// Tightening #1 (a match-condition fix, not a rune): the WHOLE trimmed content is a single bare
/// opener with no matching close at all — `"#"`, `"{"`, `"["`, `"("`. None of those four is
/// COMPLETE EDN by any reading (an EDN reader hits EOF mid-form on every one); it is a lone
/// syntax/delimiter character a hand-rolled writer pushes one token at a time (grep the offender
/// list pre-filter: `crates/wat-edn/src/json.rs` et al. build JSON/EDN text char-by-char this
/// way). Structurally can never be a violation, so exclude it at the detector, not the site.
fn is_lone_delimiter(content: &str) -> bool {
    matches!(content.trim(), "#" | "{" | "[" | "(")
}

/// Strip only the `{…}` spans that are genuine Rust `format!`/`println!`/`write!` placeholders,
/// leaving EDN maps intact. The one structural tell a format placeholder can NEVER have is internal
/// whitespace: a format spec is a single unspaced token (`{}`, `{:?}`, `{name}`, `{:>10}`, `{0}`),
/// whereas an EDN map's content is a whitespace-separated `:key val …` sequence (`{:a 1}` has a
/// space). So substitute a non-nested `{…}` span iff its inner content contains no ASCII whitespace;
/// an EDN map like `{:a 1}` (space inside) is left untouched. (Distinct from `replace_placeholders`,
/// which — for the wat-domain check — strips *every* non-nested `{…}` regardless; that looser strip
/// is right for wat templates but would swallow a whitespace-free EDN map, which cannot occur, and
/// misjudge a spaced one, which can.)
fn replace_format_placeholders(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        if chars[i] == '{' {
            if let Some(rel_close) = chars[i..].iter().position(|&c| c == '}') {
                let close = i + rel_close;
                let inner = &chars[i + 1..close];
                let is_placeholder = !inner.contains(&'{') && !inner.iter().any(|c| c.is_whitespace());
                if is_placeholder {
                    out.push_str("__ph__");
                    i = close + 1;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Tightening #2 (a match-condition fix, not a rune): a literal whose EDN-esque OPENER is really a
/// Rust `format!` Display slot, not EDN syntax. After stripping the leading Rust placeholders
/// (`replace_format_placeholders` — whitespace-free `{…}` only), if the content no longer opens with
/// an EDN opener, then its `#`/`{`/`[`/`(` was format scaffold, never EDN: `"{}"`, `"{:?}"`, `"{e}"`,
/// `format!("{}: expected {}", …)`, `panic!("{}\n---\n{:?}", …)`, `format!("{}/{}", …)` all reduce to
/// non-EDN openers (`""`, `": expected "`, `"\n---\n"`, `"/"`). A literal that STILL opens EDN-esque
/// after the strip stays a violation — a `#wat.*`-tagged golden (leading `#`, untouched), or an EDN
/// record built via format args, e.g. `format!("{{:id {} :name \"item{}\"}}", i, i)` whose `{:id …}`
/// map has internal whitespace and is preserved, still screams. (This subsumes and supersedes the
/// old bare-scaffold check: the "nothing survives" case — `"{}"`, `"{:?}"` — is just the special
/// case where the stripped opener is empty; and unlike the old check it no longer misjudges a flat
/// whitespace-free EDN map, which the loose strip would have swallowed.)
fn opener_is_format_scaffold(content: &str) -> bool {
    !is_edn_esque(&replace_format_placeholders(content))
}

/// Tightening #3 (a match-condition fix, not a rune): a human MESSAGE that quotes an EDN/wat form
/// and then explains it in English — `assert_eq!(n, 3, "[1 2 3] must have length 3")`,
/// `"(:foo r) must type-check without error; got: {}"`. The essential golden-vs-message distinction:
/// a golden (or a genuine EDN input literal) is EXACTLY ONE clean EDN form with NOTHING after it,
/// whereas a message is a form followed by prose. So for a `(`/`[`/`{`-opener, scan the first
/// balanced top-level form (bracket depth over `()`/`[]`/`{}`, ignoring brackets inside `"…"`
/// strings so a quoted `)` never closes it); if non-whitespace trails the close, it is a message,
/// not a golden. A clean single form (`[1 2 3]`, `{:a 1}`, `#wat…` — `#` never opens this scan) has
/// nothing trailing and stays a violation (golden → `.edn`; genuine input → per-offense rune). An
/// unclosed opener (`"[typo, distance"`, a search pattern) never balances → not this class (handled
/// by role below).
fn is_form_then_trailing(content: &str) -> bool {
    let chars: Vec<char> = content.trim_start().chars().collect();
    let n = chars.len();
    if !matches!(chars.first(), Some('(') | Some('[') | Some('{')) {
        return false;
    }
    let mut depth = 0i32;
    let mut in_string = false;
    let mut i = 0;
    while i < n {
        let c = chars[i];
        if in_string {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return chars[i + 1..].iter().any(|c| !c.is_whitespace());
                }
            }
            _ => {}
        }
        i += 1;
    }
    false // opener never balanced — not form-then-trailing
}

/// Tightening #4 (a match-condition fix, not a rune): the literal sits in a pure OUTPUT or SEARCH
/// position — the format string of `eprintln!`/`println!`/`print!`/`eprint!`/`write!`/`writeln!`/
/// `panic!`, or a `.matches`/`.split`/`.replace` argument — where a golden never lives. Catches the
/// message/pattern literals that `is_form_then_trailing` misses because they never balance
/// (`eprintln!("  [{}] {:?}", …)` leaves `[]` after placeholder-strip; `.matches("[typo, distance")`
/// is an unclosed search pattern). `assert!`/`assert_eq!` are deliberately NOT markers here — a
/// golden lives in an assert's VALUE position, and its message-then-prose case is already the
/// `is_form_then_trailing` class. Checked on the literal's own source line (`no_loose_string_assert`
/// scans the same way).
fn is_output_or_search_position(line: &str) -> bool {
    const MARKERS: [&str; 13] = [
        "eprintln!(",
        "println!(",
        "eprint!(",
        "print!(",
        "writeln!(",
        "write!(",
        "panic!(",
        ".matches(",
        ".split(",
        ".replace(",
        // Substring-search predicates: a literal fed to these is a search PATTERN over rendered
        // text (an error-message fragment, a tag prefix), never a golden. A real golden compared
        // via `.contains` would already be a `no_loose_string_assert` violation, so claiming these
        // here can't hide a golden.
        ".contains(",
        ".starts_with(",
        ".ends_with(",
    ];
    MARKERS.iter().any(|m| line.contains(m))
}

#[cfg(test)]
mod detector_tests {
    use super::*;

    #[test]
    fn hash_opener_is_edn_esque() {
        assert!(is_edn_esque("#uuid \"00000000-0000-0000-0000-000000000000\""));
    }

    #[test]
    fn hash_digit_opener_is_not_edn_esque() {
        // Tightening #5: `#` + ASCII digit can never open a tagged element (EDN symbols may not
        // begin with a digit) — it is a Rust-side `"#{index}"` render (a `TypeMismatch.param`
        // string like `"#2"`), never EDN. Both directions, so this proves the tightening stopped
        // flagging the digit case WITHOUT weakening the genuine-EDN cases right below it.
        assert!(!is_edn_esque("#2"));
        assert!(!is_edn_esque("#1"));
        assert!(!is_edn_esque("#0 rest"));
        // Genuine EDN openers must still flag — the tightening is scoped to `#`+digit only.
        assert!(is_edn_esque("#uuid \"00000000-0000-0000-0000-000000000000\""));
        assert!(is_edn_esque("#{1 2 3}"));
        assert!(is_edn_esque("#wat.core/Span {:line 1}"));
        // The bare lone `#` is unaffected by this tightening — still opens EDN-esque here, still
        // excluded downstream via `is_lone_delimiter` (see `lone_delimiter_chars_are_excluded`).
        assert!(is_edn_esque("#"));
    }

    #[test]
    fn brace_opener_is_edn_esque() {
        assert!(is_edn_esque("{:namespace \"probe-ns\"}"));
    }

    #[test]
    fn bracket_opener_is_edn_esque() {
        assert!(is_edn_esque("[1 2 3]"));
    }

    #[test]
    fn leading_whitespace_is_trimmed_before_the_check() {
        assert!(is_edn_esque("\n  {:a 1}"));
    }

    #[test]
    fn ordinary_prose_is_not_edn_esque() {
        assert!(!is_edn_esque("hello world"));
        assert!(!is_edn_esque("expected i32, got String"));
    }

    #[test]
    fn wat_call_form_is_excluded_via_the_reader_not_the_heuristic() {
        // The heuristic alone WOULD flag this (it opens with `(`) — the wat-domain check is what
        // excludes it, which is exactly the boundary this lint documents.
        assert!(is_edn_esque("(:wat::core::+ 2 3)"));
        assert!(is_inline_wat_form("(:wat::core::+ 2 3)"));
    }

    #[test]
    fn lone_delimiter_chars_are_excluded() {
        assert!(is_lone_delimiter("#"));
        assert!(is_lone_delimiter("{"));
        assert!(is_lone_delimiter("["));
        assert!(is_lone_delimiter("("));
        assert!(!is_lone_delimiter("{}"));
        assert!(!is_lone_delimiter("#uuid"));
    }

    #[test]
    fn bare_format_placeholders_are_excluded_as_scaffold() {
        assert!(opener_is_format_scaffold("{}"));
        assert!(opener_is_format_scaffold("{:?}"));
        assert!(opener_is_format_scaffold("{e}"));
        assert!(opener_is_format_scaffold("{}{}"));
    }

    #[test]
    fn format_message_openers_are_excluded_as_scaffold() {
        // A `format!`/`panic!` message whose leading `{}`/`{:?}` Display slot only makes it *look*
        // EDN-esque — after stripping the whitespace-free placeholders the opener is gone.
        assert!(opener_is_format_scaffold("{}: expected {}, got {}"));
        assert!(opener_is_format_scaffold("{}\n---\n{:?}"));
        assert!(opener_is_format_scaffold("{}/{}/{}"));
        assert!(opener_is_format_scaffold("{}: tuple[{}] is {:?}"));
    }

    #[test]
    fn edn_shaped_content_survives_the_scaffold_strip() {
        // Real EDN assembled via format args — the `{:id …}` map has internal whitespace so it is
        // NOT a placeholder and is preserved; the leading `{` survives, so it must NOT be excluded.
        assert!(!opener_is_format_scaffold("{:id {} :name \"item{}\" :price {}.5}"));
    }

    #[test]
    fn flat_edn_map_golden_survives_the_scaffold_strip() {
        // A whitespace-free `{…}` is a placeholder, but a real EDN map ALWAYS has internal
        // whitespace (`:key val`), so `{:a 1}` is preserved (fixes the loose strip's false-negative
        // that would have swallowed a flat map with no format args).
        assert!(!opener_is_format_scaffold("{:a 1 :b 2}"));
        assert!(!opener_is_format_scaffold("#wat.check/CheckErrors {:message \"1 error\"}"));
    }

    #[test]
    fn form_then_english_prose_is_a_message_not_a_golden() {
        assert!(is_form_then_trailing("[1 2 3] must have length 3"));
        assert!(is_form_then_trailing("(:wat::io::IOReader/read-frame r) must type-check; got: {}"));
        assert!(is_form_then_trailing("(:wat::core::Vector :wat::type::Infer 1 2 3) empty case"));
        assert!(is_form_then_trailing("[{}] {:?}")); // eprintln fmt: [{}] balances, {:?} trails
    }

    #[test]
    fn a_clean_single_form_is_not_form_then_trailing() {
        // A golden or a genuine EDN input literal — one clean form, nothing trailing.
        assert!(!is_form_then_trailing("[1 2 3]"));
        assert!(!is_form_then_trailing("(1 2 3)"));
        assert!(!is_form_then_trailing("{:message \"1 type-check error\" :causes [1 2]}"));
        // a quoted `)` inside the form must not be read as the close:
        assert!(!is_form_then_trailing("(:foo \"a)b\" :bar 1)"));
        // `#`-openers are not this class (goldens; handled as tagged EDN), and an unclosed opener
        // (a search pattern) never balances:
        assert!(!is_form_then_trailing("#wat.check/CheckErrors {:a 1}"));
        assert!(!is_form_then_trailing("[typo, distance"));
    }

    #[test]
    fn output_and_search_positions_are_recognized() {
        assert!(is_output_or_search_position("            eprintln!(\"  [{}] {:?}\", i, line);"));
        assert!(is_output_or_search_position("    let c = msg.matches(\"[typo, distance\").count();"));
        assert!(is_output_or_search_position("    panic!(\"[{}] boom\", n);"));
        // an assert holding a golden is NOT an output/search position:
        assert!(!is_output_or_search_position("assert_eq!(actual, r#\"#wat.x/Y {:a 1}\"#);"));
    }

    #[test]
    fn edn_list_form_is_not_wat() {
        assert!(is_edn_esque("(1 2 3)"));
        assert!(!is_inline_wat_form("(1 2 3)"));
    }

    #[test]
    fn extractor_tracks_line_numbers() {
        let src = "let a = \"one\";\nlet b = \"two\";\n";
        let lits = extract_string_literals(src);
        assert_eq!(lits, vec![("one".to_string(), 1), ("two".to_string(), 2)]);
    }

    #[test]
    fn extractor_skips_comments() {
        let src = "// {not a literal}\nlet x = \"{real}\";\n";
        let lits = extract_string_literals(src);
        assert_eq!(lits, vec![("{real}".to_string(), 2)]);
    }
}

#[test]
fn tests_carry_no_inlined_edn() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    // Scope: root `tests/` ONLY — the exact scan set of the sibling `no_inlined_wat_in_tests.rs`.
    // An inlined-EDN GOLDEN (a value compared inline that should live in a co-located `.edn` file)
    // is a TEST artifact by nature; it only ever occurs here. `src/` and `crates/*/src/` EDN-esque
    // literals are the edn-WRITING machinery, not goldens — Display impls, the `wat-edn` JSON/EDN
    // writer emitting tag strings (`"#bigint"`, `"#float"`), the derive codegen, FQDN glue
    // (`format!("{}::{}")`), `#N` positional markers (`"#1"`) — none has a `.edn` file to move to
    // (its job is to EMIT edn). And `crates/*/tests/` (esp. `crates/wat-edn/tests/*`) inline EDN as
    // the INPUT UNDER TEST — the reader/writer's own corpus — the parser-test carve-out this scope
    // grants structurally (they are simply not scanned), the exact carve-out `no_inlined_wat`
    // reserves for the same reason. So the one honest scan is root `tests/`.
    let mut files = Vec::new();
    collect_rs(&Path::new(manifest).join("tests"), &mut files);
    files.sort();

    let mut violations = Vec::new();
    let mut hash_hits = 0usize;
    let mut brace_hits = 0usize;
    let mut bracket_hits = 0usize;
    let mut paren_hits = 0usize;
    let mut wat_excluded = 0usize;
    let mut lone_delimiter_excluded = 0usize;
    let mut format_scaffold_excluded = 0usize;
    let mut form_trailing_excluded = 0usize;
    let mut output_role_excluded = 0usize;
    let mut runed_excluded = 0usize;

    // Scanning force-feeds many `(`-led literals to wat's reader to decide the wat/EDN boundary;
    // `is_inline_wat_form` catch_unwinds the rare lexer panic on pathological input, but the
    // default hook still prints each one — silence it for the scan, then restore it.
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    for f in &files {
        // This file names the detector in its own doc/tests — skip self.
        if f.file_name().and_then(|n| n.to_str()) == Some("no_inlined_edn.rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let lines: Vec<&str> = src.lines().collect();
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();

        for (content, line) in extract_string_literals(&src) {
            if !is_edn_esque(&content) {
                continue;
            }
            let trimmed = content.trim_start();
            if trimmed.starts_with('(') && is_inline_wat_form(&content) {
                // no_inlined_wat's offender, not ours — a real `(head ...)` wat call form.
                wat_excluded += 1;
                continue;
            }
            if is_lone_delimiter(&content) {
                // A single bare `#`/`{`/`[`/`(` with no matching close — can never be complete
                // EDN; a hand-rolled writer's syntax-character constant.
                lone_delimiter_excluded += 1;
                continue;
            }
            if opener_is_format_scaffold(&content) {
                // The EDN-esque opener is a Rust `format!` Display slot (`{}`/`{:?}`/`{name}`),
                // not EDN's map opener — strip the whitespace-free `{…}` placeholders and the
                // leading `#`/`{`/`[`/`(` is gone (a `format!`/`panic!` message, not a golden).
                format_scaffold_excluded += 1;
                continue;
            }
            if is_form_then_trailing(&content) {
                // A form quoted then explained in English prose (an assert/panic message), not a
                // clean single golden — a golden has nothing after the balanced form.
                form_trailing_excluded += 1;
                continue;
            }
            if lines.get(line - 1).is_some_and(|l| is_output_or_search_position(l)) {
                // A pure output (`eprintln!`/`println!`/`panic!`/`write!`) format string or a
                // `.matches`/`.split`/`.replace` search argument — never a golden.
                output_role_excluded += 1;
                continue;
            }
            // Per-OFFENSE rune (not file-wide): a `// rune:lint(no-inlined-edn) — <reason>` on the
            // offending line (trailing, for a single-line literal) OR the line directly above it
            // (for a multi-line raw-string literal) exempts THIS site only — so runing a genuine
            // parser-input literal never silently suppresses a co-located golden. Mirrors
            // `no_loose_string_assert`'s per-site marker; excusare audits each reason on its line.
            let runed = lines
                .get(line - 1)
                .is_some_and(|l| l.contains("// rune:lint(no-inlined-edn)"))
                || (line >= 2
                    && lines
                        .get(line - 2)
                        .is_some_and(|l| l.contains("// rune:lint(no-inlined-edn)")));
            if runed {
                runed_excluded += 1;
                continue;
            }
            match trimmed.chars().next() {
                Some('#') => hash_hits += 1,
                Some('{') => brace_hits += 1,
                Some('[') => bracket_hits += 1,
                Some('(') => paren_hits += 1,
                _ => unreachable!("is_edn_esque only admits #/{{/[/( openers"),
            }
            violations.push(format!("{}:{}", rel, line));
        }
    }

    std::panic::set_hook(prev_hook);

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 INLINED-EDN — {} site(s) carry a string literal whose content opens with\n\
         `#`/`{{`/`[`/`(` (EDN-esque, trimmed of leading whitespace).\n\
         \n\
         THE FIX (RUBRIC: docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden'): move the EDN\n\
         into a co-located, PRETTY-PRINTED `<probe>__<label>.edn` file (multi-line, indented — see\n\
         tests/services/probe_arc278_journal_service_logs__stored_log.edn), load it via\n\
         `include_str!(\"...edn\")`, and compare STRUCTURALLY via\n\
         `wat::assert_edn_eq!(actual_edn, include_str!(\"...edn\"))` — never a string-literal compare.\n\
         A literal that merely LOOKS EDN-esque but is genuinely not EDN is NOT a rune candidate —\n\
         restructure the CODE so the compared literal's trimmed content doesn't open that way.\n\
         `// rune:lint(no-inlined-edn) — <reason>` (PER-OFFENSE: on the offending line or the one\n\
         above it — NOT file-wide, unlike no_inlined_wat) is reserved for a literal that IS\n\
         genuinely EDN yet legitimately cannot be a file (e.g. a parser/reader test whose raw\n\
         literal is the input under test) — an EXTREMELY hard bar, justify hard.\n\
         \n\
         Drive it to ZERO. Opener breakdown so far: {} `#`, {} `{{`, {} `[`, {} `(` (already\n\
         excludes `(`-openers that are real wat call forms — those are `no_inlined_wat`'s: {}\n\
         excluded that way. Detector-inferred non-EDN excluded (NOT runed): {} lone-delimiter-char,\n\
         {} format-scaffold-opener, {} form-then-trailing-prose (a message, not a golden), {}\n\
         output/search-position (eprintln!/panic!/.matches/…). Plus {} site(s) exempted by a\n\
         per-offense `// rune:lint(no-inlined-edn)` marker (genuine EDN legitimately inline).\n\
         Offenders:\n\n{}\n",
        violations.len(),
        hash_hits,
        brace_hits,
        bracket_hits,
        paren_hits,
        wat_excluded,
        lone_delimiter_excluded,
        format_scaffold_excluded,
        form_trailing_excluded,
        output_role_excluded,
        runed_excluded,
        violations.join("\n"),
    );
}

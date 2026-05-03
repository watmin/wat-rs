//! Deftest discovery — paren-balanced scanner that finds the four
//! deftest-producing shapes in `.wat` source:
//!
//! 1. `(:wat::test::deftest <name> ...)` — direct, in-process (arc 121)
//! 2. `(:wat::test::deftest-hermetic <name> ...)` — direct, forked
//!    subprocess (arc 124)
//! 3. `(:alias <name> <body>)` — alias call where `:alias` was
//!    declared via `(:wat::test::make-deftest :alias <prelude>)`
//!    (arc 124)
//! 4. `(:alias <name> <body>)` — alias call where `:alias` was
//!    declared via `(:wat::test::make-deftest-hermetic :alias
//!    <prelude>)` (arc 124)
//!
//! Used by the `wat::test!` proc macro at expansion time to enumerate
//! every deftest under the configured path. The proc macro then emits
//! one `#[test] fn` per discovered site so cargo's libtest sees each
//! deftest as a first-class test.
//!
//! At the runner layer, the four shapes are indistinguishable —
//! `deftest` and `deftest-hermetic` both expand at wat-side macro
//! expansion to a `:wat::core::define` of a function returning
//! `:wat::test::TestResult`; the runner just looks up the function
//! by keyword name and calls it. Hermetic vs in-process is
//! encoded INSIDE the wat-side body (the choice between
//! `run-sandboxed-ast` and `run-sandboxed-hermetic-ast`). Same for
//! alias forms — `make-deftest` builds a defmacro that ultimately
//! expands to `deftest`, `make-deftest-hermetic` to
//! `deftest-hermetic`. The proc-macro scanner doesn't care about
//! the inner choice; it just emits the `#[test] fn`.
//!
//! This is a tiny lexer — NOT a full wat parser. Recognizing
//! `(:wat::test::deftest <name>` (or any of the other shapes) is
//! unambiguous textually:
//!
//! - paren balance (skipping comments and string literals)
//! - at depth 1, the head keyword names the discovery shape
//! - the next non-whitespace, non-comment token is the deftest's
//!   name (a keyword starting with `:`)
//!
//! Aliases are tracked per-file. The scanner walks top-to-bottom; a
//! `make-deftest` / `make-deftest-hermetic` registration must
//! precede the alias's first use (matches wat's runtime defmacro
//! ordering). Late-declared aliases silently drop — the wat-level
//! type checker surfaces the real error.
//!
//! Comments: `;` to end-of-line. Standard wat comment syntax.
//! Strings: `"..."` with `\\` and `\"` escapes.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// One discovered deftest site.
#[derive(Debug, Clone)]
pub struct DeftestSite {
    /// Absolute path of the `.wat` file the deftest was found in.
    pub file_path: PathBuf,
    /// Fully-qualified deftest keyword name, including the leading
    /// colon — e.g. `:wat-tests::holon::lru::test-foo`.
    pub name: String,
    /// Arc 138 F-NAMES-1f — 1-indexed line number where the deftest
    /// form opens in `file_path`. Threaded into the timeout panic
    /// message so test authors can navigate even when libtest's
    /// panic-header location points at `tests/test.rs`.
    pub line: usize,
    /// Arc 138 F-NAMES-1f — 1-indexed column number, paired with `line`.
    pub col: usize,
    /// Arc 122 — `(:wat::test::ignore "<reason>")` annotation
    /// preceding this deftest, if any. Causes the proc macro to
    /// emit `#[ignore = "<reason>"]` on the generated `#[test] fn`.
    pub ignore: Option<String>,
    /// Arc 122 — `(:wat::test::should-panic "<expected>")`
    /// annotation preceding this deftest, if any. Causes the proc
    /// macro to emit
    /// `#[should_panic(expected = "<expected>")]` on the
    /// generated `#[test] fn`.
    pub should_panic: Option<String>,
    /// Arc 123 — `(:wat::test::time-limit "<dur>")` annotation
    /// preceding this deftest, if any. Stores the parsed duration
    /// in milliseconds. Causes the proc macro to wrap the
    /// generated `#[test] fn`'s body in a thread-spawn +
    /// `recv_timeout`; if the budget is exceeded, the wrapper
    /// panics with a timeout message.
    pub time_limit_ms: Option<u64>,
}

/// Walk `root` (file or directory) and return every deftest site found
/// in `.wat` files under it. Recursive for directories. Sorted by
/// (file_path, name) for stable expansion order.
pub fn discover_deftests(root: &Path) -> Result<Vec<DeftestSite>, DiscoverError> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_wat_files(root, &mut files)?;
    files.sort();

    let mut sites: Vec<DeftestSite> = Vec::new();
    for file in &files {
        let src = fs::read_to_string(file)
            .map_err(|e| DiscoverError::Read(file.clone(), e.to_string()))?;
        for parsed in scan_file(&src) {
            sites.push(DeftestSite {
                file_path: file.clone(),
                name: parsed.name,
                line: parsed.line,
                col: parsed.col,
                ignore: parsed.ignore,
                should_panic: parsed.should_panic,
                time_limit_ms: parsed.time_limit_ms,
            });
        }
    }
    Ok(sites)
}

/// One deftest as parsed from a single file. `discover_deftests`
/// adds the `file_path` to produce a full `DeftestSite`.
#[derive(Debug, Clone)]
pub struct ParsedSite {
    pub name: String,
    /// Arc 138 F-NAMES-1f — 1-indexed line where the deftest opens.
    pub line: usize,
    /// Arc 138 F-NAMES-1f — 1-indexed column.
    pub col: usize,
    pub ignore: Option<String>,
    pub should_panic: Option<String>,
    pub time_limit_ms: Option<u64>,
}

/// Arc 123 — parse a `:wat::test::time-limit` duration string.
///
/// Syntax: `<digits><suffix>` where `suffix` is one of `ms`, `s`,
/// `m`. Returns the duration in **milliseconds** (the foundational
/// resolution; finer granularity is not test-scale).
///
/// Examples: `"500ms"` → `500`. `"30s"` → `30_000`. `"5m"` →
/// `300_000`.
///
/// The doctrine is ms-first (per arc 123 DESIGN.md). `s` and `m`
/// suffixes are supported but not advertised — they exist for the
/// rare exception (sandboxed integration tests, intentional sleep)
/// and don't lead the docs.
///
/// Errors: missing suffix, non-numeric prefix, unknown suffix
/// (e.g. `"500us"`, `"5min"`).
pub fn parse_duration_ms(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty duration string".into());
    }
    // Order matters — check 2-char suffixes before 1-char
    // (so "ms" is recognized before "m" or "s").
    if let Some(num_str) = s.strip_suffix("ms") {
        return num_str
            .parse::<u64>()
            .map_err(|e| format!("invalid milliseconds in {:?}: {}", s, e));
    }
    if let Some(num_str) = s.strip_suffix('s') {
        let n: u64 = num_str
            .parse()
            .map_err(|e| format!("invalid seconds in {:?}: {}", s, e))?;
        return n
            .checked_mul(1000)
            .ok_or_else(|| format!("seconds overflow in {:?}", s));
    }
    if let Some(num_str) = s.strip_suffix('m') {
        let n: u64 = num_str
            .parse()
            .map_err(|e| format!("invalid minutes in {:?}: {}", s, e))?;
        return n
            .checked_mul(60_000)
            .ok_or_else(|| format!("minutes overflow in {:?}", s));
    }
    Err(format!(
        "duration {:?} missing unit suffix; use 'ms' (preferred), 's', or 'm'",
        s
    ))
}

#[derive(Debug)]
pub enum DiscoverError {
    Read(PathBuf, String),
    Stat(PathBuf, String),
}

impl std::fmt::Display for DiscoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(p, e) => write!(f, "read {}: {}", p.display(), e),
            Self::Stat(p, e) => write!(f, "stat {}: {}", p.display(), e),
        }
    }
}

fn collect_wat_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), DiscoverError> {
    let meta = fs::metadata(root)
        .map_err(|e| DiscoverError::Stat(root.to_path_buf(), e.to_string()))?;
    if meta.is_file() {
        if root.extension().and_then(|e| e.to_str()) == Some("wat") {
            out.push(root.to_path_buf());
        }
        return Ok(());
    }
    if !meta.is_dir() {
        return Ok(());
    }
    let entries = fs::read_dir(root)
        .map_err(|e| DiscoverError::Read(root.to_path_buf(), e.to_string()))?;
    for entry in entries {
        let entry = entry
            .map_err(|e| DiscoverError::Read(root.to_path_buf(), e.to_string()))?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .map_err(|e| DiscoverError::Stat(path.clone(), e.to_string()))?;
        if ft.is_dir() {
            collect_wat_files(&path, out)?;
        } else if ft.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some("wat")
        {
            out.push(path);
        }
    }
    Ok(())
}

/// Scan one `.wat` source string for the four deftest-producing
/// shapes (per arc 121 + arc 124) and the per-test annotations that
/// may precede each (`(:wat::test::ignore "<reason>")`,
/// `(:wat::test::should-panic "<expected>")`,
/// `(:wat::test::time-limit "<dur>")`) per arcs 122 + 123.
///
/// The four shapes:
/// - `(:wat::test::deftest <name> ...)` — direct, in-process
/// - `(:wat::test::deftest-hermetic <name> ...)` — direct, forked
/// - `(:alias <name> ...)` where `(:wat::test::make-deftest :alias
///   <prelude>)` declared the alias upstream
/// - `(:alias <name> ...)` where `(:wat::test::make-deftest-hermetic
///   :alias <prelude>)` declared the alias upstream
///
/// Annotations are SIBLING forms preceding a deftest — pending state
/// attaches to the next deftest discovered (regardless of which of
/// the four shapes it takes). Encountering any non-annotation form
/// between an annotation and a deftest CLEARS the pending
/// annotations (including `make-deftest` / `make-deftest-hermetic`
/// declarations themselves — annotations only attach to the
/// immediately next deftest CALL, not to alias declarations).
///
/// Comments are skipped (`;` to end of line). String literals are
/// skipped (`"..."` with `\\` and `\"` escapes). The scanner is a
/// hand-rolled paren-balanced reader, NOT a full wat parser.
/// Arc 138 F-NAMES-1f — convert a byte offset within `src` to a
/// 1-indexed (line, col) pair. UTF-8 char-count for column matches
/// the lexer's convention. Used by [`scan_file`] to record each
/// deftest's source position for the timeout panic message.
fn byte_offset_to_line_col(src: &str, offset: usize) -> (usize, usize) {
    let off = offset.min(src.len());
    let prefix = &src[..off];
    let line = prefix.bytes().filter(|&b| b == b'\n').count() + 1;
    let last_nl = prefix.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let col = src[last_nl..off].chars().count() + 1;
    (line, col)
}

pub fn scan_file(src: &str) -> Vec<ParsedSite> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut sites: Vec<ParsedSite> = Vec::new();

    let mut pending_ignore: Option<String> = None;
    let mut pending_should_panic: Option<String> = None;
    let mut pending_time_limit_ms: Option<u64> = None;

    // Arc 124 — per-file alias table. Aliases registered by
    // `(:wat::test::make-deftest :alias ...)` or
    // `(:wat::test::make-deftest-hermetic :alias ...)` are added
    // here; subsequent top-level forms whose head keyword is in
    // the table are treated as deftest calls. Hermetic vs
    // in-process distinction is invisible at the runner layer
    // (the wat-side macro expansion handles dispatch).
    let mut aliases: HashMap<String, ()> = HashMap::new();

    while i < bytes.len() {
        let b = bytes[i];

        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }

        // Line comment — ; to end of line.
        if b == b';' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // String literal at the top level — skip (preserves
        // pending annotations; no top-level form was opened).
        if b == b'"' {
            i = skip_string(bytes, i);
            continue;
        }

        if b == b'(' {
            // Open a top-level form. Identify by head keyword.
            let after_paren = i + 1;
            let head_start = skip_ws_and_comments(bytes, after_paren);
            let head = read_keyword(bytes, head_start);
            let head_str = head
                .map(|h| std::str::from_utf8(h).unwrap_or(""))
                .unwrap_or("");

            match head_str {
                ":wat::test::ignore" => {
                    let arg_start =
                        skip_ws_and_comments(bytes, head_start + head_str.len());
                    if let Some(reason) = read_string_literal(bytes, arg_start) {
                        pending_ignore = Some(reason);
                    }
                    i = skip_form(bytes, i);
                }
                ":wat::test::should-panic" => {
                    let arg_start =
                        skip_ws_and_comments(bytes, head_start + head_str.len());
                    if let Some(expected) = read_string_literal(bytes, arg_start) {
                        pending_should_panic = Some(expected);
                    }
                    i = skip_form(bytes, i);
                }
                ":wat::test::time-limit" => {
                    let arg_start =
                        skip_ws_and_comments(bytes, head_start + head_str.len());
                    if let Some(dur_str) = read_string_literal(bytes, arg_start) {
                        if let Ok(ms) = parse_duration_ms(&dur_str) {
                            pending_time_limit_ms = Some(ms);
                        }
                        // Parse error silently dropped here — the proc
                        // macro re-parses + emits compile_error! with
                        // the message at the macro-expansion site.
                    }
                    i = skip_form(bytes, i);
                }
                ":wat::test::deftest" | ":wat::test::deftest-hermetic" => {
                    // Arc 121 + arc 124 — direct deftest forms.
                    // The wat-side `deftest-hermetic` macro expands
                    // to a `define` that calls
                    // `run-sandboxed-hermetic-ast`; the runner
                    // doesn't distinguish.
                    let name_start =
                        skip_ws_and_comments(bytes, head_start + head_str.len());
                    if let Some(name_bytes) = read_keyword(bytes, name_start) {
                        let name =
                            std::str::from_utf8(name_bytes).unwrap_or("").to_string();
                        // Arc 138 F-NAMES-1f — the `(` byte at offset `i`
                        // is the deftest form's opening paren; convert to
                        // 1-indexed (line, col).
                        let (line, col) = byte_offset_to_line_col(src, i);
                        sites.push(ParsedSite {
                            name,
                            line,
                            col,
                            ignore: pending_ignore.take(),
                            should_panic: pending_should_panic.take(),
                            time_limit_ms: pending_time_limit_ms.take(),
                        });
                    }
                    i = skip_form(bytes, i);
                }
                ":wat::test::make-deftest" | ":wat::test::make-deftest-hermetic" => {
                    // Arc 124 — register an alias keyword as a
                    // deftest-producing form for the rest of this
                    // file. The first argument after the head is
                    // the alias keyword (e.g. `:deftest-hermetic`).
                    // Annotations preceding a make-deftest call are
                    // dropped — they don't attach to the alias's
                    // declaration; an annotation must precede the
                    // alias's CALL site to attach.
                    let alias_start =
                        skip_ws_and_comments(bytes, head_start + head_str.len());
                    if let Some(alias_bytes) = read_keyword(bytes, alias_start) {
                        let alias =
                            std::str::from_utf8(alias_bytes).unwrap_or("").to_string();
                        if !alias.is_empty() {
                            aliases.insert(alias, ());
                        }
                    }
                    pending_ignore = None;
                    pending_should_panic = None;
                    pending_time_limit_ms = None;
                    i = skip_form(bytes, i);
                }
                other if !other.is_empty() && aliases.contains_key(other) => {
                    // Arc 124 — alias call. Treat as a deftest with
                    // the same shape: next keyword is the test name.
                    let name_start =
                        skip_ws_and_comments(bytes, head_start + head_str.len());
                    if let Some(name_bytes) = read_keyword(bytes, name_start) {
                        let name =
                            std::str::from_utf8(name_bytes).unwrap_or("").to_string();
                        let (line, col) = byte_offset_to_line_col(src, i);
                        sites.push(ParsedSite {
                            name,
                            line,
                            col,
                            ignore: pending_ignore.take(),
                            should_panic: pending_should_panic.take(),
                            time_limit_ms: pending_time_limit_ms.take(),
                        });
                    }
                    i = skip_form(bytes, i);
                }
                _ => {
                    // Any other top-level form clears pending
                    // annotations. An annotation only attaches to the
                    // immediately next deftest.
                    pending_ignore = None;
                    pending_should_panic = None;
                    pending_time_limit_ms = None;
                    i = skip_form(bytes, i);
                }
            }
            continue;
        }

        // Stray byte at top level — advance.
        i += 1;
    }

    sites
}

/// Read a `:keyword` starting at `pos`. Returns the keyword's byte
/// length (including the leading `:`) or `None` if the byte at
/// `pos` is not the start of a keyword.
///
/// A wat keyword: `:` followed by one or more identifier chars
/// (alphanumerics, underscore, hyphen, plus `:` for FQDN segments).
fn read_keyword(bytes: &[u8], pos: usize) -> Option<&[u8]> {
    if pos >= bytes.len() || bytes[pos] != b':' {
        return None;
    }
    let mut end = pos + 1;
    while end < bytes.len() && is_keyword_byte(bytes[end]) {
        end += 1;
    }
    if end == pos + 1 {
        return None;
    }
    Some(&bytes[pos..end])
}

fn is_keyword_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b':'
}

fn skip_ws_and_comments(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() {
        let b = bytes[pos];
        if b.is_ascii_whitespace() {
            pos += 1;
            continue;
        }
        if b == b';' {
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }
        break;
    }
    pos
}

/// Skip a string literal starting at `pos` (where `bytes[pos] ==
/// b'"'`). Returns the position one past the closing quote.
/// Honors `\\` and `\"` escapes.
fn skip_string(bytes: &[u8], mut pos: usize) -> usize {
    debug_assert_eq!(bytes[pos], b'"');
    pos += 1;
    while pos < bytes.len() {
        if bytes[pos] == b'\\' && pos + 1 < bytes.len() {
            pos += 2;
            continue;
        }
        if bytes[pos] == b'"' {
            return pos + 1;
        }
        pos += 1;
    }
    pos
}

/// Read a string literal starting at `pos` (which should point at
/// `"`). Returns the unescaped string contents, or None if `pos`
/// is not the start of a string. Handles `\\`, `\"`, `\n`, `\t`,
/// `\r` escapes.
fn read_string_literal(bytes: &[u8], pos: usize) -> Option<String> {
    if pos >= bytes.len() || bytes[pos] != b'"' {
        return None;
    }
    let mut out = String::new();
    let mut p = pos + 1;
    while p < bytes.len() {
        let b = bytes[p];
        if b == b'\\' && p + 1 < bytes.len() {
            let escape = bytes[p + 1];
            let ch = match escape {
                b'\\' => '\\',
                b'"' => '"',
                b'n' => '\n',
                b't' => '\t',
                b'r' => '\r',
                _ => {
                    // Unknown escape — preserve verbatim.
                    out.push('\\');
                    out.push(escape as char);
                    p += 2;
                    continue;
                }
            };
            out.push(ch);
            p += 2;
            continue;
        }
        if b == b'"' {
            return Some(out);
        }
        out.push(b as char);
        p += 1;
    }
    Some(out)
}

/// Skip a paren-form starting at `pos` (where `bytes[pos] == b'('`).
/// Returns the position one past the matching close paren.
/// Respects nested parens, comments, and string literals.
fn skip_form(bytes: &[u8], mut pos: usize) -> usize {
    debug_assert_eq!(bytes[pos], b'(');
    let mut depth: i32 = 0;
    while pos < bytes.len() {
        let b = bytes[pos];
        if b == b';' {
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }
        if b == b'"' {
            pos = skip_string(bytes, pos);
            continue;
        }
        if b == b'(' {
            depth += 1;
            pos += 1;
            continue;
        }
        if b == b')' {
            depth -= 1;
            pos += 1;
            if depth == 0 {
                return pos;
            }
            continue;
        }
        pos += 1;
    }
    pos
}

/// Sanitize a deftest's keyword name into a valid Rust identifier
/// suitable for `#[test] fn` emission. Replaces every non-ident
/// character with `_`.
///
/// Examples:
/// - `:wat-tests::holon::lru::test-foo` → `wat_tests_holon_lru_test_foo`
/// - `:my::test-name`                   → `my_test_name`
pub fn sanitize_name(name: &str) -> String {
    let trimmed = name.trim_start_matches(':');
    let mut out = String::with_capacity(trimmed.len());
    let mut last_was_underscore = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_was_underscore = false;
        } else if !last_was_underscore && !out.is_empty() {
            out.push('_');
            last_was_underscore = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        return "unnamed".into();
    }
    if out.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        let mut prefixed = String::from("_");
        prefixed.push_str(&out);
        prefixed
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names_only(src: &str) -> Vec<String> {
        scan_file(src).into_iter().map(|s| s.name).collect()
    }

    #[test]
    fn scan_finds_simple_deftest() {
        let src = r#"
            (:wat::test::deftest :my::test-foo
              (:wat::core::let* () ()))
        "#;
        assert_eq!(names_only(src), vec![":my::test-foo".to_string()]);
    }

    #[test]
    fn scan_finds_multiple_deftests() {
        let src = r#"
            (:wat::test::deftest :first ())
            (:wat::test::deftest :second ())
            (:wat::test::deftest :third ())
        "#;
        assert_eq!(
            names_only(src),
            vec![":first".to_string(), ":second".to_string(), ":third".to_string()]
        );
    }

    #[test]
    fn scan_skips_line_comments() {
        let src = r#"
            ;; (:wat::test::deftest :commented-out ())
            ; another comment
            (:wat::test::deftest :real ())
        "#;
        assert_eq!(names_only(src), vec![":real".to_string()]);
    }

    #[test]
    fn scan_skips_string_literals() {
        let src = r#"
            (:user::say "(:wat::test::deftest :inside-string ())")
            (:wat::test::deftest :real ())
        "#;
        assert_eq!(names_only(src), vec![":real".to_string()]);
    }

    #[test]
    fn scan_handles_string_with_escapes() {
        let src = r#"
            (:user::say "an escaped \"quote\" then (:wat::test::deftest :nope ())")
            (:wat::test::deftest :real ())
        "#;
        assert_eq!(names_only(src), vec![":real".to_string()]);
    }

    #[test]
    fn scan_finds_aliases_and_outer() {
        // Arc 124 — make-deftest-registered aliases and direct
        // deftest forms BOTH produce sites. Discovery order
        // follows source order.
        let src = r#"
            (:wat::test::make-deftest :deftest-x ())
            (:deftest-x :my::nested ())
            (:wat::test::deftest :outer ())
        "#;
        assert_eq!(
            names_only(src),
            vec![":my::nested".to_string(), ":outer".to_string()]
        );
    }

    #[test]
    fn scan_keyword_with_dashes_and_colons() {
        let src = r#"
            (:wat::test::deftest :wat-tests::holon::lru::HologramCacheService::test-step1-spawn-join
              ())
        "#;
        assert_eq!(
            names_only(src),
            vec![":wat-tests::holon::lru::HologramCacheService::test-step1-spawn-join".to_string()]
        );
    }

    #[test]
    fn scan_handles_empty_input() {
        assert!(scan_file("").is_empty());
        assert!(scan_file("   \n  \t  ").is_empty());
        assert!(scan_file(";; only comments").is_empty());
    }

    #[test]
    fn scan_handles_paren_in_string() {
        let src = r#"
            (:user::say "( ) ; not a comment")
            (:wat::test::deftest :real ())
        "#;
        assert_eq!(names_only(src), vec![":real".to_string()]);
    }

    // ─── Arc 122 — per-test attributes ───────────────────────────

    #[test]
    fn scan_attaches_ignore_to_next_deftest() {
        let src = r#"
            (:wat::test::ignore "broken on Windows")
            (:wat::test::deftest :my::flaky ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].name, ":my::flaky");
        assert_eq!(sites[0].ignore.as_deref(), Some("broken on Windows"));
        assert_eq!(sites[0].should_panic, None);
    }

    #[test]
    fn scan_attaches_should_panic_to_next_deftest() {
        let src = r#"
            (:wat::test::should-panic "divide by zero")
            (:wat::test::deftest :my::div-zero ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].name, ":my::div-zero");
        assert_eq!(sites[0].ignore, None);
        assert_eq!(sites[0].should_panic.as_deref(), Some("divide by zero"));
    }

    #[test]
    fn scan_attaches_both_annotations() {
        let src = r#"
            (:wat::test::ignore "intermittent")
            (:wat::test::should-panic "expected substring")
            (:wat::test::deftest :my::combined ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].ignore.as_deref(), Some("intermittent"));
        assert_eq!(sites[0].should_panic.as_deref(), Some("expected substring"));
    }

    #[test]
    fn scan_clears_pending_on_unrelated_form() {
        // An annotation followed by a non-deftest form should NOT
        // attach to the later deftest.
        let src = r#"
            (:wat::test::ignore "stale")
            (:user::compute 1 2 3)
            (:wat::test::deftest :my::clean ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].name, ":my::clean");
        assert_eq!(sites[0].ignore, None);
    }

    #[test]
    fn scan_orphan_annotation_silently_ignored() {
        // Annotation at the end of a file with no following deftest.
        let src = r#"
            (:wat::test::deftest :my::test ())
            (:wat::test::ignore "trailing — never attaches")
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].ignore, None);
    }

    #[test]
    fn scan_attaches_only_to_immediately_next_deftest() {
        // First annotation attaches to first deftest only; the
        // second deftest gets no annotation.
        let src = r#"
            (:wat::test::ignore "for first only")
            (:wat::test::deftest :my::first ())
            (:wat::test::deftest :my::second ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].name, ":my::first");
        assert_eq!(sites[0].ignore.as_deref(), Some("for first only"));
        assert_eq!(sites[1].name, ":my::second");
        assert_eq!(sites[1].ignore, None);
    }

    #[test]
    fn scan_handles_string_escape_in_reason() {
        let src = r#"
            (:wat::test::ignore "with \"quote\" and \\backslash")
            (:wat::test::deftest :my::escaped ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(
            sites[0].ignore.as_deref(),
            Some("with \"quote\" and \\backslash")
        );
    }

    #[test]
    fn sanitize_basic() {
        assert_eq!(
            sanitize_name(":wat-tests::holon::lru::test-foo"),
            "wat_tests_holon_lru_test_foo"
        );
    }

    #[test]
    fn sanitize_collapses_runs() {
        assert_eq!(sanitize_name(":my::::test"), "my_test");
        assert_eq!(sanitize_name(":foo--bar--baz"), "foo_bar_baz");
    }

    #[test]
    fn sanitize_strips_trailing_underscore() {
        assert_eq!(sanitize_name(":foo::"), "foo");
        assert_eq!(sanitize_name(":foo----"), "foo");
    }

    #[test]
    fn sanitize_handles_digit_start() {
        assert_eq!(sanitize_name(":1foo"), "_1foo");
    }

    // ─── Arc 123 — time-limit annotation + duration parser ──────

    #[test]
    fn parse_duration_milliseconds() {
        assert_eq!(parse_duration_ms("500ms").unwrap(), 500);
        assert_eq!(parse_duration_ms("1ms").unwrap(), 1);
        assert_eq!(parse_duration_ms("0ms").unwrap(), 0);
    }

    #[test]
    fn parse_duration_seconds() {
        assert_eq!(parse_duration_ms("30s").unwrap(), 30_000);
        assert_eq!(parse_duration_ms("1s").unwrap(), 1_000);
    }

    #[test]
    fn parse_duration_minutes() {
        assert_eq!(parse_duration_ms("5m").unwrap(), 300_000);
        assert_eq!(parse_duration_ms("1m").unwrap(), 60_000);
    }

    #[test]
    fn parse_duration_rejects_missing_suffix() {
        assert!(parse_duration_ms("500").is_err());
        assert!(parse_duration_ms("0").is_err());
    }

    #[test]
    fn parse_duration_rejects_finer_than_ms() {
        assert!(parse_duration_ms("500us").is_err());
        assert!(parse_duration_ms("500ns").is_err());
    }

    #[test]
    fn parse_duration_rejects_long_suffixes() {
        assert!(parse_duration_ms("30sec").is_err());
        assert!(parse_duration_ms("5min").is_err());
        assert!(parse_duration_ms("1hour").is_err());
    }

    #[test]
    fn parse_duration_rejects_non_numeric() {
        assert!(parse_duration_ms("ms").is_err());
        assert!(parse_duration_ms("abcms").is_err());
        assert!(parse_duration_ms("").is_err());
    }

    #[test]
    fn scan_attaches_time_limit_to_next_deftest() {
        let src = r#"
            (:wat::test::time-limit "500ms")
            (:wat::test::deftest :my::bounded ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].name, ":my::bounded");
        assert_eq!(sites[0].time_limit_ms, Some(500));
    }

    #[test]
    fn scan_time_limit_with_seconds_suffix() {
        let src = r#"
            (:wat::test::time-limit "30s")
            (:wat::test::deftest :my::slower ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites[0].time_limit_ms, Some(30_000));
    }

    #[test]
    fn scan_time_limit_with_minutes_suffix() {
        let src = r#"
            (:wat::test::time-limit "5m")
            (:wat::test::deftest :my::integration ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites[0].time_limit_ms, Some(300_000));
    }

    #[test]
    fn scan_time_limit_stacks_with_other_annotations() {
        let src = r#"
            (:wat::test::ignore "intermittent")
            (:wat::test::time-limit "100ms")
            (:wat::test::should-panic "expected")
            (:wat::test::deftest :my::all-three ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].ignore.as_deref(), Some("intermittent"));
        assert_eq!(sites[0].should_panic.as_deref(), Some("expected"));
        assert_eq!(sites[0].time_limit_ms, Some(100));
    }

    #[test]
    fn scan_time_limit_cleared_by_unrelated_form() {
        let src = r#"
            (:wat::test::time-limit "500ms")
            (:user::compute 1 2 3)
            (:wat::test::deftest :my::no-attach ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].time_limit_ms, None);
    }

    // ─── Arc 124 — hermetic + alias deftest discovery ────────────

    #[test]
    fn scan_finds_deftest_hermetic() {
        let src = r#"
            (:wat::test::deftest-hermetic :my::forked
              ((:wat::core::let* () ())))
        "#;
        assert_eq!(names_only(src), vec![":my::forked".to_string()]);
    }

    #[test]
    fn scan_alias_via_make_deftest() {
        let src = r#"
            (:wat::test::make-deftest :deftest ())
            (:deftest :my::aliased ())
        "#;
        assert_eq!(names_only(src), vec![":my::aliased".to_string()]);
    }

    #[test]
    fn scan_alias_via_make_deftest_hermetic() {
        let src = r#"
            (:wat::test::make-deftest-hermetic :deftest-hermetic ())
            (:deftest-hermetic :my::hermetic-alias ())
        "#;
        assert_eq!(
            names_only(src),
            vec![":my::hermetic-alias".to_string()]
        );
    }

    #[test]
    fn scan_alias_with_pending_annotations() {
        // An :ignore preceding an alias call attaches to that
        // call (alias calls behave identically to direct deftest
        // calls for annotation purposes).
        let src = r#"
            (:wat::test::make-deftest-hermetic :deftest-hermetic ())
            (:wat::test::ignore "hangs in arc 119")
            (:deftest-hermetic :my::flaky ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].name, ":my::flaky");
        assert_eq!(sites[0].ignore.as_deref(), Some("hangs in arc 119"));
    }

    #[test]
    fn scan_alias_does_not_attach_annotations_to_make_deftest_call() {
        // An :ignore preceding a make-deftest call is dropped —
        // make-deftest is a declaration, not a test.
        let src = r#"
            (:wat::test::ignore "stale")
            (:wat::test::make-deftest :deftest ())
            (:deftest :my::clean ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].name, ":my::clean");
        assert_eq!(sites[0].ignore, None);
    }

    #[test]
    fn scan_unknown_alias_silently_dropped() {
        // A keyword that isn't registered as an alias is just an
        // unknown form — silently skipped.
        let src = r#"
            (:not-an-alias :would-be-name ())
            (:wat::test::deftest :real ())
        "#;
        assert_eq!(names_only(src), vec![":real".to_string()]);
    }

    #[test]
    fn scan_alias_must_be_declared_before_use() {
        // An alias call BEFORE its make-deftest declaration is
        // silently dropped — the scanner walks top-to-bottom, so
        // the alias isn't yet in the alias table when the call
        // is encountered. (Matches wat's runtime behavior;
        // forward-referenced defmacros surface as type-check
        // errors at freeze time.)
        let src = r#"
            (:deftest-hermetic :my::too-early ())
            (:wat::test::make-deftest-hermetic :deftest-hermetic ())
            (:deftest-hermetic :my::on-time ())
        "#;
        assert_eq!(names_only(src), vec![":my::on-time".to_string()]);
    }

    #[test]
    fn scan_multiple_aliases_in_one_file() {
        let src = r#"
            (:wat::test::make-deftest :deftest ())
            (:wat::test::make-deftest-hermetic :deftest-hermetic ())
            (:deftest :my::in-process ())
            (:deftest-hermetic :my::forked ())
        "#;
        assert_eq!(
            names_only(src),
            vec![":my::in-process".to_string(), ":my::forked".to_string()]
        );
    }

    #[test]
    fn scan_finds_mixed_shapes() {
        // All four shapes coexist: direct deftest, direct
        // deftest-hermetic, alias-via-make-deftest,
        // alias-via-make-deftest-hermetic.
        let src = r#"
            (:wat::test::deftest :a::direct ())
            (:wat::test::deftest-hermetic :b::direct-hermetic ())
            (:wat::test::make-deftest :alias-x ())
            (:alias-x :c::aliased ())
            (:wat::test::make-deftest-hermetic :alias-y ())
            (:alias-y :d::aliased-hermetic ())
        "#;
        assert_eq!(
            names_only(src),
            vec![
                ":a::direct".to_string(),
                ":b::direct-hermetic".to_string(),
                ":c::aliased".to_string(),
                ":d::aliased-hermetic".to_string(),
            ]
        );
    }

    #[test]
    fn scan_alias_carries_time_limit() {
        // :time-limit must compose across the alias path the same
        // way it composes for direct deftest.
        let src = r#"
            (:wat::test::make-deftest-hermetic :deftest-hermetic ())
            (:wat::test::time-limit "200ms")
            (:deftest-hermetic :my::bounded ())
        "#;
        let sites = scan_file(src);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].name, ":my::bounded");
        assert_eq!(sites[0].time_limit_ms, Some(200));
    }
}

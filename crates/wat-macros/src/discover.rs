//! Deftest discovery — paren-balanced scanner that finds
//! `(:wat::test::deftest <name> ...)` forms in `.wat` source.
//!
//! Used by the `wat::test!` proc macro at expansion time to enumerate
//! every deftest under the configured path. The proc macro then emits
//! one `#[test] fn` per discovered site so cargo's libtest sees each
//! deftest as a first-class test (arc 121).
//!
//! This is a tiny lexer — NOT a full wat parser. Recognizing
//! `(:wat::test::deftest <name>` is unambiguous textually:
//!
//! - paren balance (skipping comments and string literals)
//! - at depth 1, the head keyword equals `:wat::test::deftest`
//! - the next non-whitespace, non-comment token is the deftest's
//!   name (a keyword starting with `:`)
//!
//! Comments: `;` to end-of-line. Standard wat comment syntax.
//! Strings: `"..."` with `\\` and `\"` escapes.

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
        for name in scan_file(&src) {
            sites.push(DeftestSite {
                file_path: file.clone(),
                name,
            });
        }
    }
    Ok(sites)
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

/// Scan one `.wat` source string for `(:wat::test::deftest <name> ...)`
/// forms. Returns the discovered deftest names (full keyword form,
/// leading colon included) in source order.
pub fn scan_file(src: &str) -> Vec<String> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut depth: i32 = 0;
    let mut names: Vec<String> = Vec::new();

    while i < bytes.len() {
        let b = bytes[i];

        // Line comment — ; to end of line.
        if b == b';' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // String literal — "..." with \\ and \" escapes.
        if b == b'"' {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        if b == b'(' {
            depth += 1;
            // Look for `:wat::test::deftest <name>` at the start of
            // any list (any depth — deftests can be nested under
            // sandbox forms, etc).
            let after_paren = i + 1;
            let head_start = skip_ws_and_comments(bytes, after_paren);
            if let Some(head) = read_keyword(bytes, head_start) {
                if &src[head_start..head_start + head.len()] == ":wat::test::deftest" {
                    let name_start = skip_ws_and_comments(bytes, head_start + head.len());
                    if let Some(name) = read_keyword(bytes, name_start) {
                        names.push(src[name_start..name_start + name.len()].to_string());
                    }
                }
            }
            i = after_paren;
            continue;
        }

        if b == b')' {
            depth -= 1;
            i += 1;
            continue;
        }

        i += 1;
    }

    let _ = depth; // depth tracked for future invariants; unused here
    names
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

    #[test]
    fn scan_finds_simple_deftest() {
        let src = r#"
            (:wat::test::deftest :my::test-foo
              (:wat::core::let* () ()))
        "#;
        let names = scan_file(src);
        assert_eq!(names, vec![":my::test-foo".to_string()]);
    }

    #[test]
    fn scan_finds_multiple_deftests() {
        let src = r#"
            (:wat::test::deftest :first ())
            (:wat::test::deftest :second ())
            (:wat::test::deftest :third ())
        "#;
        let names = scan_file(src);
        assert_eq!(
            names,
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
        let names = scan_file(src);
        assert_eq!(names, vec![":real".to_string()]);
    }

    #[test]
    fn scan_skips_string_literals() {
        let src = r#"
            (:user::say "(:wat::test::deftest :inside-string ())")
            (:wat::test::deftest :real ())
        "#;
        let names = scan_file(src);
        assert_eq!(names, vec![":real".to_string()]);
    }

    #[test]
    fn scan_handles_string_with_escapes() {
        let src = r#"
            (:user::say "an escaped \"quote\" then (:wat::test::deftest :nope ())")
            (:wat::test::deftest :real ())
        "#;
        let names = scan_file(src);
        assert_eq!(names, vec![":real".to_string()]);
    }

    #[test]
    fn scan_finds_nested_deftest() {
        // A deftest that's inside a make-deftest body should still
        // be discovered (the scanner doesn't gate on depth).
        let src = r#"
            (:wat::test::make-deftest :deftest-x ())
            (:deftest-x :my::nested ())
            (:wat::test::deftest :outer ())
        "#;
        let names = scan_file(src);
        assert_eq!(names, vec![":outer".to_string()]);
    }

    #[test]
    fn scan_keyword_with_dashes_and_colons() {
        let src = r#"
            (:wat::test::deftest :wat-tests::holon::lru::HologramCacheService::test-step1-spawn-join
              ())
        "#;
        let names = scan_file(src);
        assert_eq!(
            names,
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
        let names = scan_file(src);
        assert_eq!(names, vec![":real".to_string()]);
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
}

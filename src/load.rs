//! Recursive `(:wat/core/load! ...)` resolution.
//!
//! Walks the post-config forms from [`crate::config::collect_entry_file`],
//! expands each `load!` form by fetching the referenced source, parsing
//! it, and recursively resolving its own `load!`s. Returns a flat
//! `Vec<WatAST>` — every toplevel form from every loaded file, in load
//! order, with all `load!` nodes replaced by their contents.
//!
//! # The three load modes (per FOUNDATION's startup-loading section)
//!
//! ```scheme
//! (:wat/core/load! "path/to/file.wat")
//! (:wat/core/load! "path/to/file.wat" (md5 "abc123..."))
//! (:wat/core/load! "path/to/file.wat" (signed <sig> <pub-key>))
//! ```
//!
//! This slice **parse-accepts** all three forms — the `md5` / `signed`
//! arguments are captured into [`VerificationMode`] — but does NOT
//! verify them. Cryptographic verification lands with the hashing
//! slice (task #138).
//!
//! # Enforced invariants
//!
//! - **Loaded files cannot contain `(:wat/config/set-*!)`** — the
//!   entry-file discipline's second half. A setter at any level inside
//!   a loaded file halts with [`LoadError::SetterInLoadedFile`].
//! - **Commit-once.** Per FOUNDATION: loading the same path twice halts
//!   startup. [`LoadError::DuplicateLoad`] names the path and both
//!   load sites.
//! - **Cycle detection.** A load path currently on the resolution stack
//!   is a cycle. [`LoadError::CycleDetected`] names the full chain
//!   (file A → file B → ... → file A).
//!
//! # Filesystem vs in-memory
//!
//! Loading goes through the [`SourceLoader`] trait so tests can drive
//! resolution without touching the disk. Production uses [`FsLoader`]
//! (path resolution relative to the importing file's directory); tests
//! use [`InMemoryLoader`].

use crate::ast::WatAST;
use crate::parser::{parse_all, ParseError};
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

/// Verification mode attached to a `load!` form. Parse-accepted in this
/// slice; verification lives in the hashing slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationMode {
    /// `(:wat/core/load! "path")` — trust the contents.
    Unverified,
    /// `(:wat/core/load! "path" (md5 "hex..."))`.
    Md5(String),
    /// `(:wat/core/load! "path" (signed <sig> <pub-key>))`.
    ///
    /// In this slice `sig` and `pub_key` are captured as their printed
    /// WatAST form; the hash-verify slice decides the real types.
    Signed { signature: String, pub_key: String },
}

/// Parsed representation of a single `load!` form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadSpec {
    pub path: String,
    pub verification: VerificationMode,
}

/// A fetched source, with its canonical path for cycle/duplicate detection.
#[derive(Debug, Clone)]
pub struct LoadedSource {
    pub canonical_path: String,
    pub source: String,
}

/// Abstract source fetcher. The [`resolve_loads`] driver calls this
/// for every `load!` it encounters. Implementations handle path
/// resolution (relative to importing file, canonical form, etc.).
pub trait SourceLoader {
    /// Fetch the source for `path`. `base_canonical` is the canonical
    /// path of the importing file (`None` for paths from the entry
    /// file, since the entry file itself has no importer).
    fn load(
        &self,
        path: &str,
        base_canonical: Option<&str>,
    ) -> Result<LoadedSource, LoadFetchError>;
}

/// Errors the loader itself can raise (fetching, path resolution).
#[derive(Debug, Clone, PartialEq)]
pub enum LoadFetchError {
    /// Path doesn't exist under the loader's domain.
    NotFound(String),
    /// Loader-specific I/O or resolution error; prose describes.
    Other { path: String, reason: String },
}

impl fmt::Display for LoadFetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadFetchError::NotFound(p) => write!(f, "load: file not found: {}", p),
            LoadFetchError::Other { path, reason } => {
                write!(f, "load: failed to read {}: {}", path, reason)
            }
        }
    }
}

impl std::error::Error for LoadFetchError {}

/// Errors raised by the load-resolution driver.
#[derive(Debug)]
pub enum LoadError {
    /// The load form was malformed — wrong arity, non-string path,
    /// unknown verification head, etc.
    MalformedLoadForm { reason: String },
    /// A loaded file contained a `(:wat/config/set-*!)` form. Entry-file
    /// discipline: setters belong to the entry file only.
    SetterInLoadedFile {
        loaded_path: String,
        setter_head: String,
    },
    /// The same path was loaded twice (a transitive duplicate or direct
    /// repeat). Per FOUNDATION, this halts startup.
    DuplicateLoad { path: String },
    /// A load chain closed back on itself (A loads B loads A). The cycle
    /// is the full chain of canonical paths that formed the loop.
    CycleDetected { cycle: Vec<String> },
    /// The loader couldn't fetch the file.
    Fetch(LoadFetchError),
    /// Parsing the fetched source failed.
    Parse {
        path: String,
        err: ParseError,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::MalformedLoadForm { reason } => {
                write!(f, "malformed load! form: {}", reason)
            }
            LoadError::SetterInLoadedFile {
                loaded_path,
                setter_head,
            } => write!(
                f,
                "config setter {} in loaded file {}; setters belong in the entry file only",
                setter_head, loaded_path
            ),
            LoadError::DuplicateLoad { path } => {
                write!(
                    f,
                    "path {} loaded more than once; each path may be loaded at most once",
                    path
                )
            }
            LoadError::CycleDetected { cycle } => {
                write!(f, "load cycle detected: {}", cycle.join(" -> "))
            }
            LoadError::Fetch(e) => write!(f, "{}", e),
            LoadError::Parse { path, err } => {
                write!(f, "parse error in loaded file {}: {}", path, err)
            }
        }
    }
}

impl std::error::Error for LoadError {}

impl From<LoadFetchError> for LoadError {
    fn from(e: LoadFetchError) -> Self {
        LoadError::Fetch(e)
    }
}

/// Drive recursive load! resolution.
///
/// `forms` is the post-config form list from [`crate::config::collect_entry_file`].
/// `base_canonical` is the entry file's canonical path if known (used by
/// the loader for relative-path resolution of top-level `load!`s).
///
/// Returns a flat `Vec<WatAST>` containing every form from every loaded
/// file, in load order, with all `load!` forms replaced by their
/// contents. Non-load forms in `forms` are preserved in place.
pub fn resolve_loads(
    forms: Vec<WatAST>,
    base_canonical: Option<&str>,
    loader: &dyn SourceLoader,
) -> Result<Vec<WatAST>, LoadError> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = Vec::new();
    let mut out: Vec<WatAST> = Vec::new();
    process_forms(
        &mut out,
        forms,
        base_canonical,
        loader,
        &mut visited,
        &mut stack,
    )?;
    Ok(out)
}

fn process_forms(
    out: &mut Vec<WatAST>,
    forms: Vec<WatAST>,
    base_canonical: Option<&str>,
    loader: &dyn SourceLoader,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
) -> Result<(), LoadError> {
    for form in forms {
        if let Some(load_spec) = match_load_form(&form)? {
            process_single_load(load_spec, base_canonical, loader, visited, stack, out)?;
        } else {
            out.push(form);
        }
    }
    Ok(())
}

fn process_single_load(
    spec: LoadSpec,
    base_canonical: Option<&str>,
    loader: &dyn SourceLoader,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
    out: &mut Vec<WatAST>,
) -> Result<(), LoadError> {
    let fetched = loader.load(&spec.path, base_canonical)?;

    // Cycle: the target is currently on the resolution stack.
    if stack.iter().any(|p| p == &fetched.canonical_path) {
        let mut cycle = stack.clone();
        cycle.push(fetched.canonical_path.clone());
        return Err(LoadError::CycleDetected { cycle });
    }

    // Commit-once: already loaded via a previous (non-stack) path.
    if visited.contains(&fetched.canonical_path) {
        return Err(LoadError::DuplicateLoad {
            path: fetched.canonical_path,
        });
    }

    visited.insert(fetched.canonical_path.clone());
    stack.push(fetched.canonical_path.clone());

    let loaded_forms = parse_all(&fetched.source).map_err(|err| LoadError::Parse {
        path: fetched.canonical_path.clone(),
        err,
    })?;
    reject_setters_in_loaded(&loaded_forms, &fetched.canonical_path)?;

    process_forms(
        out,
        loaded_forms,
        Some(&fetched.canonical_path),
        loader,
        visited,
        stack,
    )?;

    stack.pop();
    Ok(())
}

/// Attempt to interpret `form` as `(:wat/core/load! "path" [verification])`.
///
/// - Returns `Ok(Some(spec))` if the form IS a well-formed load!.
/// - Returns `Ok(None)` if the form is NOT a load! (any other shape).
/// - Returns `Err(MalformedLoadForm)` if the head is `:wat/core/load!` but
///   the arguments don't conform to the grammar.
fn match_load_form(form: &WatAST) -> Result<Option<LoadSpec>, LoadError> {
    let items = match form {
        WatAST::List(items) => items,
        _ => return Ok(None),
    };
    let head = match items.first() {
        Some(WatAST::Keyword(k)) if k == ":wat/core/load!" => k,
        _ => return Ok(None),
    };
    let _ = head; // kept for future source-position reporting

    let args = &items[1..];
    let path = match args.first() {
        Some(WatAST::StringLit(s)) => s.clone(),
        Some(other) => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "path argument must be a string literal; got {}",
                    variant_name(other)
                ),
            });
        }
        None => {
            return Err(LoadError::MalformedLoadForm {
                reason: "load! requires at least a path string".into(),
            });
        }
    };

    let verification = match args.get(1) {
        None => VerificationMode::Unverified,
        Some(v) => parse_verification_form(v)?,
    };

    if args.len() > 2 {
        return Err(LoadError::MalformedLoadForm {
            reason: format!(
                "load! accepts at most 2 arguments (path, optional verification); got {}",
                args.len()
            ),
        });
    }

    Ok(Some(LoadSpec { path, verification }))
}

/// Parse a `(md5 "hex")` or `(signed <sig> <pub-key>)` form.
fn parse_verification_form(form: &WatAST) -> Result<VerificationMode, LoadError> {
    let items = match form {
        WatAST::List(items) => items,
        _ => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "verification must be a list like (md5 \"...\") or (signed <sig> <pub-key>); got {}",
                    variant_name(form)
                ),
            });
        }
    };
    let head_name = match items.first() {
        Some(WatAST::Symbol(s)) => s.as_str(),
        Some(other) => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "verification head must be a bare symbol (md5 or signed); got {}",
                    variant_name(other)
                ),
            });
        }
        None => {
            return Err(LoadError::MalformedLoadForm {
                reason: "verification form is empty".into(),
            });
        }
    };
    let rest = &items[1..];

    match head_name {
        "md5" => {
            if rest.len() != 1 {
                return Err(LoadError::MalformedLoadForm {
                    reason: format!("(md5 ...) takes one hex-string argument; got {}", rest.len()),
                });
            }
            let hex = match &rest[0] {
                WatAST::StringLit(s) => s.clone(),
                other => {
                    return Err(LoadError::MalformedLoadForm {
                        reason: format!(
                            "(md5 ...) argument must be a string literal; got {}",
                            variant_name(other)
                        ),
                    });
                }
            };
            Ok(VerificationMode::Md5(hex))
        }
        "signed" => {
            if rest.len() != 2 {
                return Err(LoadError::MalformedLoadForm {
                    reason: format!(
                        "(signed <sig> <pub-key>) takes two arguments; got {}",
                        rest.len()
                    ),
                });
            }
            Ok(VerificationMode::Signed {
                signature: render_placeholder(&rest[0]),
                pub_key: render_placeholder(&rest[1]),
            })
        }
        other => Err(LoadError::MalformedLoadForm {
            reason: format!(
                "unknown verification form: {}; expected md5 or signed",
                other
            ),
        }),
    }
}

/// Render a WatAST as a placeholder string for deferred verification
/// slices. The hash-verify slice will replace this with structured
/// types (bytes, Ed25519 pub-key, etc.).
fn render_placeholder(ast: &WatAST) -> String {
    match ast {
        WatAST::StringLit(s) => s.clone(),
        WatAST::Symbol(s) => s.clone(),
        WatAST::Keyword(k) => k.clone(),
        other => format!("{:?}", other),
    }
}

/// Walk a loaded file's forms looking for `(:wat/config/set-*!)`. Any
/// occurrence halts with an error naming the path and the setter head.
fn reject_setters_in_loaded(forms: &[WatAST], path: &str) -> Result<(), LoadError> {
    for form in forms {
        scan_for_setter(form, path)?;
    }
    Ok(())
}

fn scan_for_setter(form: &WatAST, path: &str) -> Result<(), LoadError> {
    if let WatAST::List(items) = form {
        if let Some(WatAST::Keyword(k)) = items.first() {
            if k.starts_with(":wat/config/set-") && k.ends_with('!') {
                return Err(LoadError::SetterInLoadedFile {
                    loaded_path: path.to_string(),
                    setter_head: k.clone(),
                });
            }
        }
        for child in items {
            scan_for_setter(child, path)?;
        }
    }
    Ok(())
}

fn variant_name(ast: &WatAST) -> &'static str {
    match ast {
        WatAST::IntLit(_) => "int literal",
        WatAST::FloatLit(_) => "float literal",
        WatAST::BoolLit(_) => "bool literal",
        WatAST::StringLit(_) => "string literal",
        WatAST::Keyword(_) => "keyword",
        WatAST::Symbol(_) => "symbol",
        WatAST::List(_) => "list",
    }
}

// ─── Loaders ────────────────────────────────────────────────────────────

/// In-memory loader for tests and embedded scenarios.
///
/// Paths are resolved against a simple map; no filesystem access. The
/// `base_canonical` parameter is accepted but not used for relative
/// resolution — the test supplies canonical paths directly.
pub struct InMemoryLoader {
    files: std::collections::HashMap<String, String>,
}

impl Default for InMemoryLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryLoader {
    pub fn new() -> Self {
        InMemoryLoader {
            files: std::collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, path: impl Into<String>, source: impl Into<String>) {
        self.files.insert(path.into(), source.into());
    }
}

impl SourceLoader for InMemoryLoader {
    fn load(
        &self,
        path: &str,
        _base_canonical: Option<&str>,
    ) -> Result<LoadedSource, LoadFetchError> {
        match self.files.get(path) {
            Some(source) => Ok(LoadedSource {
                canonical_path: path.to_string(),
                source: source.clone(),
            }),
            None => Err(LoadFetchError::NotFound(path.to_string())),
        }
    }
}

/// Filesystem loader. Resolves relative paths against the importing
/// file's directory; absolute paths are used as-is. Canonical paths
/// are `std::fs::canonicalize`'d so cycle/duplicate detection treats
/// distinct spellings of the same file as the same file.
pub struct FsLoader;

impl SourceLoader for FsLoader {
    fn load(
        &self,
        path: &str,
        base_canonical: Option<&str>,
    ) -> Result<LoadedSource, LoadFetchError> {
        let requested = Path::new(path);
        let resolved: PathBuf = if requested.is_absolute() {
            requested.to_path_buf()
        } else if let Some(base) = base_canonical {
            let base_dir = Path::new(base).parent().unwrap_or_else(|| Path::new("."));
            base_dir.join(requested)
        } else {
            requested.to_path_buf()
        };

        let canonical = std::fs::canonicalize(&resolved).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => LoadFetchError::NotFound(path.to_string()),
            _ => LoadFetchError::Other {
                path: resolved.display().to_string(),
                reason: e.to_string(),
            },
        })?;

        let source = std::fs::read_to_string(&canonical).map_err(|e| LoadFetchError::Other {
            path: canonical.display().to_string(),
            reason: e.to_string(),
        })?;

        Ok(LoadedSource {
            canonical_path: canonical.display().to_string(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_all;

    fn resolve_mem(entry: &str, files: &[(&str, &str)]) -> Result<Vec<WatAST>, LoadError> {
        let mut loader = InMemoryLoader::new();
        for (p, s) in files {
            loader.add(*p, *s);
        }
        let forms = parse_all(entry).expect("entry parse succeeds");
        resolve_loads(forms, None, &loader)
    }

    #[test]
    fn no_loads_passes_forms_through() {
        let forms = resolve_mem(
            r#"(:wat/algebra/Atom "hello") (:wat/algebra/Atom "world")"#,
            &[],
        )
        .unwrap();
        assert_eq!(forms.len(), 2);
    }

    #[test]
    fn single_load_flattens_into_tree() {
        let forms = resolve_mem(
            r#"(:wat/core/load! "lib.wat") (:wat/algebra/Atom "tail")"#,
            &[("lib.wat", r#"(:wat/algebra/Atom "from-lib")"#)],
        )
        .unwrap();
        // Expect: [Atom "from-lib", Atom "tail"] — load content inlined first.
        assert_eq!(forms.len(), 2);
        if let WatAST::List(items) = &forms[0] {
            if let WatAST::StringLit(s) = &items[1] {
                assert_eq!(s, "from-lib");
                return;
            }
        }
        panic!("first form should be (Atom \"from-lib\") from lib.wat; got {:?}", forms[0]);
    }

    #[test]
    fn transitive_load() {
        let forms = resolve_mem(
            r#"(:wat/core/load! "a.wat")"#,
            &[
                ("a.wat", r#"(:wat/core/load! "b.wat") (:wat/algebra/Atom "a")"#),
                ("b.wat", r#"(:wat/algebra/Atom "b")"#),
            ],
        )
        .unwrap();
        // Expected load order: b.wat (inside a.wat), then a.wat's trailing atom.
        assert_eq!(forms.len(), 2);
    }

    #[test]
    fn load_with_md5_parse_accepted() {
        let forms = resolve_mem(
            r#"(:wat/core/load! "lib.wat" (md5 "abc123"))"#,
            &[("lib.wat", r#"(:wat/algebra/Atom "ok")"#)],
        )
        .unwrap();
        assert_eq!(forms.len(), 1);
    }

    #[test]
    fn load_with_signed_parse_accepted() {
        let forms = resolve_mem(
            r#"(:wat/core/load! "lib.wat" (signed "sig-placeholder" "pubkey-placeholder"))"#,
            &[("lib.wat", r#"(:wat/algebra/Atom "ok")"#)],
        )
        .unwrap();
        assert_eq!(forms.len(), 1);
    }

    // ─── Error cases ────────────────────────────────────────────────────

    #[test]
    fn missing_file_errors() {
        let err = resolve_mem(r#"(:wat/core/load! "missing.wat")"#, &[]).unwrap_err();
        assert!(matches!(err, LoadError::Fetch(LoadFetchError::NotFound(_))));
    }

    #[test]
    fn cycle_detected() {
        let err = resolve_mem(
            r#"(:wat/core/load! "a.wat")"#,
            &[
                ("a.wat", r#"(:wat/core/load! "b.wat")"#),
                ("b.wat", r#"(:wat/core/load! "a.wat")"#),
            ],
        )
        .unwrap_err();
        match err {
            LoadError::CycleDetected { cycle } => {
                assert!(cycle.iter().any(|p| p == "a.wat"));
                assert!(cycle.iter().any(|p| p == "b.wat"));
            }
            other => panic!("expected CycleDetected, got {:?}", other),
        }
    }

    #[test]
    fn self_cycle_detected() {
        let err = resolve_mem(
            r#"(:wat/core/load! "a.wat")"#,
            &[("a.wat", r#"(:wat/core/load! "a.wat")"#)],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::CycleDetected { .. }));
    }

    #[test]
    fn duplicate_load_from_separate_branches_halts() {
        // A loads B, A also loads C, C loads B. B loaded twice = halt.
        let err = resolve_mem(
            r#"(:wat/core/load! "a.wat")"#,
            &[
                (
                    "a.wat",
                    r#"(:wat/core/load! "b.wat") (:wat/core/load! "c.wat")"#,
                ),
                ("b.wat", r#"(:wat/algebra/Atom "b")"#),
                ("c.wat", r#"(:wat/core/load! "b.wat")"#),
            ],
        )
        .unwrap_err();
        match err {
            LoadError::DuplicateLoad { path } => assert_eq!(path, "b.wat"),
            other => panic!("expected DuplicateLoad, got {:?}", other),
        }
    }

    #[test]
    fn setter_in_loaded_file_halts() {
        let err = resolve_mem(
            r#"(:wat/core/load! "bad.wat")"#,
            &[("bad.wat", r#"(:wat/config/set-dims! 4096)"#)],
        )
        .unwrap_err();
        match err {
            LoadError::SetterInLoadedFile {
                loaded_path,
                setter_head,
            } => {
                assert_eq!(loaded_path, "bad.wat");
                assert_eq!(setter_head, ":wat/config/set-dims!");
            }
            other => panic!("expected SetterInLoadedFile, got {:?}", other),
        }
    }

    #[test]
    fn setter_nested_in_loaded_file_halts() {
        // set-*! inside a define body is still illegal in a loaded file.
        let err = resolve_mem(
            r#"(:wat/core/load! "nest.wat")"#,
            &[(
                "nest.wat",
                r#"(:wat/core/define :foo (:wat/config/set-dims! 4096))"#,
            )],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::SetterInLoadedFile { .. }));
    }

    #[test]
    fn load_path_non_string_rejected() {
        let err = resolve_mem(r#"(:wat/core/load! 42)"#, &[]).unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn load_too_many_args_rejected() {
        let err = resolve_mem(
            r#"(:wat/core/load! "a.wat" (md5 "hex") "extra")"#,
            &[("a.wat", "")],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn load_zero_args_rejected() {
        let err = resolve_mem(r#"(:wat/core/load!)"#, &[]).unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn unknown_verification_head_rejected() {
        let err = resolve_mem(
            r#"(:wat/core/load! "a.wat" (sha256 "hex"))"#,
            &[("a.wat", "")],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn parse_error_in_loaded_file_wrapped() {
        let err = resolve_mem(
            r#"(:wat/core/load! "broken.wat")"#,
            &[("broken.wat", "((")],
        )
        .unwrap_err();
        match err {
            LoadError::Parse { path, .. } => assert_eq!(path, "broken.wat"),
            other => panic!("expected Parse, got {:?}", other),
        }
    }

    #[test]
    fn load_order_is_depth_first() {
        // Entry loads A, B. A loads A1. Expected order: A1, A-body, B-body.
        let forms = resolve_mem(
            r#"(:wat/core/load! "a.wat") (:wat/core/load! "b.wat")"#,
            &[
                (
                    "a.wat",
                    r#"(:wat/core/load! "a1.wat") (:wat/algebra/Atom "A")"#,
                ),
                ("a1.wat", r#"(:wat/algebra/Atom "A1")"#),
                ("b.wat", r#"(:wat/algebra/Atom "B")"#),
            ],
        )
        .unwrap();
        assert_eq!(forms.len(), 3);
        // Extract the atom payloads in order:
        let tags: Vec<String> = forms
            .iter()
            .map(|f| {
                if let WatAST::List(items) = f {
                    if let Some(WatAST::StringLit(s)) = items.get(1) {
                        return s.clone();
                    }
                }
                String::from("?")
            })
            .collect();
        assert_eq!(tags, vec!["A1", "A", "B"]);
    }
}

//! Recursive `load!` resolution with `:wat/load/*` and `:wat/verify/*`
//! interface keywords.
//!
//! Three toplevel forms. Each declares its source interface and, where
//! applicable, how verification payloads are fetched. The grammar is
//! explicit about IO: no hidden source of bytes, no ambient
//! authority. Every byte-producing decision is a keyword in the wat.
//!
//! # The three load forms
//!
//! ```scheme
//! ;; Unverified — trust the contents.
//! (:wat/core/load! :wat/load/file-path "path/to/file.wat")
//!
//! ;; Digest-verified — file bytes must hash to the declared digest.
//! (:wat/core/digest-load!
//!   :wat/load/file-path "path/to/file.wat"
//!   :wat/verify/digest-sha256
//!   :wat/verify/string "abc123...")
//!
//! ;; Signature-verified — parsed AST must verify under the declared
//! ;; algorithm with the declared public key.
//! (:wat/core/signed-load!
//!   :wat/load/file-path "path/to/file.wat"
//!   :wat/verify/signed-ed25519
//!   :wat/verify/string "b64-sig"
//!   :wat/verify/string "b64-pubkey")
//! ```
//!
//! # Interface keywords
//!
//! Two namespaces, by concern:
//!
//! - **`:wat/load/*`** — declares where to fetch SOURCE CODE from.
//!   Source loading owns parse, cycle detection, commit-once, and the
//!   recursive load discipline. Implemented this slice:
//!     - `:wat/load/string` — inline source literal.
//!     - `:wat/load/file-path` — filesystem path (relative to the
//!       importing file's directory; absolute paths used as-is).
//!
//! - **`:wat/verify/*`** — declares where to fetch a VERIFICATION
//!   PAYLOAD (digest hex, base64 signature, base64 public key) AND
//!   which verification algorithm to apply. Two sub-roles:
//!     - **Payload interfaces**: `:wat/verify/string` (inline) and
//!       `:wat/verify/file-path` (sidecar file; same relative-path
//!       resolution as `:wat/load/file-path`).
//!     - **Algorithms**: `:wat/verify/digest-sha256` (paired with
//!       `digest-load!`) and `:wat/verify/signed-ed25519` (paired with
//!       `signed-load!`).
//!
//! **Two concerns, two namespaces** — even though both use the
//! filesystem today, loading source has different invariants than
//! loading a payload. Source cares about parse, cycles, commit-once;
//! payload fetching just reads bytes.
//!
//! Future interfaces (`http-path`, `s3-path`, `git-ref`) slot in as
//! additional enum arms and additional `SourceLoader` trait methods.
//! Explicitly NOT implemented in this slice; the namespaces are
//! reserved to make the extension path clean.
//!
//! # Verification semantics
//!
//! - **Digest mode** gates "file bytes unchanged" — runs PRE-PARSE
//!   against the raw fetched source. Invalidated by any byte-level
//!   change (including comments and whitespace).
//! - **Signed mode** gates "AST authored by the holder of this
//!   private key" — runs POST-PARSE against the SHA-256 of the
//!   canonical-EDN. Survives comment / whitespace edits because the
//!   AST is the same.
//!
//! # Enforced invariants
//!
//! - **Loaded files cannot contain `(:wat/config/set-*!)`** — the
//!   entry-file discipline's second half. A setter at any level inside
//!   a loaded file halts with [`LoadError::SetterInLoadedFile`].
//! - **Commit-once.** Per FOUNDATION: loading the same path twice halts
//!   startup. [`LoadError::DuplicateLoad`] names the path.
//! - **Cycle detection.** A load path currently on the resolution stack
//!   is a cycle. [`LoadError::CycleDetected`] names the full chain.
//!
//! # Filesystem vs in-memory
//!
//! The [`SourceLoader`] trait abstracts file-path resolution so tests
//! can drive resolution without touching disk. Production uses
//! [`FsLoader`]; tests use [`InMemoryLoader`]. The `:wat/load/string`
//! and `:wat/verify/string` interfaces are handled directly in the
//! driver and don't go through the trait.

use crate::ast::WatAST;
use crate::parser::{parse_all, ParseError};
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

/// Where to fetch SOURCE code from. Used by all three load forms.
///
/// Each variant corresponds to a `:wat/load/<iface>` keyword in the
/// wat source. Future variants (HttpPath, S3Path, GitRef) slot in as
/// additional arms plus additional `SourceLoader` trait methods. This
/// slice implements only `String` and `FilePath`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceInterface {
    /// `:wat/load/string "source contents here"` — the value IS the
    /// source. Useful for embedded / test scenarios.
    String(String),
    /// `:wat/load/file-path "path/to/file.wat"` — fetch via filesystem.
    /// Relative paths resolve against the importing file's directory.
    FilePath(String),
    // :wat/load/http-path, :wat/load/s3-path, :wat/load/git-ref are
    // reserved but not implemented in this slice. Add new enum arms
    // and new SourceLoader trait methods when needed.
}

/// Where to fetch a VERIFICATION PAYLOAD from (digest hex, base64 sig,
/// base64 pub-key). Distinct from [`SourceInterface`] by concern —
/// payload fetching has no parse / cycle / commit-once semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadInterface {
    /// `:wat/verify/string "abc123..."` — the string IS the payload.
    String(String),
    /// `:wat/verify/file-path "sidecar.sig"` — fetch via filesystem.
    /// Relative paths resolve against the importing file's directory.
    FilePath(String),
    // :wat/verify/http-path, :wat/verify/s3-path are reserved but not
    // implemented. Add new enum arms and new SourceLoader trait
    // methods when needed.
}

/// What to verify and where the payloads come from. Parsed from the
/// tail of `digest-load!` / `signed-load!` forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationSpec {
    /// `(:wat/core/digest-load! ... :wat/verify/digest-<algo> <payload-iface>)`.
    ///
    /// `algo` is extracted from the keyword — for `:wat/verify/digest-sha256`
    /// the algo is `"sha256"`. Verified PRE-PARSE against the raw
    /// fetched bytes.
    Digest {
        algo: String,
        payload: PayloadInterface,
    },
    /// `(:wat/core/signed-load! ... :wat/verify/signed-<algo> <sig-iface> <pubkey-iface>)`.
    ///
    /// `algo` is extracted from the keyword — for
    /// `:wat/verify/signed-ed25519` the algo is `"ed25519"`. Verified
    /// POST-PARSE against the SHA-256 of the canonical-EDN.
    Signed {
        algo: String,
        sig: PayloadInterface,
        pubkey: PayloadInterface,
    },
}

/// Parsed representation of a load form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadSpec {
    pub source: SourceInterface,
    pub verification: Option<VerificationSpec>,
}

/// A fetched source, with its canonical path for cycle/duplicate detection.
#[derive(Debug, Clone)]
pub struct LoadedSource {
    pub canonical_path: String,
    pub source: String,
}

/// Abstract fetcher for file-path interfaces. `:wat/load/string` and
/// `:wat/verify/string` are handled in the driver and never call the
/// trait — the trait exists only for interfaces that need IO.
pub trait SourceLoader {
    /// Fetch source code from a filesystem path.
    fn fetch_source_file(
        &self,
        path: &str,
        base_canonical: Option<&str>,
    ) -> Result<LoadedSource, LoadFetchError>;

    /// Fetch a verification payload from a filesystem path. Relative
    /// paths resolve against the importing file's directory (same as
    /// `fetch_source_file`).
    fn fetch_payload_file(
        &self,
        path: &str,
        base_canonical: Option<&str>,
    ) -> Result<String, LoadFetchError>;
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
    /// The load form was malformed — wrong arity, wrong interface
    /// keyword, wrong value type, unknown verification algorithm, etc.
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
    Parse { path: String, err: ParseError },
    /// Cryptographic verification of the loaded source failed.
    VerificationFailed {
        path: String,
        err: crate::hash::HashError,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::MalformedLoadForm { reason } => {
                write!(f, "malformed load form: {}", reason)
            }
            LoadError::SetterInLoadedFile {
                loaded_path,
                setter_head,
            } => write!(
                f,
                "config setter {} in loaded file {}; setters belong in the entry file only",
                setter_head, loaded_path
            ),
            LoadError::DuplicateLoad { path } => write!(
                f,
                "path {} loaded more than once; each path may be loaded at most once",
                path
            ),
            LoadError::CycleDetected { cycle } => {
                write!(f, "load cycle detected: {}", cycle.join(" -> "))
            }
            LoadError::Fetch(e) => write!(f, "{}", e),
            LoadError::Parse { path, err } => {
                write!(f, "parse error in loaded file {}: {}", path, err)
            }
            LoadError::VerificationFailed { path, err } => {
                write!(f, "verification failed for {}: {}", path, err)
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

/// Drive recursive load resolution.
///
/// `forms` is the post-config form list from
/// [`crate::config::collect_entry_file`]. `base_canonical` is the
/// entry file's canonical path if known (used by the loader for
/// relative-path resolution of top-level loads).
///
/// Returns a flat `Vec<WatAST>` containing every form from every
/// loaded file, in load order, with all load-form nodes replaced by
/// their contents. Non-load forms are preserved in place.
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
    let fetched = fetch_source(&spec.source, base_canonical, loader)?;

    if stack.iter().any(|p| p == &fetched.canonical_path) {
        let mut cycle = stack.clone();
        cycle.push(fetched.canonical_path.clone());
        return Err(LoadError::CycleDetected { cycle });
    }

    if visited.contains(&fetched.canonical_path) {
        return Err(LoadError::DuplicateLoad {
            path: fetched.canonical_path,
        });
    }

    // Digest-mode verification runs PRE-PARSE against raw bytes.
    verify_pre_parse(&fetched, &spec.verification, base_canonical, loader)?;

    visited.insert(fetched.canonical_path.clone());
    stack.push(fetched.canonical_path.clone());

    let loaded_forms = parse_all(&fetched.source).map_err(|err| LoadError::Parse {
        path: fetched.canonical_path.clone(),
        err,
    })?;
    reject_setters_in_loaded(&loaded_forms, &fetched.canonical_path)?;

    // Signed-mode verification runs POST-PARSE against the parsed AST.
    verify_post_parse(
        &fetched.canonical_path,
        &loaded_forms,
        &spec.verification,
        base_canonical,
        loader,
    )?;

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

/// Dispatch on source interface. String sources are handled inline
/// (with a content-hash synthetic canonical path so duplicate inline
/// strings are still caught by commit-once).
fn fetch_source(
    iface: &SourceInterface,
    base_canonical: Option<&str>,
    loader: &dyn SourceLoader,
) -> Result<LoadedSource, LoadError> {
    match iface {
        SourceInterface::String(s) => Ok(LoadedSource {
            canonical_path: synthetic_string_path(s),
            source: s.clone(),
        }),
        SourceInterface::FilePath(p) => Ok(loader.fetch_source_file(p, base_canonical)?),
    }
}

/// Dispatch on payload interface. String payloads return inline;
/// file-path payloads go through the loader.
fn fetch_payload(
    iface: &PayloadInterface,
    base_canonical: Option<&str>,
    loader: &dyn SourceLoader,
) -> Result<String, LoadError> {
    match iface {
        PayloadInterface::String(s) => Ok(s.clone()),
        PayloadInterface::FilePath(p) => Ok(loader.fetch_payload_file(p, base_canonical)?),
    }
}

/// Canonical path for a `:wat/load/string` source. Uses SHA-256 of the
/// content so identical inlined sources still trip commit-once.
fn synthetic_string_path(source: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let digest = hasher.finalize();
    format!("<inline-string:sha256:{}>", crate::hash::hex_encode(&digest))
}

fn verify_pre_parse(
    fetched: &LoadedSource,
    verification: &Option<VerificationSpec>,
    base_canonical: Option<&str>,
    loader: &dyn SourceLoader,
) -> Result<(), LoadError> {
    match verification {
        None => Ok(()),
        Some(VerificationSpec::Digest { algo, payload }) => {
            let hex = fetch_payload(payload, base_canonical, loader)?;
            let hex_trimmed = hex.trim();
            crate::hash::verify_source_hash(fetched.source.as_bytes(), algo, hex_trimmed).map_err(
                |err| LoadError::VerificationFailed {
                    path: fetched.canonical_path.clone(),
                    err,
                },
            )
        }
        Some(VerificationSpec::Signed { .. }) => Ok(()),
    }
}

fn verify_post_parse(
    canonical_path: &str,
    forms: &[WatAST],
    verification: &Option<VerificationSpec>,
    base_canonical: Option<&str>,
    loader: &dyn SourceLoader,
) -> Result<(), LoadError> {
    match verification {
        None | Some(VerificationSpec::Digest { .. }) => Ok(()),
        Some(VerificationSpec::Signed { algo, sig, pubkey }) => {
            let sig_b64 = fetch_payload(sig, base_canonical, loader)?;
            let pk_b64 = fetch_payload(pubkey, base_canonical, loader)?;
            crate::hash::verify_program_signature(
                forms,
                algo,
                sig_b64.trim(),
                pk_b64.trim(),
            )
            .map_err(|err| LoadError::VerificationFailed {
                path: canonical_path.to_string(),
                err,
            })
        }
    }
}

// ─── Form matching ──────────────────────────────────────────────────────

/// Attempt to interpret `form` as one of the three load forms.
fn match_load_form(form: &WatAST) -> Result<Option<LoadSpec>, LoadError> {
    let items = match form {
        WatAST::List(items) => items,
        _ => return Ok(None),
    };
    let head = match items.first() {
        Some(WatAST::Keyword(k)) => k.as_str(),
        _ => return Ok(None),
    };

    match head {
        ":wat/core/load!" => parse_unverified_load(&items[1..]).map(Some),
        ":wat/core/digest-load!" => parse_digest_load(&items[1..]).map(Some),
        ":wat/core/signed-load!" => parse_signed_load(&items[1..]).map(Some),
        _ => Ok(None),
    }
}

/// `(:wat/core/load! :wat/load/<iface> <locator>)`
fn parse_unverified_load(args: &[WatAST]) -> Result<LoadSpec, LoadError> {
    if args.len() != 2 {
        return Err(LoadError::MalformedLoadForm {
            reason: format!(
                "(:wat/core/load! :wat/load/<iface> <locator>) takes exactly two arguments; got {}",
                args.len()
            ),
        });
    }
    let source = parse_source_interface(&args[0], &args[1])?;
    Ok(LoadSpec {
        source,
        verification: None,
    })
}

/// `(:wat/core/digest-load! :wat/load/<iface> <locator>
///      :wat/verify/digest-<algo> :wat/verify/<iface> <payload>)`
fn parse_digest_load(args: &[WatAST]) -> Result<LoadSpec, LoadError> {
    if args.len() != 5 {
        return Err(LoadError::MalformedLoadForm {
            reason: format!(
                "(:wat/core/digest-load! :wat/load/<iface> <locator> :wat/verify/digest-<algo> :wat/verify/<iface> <payload>) takes exactly five arguments; got {}",
                args.len()
            ),
        });
    }
    let source = parse_source_interface(&args[0], &args[1])?;
    let algo = parse_verify_algo(&args[2], "digest-")?;
    let payload = parse_payload_interface(&args[3], &args[4])?;
    Ok(LoadSpec {
        source,
        verification: Some(VerificationSpec::Digest { algo, payload }),
    })
}

/// `(:wat/core/signed-load! :wat/load/<iface> <locator>
///      :wat/verify/signed-<algo>
///      :wat/verify/<iface> <sig>
///      :wat/verify/<iface> <pubkey>)`
fn parse_signed_load(args: &[WatAST]) -> Result<LoadSpec, LoadError> {
    if args.len() != 7 {
        return Err(LoadError::MalformedLoadForm {
            reason: format!(
                "(:wat/core/signed-load! :wat/load/<iface> <locator> :wat/verify/signed-<algo> :wat/verify/<iface> <sig> :wat/verify/<iface> <pubkey>) takes exactly seven arguments; got {}",
                args.len()
            ),
        });
    }
    let source = parse_source_interface(&args[0], &args[1])?;
    let algo = parse_verify_algo(&args[2], "signed-")?;
    let sig = parse_payload_interface(&args[3], &args[4])?;
    let pubkey = parse_payload_interface(&args[5], &args[6])?;
    Ok(LoadSpec {
        source,
        verification: Some(VerificationSpec::Signed { algo, sig, pubkey }),
    })
}

fn parse_source_interface(
    iface_ast: &WatAST,
    locator_ast: &WatAST,
) -> Result<SourceInterface, LoadError> {
    let iface = match iface_ast {
        WatAST::Keyword(k) => k.as_str(),
        other => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "source interface must be a :wat/load/<iface> keyword; got {}",
                    variant_name(other)
                ),
            });
        }
    };
    let locator = match locator_ast {
        WatAST::StringLit(s) => s.clone(),
        other => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "source locator after {} must be a string literal; got {}",
                    iface,
                    variant_name(other)
                ),
            });
        }
    };
    match iface {
        ":wat/load/string" => Ok(SourceInterface::String(locator)),
        ":wat/load/file-path" => Ok(SourceInterface::FilePath(locator)),
        ":wat/load/http-path" | ":wat/load/s3-path" | ":wat/load/git-ref" => {
            Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "source interface {} is reserved but not implemented in this build; use :wat/load/string or :wat/load/file-path",
                    iface
                ),
            })
        }
        other => Err(LoadError::MalformedLoadForm {
            reason: format!(
                "unknown source interface {}; expected :wat/load/string or :wat/load/file-path",
                other
            ),
        }),
    }
}

fn parse_payload_interface(
    iface_ast: &WatAST,
    locator_ast: &WatAST,
) -> Result<PayloadInterface, LoadError> {
    let iface = match iface_ast {
        WatAST::Keyword(k) => k.as_str(),
        other => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "payload interface must be a :wat/verify/<iface> keyword; got {}",
                    variant_name(other)
                ),
            });
        }
    };
    let locator = match locator_ast {
        WatAST::StringLit(s) => s.clone(),
        other => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "payload value after {} must be a string literal; got {}",
                    iface,
                    variant_name(other)
                ),
            });
        }
    };
    match iface {
        ":wat/verify/string" => Ok(PayloadInterface::String(locator)),
        ":wat/verify/file-path" => Ok(PayloadInterface::FilePath(locator)),
        ":wat/verify/http-path" | ":wat/verify/s3-path" => Err(LoadError::MalformedLoadForm {
            reason: format!(
                "payload interface {} is reserved but not implemented in this build; use :wat/verify/string or :wat/verify/file-path",
                iface
            ),
        }),
        other => Err(LoadError::MalformedLoadForm {
            reason: format!(
                "unknown payload interface {}; expected :wat/verify/string or :wat/verify/file-path",
                other
            ),
        }),
    }
}

/// Parse an algorithm keyword like `:wat/verify/digest-sha256` or
/// `:wat/verify/signed-ed25519`. `expected_prefix` is the kind marker
/// (`"digest-"` or `"signed-"`) the form requires.
fn parse_verify_algo(ast: &WatAST, expected_prefix: &str) -> Result<String, LoadError> {
    let keyword = match ast {
        WatAST::Keyword(k) => k.as_str(),
        other => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "verification algorithm must be a :wat/verify/<kind>-<algo> keyword; got {}",
                    variant_name(other)
                ),
            });
        }
    };
    let stripped = match keyword.strip_prefix(":wat/verify/") {
        Some(s) => s,
        None => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "verification algorithm keyword must start with :wat/verify/; got {}",
                    keyword
                ),
            });
        }
    };
    let algo = match stripped.strip_prefix(expected_prefix) {
        Some(a) => a,
        None => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "this load form expects a :wat/verify/{}<algo> keyword; got {}",
                    expected_prefix, keyword
                ),
            });
        }
    };
    if algo.is_empty() {
        return Err(LoadError::MalformedLoadForm {
            reason: format!(
                "verification algorithm keyword names no algorithm after {}; got {}",
                expected_prefix, keyword
            ),
        });
    }
    Ok(algo.to_string())
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

/// In-memory loader for tests and embedded scenarios. Separate maps
/// for source files and payload files because the two concerns are
/// distinct (and tests often want to supply one without the other).
pub struct InMemoryLoader {
    source_files: std::collections::HashMap<String, String>,
    payload_files: std::collections::HashMap<String, String>,
}

impl Default for InMemoryLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryLoader {
    pub fn new() -> Self {
        InMemoryLoader {
            source_files: std::collections::HashMap::new(),
            payload_files: std::collections::HashMap::new(),
        }
    }

    /// Register a source file at `path` (for `:wat/load/file-path`).
    pub fn add_source(&mut self, path: impl Into<String>, contents: impl Into<String>) {
        self.source_files.insert(path.into(), contents.into());
    }

    /// Register a verification payload at `path` (for `:wat/verify/file-path`).
    pub fn add_payload(&mut self, path: impl Into<String>, contents: impl Into<String>) {
        self.payload_files.insert(path.into(), contents.into());
    }
}

impl SourceLoader for InMemoryLoader {
    fn fetch_source_file(
        &self,
        path: &str,
        _base_canonical: Option<&str>,
    ) -> Result<LoadedSource, LoadFetchError> {
        match self.source_files.get(path) {
            Some(source) => Ok(LoadedSource {
                canonical_path: path.to_string(),
                source: source.clone(),
            }),
            None => Err(LoadFetchError::NotFound(path.to_string())),
        }
    }

    fn fetch_payload_file(
        &self,
        path: &str,
        _base_canonical: Option<&str>,
    ) -> Result<String, LoadFetchError> {
        match self.payload_files.get(path) {
            Some(contents) => Ok(contents.clone()),
            None => Err(LoadFetchError::NotFound(path.to_string())),
        }
    }
}

/// Filesystem loader. Resolves relative paths against the importing
/// file's directory; absolute paths are used as-is. Source fetches
/// canonicalize the path via `std::fs::canonicalize` so duplicate /
/// cycle detection treats distinct spellings of the same file as the
/// same file.
pub struct FsLoader;

impl SourceLoader for FsLoader {
    fn fetch_source_file(
        &self,
        path: &str,
        base_canonical: Option<&str>,
    ) -> Result<LoadedSource, LoadFetchError> {
        let resolved = resolve_relative(path, base_canonical);
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

    fn fetch_payload_file(
        &self,
        path: &str,
        base_canonical: Option<&str>,
    ) -> Result<String, LoadFetchError> {
        let resolved = resolve_relative(path, base_canonical);
        std::fs::read_to_string(&resolved).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => LoadFetchError::NotFound(path.to_string()),
            _ => LoadFetchError::Other {
                path: resolved.display().to_string(),
                reason: e.to_string(),
            },
        })
    }
}

fn resolve_relative(path: &str, base_canonical: Option<&str>) -> PathBuf {
    let requested = Path::new(path);
    if requested.is_absolute() {
        requested.to_path_buf()
    } else if let Some(base) = base_canonical {
        let base_dir = Path::new(base).parent().unwrap_or_else(|| Path::new("."));
        base_dir.join(requested)
    } else {
        requested.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_all;

    fn resolve_mem(entry: &str, source_files: &[(&str, &str)]) -> Result<Vec<WatAST>, LoadError> {
        let mut loader = InMemoryLoader::new();
        for (p, s) in source_files {
            loader.add_source(*p, *s);
        }
        let forms = parse_all(entry).expect("entry parse succeeds");
        resolve_loads(forms, None, &loader)
    }

    fn resolve_mem_with_payloads(
        entry: &str,
        source_files: &[(&str, &str)],
        payload_files: &[(&str, &str)],
    ) -> Result<Vec<WatAST>, LoadError> {
        let mut loader = InMemoryLoader::new();
        for (p, s) in source_files {
            loader.add_source(*p, *s);
        }
        for (p, s) in payload_files {
            loader.add_payload(*p, *s);
        }
        let forms = parse_all(entry).expect("entry parse succeeds");
        resolve_loads(forms, None, &loader)
    }

    // ─── Unverified load! ──────────────────────────────────────────────

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
    fn single_file_path_load_inlines() {
        let forms = resolve_mem(
            r#"(:wat/core/load! :wat/load/file-path "lib.wat") (:wat/algebra/Atom "tail")"#,
            &[("lib.wat", r#"(:wat/algebra/Atom "from-lib")"#)],
        )
        .unwrap();
        assert_eq!(forms.len(), 2);
    }

    #[test]
    fn inline_string_source() {
        let forms = resolve_mem(
            r#"(:wat/core/load! :wat/load/string "(:wat/algebra/Atom \"inlined\")")"#,
            &[],
        )
        .unwrap();
        assert_eq!(forms.len(), 1);
        match &forms[0] {
            WatAST::List(items) => match (&items[0], &items[1]) {
                (WatAST::Keyword(k), WatAST::StringLit(s)) => {
                    assert_eq!(k, ":wat/algebra/Atom");
                    assert_eq!(s, "inlined");
                }
                _ => panic!("unexpected children"),
            },
            _ => panic!("expected a list form"),
        }
    }

    #[test]
    fn transitive_load() {
        let forms = resolve_mem(
            r#"(:wat/core/load! :wat/load/file-path "a.wat")"#,
            &[
                (
                    "a.wat",
                    r#"(:wat/core/load! :wat/load/file-path "b.wat") (:wat/algebra/Atom "a")"#,
                ),
                ("b.wat", r#"(:wat/algebra/Atom "b")"#),
            ],
        )
        .unwrap();
        assert_eq!(forms.len(), 2);
    }

    // ─── Digest-load! ──────────────────────────────────────────────────

    #[test]
    fn digest_load_inline_string_verified() {
        use sha2::Digest;
        let source = r#"(:wat/algebra/Atom "ok")"#;
        let mut hasher = sha2::Sha256::new();
        hasher.update(source.as_bytes());
        let hex = crate::hash::hex_encode(&hasher.finalize());
        let entry = format!(
            r#"(:wat/core/digest-load!
                 :wat/load/file-path "lib.wat"
                 :wat/verify/digest-sha256
                 :wat/verify/string "{}")"#,
            hex
        );
        let forms = resolve_mem(&entry, &[("lib.wat", source)]).unwrap();
        assert_eq!(forms.len(), 1);
    }

    #[test]
    fn digest_load_payload_in_file() {
        use sha2::Digest;
        let source = r#"(:wat/algebra/Atom "ok")"#;
        let mut hasher = sha2::Sha256::new();
        hasher.update(source.as_bytes());
        let hex = crate::hash::hex_encode(&hasher.finalize());
        let entry = r#"(:wat/core/digest-load!
                         :wat/load/file-path "lib.wat"
                         :wat/verify/digest-sha256
                         :wat/verify/file-path "lib.wat.sha256")"#;
        let forms = resolve_mem_with_payloads(
            entry,
            &[("lib.wat", source)],
            &[("lib.wat.sha256", &hex)],
        )
        .unwrap();
        assert_eq!(forms.len(), 1);
    }

    #[test]
    fn digest_load_mismatch_rejected() {
        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        let entry = format!(
            r#"(:wat/core/digest-load!
                 :wat/load/file-path "lib.wat"
                 :wat/verify/digest-sha256
                 :wat/verify/string "{}")"#,
            wrong
        );
        let err = resolve_mem(&entry, &[("lib.wat", r#"(:wat/algebra/Atom "ok")"#)])
            .unwrap_err();
        assert!(matches!(err, LoadError::VerificationFailed { .. }));
    }

    #[test]
    fn digest_load_unsupported_algo_rejected() {
        let entry = r#"(:wat/core/digest-load!
                         :wat/load/file-path "lib.wat"
                         :wat/verify/digest-md5
                         :wat/verify/string "abc")"#;
        let err = resolve_mem(entry, &[("lib.wat", r#"(:wat/algebra/Atom "ok")"#)]).unwrap_err();
        match err {
            LoadError::VerificationFailed { err, .. } => {
                assert!(matches!(
                    err,
                    crate::hash::HashError::UnsupportedAlgorithm { .. }
                ));
            }
            other => panic!("expected UnsupportedAlgorithm, got {:?}", other),
        }
    }

    // ─── Signed-load! ──────────────────────────────────────────────────

    fn fixed_signing_key() -> ed25519_dalek::SigningKey {
        ed25519_dalek::SigningKey::from_bytes(&[7u8; 32])
    }

    fn sign_source_ed25519(
        source: &str,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> (String, String) {
        use base64::engine::general_purpose::STANDARD as B64;
        use base64::Engine;
        use ed25519_dalek::Signer;
        let forms = parse_all(source).expect("source parses");
        let hash = crate::hash::hash_canonical_program(&forms);
        let sig = signing_key.sign(&hash);
        let sig_b64 = B64.encode(sig.to_bytes());
        let pk_b64 = B64.encode(signing_key.verifying_key().as_bytes());
        (sig_b64, pk_b64)
    }

    #[test]
    fn signed_load_inline_strings_verified() {
        let source = r#"(:wat/algebra/Atom "ok")"#;
        let (sig, pk) = sign_source_ed25519(source, &fixed_signing_key());
        let entry = format!(
            r#"(:wat/core/signed-load!
                 :wat/load/file-path "lib.wat"
                 :wat/verify/signed-ed25519
                 :wat/verify/string "{}"
                 :wat/verify/string "{}")"#,
            sig, pk
        );
        let forms = resolve_mem(&entry, &[("lib.wat", source)]).unwrap();
        assert_eq!(forms.len(), 1);
    }

    #[test]
    fn signed_load_sidecar_files_verified() {
        let source = r#"(:wat/algebra/Atom "ok")"#;
        let (sig, pk) = sign_source_ed25519(source, &fixed_signing_key());
        let entry = r#"(:wat/core/signed-load!
                         :wat/load/file-path "lib.wat"
                         :wat/verify/signed-ed25519
                         :wat/verify/file-path "lib.wat.sig"
                         :wat/verify/file-path "lib.wat.pubkey")"#;
        let forms = resolve_mem_with_payloads(
            entry,
            &[("lib.wat", source)],
            &[("lib.wat.sig", &sig), ("lib.wat.pubkey", &pk)],
        )
        .unwrap();
        assert_eq!(forms.len(), 1);
    }

    #[test]
    fn signed_load_tampered_source_rejected() {
        let signed_source = r#"(:wat/algebra/Atom "original")"#;
        let tampered_source = r#"(:wat/algebra/Atom "tampered")"#;
        let (sig, pk) = sign_source_ed25519(signed_source, &fixed_signing_key());
        let entry = format!(
            r#"(:wat/core/signed-load!
                 :wat/load/file-path "lib.wat"
                 :wat/verify/signed-ed25519
                 :wat/verify/string "{}"
                 :wat/verify/string "{}")"#,
            sig, pk
        );
        let err = resolve_mem(&entry, &[("lib.wat", tampered_source)]).unwrap_err();
        match err {
            LoadError::VerificationFailed { err, .. } => {
                assert!(matches!(err, crate::hash::HashError::SignatureMismatch { .. }));
            }
            other => panic!("expected SignatureMismatch, got {:?}", other),
        }
    }

    #[test]
    fn signed_load_unsupported_algo_rejected() {
        let entry = r#"(:wat/core/signed-load!
                         :wat/load/file-path "lib.wat"
                         :wat/verify/signed-rsa
                         :wat/verify/string "c2lnLXBsYWNlaG9sZGVy"
                         :wat/verify/string "cGstcGxhY2Vob2xkZXI=")"#;
        let err = resolve_mem(entry, &[("lib.wat", r#"(:wat/algebra/Atom "x")"#)]).unwrap_err();
        match err {
            LoadError::VerificationFailed { err, .. } => {
                assert!(matches!(
                    err,
                    crate::hash::HashError::UnsupportedSignatureAlgorithm { .. }
                ));
            }
            other => panic!("expected UnsupportedSignatureAlgorithm, got {:?}", other),
        }
    }

    // ─── Grammar errors ────────────────────────────────────────────────

    #[test]
    fn load_missing_source_iface_rejected() {
        let err = resolve_mem(r#"(:wat/core/load! "lib.wat")"#, &[("lib.wat", "")]).unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn load_non_keyword_iface_rejected() {
        let err = resolve_mem(
            r#"(:wat/core/load! "wat/load/file-path" "lib.wat")"#,
            &[("lib.wat", "")],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn load_unsupported_source_iface_rejected() {
        let err = resolve_mem(
            r#"(:wat/core/load! :wat/load/http-path "https://example.com/x.wat")"#,
            &[],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn digest_load_wrong_algo_kind_rejected() {
        // :wat/verify/signed-ed25519 in a digest-load! is a grammar error
        // (the keyword names a signature algo, not a digest algo).
        let entry = r#"(:wat/core/digest-load!
                         :wat/load/file-path "lib.wat"
                         :wat/verify/signed-ed25519
                         :wat/verify/string "abc")"#;
        let err = resolve_mem(entry, &[("lib.wat", "")]).unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn signed_load_wrong_algo_kind_rejected() {
        let entry = r#"(:wat/core/signed-load!
                         :wat/load/file-path "lib.wat"
                         :wat/verify/digest-sha256
                         :wat/verify/string "sig"
                         :wat/verify/string "pk")"#;
        let err = resolve_mem(entry, &[("lib.wat", "")]).unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn signed_load_wrong_arity_rejected() {
        let entry = r#"(:wat/core/signed-load!
                         :wat/load/file-path "lib.wat"
                         :wat/verify/signed-ed25519
                         :wat/verify/string "sig-only")"#;
        let err = resolve_mem(entry, &[("lib.wat", "")]).unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn non_string_locator_rejected() {
        let err = resolve_mem(
            r#"(:wat/core/load! :wat/load/file-path 42)"#,
            &[],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    // ─── Commit-once / cycles / setters ────────────────────────────────

    #[test]
    fn missing_file_errors() {
        let err = resolve_mem(
            r#"(:wat/core/load! :wat/load/file-path "missing.wat")"#,
            &[],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::Fetch(LoadFetchError::NotFound(_))));
    }

    #[test]
    fn cycle_detected() {
        let err = resolve_mem(
            r#"(:wat/core/load! :wat/load/file-path "a.wat")"#,
            &[
                ("a.wat", r#"(:wat/core/load! :wat/load/file-path "b.wat")"#),
                ("b.wat", r#"(:wat/core/load! :wat/load/file-path "a.wat")"#),
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
            r#"(:wat/core/load! :wat/load/file-path "a.wat")"#,
            &[("a.wat", r#"(:wat/core/load! :wat/load/file-path "a.wat")"#)],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::CycleDetected { .. }));
    }

    #[test]
    fn duplicate_load_halts() {
        let err = resolve_mem(
            r#"(:wat/core/load! :wat/load/file-path "a.wat")"#,
            &[
                (
                    "a.wat",
                    r#"(:wat/core/load! :wat/load/file-path "b.wat") (:wat/core/load! :wat/load/file-path "c.wat")"#,
                ),
                ("b.wat", r#"(:wat/algebra/Atom "b")"#),
                ("c.wat", r#"(:wat/core/load! :wat/load/file-path "b.wat")"#),
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
            r#"(:wat/core/load! :wat/load/file-path "bad.wat")"#,
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
    fn load_order_is_depth_first() {
        let forms = resolve_mem(
            r#"(:wat/core/load! :wat/load/file-path "a.wat") (:wat/core/load! :wat/load/file-path "b.wat")"#,
            &[
                (
                    "a.wat",
                    r#"(:wat/core/load! :wat/load/file-path "a1.wat") (:wat/algebra/Atom "A")"#,
                ),
                ("a1.wat", r#"(:wat/algebra/Atom "A1")"#),
                ("b.wat", r#"(:wat/algebra/Atom "B")"#),
            ],
        )
        .unwrap();
        assert_eq!(forms.len(), 3);
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

    #[test]
    fn inline_string_duplicate_halts() {
        // Two identical inline-string loads hash to the same synthetic
        // canonical path, so commit-once fires.
        let entry = r#"(:wat/core/load! :wat/load/string "(:wat/algebra/Atom \"x\")")
                       (:wat/core/load! :wat/load/string "(:wat/algebra/Atom \"x\")")"#;
        let err = resolve_mem(entry, &[]).unwrap_err();
        assert!(matches!(err, LoadError::DuplicateLoad { .. }));
    }
}

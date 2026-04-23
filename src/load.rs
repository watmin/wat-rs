//! Recursive `load!` resolution with `:wat::verify::*` interface
//! keywords.
//!
//! Four toplevel forms after arc 028 slice 1 — each with its source
//! argument as a plain string (path or inline source), no interface
//! keyword dispatch. The grammar is explicit about IO: one form per
//! transport shape; verification payloads stay keyword-dispatched
//! because they have multiple payload locations.
//!
//! # The four load forms
//!
//! ```scheme
//! ;; Unverified file load — first arg is the path.
//! (:wat::load-file! "path/to/file.wat")
//!
//! ;; Unverified inline-source load — first arg is the source text.
//! (:wat::load-string! "(:wat::holon::Atom \"x\")")
//!
//! ;; Digest-verified — file bytes must hash to the declared digest.
//! (:wat::digest-load!
//!   "path/to/file.wat"
//!   :wat::verify::digest-sha256
//!   :wat::verify::string "abc123...")
//!
//! ;; Signature-verified — parsed AST must verify under the declared
//! ;; algorithm with the declared public key.
//! (:wat::signed-load!
//!   "path/to/file.wat"
//!   :wat::verify::signed-ed25519
//!   :wat::verify::string "b64-sig"
//!   :wat::verify::string "b64-pubkey")
//! ```
//!
//! # The remaining keyword namespace — `:wat::verify::*`
//!
//! Verification stays keyword-dispatched because a verification payload
//! has multiple genuinely-different source locations (inline vs
//! sidecar file) AND multiple algorithms. The two concerns — location
//! and algorithm — share the namespace:
//!
//! - **Payload-location keywords**: `:wat::verify::string` (inline),
//!   `:wat::verify::file-path` (sidecar file; same relative-path
//!   resolution as load source).
//! - **Algorithm keywords**: `:wat::verify::digest-sha256` (paired
//!   with `digest-load!`), `:wat::verify::signed-ed25519` (paired
//!   with `signed-load!`).
//!
//! Future network-variant LOAD forms (`load-http!`, `load-s3!`,
//! `load-github!`) will be additional named forms, each taking its
//! source address directly — mirroring how `digest-load!` /
//! `signed-load!` sit as named siblings today. Not `:wat::load::http`
//! etc. as interface keywords — the iface-keyword shape retired in
//! arc 028 slice 1.
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
//! - **Loaded files cannot contain `(:wat::config::set-*!)`** — the
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
//! [`FsLoader`]; tests use [`InMemoryLoader`]. The `:wat::load::string`
//! and `:wat::verify::string` interfaces are handled directly in the
//! driver and don't go through the trait.

use crate::ast::WatAST;
use crate::parser::{parse_all, ParseError};
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

/// Where to fetch SOURCE code from. Used internally by the load
/// pipeline; each load form picks its variant directly.
///
/// - `FilePath` — constructed by `load!` / `digest-load!` /
///   `signed-load!` (all file-only post-arc-028-slice-1).
/// - `String` — constructed by `load-string!` (the inline source form).
///
/// Future network-variant forms (`load-http!`, `load-s3!`,
/// `load-github!`) add additional arms here alongside additional
/// `SourceLoader` trait methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceInterface {
    /// Inline source literal — the value IS the wat source. Used by
    /// `(:wat::load-string! <source>)`.
    String(String),
    /// Filesystem path. Relative paths resolve against the importing
    /// file's directory. Used by `(:wat::load-file! <path>)` and the
    /// verified variants.
    FilePath(String),
    // Future arms: HttpPath, S3Path, GitRef. Not present.
}

/// Where to fetch a VERIFICATION PAYLOAD from (digest hex, base64 sig,
/// base64 pub-key). Distinct from [`SourceInterface`] by concern —
/// payload fetching has no parse / cycle / commit-once semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadInterface {
    /// `:wat::verify::string "abc123..."` — the string IS the payload.
    String(String),
    /// `:wat::verify::file-path "sidecar.sig"` — fetch via filesystem.
    /// Relative paths resolve against the importing file's directory.
    FilePath(String),
    // :wat::verify::http-path, :wat::verify::s3-path are reserved but not
    // implemented. Add new enum arms and new SourceLoader trait
    // methods when needed.
}

/// What to verify and where the payloads come from. Parsed from the
/// tail of `digest-load!` / `signed-load!` forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationSpec {
    /// `(:wat::digest-load! ... :wat::verify::digest-<algo> <payload-iface>)`.
    ///
    /// `algo` is extracted from the keyword — for `:wat::verify::digest-sha256`
    /// the algo is `"sha256"`. Verified PRE-PARSE against the raw
    /// fetched bytes.
    Digest {
        algo: String,
        payload: PayloadInterface,
    },
    /// `(:wat::signed-load! ... :wat::verify::signed-<algo> <sig-iface> <pubkey-iface>)`.
    ///
    /// `algo` is extracted from the keyword — for
    /// `:wat::verify::signed-ed25519` the algo is `"ed25519"`. Verified
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

/// Abstract fetcher for file-path interfaces. `:wat::load::string` and
/// `:wat::verify::string` are handled in the driver and never call the
/// trait — the trait exists only for interfaces that need IO.
pub trait SourceLoader: Send + Sync {
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
    /// Path resolves to a location outside the loader's allowed scope.
    /// Raised by [`ScopedLoader`] when the canonical target of a
    /// request escapes the loader's root (e.g., `../../etc/passwd` or
    /// a symlink pointing outside the scope).
    OutOfScope { path: String, scope: String },
}

impl fmt::Display for LoadFetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadFetchError::NotFound(p) => write!(f, "load: file not found: {}", p),
            LoadFetchError::Other { path, reason } => {
                write!(f, "load: failed to read {}: {}", path, reason)
            }
            LoadFetchError::OutOfScope { path, scope } => write!(
                f,
                "load: path {} escapes scope {}",
                path, scope
            ),
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
    /// A loaded file contained a `(:wat::config::set-*!)` form. Entry-file
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

    // Arc 027 slice 1 — canonical-path dedup. A previously loaded
    // file is skipped silently. Its defines are already in `out`
    // from the first load; re-parsing would emit them a second time
    // and trip freeze-layer duplicate-define detection. Matches every
    // mature module system (Python import cache, Node require.cache,
    // Rust use chains, TS module resolution). Cycle detection runs
    // above; dedup never masks a cycle.
    if visited.contains(&fetched.canonical_path) {
        return Ok(());
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

/// Canonical path for a `:wat::load::string` source. Uses SHA-256 of the
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
        WatAST::List(items, _) => items,
        _ => return Ok(None),
    };
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return Ok(None),
    };

    match head {
        // Arc 028 slice 1 — drop the :wat::load::* interface keyword
        // and split each source shape into its own named form. One
        // form per (source-shape × integrity-shape) cell. Future
        // network variants (load-http!, load-s3!, etc.) land as
        // more named siblings following the same shape.
        ":wat::load-file!" => parse_unverified_load(&items[1..]).map(Some),
        ":wat::load-string!" => parse_unverified_load_string(&items[1..]).map(Some),
        ":wat::digest-load!" => parse_digest_load_file(&items[1..]).map(Some),
        ":wat::digest-load-string!" => parse_digest_load_string(&items[1..]).map(Some),
        ":wat::signed-load!" => parse_signed_load_file(&items[1..]).map(Some),
        ":wat::signed-load-string!" => parse_signed_load_string(&items[1..]).map(Some),
        _ => Ok(None),
    }
}

/// `(:wat::load-file! <path>)` — file-path load, single arg.
fn parse_unverified_load(args: &[WatAST]) -> Result<LoadSpec, LoadError> {
    if args.len() != 1 {
        return Err(LoadError::MalformedLoadForm {
            reason: format!(
                "(:wat::load-file! <path>) takes exactly one argument; got {}",
                args.len()
            ),
        });
    }
    let source = expect_string_arg(&args[0], ":wat::load-file!", "path")?;
    Ok(LoadSpec {
        source: SourceInterface::FilePath(source),
        verification: None,
    })
}

/// `(:wat::load-string! <source>)` — inline-source load, single arg.
fn parse_unverified_load_string(args: &[WatAST]) -> Result<LoadSpec, LoadError> {
    if args.len() != 1 {
        return Err(LoadError::MalformedLoadForm {
            reason: format!(
                "(:wat::load-string! <source>) takes exactly one argument; got {}",
                args.len()
            ),
        });
    }
    let source = expect_string_arg(&args[0], ":wat::load-string!", "source")?;
    Ok(LoadSpec {
        source: SourceInterface::String(source),
        verification: None,
    })
}

fn parse_digest_load_file(args: &[WatAST]) -> Result<LoadSpec, LoadError> {
    parse_digest_load_shared(args, ":wat::digest-load!", false)
}

fn parse_digest_load_string(args: &[WatAST]) -> Result<LoadSpec, LoadError> {
    parse_digest_load_shared(args, ":wat::digest-load-string!", true)
}

/// Shared parser for `digest-load!` (file) and `digest-load-string!`
/// (inline). Four args: <source-or-path> :wat::verify::digest-<algo>
/// :wat::verify::<iface> <payload>.
fn parse_digest_load_shared(
    args: &[WatAST],
    op: &'static str,
    is_string: bool,
) -> Result<LoadSpec, LoadError> {
    if args.len() != 4 {
        let shape = if is_string { "<source>" } else { "<path>" };
        return Err(LoadError::MalformedLoadForm {
            reason: format!(
                "({} {} :wat::verify::digest-<algo> :wat::verify::<iface> <payload>) takes exactly four arguments; got {}",
                op, shape, args.len()
            ),
        });
    }
    let source = expect_string_arg(&args[0], op, if is_string { "source" } else { "path" })?;
    let algo = parse_verify_algo(&args[1], "digest-")?;
    let payload = parse_payload_interface(&args[2], &args[3])?;
    let source_iface = if is_string {
        SourceInterface::String(source)
    } else {
        SourceInterface::FilePath(source)
    };
    Ok(LoadSpec {
        source: source_iface,
        verification: Some(VerificationSpec::Digest { algo, payload }),
    })
}

fn parse_signed_load_file(args: &[WatAST]) -> Result<LoadSpec, LoadError> {
    parse_signed_load_shared(args, ":wat::signed-load!", false)
}

fn parse_signed_load_string(args: &[WatAST]) -> Result<LoadSpec, LoadError> {
    parse_signed_load_shared(args, ":wat::signed-load-string!", true)
}

/// Shared parser for `signed-load!` and `signed-load-string!`. Six args:
/// <source-or-path> :wat::verify::signed-<algo>
/// :wat::verify::<iface> <sig> :wat::verify::<iface> <pubkey>.
fn parse_signed_load_shared(
    args: &[WatAST],
    op: &'static str,
    is_string: bool,
) -> Result<LoadSpec, LoadError> {
    if args.len() != 6 {
        let shape = if is_string { "<source>" } else { "<path>" };
        return Err(LoadError::MalformedLoadForm {
            reason: format!(
                "({} {} :wat::verify::signed-<algo> :wat::verify::<iface> <sig> :wat::verify::<iface> <pubkey>) takes exactly six arguments; got {}",
                op, shape, args.len()
            ),
        });
    }
    let source = expect_string_arg(&args[0], op, if is_string { "source" } else { "path" })?;
    let algo = parse_verify_algo(&args[1], "signed-")?;
    let sig = parse_payload_interface(&args[2], &args[3])?;
    let pubkey = parse_payload_interface(&args[4], &args[5])?;
    let source_iface = if is_string {
        SourceInterface::String(source)
    } else {
        SourceInterface::FilePath(source)
    };
    Ok(LoadSpec {
        source: source_iface,
        verification: Some(VerificationSpec::Signed { algo, sig, pubkey }),
    })
}

/// Arc 028 slice 1 — shared helper for the new form shapes. Each
/// load/load-string/digest/signed takes its locator or source as a
/// plain string literal (or an AST string) at a known position.
fn expect_string_arg(
    arg: &WatAST,
    op: &'static str,
    arg_name: &'static str,
) -> Result<String, LoadError> {
    match arg {
        WatAST::StringLit(s, _) => Ok(s.clone()),
        other => Err(LoadError::MalformedLoadForm {
            reason: format!(
                "{}: {} must be a string literal; got {:?}",
                op, arg_name, other
            ),
        }),
    }
}

// Arc 028 slice 1 — parse_source_interface retired alongside the
// :wat::load::* keyword namespace. Each load form now takes its
// source directly (path or string) as a plain argument; no
// interface keyword dispatch.

fn parse_payload_interface(
    iface_ast: &WatAST,
    locator_ast: &WatAST,
) -> Result<PayloadInterface, LoadError> {
    let iface = match iface_ast {
        WatAST::Keyword(k, _) => k.as_str(),
        other => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "payload interface must be a :wat::verify::<iface> keyword; got {}",
                    variant_name(other)
                ),
            });
        }
    };
    let locator = match locator_ast {
        WatAST::StringLit(s, _) => s.clone(),
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
        ":wat::verify::string" => Ok(PayloadInterface::String(locator)),
        ":wat::verify::file-path" => Ok(PayloadInterface::FilePath(locator)),
        ":wat::verify::http-path" | ":wat::verify::s3-path" => Err(LoadError::MalformedLoadForm {
            reason: format!(
                "payload interface {} is reserved but not implemented in this build; use :wat::verify::string or :wat::verify::file-path",
                iface
            ),
        }),
        other => Err(LoadError::MalformedLoadForm {
            reason: format!(
                "unknown payload interface {}; expected :wat::verify::string or :wat::verify::file-path",
                other
            ),
        }),
    }
}

/// Parse an algorithm keyword like `:wat::verify::digest-sha256` or
/// `:wat::verify::signed-ed25519`. `expected_prefix` is the kind marker
/// (`"digest-"` or `"signed-"`) the form requires.
fn parse_verify_algo(ast: &WatAST, expected_prefix: &str) -> Result<String, LoadError> {
    let keyword = match ast {
        WatAST::Keyword(k, _) => k.as_str(),
        other => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "verification algorithm must be a :wat::verify::<kind>-<algo> keyword; got {}",
                    variant_name(other)
                ),
            });
        }
    };
    let stripped = match keyword.strip_prefix(":wat::verify::") {
        Some(s) => s,
        None => {
            return Err(LoadError::MalformedLoadForm {
                reason: format!(
                    "verification algorithm keyword must start with :wat::verify::; got {}",
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
                    "this load form expects a :wat::verify::{}<algo> keyword; got {}",
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

/// Walk a loaded file's forms looking for `(:wat::config::set-*!)`. Any
/// occurrence halts with an error naming the path and the setter head.
fn reject_setters_in_loaded(forms: &[WatAST], path: &str) -> Result<(), LoadError> {
    for form in forms {
        scan_for_setter(form, path)?;
    }
    Ok(())
}

fn scan_for_setter(form: &WatAST, path: &str) -> Result<(), LoadError> {
    if let WatAST::List(items, _) = form {
        if let Some(WatAST::Keyword(k, _)) = items.first() {
            if k.starts_with(":wat::config::set-") && k.ends_with('!') {
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
        WatAST::IntLit(_, _) => "int literal",
        WatAST::FloatLit(_, _) => "float literal",
        WatAST::BoolLit(_, _) => "bool literal",
        WatAST::StringLit(_, _) => "string literal",
        WatAST::Keyword(_, _) => "keyword",
        WatAST::Symbol(_, _) => "symbol",
        WatAST::List(_, _) => "list",
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

    /// Register a source file at `path` (for `:wat::load::file-path`).
    pub fn add_source(&mut self, path: impl Into<String>, contents: impl Into<String>) {
        self.source_files.insert(path.into(), contents.into());
    }

    /// Register a verification payload at `path` (for `:wat::verify::file-path`).
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
        // Arc 027 slice 1 — strip a leading `./` so the honest
        // TypeScript-style notation works identically to bare in
        // in-memory loaders. Filesystem loaders canonicalize the
        // path which strips `./` as a side effect; this loader
        // does key lookup so needs the normalization explicit.
        let key = path.strip_prefix("./").unwrap_or(path);
        match self.source_files.get(key) {
            Some(source) => Ok(LoadedSource {
                canonical_path: key.to_string(),
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
        let key = path.strip_prefix("./").unwrap_or(path);
        match self.payload_files.get(key) {
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

/// Scope-restricted filesystem loader. Resolves every path request
/// against `canonical_root` and refuses reads whose canonical target
/// falls outside the root — rejects `../` traversal, absolute-path
/// escape, and symlinks pointing out of scope.
///
/// Construction canonicalizes the root once; all subsequent containment
/// checks use the canonical root. This catches the case where the root
/// itself is a symlink (we resolve once, consistently).
///
/// TOCTOU note: the canonicalize-then-read sequence has a small window
/// where a symlink could be swapped between the check and the actual
/// read. This is the same window [`FsLoader`] has today. Stronger
/// guarantees (openat + O_NOFOLLOW) are a follow-up if a production
/// caller demands; v1 matches FsLoader's model.
#[derive(Debug, Clone)]
pub struct ScopedLoader {
    canonical_root: PathBuf,
}

impl ScopedLoader {
    /// Construct a new ScopedLoader rooted at `root`. Canonicalizes the
    /// path; fails if the root doesn't exist or isn't reachable.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, LoadFetchError> {
        let root = root.as_ref();
        let canonical_root = std::fs::canonicalize(root).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => LoadFetchError::NotFound(root.display().to_string()),
            _ => LoadFetchError::Other {
                path: root.display().to_string(),
                reason: e.to_string(),
            },
        })?;
        Ok(Self { canonical_root })
    }

    /// The canonical root this loader clamps to. Exposed for diagnostics
    /// and for tests.
    pub fn root(&self) -> &Path {
        &self.canonical_root
    }

    /// Resolve and containment-check a single path. Returns the canonical
    /// target inside the scope, or a `LoadFetchError`.
    ///
    /// Relative-path resolution has two regimes:
    /// - **With caller base** (`base_canonical = Some(...)`) — resolve
    ///   relative to the importing file's directory, matching
    ///   [`FsLoader`]. This is the `(:wat::load-file! ...)` case from
    ///   inside a file that itself has a canonical path.
    /// - **Without caller base** (`base_canonical = None`) — resolve
    ///   relative to this loader's scope root. This is the entry-source
    ///   case: `compose_and_run_with_loader` / the `wat::main!`
    ///   `loader:` argument passes `None` for the entry's base because
    ///   `include_str!`'d source has no disk location. Rooting base-less
    ///   relative paths at the scope means `(load!
    ///   :wat::load::file-path "helper.wat")` from the entry file finds
    ///   `<scope>/helper.wat`, not `<cwd>/helper.wat`.
    ///
    /// Absolute paths bypass both regimes and are canonicalized as-is;
    /// the containment check then rejects anything outside the scope.
    fn resolve_within_scope(
        &self,
        path: &str,
        base_canonical: Option<&str>,
    ) -> Result<PathBuf, LoadFetchError> {
        let requested = Path::new(path);
        let pre_canonical = if requested.is_absolute() {
            requested.to_path_buf()
        } else if let Some(base) = base_canonical {
            Path::new(base)
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(requested)
        } else {
            self.canonical_root.join(requested)
        };
        // Paths that don't exist yet can't be canonicalized. Source
        // reads require an existing file; not-found is a legitimate
        // error signal. Canonicalize first so containment is checked
        // against the real target (handles intermediate symlinks).
        let canonical = std::fs::canonicalize(&pre_canonical).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => LoadFetchError::NotFound(path.to_string()),
            _ => LoadFetchError::Other {
                path: pre_canonical.display().to_string(),
                reason: e.to_string(),
            },
        })?;
        if !canonical.starts_with(&self.canonical_root) {
            return Err(LoadFetchError::OutOfScope {
                path: canonical.display().to_string(),
                scope: self.canonical_root.display().to_string(),
            });
        }
        Ok(canonical)
    }
}

impl SourceLoader for ScopedLoader {
    fn fetch_source_file(
        &self,
        path: &str,
        base_canonical: Option<&str>,
    ) -> Result<LoadedSource, LoadFetchError> {
        let canonical = self.resolve_within_scope(path, base_canonical)?;
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
        let canonical = self.resolve_within_scope(path, base_canonical)?;
        std::fs::read_to_string(&canonical).map_err(|e| LoadFetchError::Other {
            path: canonical.display().to_string(),
            reason: e.to_string(),
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
            r#"(:wat::holon::Atom "hello") (:wat::holon::Atom "world")"#,
            &[],
        )
        .unwrap();
        assert_eq!(forms.len(), 2);
    }

    #[test]
    fn single_file_path_load_inlines() {
        let forms = resolve_mem(
            r#"(:wat::load-file! "lib.wat") (:wat::holon::Atom "tail")"#,
            &[("lib.wat", r#"(:wat::holon::Atom "from-lib")"#)],
        )
        .unwrap();
        assert_eq!(forms.len(), 2);
    }

    #[test]
    fn inline_string_source() {
        let forms = resolve_mem(
            r#"(:wat::load-string! "(:wat::holon::Atom \"inlined\")")"#,
            &[],
        )
        .unwrap();
        assert_eq!(forms.len(), 1);
        match &forms[0] {
            WatAST::List(items, _) => match (&items[0], &items[1]) {
                (WatAST::Keyword(k, _), WatAST::StringLit(s, _)) => {
                    assert_eq!(k, ":wat::holon::Atom");
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
            r#"(:wat::load-file! "a.wat")"#,
            &[
                (
                    "a.wat",
                    r#"(:wat::load-file! "b.wat") (:wat::holon::Atom "a")"#,
                ),
                ("b.wat", r#"(:wat::holon::Atom "b")"#),
            ],
        )
        .unwrap();
        assert_eq!(forms.len(), 2);
    }

    // ─── Digest-load! ──────────────────────────────────────────────────

    #[test]
    fn digest_load_inline_string_verified() {
        use sha2::Digest;
        let source = r#"(:wat::holon::Atom "ok")"#;
        let mut hasher = sha2::Sha256::new();
        hasher.update(source.as_bytes());
        let hex = crate::hash::hex_encode(&hasher.finalize());
        let entry = format!(
            r#"(:wat::digest-load! "lib.wat"
                 :wat::verify::digest-sha256
                 :wat::verify::string "{}")"#,
            hex
        );
        let forms = resolve_mem(&entry, &[("lib.wat", source)]).unwrap();
        assert_eq!(forms.len(), 1);
    }

    #[test]
    fn digest_load_payload_in_file() {
        use sha2::Digest;
        let source = r#"(:wat::holon::Atom "ok")"#;
        let mut hasher = sha2::Sha256::new();
        hasher.update(source.as_bytes());
        let hex = crate::hash::hex_encode(&hasher.finalize());
        let entry = r#"(:wat::digest-load! "lib.wat"
                         :wat::verify::digest-sha256
                         :wat::verify::file-path "lib.wat.sha256")"#;
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
            r#"(:wat::digest-load! "lib.wat"
                 :wat::verify::digest-sha256
                 :wat::verify::string "{}")"#,
            wrong
        );
        let err = resolve_mem(&entry, &[("lib.wat", r#"(:wat::holon::Atom "ok")"#)])
            .unwrap_err();
        assert!(matches!(err, LoadError::VerificationFailed { .. }));
    }

    #[test]
    fn digest_load_unsupported_algo_rejected() {
        let entry = r#"(:wat::digest-load! "lib.wat"
                         :wat::verify::digest-md5
                         :wat::verify::string "abc")"#;
        let err = resolve_mem(entry, &[("lib.wat", r#"(:wat::holon::Atom "ok")"#)]).unwrap_err();
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
        let source = r#"(:wat::holon::Atom "ok")"#;
        let (sig, pk) = sign_source_ed25519(source, &fixed_signing_key());
        let entry = format!(
            r#"(:wat::signed-load! "lib.wat"
                 :wat::verify::signed-ed25519
                 :wat::verify::string "{}"
                 :wat::verify::string "{}")"#,
            sig, pk
        );
        let forms = resolve_mem(&entry, &[("lib.wat", source)]).unwrap();
        assert_eq!(forms.len(), 1);
    }

    #[test]
    fn signed_load_sidecar_files_verified() {
        let source = r#"(:wat::holon::Atom "ok")"#;
        let (sig, pk) = sign_source_ed25519(source, &fixed_signing_key());
        let entry = r#"(:wat::signed-load! "lib.wat"
                         :wat::verify::signed-ed25519
                         :wat::verify::file-path "lib.wat.sig"
                         :wat::verify::file-path "lib.wat.pubkey")"#;
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
        let signed_source = r#"(:wat::holon::Atom "original")"#;
        let tampered_source = r#"(:wat::holon::Atom "tampered")"#;
        let (sig, pk) = sign_source_ed25519(signed_source, &fixed_signing_key());
        let entry = format!(
            r#"(:wat::signed-load! "lib.wat"
                 :wat::verify::signed-ed25519
                 :wat::verify::string "{}"
                 :wat::verify::string "{}")"#,
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
        let entry = r#"(:wat::signed-load! "lib.wat"
                         :wat::verify::signed-rsa
                         :wat::verify::string "c2lnLXBsYWNlaG9sZGVy"
                         :wat::verify::string "cGstcGxhY2Vob2xkZXI=")"#;
        let err = resolve_mem(entry, &[("lib.wat", r#"(:wat::holon::Atom "x")"#)]).unwrap_err();
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
    //
    // Arc 028 slice 1 retired `load_missing_source_iface_rejected` and
    // `load_non_keyword_iface_rejected`. Both asserted the absence of
    // shapes that are now the CORRECT forms after the iface drop:
    //   (:wat::load-file! "lib.wat")   — this IS the valid shape now
    //   (:wat::load-file! <x> "lib.wat") — arity mismatch, caught by
    //                                      new arity check
    // New grammar test: load! with two args (wrong arity) fails loud.
    #[test]
    fn load_wrong_arity_rejected() {
        let err = resolve_mem(
            r#"(:wat::load-file! "lib.wat" "extra-arg")"#,
            &[("lib.wat", "")],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    // Arc 028 slice 1 retired `load_unsupported_source_iface_rejected`.
    // The :wat::load::* iface keyword namespace is gone; unsupported
    // transport is no longer expressible. When future network-variant
    // forms land (load-http!, load-s3!, etc.), each is a distinct
    // named form and its own rejection tests travel with it.

    #[test]
    fn digest_load_wrong_algo_kind_rejected() {
        // :wat::verify::signed-ed25519 in a digest-load! is a grammar error
        // (the keyword names a signature algo, not a digest algo).
        let entry = r#"(:wat::digest-load! "lib.wat"
                         :wat::verify::signed-ed25519
                         :wat::verify::string "abc")"#;
        let err = resolve_mem(entry, &[("lib.wat", "")]).unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn signed_load_wrong_algo_kind_rejected() {
        let entry = r#"(:wat::signed-load! "lib.wat"
                         :wat::verify::digest-sha256
                         :wat::verify::string "sig"
                         :wat::verify::string "pk")"#;
        let err = resolve_mem(entry, &[("lib.wat", "")]).unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn signed_load_wrong_arity_rejected() {
        let entry = r#"(:wat::signed-load! "lib.wat"
                         :wat::verify::signed-ed25519
                         :wat::verify::string "sig-only")"#;
        let err = resolve_mem(entry, &[("lib.wat", "")]).unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    #[test]
    fn non_string_locator_rejected() {
        let err = resolve_mem(
            r#"(:wat::load-file! 42)"#,
            &[],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::MalformedLoadForm { .. }));
    }

    // ─── Commit-once / cycles / setters ────────────────────────────────

    #[test]
    fn missing_file_errors() {
        let err = resolve_mem(
            r#"(:wat::load-file! "missing.wat")"#,
            &[],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::Fetch(LoadFetchError::NotFound(_))));
    }

    #[test]
    fn cycle_detected() {
        let err = resolve_mem(
            r#"(:wat::load-file! "a.wat")"#,
            &[
                ("a.wat", r#"(:wat::load-file! "b.wat")"#),
                ("b.wat", r#"(:wat::load-file! "a.wat")"#),
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
            r#"(:wat::load-file! "a.wat")"#,
            &[("a.wat", r#"(:wat::load-file! "a.wat")"#)],
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::CycleDetected { .. }));
    }

    // Arc 027 slice 1 — what used to be `duplicate_load_halts` now
    // tests that a diamond dependency loads `b.wat` ONCE. The second
    // load in `c.wat` is a silent no-op (canonical-path dedup). The
    // freeze-layer duplicate-define detection would fire if b.wat's
    // forms appeared twice in `out`; this test proves they don't.
    #[test]
    fn diamond_dependency_deduplicates() {
        let forms = resolve_mem(
            r#"(:wat::load-file! "a.wat")"#,
            &[
                (
                    "a.wat",
                    r#"(:wat::load-file! "b.wat") (:wat::load-file! "c.wat")"#,
                ),
                ("b.wat", r#"(:wat::holon::Atom "b")"#),
                ("c.wat", r#"(:wat::load-file! "b.wat")"#),
            ],
        )
        .expect("diamond dep resolves silently");
        // Only b.wat's single form should be present. `a.wat`,
        // `c.wat` have no standalone forms.
        assert_eq!(forms.len(), 1, "expected 1 form (b.wat's Atom), got {:?}", forms);
    }

    #[test]
    fn explicit_dot_prefix_parity_with_bare() {
        // Arc 027 — `./foo.wat` and `foo.wat` from a file that has
        // base_canonical resolve identically. Document the honest-
        // prefix notation at test tier.
        let bare = resolve_mem(
            r#"(:wat::load-file! "entry.wat")"#,
            &[
                (
                    "entry.wat",
                    r#"(:wat::load-file! "helper.wat")"#,
                ),
                ("helper.wat", r#"(:wat::holon::Atom "h")"#),
            ],
        )
        .expect("bare path resolves");

        let dotted = resolve_mem(
            r#"(:wat::load-file! "entry.wat")"#,
            &[
                (
                    "entry.wat",
                    r#"(:wat::load-file! "./helper.wat")"#,
                ),
                ("helper.wat", r#"(:wat::holon::Atom "h")"#),
            ],
        )
        .expect("./ prefix resolves");

        assert_eq!(bare.len(), dotted.len());
        assert_eq!(bare.len(), 1);
    }

    #[test]
    fn setter_in_loaded_file_halts() {
        let err = resolve_mem(
            r#"(:wat::load-file! "bad.wat")"#,
            &[("bad.wat", r#"(:wat::config::set-dims! 4096)"#)],
        )
        .unwrap_err();
        match err {
            LoadError::SetterInLoadedFile {
                loaded_path,
                setter_head,
            } => {
                assert_eq!(loaded_path, "bad.wat");
                assert_eq!(setter_head, ":wat::config::set-dims!");
            }
            other => panic!("expected SetterInLoadedFile, got {:?}", other),
        }
    }

    #[test]
    fn load_order_is_depth_first() {
        let forms = resolve_mem(
            r#"(:wat::load-file! "a.wat") (:wat::load-file! "b.wat")"#,
            &[
                (
                    "a.wat",
                    r#"(:wat::load-file! "a1.wat") (:wat::holon::Atom "A")"#,
                ),
                ("a1.wat", r#"(:wat::holon::Atom "A1")"#),
                ("b.wat", r#"(:wat::holon::Atom "B")"#),
            ],
        )
        .unwrap();
        assert_eq!(forms.len(), 3);
        let tags: Vec<String> = forms
            .iter()
            .map(|f| {
                if let WatAST::List(items, _) = f {
                    if let Some(WatAST::StringLit(s, _)) = items.get(1) {
                        return s.clone();
                    }
                }
                String::from("?")
            })
            .collect();
        assert_eq!(tags, vec!["A1", "A", "B"]);
    }

    #[test]
    fn inline_string_duplicate_deduplicates() {
        // Arc 027 slice 1 — two identical inline-string loads hash to
        // the same synthetic canonical path; the second load is a
        // silent no-op under the new dedup semantic. The form inside
        // `"x"` (a single `:wat::holon::Atom`) appears once in `out`.
        let entry = r#"(:wat::load-string! "(:wat::holon::Atom \"x\")")
                       (:wat::load-string! "(:wat::holon::Atom \"x\")")"#;
        let forms = resolve_mem(entry, &[]).expect("dedup path succeeds");
        assert_eq!(forms.len(), 1, "expected 1 form (single Atom), got {:?}", forms);
    }

    // ─── ScopedLoader ────────────────────────────────────────────────────

    /// RAII wrapper around a unique test directory under std::env::temp_dir.
    /// Dropped at end of scope; best-effort recursive delete.
    struct ScopeDir {
        path: PathBuf,
    }

    impl ScopeDir {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!(
                "wat-scope-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&path).expect("create scope dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for ScopeDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn make_scope_dir() -> ScopeDir {
        ScopeDir::new()
    }

    #[test]
    fn scoped_loader_reads_in_scope_file() {
        let dir = make_scope_dir();
        let file_path = dir.path().join("a.wat");
        std::fs::write(&file_path, "hello").unwrap();
        let loader = ScopedLoader::new(dir.path()).expect("scope");
        let loaded = loader
            .fetch_source_file(&file_path.to_string_lossy(), None)
            .expect("in-scope read");
        assert_eq!(loaded.source, "hello");
    }

    /// Arc 017: base-less relative paths resolve against the scope
    /// root (the `wat::main! { loader: "..." }` entry-source case
    /// where `include_str!`-sourced program text has no canonical
    /// path).
    #[test]
    fn scoped_loader_resolves_base_less_relative_path_against_scope_root() {
        let dir = make_scope_dir();
        std::fs::write(dir.path().join("helper.wat"), "hi").unwrap();
        let loader = ScopedLoader::new(dir.path()).expect("scope");
        let loaded = loader
            .fetch_source_file("helper.wat", None)
            .expect("base-less relative path resolves against scope root");
        assert_eq!(loaded.source, "hi");
    }

    #[test]
    fn scoped_loader_reads_payload_in_scope() {
        let dir = make_scope_dir();
        let file_path = dir.path().join("digest.txt");
        std::fs::write(&file_path, "abc123").unwrap();
        let loader = ScopedLoader::new(dir.path()).expect("scope");
        let payload = loader
            .fetch_payload_file(&file_path.to_string_lossy(), None)
            .expect("in-scope payload read");
        assert_eq!(payload, "abc123");
    }

    #[test]
    fn scoped_loader_rejects_absolute_path_escape() {
        // A second temp dir OUTSIDE the scope.
        let scope = make_scope_dir();
        let outside = make_scope_dir();
        let outside_file = outside.path().join("leak.txt");
        std::fs::write(&outside_file, "secrets").unwrap();
        let loader = ScopedLoader::new(scope.path()).expect("scope");
        let err = loader
            .fetch_source_file(&outside_file.to_string_lossy(), None)
            .expect_err("should reject");
        assert!(matches!(err, LoadFetchError::OutOfScope { .. }));
    }

    #[test]
    fn scoped_loader_rejects_dotdot_escape() {
        // Create scope/subdir/here; ask for `../../etc/passwd` style.
        let scope = make_scope_dir();
        let sub = scope.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        let loader = ScopedLoader::new(&sub).expect("scope");
        // Request a path via relative traversal escaping the sub-scope.
        // The path still resolves to something real on disk (the parent
        // temp dir), but that's outside `sub`.
        let escape = format!("{}/..", sub.display());
        let err = loader
            .fetch_source_file(&escape, None)
            .expect_err("should reject");
        assert!(
            matches!(err, LoadFetchError::OutOfScope { .. }),
            "got {:?}",
            err
        );
    }

    #[test]
    #[cfg(unix)]
    fn scoped_loader_rejects_symlink_escape() {
        // symlink inside scope pointing to a file outside scope.
        let scope = make_scope_dir();
        let outside = make_scope_dir();
        let secret = outside.path().join("secret.txt");
        std::fs::write(&secret, "do-not-read").unwrap();
        let link = scope.path().join("link");
        std::os::unix::fs::symlink(&secret, &link).unwrap();
        let loader = ScopedLoader::new(scope.path()).expect("scope");
        let err = loader
            .fetch_source_file(&link.to_string_lossy(), None)
            .expect_err("should reject");
        assert!(matches!(err, LoadFetchError::OutOfScope { .. }));
    }

    #[test]
    fn scoped_loader_returns_not_found_for_missing_file() {
        let scope = make_scope_dir();
        let loader = ScopedLoader::new(scope.path()).expect("scope");
        let missing = scope.path().join("missing.wat");
        let err = loader
            .fetch_source_file(&missing.to_string_lossy(), None)
            .expect_err("should fail");
        assert!(matches!(err, LoadFetchError::NotFound(_)), "got {:?}", err);
    }

    #[test]
    fn scoped_loader_construction_fails_for_missing_root() {
        let err = ScopedLoader::new("/nonexistent/path/that/does/not/exist-abc")
            .expect_err("should fail");
        assert!(matches!(err, LoadFetchError::NotFound(_)));
    }
}

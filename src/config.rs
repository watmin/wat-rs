//! Entry-file discipline + config pass.
//!
//! A wat entry file has a two-part shape per FOUNDATION's `:wat::config`
//! section: all `(:wat::config::set-*!)` setters first, then all
//! `(:wat::load-file!)` forms (and any trailing program body). This
//! module enforces that shape and commits the setters into a [`Config`]
//! struct.
//!
//! The three fields currently on `:wat::config`:
//!
//! - `dims` (`:usize`) — vector dimension. **Default [`DEFAULT_DIMS`] (10000).**
//!   Optional. Arc 037 slice 1: made optional (was required) so entry
//!   files can omit the setter and get the opinionated default that
//!   matches the pre-arc-037 trading lab convention.
//! - `capacity-mode` (`:wat::config::CapacityMode`) — overflow policy.
//!   **Default [`DEFAULT_CAPACITY_MODE`] (`:error`).** Optional. Arc 037
//!   slice 1: made optional with a safe default (overflow surfaces as
//!   catchable `CapacityExceeded` rather than silently corrupting).
//!   Variants: `:error` / `:panic` (arc 037 retired `:silent` and
//!   `:warn` — overflow either crashes or is handled; arc 045
//!   renamed `:abort` → `:panic`).
//! - `global-seed` (`:u64`) — deterministic-vector seed. **Default 42.**
//!   Optional. Users should rarely set this; the override exists for
//!   deliberate cross-deployment isolation.
//!
//! # Invariants
//!
//! - Every `set-*!` must syntactically precede every non-setter form.
//!   A setter appearing after a non-setter is an error.
//! - Each field can be set at most once across the entry file.
//! - Required fields (`dims`, `capacity-mode`) must be set. Optional
//!   fields fall back to their default.
//! - Value types match the field schema. Arity is enforced per setter.
//! - As of arc 037: every field has a default. Empty entry files commit
//!   a fully-defaulted Config. See [`DEFAULT_DIMS`] and
//!   [`DEFAULT_CAPACITY_MODE`].
//!
//! # What this module does NOT do
//!
//! - It does NOT construct a `VectorManager` / `ScalarEncoder` /
//!   `AtomTypeRegistry` from the config. That's a runtime concern
//!   (the runtime slice + wat binary). This module just collects
//!   the values.
//!
//! **The second half of entry-file discipline** — "setter in a loaded
//! file halts parse" — is enforced by [`crate::load`], not here.
//! Setters in a loaded file reach `load::reject_setters_in_loaded`
//! and raise `LoadError::SetterInLoadedFile`.

use crate::ast::WatAST;
use std::fmt;

/// Default `capacity-mode` when `(:wat::config::set-capacity-mode!)`
/// is omitted. `:error` is safe — overflow surfaces as a catchable
/// `CapacityExceeded` struct rather than silently corrupting the
/// computation. This constant is permanent; `set-capacity-mode!`
/// stays forever as user override.
pub const DEFAULT_CAPACITY_MODE: CapacityMode = CapacityMode::Error;

/// Default `dim-count` when `(:wat::config::set-dim-count!)` is
/// omitted. Arc 077 — the program runs at one d, user-chosen at
/// startup, with 10000 as the substrate-blessed default. Capacity in
/// any `:wat::holon::Hologram` is `floor(sqrt(dim-count))` (e.g. 100
/// at dim-count=10000).
pub const DEFAULT_DIM_COUNT: usize = 10000;

/// Committed configuration values.
///
/// Arc 037 slice 6: every substrate default is a FUNCTION; users
/// override with their own function via AST-accepting setters.
/// `presence_sigma_ast` and `coincident_sigma_ast` carry user-supplied
/// sigma functions that freeze evaluates into `SigmaFn` capabilities
/// on `SymbolTable`. Arc 077 retired the dim-router AST field; the
/// program's `dim_count` is now a stored value, not a per-form picker.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub capacity_mode: CapacityMode,
    pub global_seed: u64,
    /// Arc 077 — the program's encoding dim. One d per program;
    /// capacity for any `Hologram` is derived as
    /// `floor(sqrt(dim_count))`. Default [`DEFAULT_DIM_COUNT`] (10000);
    /// user override via `(:wat::config::set-dim-count! n)`. Restores
    /// the pre-arc-067 surface: arc 037 slice 6 deleted this field in
    /// favor of a router-pick-d-per-form story; arc 077 brings it
    /// back having learned that real programs run at one d.
    pub dim_count: usize,
    /// User-supplied presence-sigma function AST. Signature
    /// `:fn(:i64) -> :i64` — takes d, returns sigma count.
    /// `None` → built-in default `floor(sqrt(d)/2) - 1` (arc 024's
    /// formula). Arc 037 slice 6.
    pub presence_sigma_ast: Option<WatAST>,
    /// User-supplied coincident-sigma function AST. Signature
    /// `:fn(:i64) -> :i64`. `None` → built-in default `1` constant
    /// (the 1σ native granularity). Arc 037 slice 6.
    pub coincident_sigma_ast: Option<WatAST>,
}

/// `:wat::config::CapacityMode` — overflow policy when a frame exceeds
/// Kanerva's capacity budget.
///
/// Two variants only: overflow either crashes or is handled. No
/// middle ground. Arc 037 (2026-04-24) retired `:silent` and `:warn`
/// — they implied the substrate could silently proceed with a
/// corrupted vector, which is never the right answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityMode {
    /// Default — catchable `CapacityExceeded`. Program continues with
    /// `Err(...)`; type system forces the caller to handle it.
    Error,
    /// Production fail-closed — `panic!`. The bad frame never leaves
    /// the dispatcher.
    Panic,
}

/// Errors from the entry-file shape check and config pass.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// A `set-*!` form appeared after a non-setter form in the entry file.
    /// Entry-file discipline: all setters precede all other forms.
    SetterAfterNonSetter {
        form_index: usize,
        setter_head: String,
    },
    /// The same field was set more than once.
    DuplicateField { field: String },
    /// A required field was not set. `global-seed` is optional
    /// (defaults to 42); `dims` and `capacity-mode` are required.
    RequiredFieldMissing { field: String },
    /// A setter head didn't match any known `:wat::config::set-*!`.
    UnknownSetter { head: String },
    /// A setter was called with the wrong number of arguments.
    BadArity {
        head: String,
        expected: usize,
        got: usize,
    },
    /// A setter's argument was the wrong kind of WatAST.
    BadType {
        field: String,
        expected: &'static str,
        got: &'static str,
    },
    /// A setter's argument was well-typed but out-of-range / not a
    /// recognized variant.
    BadValue {
        field: String,
        reason: String,
    },
    /// A setter form was malformed (empty list, head not a keyword).
    MalformedSetter { form_index: usize },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::SetterAfterNonSetter { form_index, setter_head } => {
                write!(
                    f,
                    "config setter follows non-setter; entry-file discipline requires all {} setters before any load! or program body (form index {})",
                    setter_head, form_index
                )
            }
            ConfigError::DuplicateField { field } => {
                write!(f, "config field :{} set more than once; each field may be committed at most once", field)
            }
            ConfigError::RequiredFieldMissing { field } => {
                write!(
                    f,
                    "required config field :{} not set; :dims and :capacity-mode must be committed by the entry file",
                    field
                )
            }
            ConfigError::UnknownSetter { head } => {
                write!(f, "unknown config setter {}", head)
            }
            ConfigError::BadArity { head, expected, got } => {
                write!(
                    f,
                    "config setter {} expects {} argument(s); got {}",
                    head, expected, got
                )
            }
            ConfigError::BadType { field, expected, got } => {
                write!(
                    f,
                    "config field :{} expects {}; got {}",
                    field, expected, got
                )
            }
            ConfigError::BadValue { field, reason } => {
                write!(f, "config field :{}: {}", field, reason)
            }
            ConfigError::MalformedSetter { form_index } => {
                write!(
                    f,
                    "malformed config setter at form index {} (empty list or head not a keyword)",
                    form_index
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Collect the entry file's leading `(:wat::config::set-*!)` setters and
/// commit them to a [`Config`]. Returns the config plus the remaining
/// forms (load!s, program body) for further processing.
///
/// Enforces:
/// - Every setter precedes every non-setter (entry-file discipline).
/// - Each field committed at most once in the forms.
/// - Required fields (`dims`, `capacity-mode`) set; `global-seed`
///   defaults to 42 if unset.
/// - Argument arity and type match each setter's schema.
pub fn collect_entry_file(forms: Vec<WatAST>) -> Result<(Config, Vec<WatAST>), ConfigError> {
    collect_entry_file_inner(forms, None)
}

/// Same as [`collect_entry_file`], but seeded from an inherited baseline
/// [`Config`]. Setters absent from the forms take their value from
/// `inherit`; present setters still override (and duplicate-field
/// checking still applies to the forms themselves). Required-field
/// checking dissolves because `inherit` already has every field set.
///
/// Used by sandbox freezes (`:wat::kernel::run-sandboxed-ast`,
/// `run-sandboxed-hermetic-ast`, fork children) so a sandbox that
/// omits setters inherits the caller's committed config — same
/// scope-inheritance move arc 027 made for the source loader.
pub fn collect_entry_file_with_inherit(
    forms: Vec<WatAST>,
    inherit: &Config,
) -> Result<(Config, Vec<WatAST>), ConfigError> {
    collect_entry_file_inner(forms, Some(inherit))
}

fn collect_entry_file_inner(
    forms: Vec<WatAST>,
    inherit: Option<&Config>,
) -> Result<(Config, Vec<WatAST>), ConfigError> {
    // When `inherit` is set, each field starts at the inherited value.
    // Setters in `forms` override; duplicate-in-forms still errors.
    //
    // Arc 037 slice 6: scalar dims / noise_floor / presence_sigma /
    // coincident_sigma retired. Every override is a FUNCTION AST
    // (dim_router_ast, presence_sigma_ast, coincident_sigma_ast)
    // evaluated at freeze time.
    let mut capacity_mode: Option<CapacityMode> = inherit.map(|c| c.capacity_mode);
    let mut global_seed: Option<u64> = inherit.map(|c| c.global_seed);
    let mut dim_count: usize = inherit.map(|c| c.dim_count).unwrap_or(DEFAULT_DIM_COUNT);
    let mut presence_sigma_ast: Option<WatAST> =
        inherit.and_then(|c| c.presence_sigma_ast.clone());
    let mut coincident_sigma_ast: Option<WatAST> =
        inherit.and_then(|c| c.coincident_sigma_ast.clone());

    // Separate tracker: has this field's setter appeared in THIS forms
    // list? Inheritance pre-seeds the Some; duplicate-in-forms still
    // errors.
    let mut set_capacity_mode = false;
    let mut set_global_seed = false;
    let mut set_dim_count = false;
    let mut set_presence_sigma = false;
    let mut set_coincident_sigma = false;

    let mut remainder_start: Option<usize> = None;

    for (i, form) in forms.iter().enumerate() {
        let setter_head = match setter_head_of(form) {
            Some(head) if head.starts_with(":wat::config::set-") && head.ends_with('!') => {
                head.to_string()
            }
            _ => {
                // First non-setter form — ends the setter section.
                remainder_start = Some(i);
                break;
            }
        };

        // Make sure we haven't already passed the setter section.
        if remainder_start.is_some() {
            return Err(ConfigError::SetterAfterNonSetter {
                form_index: i,
                setter_head,
            });
        }

        let args = setter_args_of(form).ok_or(ConfigError::MalformedSetter { form_index: i })?;

        match setter_head.as_str() {
            ":wat::config::set-capacity-mode!" => {
                if set_capacity_mode {
                    return Err(ConfigError::DuplicateField {
                        field: "capacity-mode".into(),
                    });
                }
                set_capacity_mode = true;
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                capacity_mode = Some(parse_capacity_mode(&args[0])?);
            }
            ":wat::config::set-global-seed!" => {
                if set_global_seed {
                    return Err(ConfigError::DuplicateField {
                        field: "global-seed".into(),
                    });
                }
                set_global_seed = true;
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                global_seed = Some(parse_u64(&args[0], "global-seed")?);
            }
            ":wat::config::set-dim-count!" => {
                if set_dim_count {
                    return Err(ConfigError::DuplicateField {
                        field: "dim-count".into(),
                    });
                }
                set_dim_count = true;
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                let n = parse_u64(&args[0], "dim-count")?;
                if n == 0 {
                    return Err(ConfigError::BadValue {
                        field: "dim-count".into(),
                        reason: "dim-count must be > 0".into(),
                    });
                }
                dim_count = n as usize;
            }
            // Arc 077: `:wat::config::set-dim-router!` retired. The
            // single-d program model dropped the per-form router; use
            // `:wat::config::set-dim-count!` instead.
            ":wat::config::set-presence-sigma!" => {
                if set_presence_sigma {
                    return Err(ConfigError::DuplicateField {
                        field: "presence-sigma".into(),
                    });
                }
                set_presence_sigma = true;
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                // Arc 037 slice 6: AST-valued, not scalar. Freeze
                // evaluates to a `:fn(:i64) -> :i64`. Users who want
                // a constant sigma write `(fn (_d) N)`.
                presence_sigma_ast = Some(args[0].clone());
            }
            ":wat::config::set-coincident-sigma!" => {
                if set_coincident_sigma {
                    return Err(ConfigError::DuplicateField {
                        field: "coincident-sigma".into(),
                    });
                }
                set_coincident_sigma = true;
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                // Arc 037 slice 6: AST-valued. Signature `:fn(:i64) -> :i64`.
                coincident_sigma_ast = Some(args[0].clone());
            }
            _ => {
                return Err(ConfigError::UnknownSetter {
                    head: setter_head,
                });
            }
        }
    }

    // Arc 037 slice 6: all setters optional. capacity-mode defaults
    // to :error; global-seed defaults to 42; the three function ASTs
    // default to None (freeze installs built-in defaults on
    // SymbolTable capability slots).
    let capacity_mode = capacity_mode.unwrap_or(DEFAULT_CAPACITY_MODE);
    let global_seed = global_seed.unwrap_or(42);

    let config = Config {
        capacity_mode,
        global_seed,
        dim_count,
        presence_sigma_ast,
        coincident_sigma_ast,
    };

    let remainder = match remainder_start {
        Some(start) => forms.into_iter().skip(start).collect(),
        None => Vec::new(),
    };

    Ok((config, remainder))
}

/// If `form` is a `WatAST::List` whose first element is a `Keyword`,
/// return that keyword's string. Otherwise return `None` (signaling
/// "not a setter-shaped form").
fn setter_head_of(form: &WatAST) -> Option<&str> {
    match form {
        WatAST::List(items, _) => match items.first()? {
            WatAST::Keyword(k, _) => Some(k),
            _ => None,
        },
        _ => None,
    }
}

/// If `form` is a `WatAST::List`, return the children after the head
/// (i.e. the argument slice).
fn setter_args_of(form: &WatAST) -> Option<&[WatAST]> {
    match form {
        WatAST::List(items, _) => items.get(1..),
        _ => None,
    }
}

fn parse_u64(ast: &WatAST, field: &'static str) -> Result<u64, ConfigError> {
    match ast {
        WatAST::IntLit(n, _) => {
            if *n < 0 {
                return Err(ConfigError::BadValue {
                    field: field.into(),
                    reason: format!("expected non-negative integer, got {}", n),
                });
            }
            Ok(*n as u64)
        }
        other => Err(ConfigError::BadType {
            field: field.into(),
            expected: "integer literal",
            got: variant_name(other),
        }),
    }
}

fn parse_capacity_mode(ast: &WatAST) -> Result<CapacityMode, ConfigError> {
    match ast {
        WatAST::Keyword(k, _) => match k.as_str() {
            ":error" => Ok(CapacityMode::Error),
            ":panic" => Ok(CapacityMode::Panic),
            other => Err(ConfigError::BadValue {
                field: "capacity-mode".into(),
                reason: format!(
                    "unknown variant {}; expected :error / :panic (arc 037 retired :silent and :warn; arc 045 renamed :abort → :panic)",
                    other
                ),
            }),
        },
        other => Err(ConfigError::BadType {
            field: "capacity-mode".into(),
            expected: "keyword (:error / :panic)",
            got: variant_name(other),
        }),
    }
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_all;

    fn collect(src: &str) -> Result<(Config, Vec<WatAST>), ConfigError> {
        let forms = parse_all(src).expect("parse succeeds");
        collect_entry_file(forms)
    }

    // ─── Minimum / defaults ─────────────────────────────────────────────

    #[test]
    fn empty_entry_file_commits_defaults() {
        let (cfg, _) = collect("").unwrap();
        assert_eq!(cfg.capacity_mode, CapacityMode::Error);
        assert_eq!(cfg.global_seed, 42);
        assert_eq!(cfg.dim_count, DEFAULT_DIM_COUNT);
        assert!(cfg.presence_sigma_ast.is_none());
        assert!(cfg.coincident_sigma_ast.is_none());
    }

    #[test]
    fn dim_count_override() {
        let (cfg, _) = collect("(:wat::config::set-dim-count! 4096)").unwrap();
        assert_eq!(cfg.dim_count, 4096);
    }

    #[test]
    fn dim_count_zero_rejected() {
        let err = collect("(:wat::config::set-dim-count! 0)").unwrap_err();
        assert!(matches!(
            err,
            ConfigError::BadValue { ref field, .. } if field == "dim-count"
        ));
    }

    #[test]
    fn dim_count_duplicate_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dim-count! 4096)
            (:wat::config::set-dim-count! 8192)
            "#,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ConfigError::DuplicateField { ref field } if field == "dim-count"
        ));
    }

    #[test]
    fn capacity_mode_error_parses() {
        let (cfg, _) = collect("(:wat::config::set-capacity-mode! :error)").unwrap();
        assert_eq!(cfg.capacity_mode, CapacityMode::Error);
    }

    #[test]
    fn capacity_mode_panic_parses() {
        let (cfg, _) = collect("(:wat::config::set-capacity-mode! :panic)").unwrap();
        assert_eq!(cfg.capacity_mode, CapacityMode::Panic);
    }

    #[test]
    fn global_seed_default_is_42() {
        let (cfg, _) = collect("").unwrap();
        assert_eq!(cfg.global_seed, 42);
    }

    #[test]
    fn global_seed_override() {
        let (cfg, _) = collect("(:wat::config::set-global-seed! 1337)").unwrap();
        assert_eq!(cfg.global_seed, 1337);
    }

    // ─── CapacityMode retirement (arc 037 Layer 1) ──────────────────────

    #[test]
    fn retired_silent_variant_rejected_at_parse() {
        let err = collect("(:wat::config::set-capacity-mode! :silent)").unwrap_err();
        match err {
            ConfigError::BadValue { field, reason } => {
                assert_eq!(field, "capacity-mode");
                assert!(reason.contains(":silent"), "reason: {}", reason);
            }
            other => panic!("expected BadValue, got {:?}", other),
        }
    }

    #[test]
    fn retired_warn_variant_rejected_at_parse() {
        let err = collect("(:wat::config::set-capacity-mode! :warn)").unwrap_err();
        match err {
            ConfigError::BadValue { field, reason } => {
                assert_eq!(field, "capacity-mode");
                assert!(reason.contains(":warn"), "reason: {}", reason);
            }
            other => panic!("expected BadValue, got {:?}", other),
        }
    }

    // Arc 077: set-dim-router! retired; tests removed.

    // ─── set-presence-sigma! / set-coincident-sigma! AST storage ────────

    #[test]
    fn set_presence_sigma_stores_ast_verbatim() {
        let (cfg, _) = collect("(:wat::config::set-presence-sigma! :my::sigma)").unwrap();
        assert!(cfg.presence_sigma_ast.is_some());
    }

    #[test]
    fn set_coincident_sigma_stores_ast_verbatim() {
        let (cfg, _) = collect("(:wat::config::set-coincident-sigma! :my::sigma)").unwrap();
        assert!(cfg.coincident_sigma_ast.is_some());
    }

    #[test]
    fn set_presence_sigma_duplicate_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-presence-sigma! :a)
            (:wat::config::set-presence-sigma! :b)
            "#,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ConfigError::DuplicateField { ref field } if field == "presence-sigma"
        ));
    }

    #[test]
    fn set_coincident_sigma_duplicate_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-coincident-sigma! :a)
            (:wat::config::set-coincident-sigma! :b)
            "#,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ConfigError::DuplicateField { ref field } if field == "coincident-sigma"
        ));
    }

    // ─── Retired setters (arc 037 slice 6 rip) ──────────────────────────

    #[test]
    fn set_dims_is_unknown_setter() {
        let err = collect("(:wat::config::set-dims! 1024)").unwrap_err();
        assert!(matches!(
            err,
            ConfigError::UnknownSetter { ref head } if head == ":wat::config::set-dims!"
        ));
    }

    #[test]
    fn set_noise_floor_is_unknown_setter() {
        let err = collect("(:wat::config::set-noise-floor! 0.1)").unwrap_err();
        assert!(matches!(
            err,
            ConfigError::UnknownSetter { ref head } if head == ":wat::config::set-noise-floor!"
        ));
    }

    // ─── Entry-file discipline ──────────────────────────────────────────

    #[test]
    fn setters_then_body() {
        let (_, rest) = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (some-body-form)
            "#,
        )
        .unwrap();
        assert_eq!(rest.len(), 1);
    }

    #[test]
    fn setter_after_non_setter_is_silently_dropped() {
        let (_cfg, rest) = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:some::body)
            (:wat::config::set-capacity-mode! :panic)
            "#,
        )
        .unwrap();
        assert_eq!(rest.len(), 2);
    }

    #[test]
    fn duplicate_capacity_mode_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-capacity-mode! :panic)
            "#,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ConfigError::DuplicateField { ref field } if field == "capacity-mode"
        ));
    }

    #[test]
    fn unknown_setter_rejected() {
        let err = collect("(:wat::config::set-bogus! 1)").unwrap_err();
        assert!(matches!(
            err,
            ConfigError::UnknownSetter { ref head } if head == ":wat::config::set-bogus!"
        ));
    }

    #[test]
    fn capacity_mode_wrong_type_rejected() {
        let err = collect(r#"(:wat::config::set-capacity-mode! "oops")"#).unwrap_err();
        assert!(matches!(err, ConfigError::BadType { ref field, .. } if field == "capacity-mode"));
    }

    #[test]
    fn wrong_arity_rejected() {
        let err = collect("(:wat::config::set-capacity-mode! :error :panic)").unwrap_err();
        assert!(matches!(
            err,
            ConfigError::BadArity { expected: 1, got: 2, .. }
        ));
    }

    #[test]
    fn negative_global_seed_rejected() {
        let err = collect("(:wat::config::set-global-seed! -5)").unwrap_err();
        assert!(matches!(err, ConfigError::BadValue { ref field, .. } if field == "global-seed"));
    }

    // ─── Inheritance (arc 031) ──────────────────────────────────────────

    fn parent_config() -> Config {
        Config {
            capacity_mode: CapacityMode::Panic,
            global_seed: 99,
            dim_count: DEFAULT_DIM_COUNT,
            presence_sigma_ast: None,
            coincident_sigma_ast: None,
        }
    }

    fn collect_inherit(src: &str, inherit: &Config) -> Result<(Config, Vec<WatAST>), ConfigError> {
        let forms = parse_all(src).expect("parse succeeds");
        collect_entry_file_with_inherit(forms, inherit)
    }

    #[test]
    fn inherit_empty_forms_takes_every_parent_field() {
        let parent = parent_config();
        let (cfg, rest) = collect_inherit("", &parent).unwrap();
        assert_eq!(cfg, parent);
        assert!(rest.is_empty());
    }

    #[test]
    fn inherit_setter_overrides_single_field() {
        let parent = parent_config();
        let (cfg, _) = collect_inherit(
            "(:wat::config::set-global-seed! 7)",
            &parent,
        )
        .unwrap();
        assert_eq!(cfg.global_seed, 7);
        assert_eq!(cfg.capacity_mode, parent.capacity_mode);
    }

    #[test]
    fn inherit_duplicate_setter_in_forms_still_errors() {
        let parent = parent_config();
        let err = collect_inherit(
            r#"
            (:wat::config::set-global-seed! 1)
            (:wat::config::set-global-seed! 2)
            "#,
            &parent,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateField { ref field } if field == "global-seed"));
    }
}


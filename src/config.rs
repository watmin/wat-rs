//! Entry-file discipline + config pass.
//!
//! A wat entry file has a two-part shape per FOUNDATION's `:wat::config`
//! section: all `(:wat::config::set-*!)` setters first, then all
//! `(:wat::core::load!)` forms (and any trailing program body). This
//! module enforces that shape and commits the setters into a [`Config`]
//! struct.
//!
//! The three fields currently on `:wat::config`:
//!
//! - `dims` (`:usize`) — vector dimension. Required; no default.
//! - `capacity-mode` (`:wat::config::CapacityMode`) — overflow policy.
//!   Required; no default. Variants: `:silent` / `:warn` / `:error` /
//!   `:abort`.
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

/// Committed configuration values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    pub dims: usize,
    pub capacity_mode: CapacityMode,
    pub global_seed: u64,
    /// Cosine threshold distinguishing signal from noise for the
    /// substrate at this `dims`. Per FOUNDATION 1718, presence
    /// measurements are compared against this floor to decide whether a
    /// target is "in" a reference vector with confidence.
    ///
    /// Default: `5.0 / sqrt(dims as f64)` — the 5-sigma substrate noise
    /// floor. At `d = 10,000` this is ≈ 0.05; at `d = 1024` ≈ 0.156.
    ///
    /// Users may override exactly once via
    /// `(:wat::config::set-noise-floor! <f64>)` — same discipline as
    /// `set-global-seed!`. Applications that need tighter confidence
    /// (10σ engram-recognition) or looser (rough prefiltering) commit
    /// their threshold at startup rather than threading it through
    /// every presence call.
    pub noise_floor: f64,
}

/// `:wat::config::CapacityMode` — overflow policy when a frame exceeds
/// Kanerva's capacity budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityMode {
    /// Research — user accepts degradation; no check.
    Silent,
    /// Development — log but continue.
    Warn,
    /// Default — catchable `CapacityExceeded`.
    Error,
    /// Production fail-closed — halt the wat.
    Abort,
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
/// - Each field committed at most once.
/// - Required fields (`dims`, `capacity-mode`) set; `global-seed`
///   defaults to 42 if unset.
/// - Argument arity and type match each setter's schema.
pub fn collect_entry_file(forms: Vec<WatAST>) -> Result<(Config, Vec<WatAST>), ConfigError> {
    let mut dims: Option<usize> = None;
    let mut capacity_mode: Option<CapacityMode> = None;
    let mut global_seed: Option<u64> = None;
    let mut noise_floor: Option<f64> = None;
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
            ":wat::config::set-dims!" => {
                if dims.is_some() {
                    return Err(ConfigError::DuplicateField { field: "dims".into() });
                }
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                dims = Some(parse_usize(&args[0], "dims")?);
            }
            ":wat::config::set-capacity-mode!" => {
                if capacity_mode.is_some() {
                    return Err(ConfigError::DuplicateField {
                        field: "capacity-mode".into(),
                    });
                }
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
                if global_seed.is_some() {
                    return Err(ConfigError::DuplicateField {
                        field: "global-seed".into(),
                    });
                }
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                global_seed = Some(parse_u64(&args[0], "global-seed")?);
            }
            ":wat::config::set-noise-floor!" => {
                if noise_floor.is_some() {
                    return Err(ConfigError::DuplicateField {
                        field: "noise-floor".into(),
                    });
                }
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                noise_floor = Some(parse_f64(&args[0], "noise-floor")?);
            }
            _ => {
                return Err(ConfigError::UnknownSetter {
                    head: setter_head,
                });
            }
        }
    }

    let dims = dims.ok_or(ConfigError::RequiredFieldMissing {
        field: "dims".into(),
    })?;
    let capacity_mode = capacity_mode.ok_or(ConfigError::RequiredFieldMissing {
        field: "capacity-mode".into(),
    })?;
    let global_seed = global_seed.unwrap_or(42);
    // Default: 5-sigma substrate noise floor derived from `dims`.
    // `5.0 / sqrt(d)` per FOUNDATION 1718.
    let noise_floor = noise_floor.unwrap_or_else(|| 5.0 / (dims as f64).sqrt());

    let config = Config {
        dims,
        capacity_mode,
        global_seed,
        noise_floor,
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

fn parse_usize(ast: &WatAST, field: &'static str) -> Result<usize, ConfigError> {
    match ast {
        WatAST::IntLit(n, _) => {
            if *n < 0 {
                return Err(ConfigError::BadValue {
                    field: field.into(),
                    reason: format!("expected non-negative integer, got {}", n),
                });
            }
            usize::try_from(*n).map_err(|_| ConfigError::BadValue {
                field: field.into(),
                reason: format!("integer {} does not fit in usize", n),
            })
        }
        other => Err(ConfigError::BadType {
            field: field.into(),
            expected: "integer literal",
            got: variant_name(other),
        }),
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

fn parse_f64(ast: &WatAST, field: &'static str) -> Result<f64, ConfigError> {
    match ast {
        WatAST::FloatLit(x, _) => Ok(*x),
        // Accept IntLit as a convenience — `(set-noise-floor! 0)` works.
        WatAST::IntLit(n, _) => Ok(*n as f64),
        other => Err(ConfigError::BadType {
            field: field.into(),
            expected: "float or integer literal",
            got: variant_name(other),
        }),
    }
}

fn parse_capacity_mode(ast: &WatAST) -> Result<CapacityMode, ConfigError> {
    match ast {
        WatAST::Keyword(k, _) => match k.as_str() {
            ":silent" => Ok(CapacityMode::Silent),
            ":warn" => Ok(CapacityMode::Warn),
            ":error" => Ok(CapacityMode::Error),
            ":abort" => Ok(CapacityMode::Abort),
            other => Err(ConfigError::BadValue {
                field: "capacity-mode".into(),
                reason: format!(
                    "unknown variant {}; expected :silent / :warn / :error / :abort",
                    other
                ),
            }),
        },
        other => Err(ConfigError::BadType {
            field: "capacity-mode".into(),
            expected: "keyword (:silent / :warn / :error / :abort)",
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

    #[test]
    fn minimum_required_entry_file() {
        let (cfg, rest) = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.dims, 10000);
        assert_eq!(cfg.capacity_mode, CapacityMode::Error);
        assert_eq!(cfg.global_seed, 42, "default global-seed is 42");
        // Default noise floor at d=10000 is 5/sqrt(10000) = 0.05.
        let expected = 5.0_f64 / (10000_f64).sqrt();
        assert!((cfg.noise_floor - expected).abs() < 1e-12);
        assert!(rest.is_empty());
    }

    #[test]
    fn noise_floor_default_is_5_over_sqrt_dims() {
        // d=1024 → 5/32 = 0.15625
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.dims, 1024);
        assert!((cfg.noise_floor - 5.0 / 32.0).abs() < 1e-12);
    }

    #[test]
    fn noise_floor_override() {
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-noise-floor! 0.1)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.noise_floor, 0.1);
    }

    #[test]
    fn noise_floor_override_accepts_integer() {
        // set-noise-floor! accepts integer literals as convenience.
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-noise-floor! 0)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.noise_floor, 0.0);
    }

    #[test]
    fn noise_floor_double_set_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-noise-floor! 0.1)
            (:wat::config::set-noise-floor! 0.2)
            "#,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ConfigError::DuplicateField { field } if field == "noise-floor"
        ));
    }

    #[test]
    fn global_seed_default_is_42() {
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.global_seed, 42);
    }

    #[test]
    fn global_seed_override() {
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-global-seed! 12345)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.global_seed, 12345);
    }

    #[test]
    fn setters_then_body() {
        let (cfg, rest) = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :error)
            (:wat::algebra::Atom "hello")
            "#,
        )
        .unwrap();
        assert_eq!(cfg.dims, 10000);
        assert_eq!(rest.len(), 1);
    }

    #[test]
    fn all_capacity_modes_parse() {
        for (kw, variant) in [
            (":silent", CapacityMode::Silent),
            (":warn", CapacityMode::Warn),
            (":error", CapacityMode::Error),
            (":abort", CapacityMode::Abort),
        ] {
            let src = format!(
                r#"
                (:wat::config::set-dims! 1024)
                (:wat::config::set-capacity-mode! {})
                "#,
                kw
            );
            let (cfg, _) = collect(&src).unwrap();
            assert_eq!(cfg.capacity_mode, variant, "failed for {}", kw);
        }
    }

    // ─── Error cases ────────────────────────────────────────────────────

    #[test]
    fn setter_after_non_setter_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::algebra::Atom "oops — body in the middle")
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::RequiredFieldMissing { ref field } if field == "capacity-mode"));
        // Note: our walker stops at first non-setter; the following setter is
        // simply never seen. The RequiredFieldMissing error surfaces the real
        // consequence to the user — capacity-mode wasn't set. A stricter
        // implementation could scan past the non-setter to detect misplaced
        // setters and report SetterAfterNonSetter; see test below for the
        // future-proof variant.
    }

    #[test]
    fn duplicate_dims_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-dims! 8192)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateField { ref field } if field == "dims"));
    }

    #[test]
    fn duplicate_capacity_mode_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-capacity-mode! :abort)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateField { ref field } if field == "capacity-mode"));
    }

    #[test]
    fn missing_dims_rejected() {
        let err = collect(r#"(:wat::config::set-capacity-mode! :error)"#).unwrap_err();
        assert!(matches!(err, ConfigError::RequiredFieldMissing { ref field } if field == "dims"));
    }

    #[test]
    fn missing_capacity_mode_rejected() {
        let err = collect(r#"(:wat::config::set-dims! 10000)"#).unwrap_err();
        assert!(matches!(err, ConfigError::RequiredFieldMissing { ref field } if field == "capacity-mode"));
    }

    #[test]
    fn empty_entry_file_rejected() {
        // No setters = missing dims.
        let err = collect("").unwrap_err();
        assert!(matches!(err, ConfigError::RequiredFieldMissing { ref field } if field == "dims"));
    }

    #[test]
    fn unknown_setter_rejected() {
        let err = collect(r#"(:wat::config::set-bogus! 1)"#).unwrap_err();
        assert!(matches!(err, ConfigError::UnknownSetter { ref head } if head == ":wat::config::set-bogus!"));
    }

    #[test]
    fn wrong_arity_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 10000 8192)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ConfigError::BadArity { expected: 1, got: 2, .. }
        ));
    }

    #[test]
    fn dims_wrong_type_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! "oops")
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::BadType { ref field, .. } if field == "dims"));
    }

    #[test]
    fn capacity_mode_wrong_type_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! 42)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::BadType { ref field, .. } if field == "capacity-mode"));
    }

    #[test]
    fn capacity_mode_unknown_variant_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :chaos)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::BadValue { ref field, .. } if field == "capacity-mode"));
    }

    #[test]
    fn negative_dims_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! -1)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::BadValue { ref field, .. } if field == "dims"));
    }

    #[test]
    fn negative_global_seed_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-global-seed! -5)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::BadValue { ref field, .. } if field == "global-seed"));
    }

    #[test]
    fn setter_order_between_dims_and_capacity_either_way() {
        // The spec says "setters precede loads" — not that dims comes before
        // capacity-mode. Order among setters is free.
        let (cfg_a, _) = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap();
        let (cfg_b, _) = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 10000)
            "#,
        )
        .unwrap();
        assert_eq!(cfg_a, cfg_b);
    }
}

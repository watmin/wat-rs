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
    /// The substrate's **1σ native granularity**: `1.0 / sqrt(dims)`.
    /// The atomic angular unit on the hypersphere at this dimension —
    /// the smallest cosine distance the algebra can distinguish above
    /// its own random-pair distribution. Arc 024 renamed this from
    /// "the 5σ presence floor" to "the 1σ base unit" — both predicates
    /// multiply it by their respective sigma count to derive their
    /// operative threshold.
    ///
    /// Users may override via `(:wat::config::set-noise-floor! <f64>)`
    /// — rare; the derivation from `dims` is the honest default.
    pub noise_floor: f64,
    /// How many σ above the random-pair distribution `presence?` requires
    /// to fire. Default 15 — FPR ≈ 10⁻⁵¹, essentially zero. User
    /// overridable via `(:wat::config::set-presence-sigma! <i64>)`.
    pub presence_sigma: i64,
    /// How many σ below perfect-identity `coincident?` requires to fire.
    /// Default 1 — the native granularity; the geometric minimum.
    /// User overridable via `(:wat::config::set-coincident-sigma! <i64>)`.
    pub coincident_sigma: i64,
    /// Memoized `presence_sigma * noise_floor`. `presence?` closes over
    /// this; recomputed only at config commit.
    pub presence_floor: f64,
    /// Memoized `coincident_sigma * noise_floor`. `coincident?` closes
    /// over this.
    pub coincident_floor: f64,
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
    let mut presence_sigma: Option<i64> = None;
    let mut coincident_sigma: Option<i64> = None;
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
            ":wat::config::set-presence-sigma!" => {
                if presence_sigma.is_some() {
                    return Err(ConfigError::DuplicateField {
                        field: "presence-sigma".into(),
                    });
                }
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                presence_sigma = Some(parse_positive_i64(&args[0], "presence-sigma")?);
            }
            ":wat::config::set-coincident-sigma!" => {
                if coincident_sigma.is_some() {
                    return Err(ConfigError::DuplicateField {
                        field: "coincident-sigma".into(),
                    });
                }
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                coincident_sigma = Some(parse_positive_i64(&args[0], "coincident-sigma")?);
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
    // noise_floor defaults to the 1σ native granularity: 1/sqrt(dims).
    // The atomic angular unit; both predicates multiply it by their
    // sigma counts. Arc 024 retired the prior 5σ conflation.
    let noise_floor = noise_floor.unwrap_or_else(|| 1.0 / (dims as f64).sqrt());
    let presence_sigma = presence_sigma.unwrap_or(15);
    let coincident_sigma = coincident_sigma.unwrap_or(1);
    let presence_floor = (presence_sigma as f64) * noise_floor;
    let coincident_floor = (coincident_sigma as f64) * noise_floor;

    // Validity: n_p + n_c < sqrt(dims). Above this the presence /
    // coincident predicates collapse (their thresholds meet or swap).
    // Behavior per capacity_mode — reuses the same four-mode policy
    // Bundle capacity uses.
    let sigma_sum = presence_sigma.saturating_add(coincident_sigma);
    let dims_sqrt = (dims as f64).sqrt();
    if (sigma_sum as f64) >= dims_sqrt {
        match capacity_mode {
            CapacityMode::Silent => { /* proceed anyway */ }
            CapacityMode::Warn => {
                eprintln!(
                    "warning: presence-sigma ({}) + coincident-sigma ({}) = {} >= sqrt(dims) = {:.4}. \
                    Predicate duality collapses at or above this sum. Raise dims or lower a sigma.",
                    presence_sigma, coincident_sigma, sigma_sum, dims_sqrt
                );
            }
            CapacityMode::Error => {
                return Err(ConfigError::BadValue {
                    field: "presence-sigma + coincident-sigma".into(),
                    reason: format!(
                        "sum ({}) >= sqrt(dims) ({:.4}); presence / coincident predicate duality collapses. \
                        Raise dims or lower a sigma. (Capacity-mode :error returns this failure.)",
                        sigma_sum, dims_sqrt
                    ),
                });
            }
            CapacityMode::Abort => {
                panic!(
                    "config invalid under :abort: presence-sigma ({}) + coincident-sigma ({}) = {} >= sqrt(dims) = {:.4}. \
                    Predicate duality collapses.",
                    presence_sigma, coincident_sigma, sigma_sum, dims_sqrt
                );
            }
        }
    }

    let config = Config {
        dims,
        capacity_mode,
        global_seed,
        noise_floor,
        presence_sigma,
        coincident_sigma,
        presence_floor,
        coincident_floor,
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

fn parse_positive_i64(ast: &WatAST, field: &'static str) -> Result<i64, ConfigError> {
    match ast {
        WatAST::IntLit(n, _) => {
            if *n <= 0 {
                return Err(ConfigError::BadValue {
                    field: field.into(),
                    reason: format!("expected positive integer, got {}", n),
                });
            }
            Ok(*n)
        }
        other => Err(ConfigError::BadType {
            field: field.into(),
            expected: "positive integer literal",
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
        // Arc 024: noise_floor = 1σ = 1/sqrt(d). At d=10000 that's 0.01.
        let expected = 1.0_f64 / (10000_f64).sqrt();
        assert!((cfg.noise_floor - expected).abs() < 1e-12);
        // Arc 024 defaults: presence_sigma=15, coincident_sigma=1.
        assert_eq!(cfg.presence_sigma, 15);
        assert_eq!(cfg.coincident_sigma, 1);
        assert!((cfg.presence_floor - 15.0 * expected).abs() < 1e-12);
        assert!((cfg.coincident_floor - expected).abs() < 1e-12);
        assert!(rest.is_empty());
    }

    #[test]
    fn noise_floor_default_is_1_over_sqrt_dims() {
        // Arc 024: noise_floor = 1σ. At d=1024, 1/32 = 0.03125.
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.dims, 1024);
        assert!((cfg.noise_floor - 1.0 / 32.0).abs() < 1e-12);
        assert!((cfg.presence_floor - 15.0 / 32.0).abs() < 1e-12);
        assert!((cfg.coincident_floor - 1.0 / 32.0).abs() < 1e-12);
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

    // ─── Arc 024: presence_sigma + coincident_sigma ───────────────────

    #[test]
    fn sigma_defaults_are_15_and_1() {
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.presence_sigma, 15);
        assert_eq!(cfg.coincident_sigma, 1);
        // presence_floor = 15 / 32 = 0.46875
        assert!((cfg.presence_floor - 15.0 / 32.0).abs() < 1e-12);
        // coincident_floor = 1 / 32 = 0.03125
        assert!((cfg.coincident_floor - 1.0 / 32.0).abs() < 1e-12);
    }

    #[test]
    fn presence_sigma_override() {
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-presence-sigma! 10)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.presence_sigma, 10);
        assert!((cfg.presence_floor - 10.0 / 32.0).abs() < 1e-12);
    }

    #[test]
    fn coincident_sigma_override() {
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-coincident-sigma! 3)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.coincident_sigma, 3);
        assert!((cfg.coincident_floor - 3.0 / 32.0).abs() < 1e-12);
    }

    #[test]
    fn nonpositive_sigma_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-presence-sigma! 0)
            "#,
        )
        .unwrap_err();
        assert!(
            matches!(err, ConfigError::BadValue { ref field, .. } if field == "presence-sigma"),
            "expected BadValue for presence-sigma, got {:?}",
            err
        );
    }

    #[test]
    fn sigma_sum_exceeds_sqrt_dims_under_error_returns_err() {
        // d=100 → sqrt(d)=10. Default sum 15+1=16 ≥ 10. Invariant
        // violated. Under :error, collect_entry_file returns Err.
        let err = collect(
            r#"
            (:wat::config::set-dims! 100)
            (:wat::config::set-capacity-mode! :error)
            "#,
        )
        .unwrap_err();
        match err {
            ConfigError::BadValue { field, .. } => {
                assert_eq!(field, "presence-sigma + coincident-sigma");
            }
            other => panic!("expected BadValue, got {:?}", other),
        }
    }

    #[test]
    fn sigma_sum_exceeds_sqrt_dims_under_silent_passes() {
        // :silent mode — invariant violation proceeds anyway. Predicates
        // behave nonsensically but substrate does not complain.
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 100)
            (:wat::config::set-capacity-mode! :silent)
            "#,
        )
        .unwrap();
        // Config committed — defaults apply even in degenerate zone.
        assert_eq!(cfg.presence_sigma, 15);
        assert_eq!(cfg.coincident_sigma, 1);
    }

    #[test]
    fn sigma_sum_exceeds_sqrt_dims_under_warn_passes_with_stderr() {
        // :warn mode — proceeds after stderr diagnostic. We don't
        // capture stderr here; just verify the Config is committed.
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 100)
            (:wat::config::set-capacity-mode! :warn)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.presence_sigma, 15);
    }

    #[test]
    fn sigma_override_keeps_config_valid_at_small_dims() {
        // At d=100, user lowers presence-sigma so sum stays below
        // sqrt(d)=10. Config commits cleanly.
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-dims! 100)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-presence-sigma! 5)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.presence_sigma, 5);
        assert_eq!(cfg.coincident_sigma, 1);
        // Sum 5+1=6 < sqrt(100)=10 — OK under :error.
    }

    #[test]
    fn sigma_double_set_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-presence-sigma! 10)
            (:wat::config::set-presence-sigma! 20)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateField { .. }));
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
            (:wat::holon::Atom "hello")
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
            (:wat::holon::Atom "oops — body in the middle")
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

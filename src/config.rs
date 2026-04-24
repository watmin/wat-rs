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
//!   Variants: `:error` / `:abort` (arc 037 retired `:silent` and
//!   `:warn` — overflow either crashes or is handled).
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

/// Committed configuration values.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub dims: usize,
    pub capacity_mode: CapacityMode,
    pub global_seed: u64,
    /// User-supplied dim router AST, captured verbatim at
    /// setter time. Freeze evaluates it against the fully-built
    /// frozen world (arc 009 "names are values" — a keyword-path
    /// lifts to a function value; a lambda expression constructs
    /// one; any AST that reduces to a function works). The result
    /// is wrapped in [`crate::dim_router::WatLambdaRouter`] and
    /// installed as the ambient router. When `None`, freeze
    /// installs the default
    /// `SizingRouter::with_default_tiers()`. Arc 037 slice 4.
    pub dim_router_ast: Option<WatAST>,
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
    /// to fire. Default function of `dims`: `floor(sqrt(dims)/2) − 1`
    /// ("one before the zero point" — the sliver below `middle_width = 0`).
    /// At d=1024 the default is 15; at d=10000 it's 49; at d=100 it's
    /// 4. User overridable via `(:wat::config::set-presence-sigma! <i64>)`.
    pub presence_sigma: i64,
    /// How many σ below perfect-identity `coincident?` requires to fire.
    /// Default 1 — the 1σ native granularity; the geometric minimum.
    /// Constant function of dims (always 1). User overridable via
    /// `(:wat::config::set-coincident-sigma! <i64>)`.
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
    let mut dims: Option<usize> = inherit.map(|c| c.dims);
    let mut capacity_mode: Option<CapacityMode> = inherit.map(|c| c.capacity_mode);
    let mut global_seed: Option<u64> = inherit.map(|c| c.global_seed);
    let mut noise_floor: Option<f64> = inherit.map(|c| c.noise_floor);
    let mut presence_sigma: Option<i64> = inherit.map(|c| c.presence_sigma);
    let mut coincident_sigma: Option<i64> = inherit.map(|c| c.coincident_sigma);
    let mut dim_router_ast: Option<WatAST> = inherit.and_then(|c| c.dim_router_ast.clone());

    // Separate tracker: has this field's setter appeared in THIS forms
    // list? Distinct from `.is_some()` because inheritance pre-seeds
    // the Some. A setter is permitted once per forms list; inheritance
    // does not count as a prior set.
    let mut set_dims = false;
    let mut set_capacity_mode = false;
    let mut set_global_seed = false;
    let mut set_noise_floor = false;
    let mut set_presence_sigma = false;
    let mut set_coincident_sigma = false;
    let mut set_dim_router = false;

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
                if set_dims {
                    return Err(ConfigError::DuplicateField { field: "dims".into() });
                }
                set_dims = true;
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
            ":wat::config::set-noise-floor!" => {
                if set_noise_floor {
                    return Err(ConfigError::DuplicateField {
                        field: "noise-floor".into(),
                    });
                }
                set_noise_floor = true;
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
                presence_sigma = Some(parse_positive_i64(&args[0], "presence-sigma")?);
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
                coincident_sigma = Some(parse_positive_i64(&args[0], "coincident-sigma")?);
            }
            ":wat::config::set-dim-router!" => {
                if set_dim_router {
                    return Err(ConfigError::DuplicateField {
                        field: "dim-router".into(),
                    });
                }
                set_dim_router = true;
                if args.len() != 1 {
                    return Err(ConfigError::BadArity {
                        head: setter_head,
                        expected: 1,
                        got: args.len(),
                    });
                }
                // Arc 037 slice 4: store the AST verbatim. Freeze
                // evaluates it against the fully-built frozen world
                // (names-are-values per arc 009 lifts a keyword path
                // to a function value; a lambda expression constructs
                // one; any AST that reduces to a function works).
                dim_router_ast = Some(args[0].clone());
            }
            _ => {
                return Err(ConfigError::UnknownSetter {
                    head: setter_head,
                });
            }
        }
    }

    // Arc 037 slice 1: dims and capacity_mode are optional. When unset
    // (and no inherited value is present), fall back to opinionated
    // defaults. The existing `RequiredFieldMissing` error variant is
    // retained for potential future required fields but is no longer
    // raised for these two.
    //
    // dims default is inlined (10000) rather than named — the single-
    // dim concept retires when arc 037 slice 3 introduces the router;
    // a constant named `DEFAULT_DIMS` would outlive its meaning.
    let dims = dims.unwrap_or(10000);
    let capacity_mode = capacity_mode.unwrap_or(DEFAULT_CAPACITY_MODE);
    let global_seed = global_seed.unwrap_or(42);
    // Opinionated defaults — all FUNCTIONS of dims. The user picks
    // dims; everything else derives. Arc 024 slice 2.
    //
    //   noise_floor(d)    = 1 / sqrt(d)            ; 1σ native granularity
    //   coincident_sigma  = 1                      ; constant: 1σ always
    //   presence_sigma(d) = floor(sqrt(d)/2) - 1   ; one before zero-point
    //
    // The presence formula derives "the thing one before zero" — the
    // zero-point of middle_width is sqrt(d)/2 (where the two predicates
    // collapse). Presence sits one sliver below, leaving the smallest
    // non-zero separation. Coincident stays at 1σ because that's the
    // geometric minimum the substrate can physically resolve.
    //
    // Validity (presence_sigma + coincident_sigma < sqrt(d)) holds for
    // defaults at d ≥ 16. Below that, the user must override.
    let noise_floor = noise_floor.unwrap_or_else(|| 1.0 / (dims as f64).sqrt());
    let coincident_sigma = coincident_sigma.unwrap_or(1);
    let presence_sigma = presence_sigma.unwrap_or_else(|| {
        let half_sqrt_dims = ((dims as f64).sqrt() / 2.0).floor() as i64;
        // "one before the zero point"; clamp positive so parse invariant
        // still holds for tiny d even though the validity check below
        // will reject.
        (half_sqrt_dims - 1).max(1)
    });
    let presence_floor = (presence_sigma as f64) * noise_floor;
    let coincident_floor = (coincident_sigma as f64) * noise_floor;

    // Validity: n_p + n_c < sqrt(dims). Above this the presence /
    // coincident predicates collapse (their thresholds meet or swap).
    // Behavior per capacity_mode — reuses the same two-mode policy
    // Bundle capacity uses.
    let sigma_sum = presence_sigma.saturating_add(coincident_sigma);
    let dims_sqrt = (dims as f64).sqrt();
    if (sigma_sum as f64) >= dims_sqrt {
        match capacity_mode {
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
        dim_router_ast,
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
            ":error" => Ok(CapacityMode::Error),
            ":abort" => Ok(CapacityMode::Abort),
            other => Err(ConfigError::BadValue {
                field: "capacity-mode".into(),
                reason: format!(
                    "unknown variant {}; expected :error / :abort (arc 037 retired :silent and :warn)",
                    other
                ),
            }),
        },
        other => Err(ConfigError::BadType {
            field: "capacity-mode".into(),
            expected: "keyword (:error / :abort)",
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 10000)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.dims, 10000);
        assert_eq!(cfg.capacity_mode, CapacityMode::Error);
        assert_eq!(cfg.global_seed, 42, "default global-seed is 42");
        // Arc 024: noise_floor = 1σ = 1/sqrt(d). At d=10000 that's 0.01.
        let expected = 1.0_f64 / (10000_f64).sqrt();
        assert!((cfg.noise_floor - expected).abs() < 1e-12);
        // Arc 024 slice 2: defaults are FUNCTIONS of dims.
        //   presence_sigma = floor(sqrt(d)/2) - 1
        //   coincident_sigma = 1
        // At d=10000: floor(100/2) - 1 = 49.
        assert_eq!(cfg.presence_sigma, 49);
        assert_eq!(cfg.coincident_sigma, 1);
        assert!((cfg.presence_floor - 49.0 * expected).abs() < 1e-12);
        assert!((cfg.coincident_floor - expected).abs() < 1e-12);
        assert!(rest.is_empty());
    }

    #[test]
    fn noise_floor_default_is_1_over_sqrt_dims() {
        // Arc 024: noise_floor = 1σ. At d=1024, 1/32 = 0.03125.
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 10000)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
    fn defaults_stay_valid_at_small_dims() {
        // Arc 024 slice 2: the default formula derives presence_sigma
        // from dims. At d=100 → floor(10/2) - 1 = 4; sum 4+1=5 < 10.
        // Valid. The default works wherever the substrate geometry
        // permits.
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 100)
            "#,
        )
        .unwrap();
        assert_eq!(cfg.presence_sigma, 4);
        assert_eq!(cfg.coincident_sigma, 1);
    }

    #[test]
    fn user_override_that_breaks_invariant_under_error_returns_err() {
        // User picks a sigma that violates the geometric ceiling.
        // At d=100 sqrt=10, so presence_sigma=15 + coincident=1 = 16 ≥ 10.
        // Under :error, collect_entry_file returns Err.
        let err = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 100)
            (:wat::config::set-presence-sigma! 15)
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
    fn retired_silent_variant_rejected_at_parse() {
        // Arc 037 retired :silent. Parser errors cleanly.
        let err = collect(
            r#"
            (:wat::config::set-capacity-mode! :silent)
            (:wat::config::set-dims! 100)
            "#,
        )
        .unwrap_err();
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
        // Arc 037 retired :warn. Parser errors cleanly.
        let err = collect(
            r#"
            (:wat::config::set-capacity-mode! :warn)
            (:wat::config::set-dims! 100)
            "#,
        )
        .unwrap_err();
        match err {
            ConfigError::BadValue { field, reason } => {
                assert_eq!(field, "capacity-mode");
                assert!(reason.contains(":warn"), "reason: {}", reason);
            }
            other => panic!("expected BadValue, got {:?}", other),
        }
    }

    #[test]
    fn sigma_override_keeps_config_valid_at_small_dims() {
        // At d=100, user lowers presence-sigma so sum stays below
        // sqrt(d)=10. Config commits cleanly.
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 100)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 10000)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 10000)
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
            (":error", CapacityMode::Error),
            (":abort", CapacityMode::Abort),
        ] {
            let src = format!(
                r#"
                (:wat::config::set-capacity-mode! {})
                (:wat::config::set-dims! 1024)
                "#,
                kw
            );
            let (cfg, _) = collect(&src).unwrap();
            assert_eq!(cfg.capacity_mode, variant, "failed for {}", kw);
        }
    }

    // ─── Error cases ────────────────────────────────────────────────────

    #[test]
    fn setter_after_non_setter_is_silently_dropped() {
        // Walker stops at the first non-setter; the trailing setter is
        // never seen. Pre-arc-037 this surfaced as RequiredFieldMissing
        // because capacity-mode was required. Arc 037 made it optional
        // (defaults to :error), so the walker's silent-drop behavior is
        // directly visible: Config commits with defaults for anything
        // not reached, and the trailing setter is returned as a rest
        // form for downstream processing. A future arc could tighten
        // this by scanning past the non-setter and reporting
        // SetterAfterNonSetter; today the walker is best-effort.
        let (cfg, rest) = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::holon::Atom "oops — body in the middle")
            (:wat::config::set-capacity-mode! :abort)
            "#,
        )
        .unwrap();
        // capacity-mode came from the default (walker never saw the trailing
        // :abort setter), not from the ignored setter below.
        assert_eq!(cfg.capacity_mode, CapacityMode::Error);
        // The ignored setter is present in the rest-forms for the next pass.
        assert_eq!(rest.len(), 2, "Atom + trailing setter both in rest");
    }

    #[test]
    fn duplicate_dims_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 8192)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateField { ref field } if field == "dims"));
    }

    #[test]
    fn duplicate_capacity_mode_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 10000)
            (:wat::config::set-capacity-mode! :abort)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateField { ref field } if field == "capacity-mode"));
    }

    #[test]
    fn missing_dims_defaults() {
        // Arc 037: dims is optional. Its value on the encoder path is
        // irrelevant (router answers on demand). Config commits with
        // whatever the collector's fallback is.
        let (cfg, _) = collect(r#"(:wat::config::set-capacity-mode! :error)"#).unwrap();
        assert_eq!(cfg.capacity_mode, CapacityMode::Error);
    }

    #[test]
    fn missing_capacity_mode_defaults_to_error() {
        // Arc 037: capacity-mode optional; defaults to :error (safe —
        // overflow surfaces as catchable CapacityExceeded).
        let (cfg, _) = collect(r#"(:wat::config::set-dims! 10000)"#).unwrap();
        assert_eq!(cfg.capacity_mode, CapacityMode::Error);
    }

    #[test]
    fn empty_entry_file_commits_defaults() {
        // Arc 037: no setters required. Empty entry file produces a
        // fully-defaulted Config.
        let (cfg, _) = collect("").unwrap();
        assert_eq!(cfg.capacity_mode, CapacityMode::Error);
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 10000 8192)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! "oops")
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::BadType { ref field, .. } if field == "dims"));
    }

    #[test]
    fn capacity_mode_wrong_type_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-capacity-mode! 42)
            (:wat::config::set-dims! 10000)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::BadType { ref field, .. } if field == "capacity-mode"));
    }

    #[test]
    fn capacity_mode_unknown_variant_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-capacity-mode! :chaos)
            (:wat::config::set-dims! 10000)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::BadValue { ref field, .. } if field == "capacity-mode"));
    }

    #[test]
    fn negative_dims_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! -1)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::BadValue { ref field, .. } if field == "dims"));
    }

    #[test]
    fn negative_global_seed_rejected() {
        let err = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 10000)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 10000)
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

    // ─── Arc 031 — collect_entry_file_with_inherit ──────────────────

    fn parent_config() -> Config {
        // A fully-populated parent config for inheritance tests. Built
        // via the non-inheriting collector so the exact default-derivation
        // rules (noise_floor, presence_sigma, coincident_sigma) match
        // what production callers would carry in.
        let (cfg, _) = collect(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            "#,
        )
        .unwrap();
        cfg
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
    fn inherit_with_no_setters_but_body_still_inherits() {
        let parent = parent_config();
        // Body-only form — no setters at all. Would normally error with
        // RequiredFieldMissing; with inheritance, it's fine.
        let (cfg, rest) = collect_inherit(
            r#"
            (:wat::core::define (:user::main -> :())
              ())
            "#,
            &parent,
        )
        .unwrap();
        assert_eq!(cfg.dims, parent.dims);
        assert_eq!(cfg.capacity_mode, parent.capacity_mode);
        assert_eq!(rest.len(), 1);
    }

    #[test]
    fn inherit_setter_overrides_single_field() {
        let parent = parent_config();
        let (cfg, _) = collect_inherit(
            r#"
            (:wat::config::set-dims! 4096)
            "#,
            &parent,
        )
        .unwrap();
        assert_eq!(cfg.dims, 4096, "explicit setter overrides inherited");
        assert_eq!(cfg.capacity_mode, parent.capacity_mode, "unset fields still inherit");
    }

    #[test]
    fn inherit_both_setters_override_everything_explicit() {
        let parent = parent_config();
        // Parent has :error + 1024; forms set :abort + 4096.
        let (cfg, _) = collect_inherit(
            r#"
            (:wat::config::set-capacity-mode! :abort)
            (:wat::config::set-dims! 4096)
            "#,
            &parent,
        )
        .unwrap();
        assert_eq!(cfg.dims, 4096);
        assert_eq!(cfg.capacity_mode, CapacityMode::Abort);
    }

    #[test]
    fn inherit_duplicate_setter_in_forms_still_errors() {
        // Inheritance pre-seeds dims, but that's not a prior "set" in
        // the forms. A single setter overrides cleanly; a SECOND setter
        // for the same field in the forms trips DuplicateField.
        let parent = parent_config();
        let err = collect_inherit(
            r#"
            (:wat::config::set-dims! 4096)
            (:wat::config::set-dims! 8192)
            "#,
            &parent,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateField { ref field, .. } if field == "dims"));
    }

    #[test]
    fn inherit_preserves_derived_fields_when_not_overridden() {
        // Parent at d=1024 has presence_sigma=15, coincident_sigma=1,
        // noise_floor=1/32. Child with no sigma/noise setters should
        // take all three from parent unchanged.
        let parent = parent_config();
        let (cfg, _) = collect_inherit("", &parent).unwrap();
        assert_eq!(cfg.presence_sigma, parent.presence_sigma);
        assert_eq!(cfg.coincident_sigma, parent.coincident_sigma);
        assert!((cfg.noise_floor - parent.noise_floor).abs() < 1e-12);
        assert!((cfg.presence_floor - parent.presence_floor).abs() < 1e-12);
        assert!((cfg.coincident_floor - parent.coincident_floor).abs() < 1e-12);
    }

    #[test]
    fn inherit_dims_override_recomputes_nothing_automatically() {
        // Corollary: overriding dims DOES NOT recompute the
        // noise_floor default. Inheritance carries the parent's
        // noise_floor as-is. If the caller wants the new dims' 1σ
        // floor, they must also set-noise-floor! explicitly. This
        // matches "inheritance is a baseline, setters override per
        // field" semantics — simpler than a recompute-on-cascade
        // rule.
        let parent = parent_config();
        let (cfg, _) = collect_inherit(
            r#"
            (:wat::config::set-dims! 4096)
            "#,
            &parent,
        )
        .unwrap();
        assert_eq!(cfg.dims, 4096);
        assert!(
            (cfg.noise_floor - parent.noise_floor).abs() < 1e-12,
            "noise_floor inherits parent's 1/sqrt(1024), does not recompute for 4096"
        );
    }
}

//! `:wat::uuid::*` intrinsics — arc 255 carve (builder amendment to
//! home #4 phase 2, the string carve): "own home, same shape" as
//! `intrinsic/bytes.rs`. Arc 207 slice 2 minted `:wat::core::Uuid` as a
//! typed primitive; arc 299 slice 1 added the `version`/`rfc4122-variant?`
//! accessors. Seven verbs, self-contained (no separate `src/uuid/`
//! implementation home — each handler's body IS its whole implementation,
//! same shape `bytes.rs` uses).

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};

/// Returns `true` iff `s` is a canonical 8-4-4-4-12 lowercase hyphenated
/// UUID. Exactly matches what `uuid::Uuid::to_string()` produces (and what
/// the `wat-edn` parser + `Uuid/to-string` emit). Rejects uppercase hex,
/// URN prefix, braced form, simple 32-char hex, and any other non-canonical
/// variant. Used by `eval_uuid_from_string` to enforce parse strictness.
fn is_canonical_uuid_string(s: &str) -> bool {
    s.len() == 36
        && s.as_bytes()[8] == b'-'
        && s.as_bytes()[13] == b'-'
        && s.as_bytes()[18] == b'-'
        && s.as_bytes()[23] == b'-'
        && s.chars().enumerate().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                c == '-'
            } else {
                c.is_ascii_hexdigit() && (!c.is_alphabetic() || c.is_ascii_lowercase())
            }
        })
}

/// `(:wat::uuid::v4)` → a fresh, randomly-minted `:wat::core::Uuid`.
///
/// Mints a fresh v4 (random) UUID on every call — different on every
/// invocation, hence Nondeterministic (matches `:wat::time::now`'s
/// Pure∧Nondeterministic∧Entropic shape: no I/O, but the result depends on
/// an external entropy source rather than only its arguments). Arc 207
/// slice 2.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @ret     :wat::core::Uuid a freshly-minted random UUID
/// @example-norun (:wat::uuid::v4) #=> #uuid "a random v4 UUID, different every call"
/// @see     :wat::uuid::v5
#[wat_intrinsic(":wat::uuid::v4")]
pub(crate) fn eval_uuid_v4() -> Result<Value, EvalBreak> {
    Ok(Value::wat__core__Uuid(wat_edn::new_uuid_v4()))
}

/// `(:wat::uuid::v5 ns name)` → a deterministic SHA-1-based
/// `:wat::core::Uuid`.
///
/// `ns` is `:wat::core::Uuid` (type-enforced, eliminating the runtime-panic
/// foot-gun in arc 206's string-typed namespace). Same `(ns, name)` pair
/// always yields the same UUID. Arc 207 slice 2.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     ns   :wat::core::Uuid   the namespace UUID
/// @arg     name :wat::core::String the name, scoped by `ns`
/// @ret     :wat::core::Uuid the deterministic SHA-1-based UUID for `(ns, name)`
/// @example (:wat::uuid::v5 (:wat::uuid::nil) "x") #=> (:wat::uuid::v5 (:wat::uuid::nil) "x")
/// @see     :wat::uuid::v4
#[wat_intrinsic(":wat::uuid::v5")]
pub(crate) fn eval_uuid_v5(
    ns: &WatAST,
    name: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: every error (TypeMismatch) locates at its own arg's span (`ns`'s or `name`'s)
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::uuid::v5";
    let ns_val = eval_inner(ns, env, sym)?.value_owned();
    let ns_uuid = match ns_val {
        Value::wat__core__Uuid(u) => u,
        other => {
            return Err(RuntimeError::new(ns.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid",
                got: Box::new(ValueSnapshot::of(&other)),
            })
            .into());
        }
    };
    let name_val = eval_inner(name, env, sym)?.value_owned();
    let name_str = match &name_val {
        Value::String(s) => s.as_str().to_string(),
        other => {
            return Err(RuntimeError::new(name.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(ValueSnapshot::of(other)),
            })
            .into());
        }
    };
    Ok(Value::wat__core__Uuid(wat_edn::new_uuid_v5(ns_uuid, &name_str)))
}

/// `(:wat::uuid::from-string s)` → `s` parsed as a canonical UUID, or
/// `None`.
///
/// Parse-safe constructor. Accepts ONLY canonical 8-4-4-4-12 lowercase
/// hyphenated form; returns `None` for uppercase, URN prefix, braced,
/// simple (no-hyphen), or otherwise non-canonical input. Arc 207 slice 2.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the candidate UUID text
/// @ret     (:wat::core::Option :- [:wat::core::Uuid]) `Some(u)` iff `s` is a canonical UUID string, `None` otherwise
/// @example (:wat::uuid::from-string "not-a-uuid") #=> :None
/// @see     :wat::uuid::to-string
#[wat_intrinsic(":wat::uuid::from-string")]
pub(crate) fn eval_uuid_from_string(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `s`'s own span; malformed UUID text is a non-error `Ok(None)`
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::uuid::from-string";
    let s_val = eval_inner(s, env, sym)?.value_owned();
    let s_str = match &s_val {
        Value::String(v) => v.as_str().to_string(),
        other => {
            return Err(RuntimeError::new(s.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(ValueSnapshot::of(other)),
            })
            .into());
        }
    };
    let result = if is_canonical_uuid_string(&s_str) {
        uuid::Uuid::parse_str(&s_str).ok().map(Value::wat__core__Uuid)
    } else {
        None
    };
    Ok(Value::Option(Arc::new(result)))
}

/// `(:wat::uuid::to-string u)` → the canonical 8-4-4-4-12 lowercase
/// hyphenated rendering of `u`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     u :wat::core::Uuid the UUID to render
/// @ret     :wat::core::String the canonical 8-4-4-4-12 lowercase hyphenated rendering of `u`
/// @example (:wat::uuid::to-string (:wat::uuid::nil)) #=> "00000000-0000-0000-0000-000000000000"
/// @see     :wat::uuid::from-string
#[wat_intrinsic(":wat::uuid::to-string")]
pub(crate) fn eval_uuid_to_string(
    u: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `u`'s own span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::uuid::to-string";
    let u_val = eval_inner(u, env, sym)?.value_owned();
    let uu = match &u_val {
        Value::wat__core__Uuid(v) => *v,
        other => {
            return Err(RuntimeError::new(u.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid",
                got: Box::new(ValueSnapshot::of(other)),
            })
            .into());
        }
    };
    Ok(Value::String(Arc::new(uu.to_string())))
}

/// `(:wat::uuid::nil)` → the nil UUID
/// (`00000000-0000-0000-0000-000000000000`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @ret     :wat::core::Uuid the nil UUID (`00000000-0000-0000-0000-000000000000`)
/// @example (:wat::uuid::nil) #=> (:wat::uuid::nil)
#[wat_intrinsic(":wat::uuid::nil")]
pub(crate) fn eval_uuid_nil() -> Result<Value, EvalBreak> {
    Ok(Value::wat__core__Uuid(uuid::Uuid::nil()))
}

/// `(:wat::uuid::version u)` → the version nibble of `u`.
///
/// Returns the version nibble as an integer (e.g. `4` for a v4 UUID). Arc
/// 299 slice 1.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Projection
/// @arg     u :wat::core::Uuid the UUID to inspect
/// @ret     :wat::core::i64 the version nibble of `u` (e.g. 4 for a v4 UUID)
/// @example (:wat::uuid::version (:wat::uuid::nil)) #=> 0
#[wat_intrinsic(":wat::uuid::version")]
pub(crate) fn eval_uuid_version(
    u: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `u`'s own span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::uuid::version";
    let u_val = eval_inner(u, env, sym)?.value_owned();
    let uu = match &u_val {
        Value::wat__core__Uuid(v) => *v,
        other => {
            return Err(RuntimeError::new(u.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid",
                got: Box::new(ValueSnapshot::of(other)),
            })
            .into());
        }
    };
    Ok(Value::i64(uu.get_version_num() as i64))
}

/// `(:wat::uuid::rfc4122-variant? u)` → whether `u`'s variant nibble
/// indicates RFC-4122.
///
/// True iff the variant bits are `10xx` (nibble ∈ {8,9,a,b}). Arc 299
/// slice 1.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     u :wat::core::Uuid the UUID to inspect
/// @ret     :wat::core::bool true iff `u`'s variant nibble indicates RFC-4122
/// @example (:wat::uuid::rfc4122-variant? (:wat::uuid::nil)) #=> false
#[wat_intrinsic(":wat::uuid::rfc4122-variant?")]
pub(crate) fn eval_uuid_rfc4122_variant(
    u: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `u`'s own span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::uuid::rfc4122-variant?";
    let u_val = eval_inner(u, env, sym)?.value_owned();
    let uu = match &u_val {
        Value::wat__core__Uuid(v) => *v,
        other => {
            return Err(RuntimeError::new(u.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid",
                got: Box::new(ValueSnapshot::of(other)),
            })
            .into());
        }
    };
    Ok(Value::bool(uu.get_variant() == uuid::Variant::RFC4122))
}

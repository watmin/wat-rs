//! `:wat::string::*` implementation — the NAMESPACE home for the algorithms
//! the `:wat::string::*` REGISTRY home (`src/intrinsic/string.rs`) calls,
//! plus the shared total-string-renderer `str`/`join`/`interpolate` all
//! route through.
//!
//! Two-home split, builder amendment (arc 255 home #4 phase 2, the string
//! carve, mid-flight): `intrinsic/string.rs` holds the dispatch shim + `///`
//! preamble per verb; this module holds the actual algorithm each handler
//! calls — a peer to the `src/` namespace homes that already exist
//! (`collection/`, `channel/`, `stream/`, `value/`, `types/`, …).
//!
//! `src/string_ops.rs` — the pre-discipline loose root file that held all
//! FOUR `:wat::string::*` / `:wat::core::Uuid/*` / `:wat::core::char/*` /
//! `:wat::core::regex::*` families in one 1254-line junk drawer — is
//! retired. Each family now has its own home (`intrinsic/string.rs` +
//! `string/` for the string family; `intrinsic/uuid.rs`, `intrinsic/char.rs`,
//! `intrinsic/regex.rs` — each self-contained, "own home, same shape" as
//! `intrinsic/bytes.rs` — for the other three, none of which had a helper
//! shared with a call site outside their own family).
//!
//! `kebab_to_pascal_with_acronyms` and `render_str_total` live here rather
//! than in `intrinsic/string.rs` for a reason beyond "this is the namespace
//! home": BOTH have callers outside the carved `:wat::string::*` verbs —
//! `types.rs` and `runtime.rs`'s enum-variant renderer call the former;
//! `runtime.rs`'s own `str` verb calls the latter. Moving them into
//! `intrinsic/string.rs` would make `intrinsic/` (a registry-only layer)
//! reach back into by non-registry callers — this module is the shared
//! floor both the registry and the runtime's own non-carved code stand on.
//!
//! `one_string`/`two_strings` (`string_ops.rs`'s old slice-based per-arg
//! helpers) are NOT here — deleted. After all four families moved to
//! fixed-arg `#[wat_intrinsic]` handlers, every one of their call sites
//! (`contains?`/`starts-with?`/`ends-with?`/`length`/`trim`/
//! `to-lowercase`/`to-uppercase`/`pascal->kebab`/`split`/
//! `:wat::regex::matches?`) moved to a handler that type-checks its
//! own named `&WatAST` arg directly (`arg_string` in `intrinsic/string.rs`,
//! or an inline match in `intrinsic/regex.rs` — the arity half of
//! `one_string`/`two_strings`' job is now the `#[wat_intrinsic]` shim's,
//! which runs before any handler is called). Zero remaining callers, so
//! there was nowhere honest to relocate them TO — carrying them here as
//! unused-but-preserved code would be exactly the kind of measured, unforced
//! deletion the accretion discipline in `intrinsic/mod.rs` asks for.

use crate::ast::WatAST;
use crate::runtime::eval;
use crate::value::{Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};

/// `:wat::core::str`'s rendering, factored so `str` and `join` cannot drift
/// (279.3). Total over every `Value`: two arms, both load-bearing.
///
/// - `Value::String` → itself, BARE (no surrounding quotes). The EDN encoder
///   quotes strings; routing a top-level `String` through it would corrupt
///   every caller that expects unquoted text (`(join "-" ["a" "b"])` would
///   render `"\"a\"-\"b\""` instead of `"a-b"`).
/// - everything else → `value_to_edn_string_with`, passed `types` so a
///   record renders by field NAME (`{:x 1}`) rather than positionally
///   (`{:field-0 1}`) — the 296/279.2 fix. Callers with no registry pass
///   `None` explicitly, per `edn::render::value_to_edn_string_with`'s stated discipline.
pub(crate) fn render_str_total(v: &Value, types: Option<&crate::types::TypeEnv>) -> String {
    match v {
        Value::String(s) => (**s).clone(),
        other => crate::edn::render::value_to_edn_string_with(other, types),
    }
}

/// Core algorithm: kebab-case → PascalCase using a known acronym set.
///
/// Splits on `-`; each segment matching a registered acronym
/// (case-insensitive) → the canonical form (e.g. `"acl"` → `"ACL"`); else
/// capitalize (first char upper, rest as-is). Empty acronym set → plain
/// `kebab->pascal` behavior.
pub(crate) fn kebab_to_pascal_with_acronyms(s: &str, acronyms: &[String]) -> String {
    let mut result = String::with_capacity(s.len());
    for segment in s.split('-') {
        let canonical = acronyms.iter().find(|acr| acr.eq_ignore_ascii_case(segment));
        if let Some(acr) = canonical {
            result.push_str(acr);
        } else {
            let mut chars = segment.chars();
            if let Some(first) = chars.next() {
                for uc in first.to_uppercase() {
                    result.push(uc);
                }
                result.push_str(chars.as_str());
            }
        }
    }
    result
}

/// Core algorithm: PascalCase → kebab-case using a known acronym set.
///
/// A registered acronym is ONE segment (e.g. `"ACL"` starts at an uppercase
/// boundary and consumes all its chars as a single token). Capital-boundary
/// for everything else. Empty acronym set → plain `pascal->kebab` behavior.
pub(crate) fn pascal_to_kebab_with_acronyms(s: &str, acronyms: &[String]) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if i > 0 && ch.is_uppercase() {
            let mut matched: Option<usize> = None;
            for acr in acronyms {
                let acr_chars: Vec<char> = acr.chars().collect();
                if chars[i..].len() >= acr_chars.len()
                    && chars[i..i + acr_chars.len()].iter().zip(&acr_chars).all(|(a, b)| a == b)
                {
                    let end = i + acr_chars.len();
                    let at_boundary = end >= chars.len() || chars[end].is_uppercase();
                    if at_boundary {
                        matched = Some(acr_chars.len());
                        break;
                    }
                }
            }
            if let Some(len) = matched {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
                let acr_str: String = chars[i..i + len].iter().collect();
                segments.push(acr_str.to_lowercase());
                i += len;
                continue;
            } else if !current.is_empty() {
                segments.push(std::mem::take(&mut current));
            }
        }
        for lc in ch.to_lowercase() {
            current.push(lc);
        }
        i += 1;
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments.join("-")
}

/// Resolve a keyword argument (used as a namespace) to the registry key
/// form.
///
/// The registry key is the full keyword string with leading colon (e.g.
/// `":my::aws"`). Two value forms are accepted:
/// - `Value::wat__core__keyword(k)` — a keyword in value position (e.g.
///   `:my::aws` literal). The value already carries the leading colon.
/// - `Value::wat__WatAST(WatAST::Keyword(k, _))` — a keyword in value
///   position bound as a macro argument (type `:wat::WatAST`), e.g.
///   `defservice`'s expand-time call.
pub(crate) fn keyword_value_to_registry_key(
    op: &str,
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, RuntimeError> {
    let v = eval(arg, env, sym)?.value_owned();
    match v {
        Value::wat__core__keyword(ref k) => Ok((**k).clone()),
        Value::wat__WatAST(ref ast) => {
            if let WatAST::Keyword(k, _) = ast.as_ref() {
                Ok(k.clone())
            } else {
                Err(RuntimeError::new(
                    arg.span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: op.into(),
                        expected: "keyword",
                        got: Box::new(ValueSnapshot::of(&v)),
                    },
                ))
            }
        }
        ref other => Err(RuntimeError::new(
            arg.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "keyword",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )),
    }
}

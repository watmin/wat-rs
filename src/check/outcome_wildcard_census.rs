//! Excursus 001 item 3 — *an outcome arm cannot be a wildcard*, **PHASE 1: CENSUS ONLY**.
//!
//! A `match` on a FIXED, registered outcome/event enum that carries a `_` wildcard arm is a
//! latent bug: a variant added to the enum later is silently absorbed instead of becoming a
//! checker error. Two stones this session removed such arms by hand
//! (`the-blind-sends-face-their-outcome`, `every-status-arm-is-named`) and **both were found by
//! grepping, and both nearly weren't** — 12 of `wat/service.wat`'s 14 wildcards sit inside
//! quasiquoted macro bodies whose arm heads are unquoted symbols (`~status-stopped-kw`), so no
//! regex can attribute them to an enum.
//!
//! ⭐ **Therefore this lives in the CHECKER, not in `tests/lint/`.** It hooks
//! `infer_match`'s exhaustiveness decision — the one point that already knows the scrutinee's
//! real enum path and that a wildcard was used — and so it sees macro-EXPANDED forms. The cost
//! is that the checker cannot read comments, so a comment cannot ever be the exemption
//! mechanism (see the DESIGN's §Exemptions; phase 1 reports the options, chooses none).
//!
//! ⛔ **Nothing here is an error.** `record_if_enabled()` is a side-channel, off by default
//! (`ENABLED`), that a reporting test switches on. The checker's own diagnostics are
//! byte-identical whether or not the census is running: a wildcard still BLESSES a missing arm.
//! Turning any of this into a hard error is a later ruling the builder has not been asked for.
//!
//! ## What the scope predicate is, stated as a property and not as a path list
//!
//! `classify()` answers from the enum's **path shape and declared variant set**, never from
//! "which file is being checked". Three kinds are IN:
//!
//! 1. a `:wat::`-namespaced enum whose last path segment ends in `Outcome` — the whole
//!    `RecvOutcome` / `SendOutcome` / `TrySendOutcome` / `ConnectOutcome` / `CloseOutcome` /
//!    `StopOutcome` / `GateOutcome` / `CallOutcome` / `ReadlnOutcome` / `AcceptOutcome` /
//!    `SignalOutcome` / `NextOutcome` family, *derived* rather than hand-listed so a new
//!    `*Outcome` minted next month is in scope the day it is declared;
//! 2. `:wat::spawn::ServiceEvent` — the one event enum in the family, named because it does not
//!    carry the `Outcome` suffix;
//! 3. any enum whose last segment is `Status` **and** whose declared variant set is exactly the
//!    six of `defservice`'s generated `Status` (`Started · Stopped · Hibernated · PeersAllowed ·
//!    PeersDenied · Faulted`). `Status` is generated *per service*, so its path is
//!    `:<svc-ns>::Status` and cannot be listed; its SHAPE is fixed, so the shape is the key.
//!    A domain enum that merely happens to be called `Status` has a different variant set and is
//!    therefore OUT — the predicate does not fire on a name alone.
//!
//! Everything else is OUT, and `Verdict::Out` carries the reason so the report can show what was
//! seen-and-classified-out rather than merely not-seen. In particular the per-surface generated
//! enums (`<S>::Reply`, the per-op response enums, `Admin`) are out **by construction**: their
//! variant sets are generated per surface, so "name every variant" has no fixed meaning there and
//! the `_` IS the desync detector (a reply for a *different* op). `Option` / `Result` never reach
//! here at all — they are their own `MatchShape`s, not `MatchShape::Enum`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// The six variants of `defservice`'s generated `Status`, in declaration order
/// (`wat/service.wat`'s `status-enum-def`). Shape, not spelling: this is what makes the
/// per-service `Status` reachable by a rule that cannot know its namespace.
pub const STATUS_SHAPE: [&str; 6] = [
    "Started",
    "Stopped",
    "Hibernated",
    "PeersAllowed",
    "PeersDenied",
    "Faulted",
];

/// `:wat::spawn::ServiceEvent` — the one in-scope enum without the `Outcome` suffix.
pub const SERVICE_EVENT: &str = ":wat::spawn::ServiceEvent";

/// Why a wildcarded enum match is, or is not, one this rule is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The rule fires. The payload names WHICH of the three in-scope kinds matched.
    In(&'static str),
    /// The rule does not fire. The payload names why, so a report can show the control.
    Out(&'static str),
}

impl Verdict {
    pub fn is_in(&self) -> bool {
        matches!(self, Verdict::In(_))
    }
    pub fn why(&self) -> &'static str {
        match self {
            Verdict::In(w) | Verdict::Out(w) => w,
        }
    }
}

/// The scope predicate. `variants` is the enum's DECLARED variant set (from the type
/// environment), needed for the `Status`-by-shape arm.
pub fn classify(enum_path: &str, variants: &[String]) -> Verdict {
    // ⭐ The ONE name grammar. `one_name_grammar` (`tests/lint/one_name_grammar.rs`) caught this
    // line hand-rolling `rsplit("::")` on my first floor — a red that was mine, and a FIX rather
    // than a rune: an enum path IS a wat name, so its leaf must be taken the one way the tree takes
    // a leaf. Two parsers of a name will eventually disagree (arc 109's census found 33 that had).
    let last = wat_reader::identifier::leaf(enum_path);
    if enum_path.starts_with(":wat::") && last.ends_with("Outcome") {
        return Verdict::In("wat-namespaced *Outcome");
    }
    if enum_path == SERVICE_EVENT {
        return Verdict::In("ServiceEvent");
    }
    if last == "Status" {
        let mut got: Vec<&str> = variants.iter().map(String::as_str).collect();
        got.sort_unstable();
        let mut want: Vec<&str> = STATUS_SHAPE.to_vec();
        want.sort_unstable();
        if got == want {
            return Verdict::In("defservice Status shape");
        }
        return Verdict::Out("named Status but NOT the defservice six-variant shape");
    }
    if enum_path.starts_with(":wat::") {
        return Verdict::Out("wat-namespaced, but not an outcome/event enum");
    }
    Verdict::Out("not a fixed registered outcome/event enum (domain, or per-surface generated)")
}

/// One wildcarded `match` over an enum, as the CHECKER saw it.
#[derive(Clone, Debug)]
pub struct WildcardSite {
    /// Full enum path the scrutinee resolved to (`:wat::kernel::RecvOutcome`).
    pub enum_path: String,
    /// Span of the `_` arm itself where one was reached, else the `match` head.
    pub file: String,
    pub line: i64,
    pub col: i64,
    /// Variants with a fully-general arm of their own (narrowed arms are NOT counted — see
    /// `Coverage::EnumVariant { full: false }` in `check.rs`; stated because it bounds the
    /// `missing` column, not the site count).
    pub named: Vec<String>,
    /// Declared variants with no arm — the ones the `_` is currently swallowing. Empty means
    /// the wildcard is redundant TODAY and becomes a swallower the moment a variant is added.
    pub missing: Vec<String>,
    pub verdict: Verdict,
    /// The file the checker was pointed at when this fired. A site whose `file` differs from
    /// its `root` came out of a MACRO EXPANSION — that difference is the whole argument for
    /// choosing the checker over a text lint.
    pub root: Option<String>,
}

static ENABLED: AtomicBool = AtomicBool::new(false);
static SITES: Mutex<Vec<WildcardSite>> = Mutex::new(Vec::new());
static ROOT: Mutex<Option<String>> = Mutex::new(None);

/// Switch collection on. Reporting tests only; the checker's diagnostics do not change.
pub fn enable() {
    ENABLED.store(true, Ordering::Relaxed);
}

/// Switch collection off.
pub fn disable() {
    ENABLED.store(false, Ordering::Relaxed);
}

/// Whether collection is on. Read by `infer_match` BEFORE it builds anything: the census's cost
/// to a normal `wat --check` must be one relaxed atomic load, not an allocation per wildcarded
/// enum match. The type-check path is the floor's dominant cost (`.config/nextest.toml` records
/// the scripts gate as ~98% of the suite's wall-clock), so this is not a hypothetical.
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Label the file the checker is about to be pointed at, so each site can be attributed to
/// the root that triggered it (and a span from a *different* file identified as generated).
pub fn set_root(root: Option<String>) {
    if let Ok(mut g) = ROOT.lock() {
        *g = root;
    }
}

/// Take everything collected so far, leaving the buffer empty.
pub fn drain() -> Vec<WildcardSite> {
    match SITES.lock() {
        Ok(mut g) => std::mem::take(&mut *g),
        Err(_) => Vec::new(),
    }
}

/// The ONE door `infer_match` calls, for EVERY wildcarded match over an enum — in scope or not, so
/// the report can show the out-of-scope population as its own control rather than as an absence.
///
/// The enabled check is FIRST and the registry lookup + variant-name allocation happen only behind
/// it, so a normal `wat --check` pays one relaxed atomic load. Kept here rather than inlined at the
/// call site so `check.rs` gains no nesting and the cost ordering cannot drift.
pub(crate) fn record_if_enabled(
    enum_path: &str,
    span: &crate::span::Span,
    types: &crate::types::TypeEnv,
    covered: &std::collections::HashSet<String>,
) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let Some(crate::types::TypeDef::Enum(e)) = types.get(enum_path) else {
        return;
    };
    let declared: Vec<String> = e
        .variants
        .iter()
        .map(|v| match v {
            crate::types::EnumVariant::Unit(n) => n.clone(),
            crate::types::EnumVariant::Tagged { name, .. } => name.clone(),
        })
        .collect();
    record(enum_path, span, &declared, covered);
}

/// The collection itself, split from the guard so the cheap-path ordering in `record_if_enabled`
/// reads in one glance.
fn record(
    enum_path: &str,
    span: &crate::span::Span,
    declared: &[String],
    covered: &std::collections::HashSet<String>,
) {
    let mut named: Vec<String> = declared
        .iter()
        .filter(|v| covered.contains(v.as_str()))
        .cloned()
        .collect();
    named.sort();
    let missing: Vec<String> = declared
        .iter()
        .filter(|v| !covered.contains(v.as_str()))
        .cloned()
        .collect();
    let verdict = classify(enum_path, declared);
    let root = ROOT.lock().ok().and_then(|g| g.clone());
    if let Ok(mut g) = SITES.lock() {
        g.push(WildcardSite {
            enum_path: enum_path.to_string(),
            file: span.file.as_str().to_string(),
            line: span.line,
            col: span.col,
            named,
            missing,
            verdict,
            root,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_outcome_family_is_in_scope_by_suffix_not_by_list() {
        for p in [
            ":wat::kernel::RecvOutcome",
            ":wat::kernel::SendOutcome",
            ":wat::kernel::TrySendOutcome",
            ":wat::kernel::ConnectOutcome",
            ":wat::kernel::CloseOutcome",
            ":wat::kernel::AcceptOutcome",
            ":wat::kernel::SignalOutcome",
            ":wat::kernel::ReadlnOutcome",
            ":wat::stream::NextOutcome",
            ":wat::service::StopOutcome",
            ":wat::service::GateOutcome",
            ":wat::service::CallOutcome",
        ] {
            assert!(classify(p, &[]).is_in(), "{p} must be in scope");
        }
        assert!(classify(SERVICE_EVENT, &[]).is_in());
    }

    #[test]
    fn a_domain_enum_is_out_even_when_it_is_called_status() {
        assert!(!classify(":my::Color", &v(&["Red", "Green"])).is_in());
        assert!(!classify(":my::PeerKind", &v(&["Thread", "Process"])).is_in());
        // Named Status, different shape → OUT. The name alone never fires the rule.
        assert!(!classify(":my::Status", &v(&["Up", "Down"])).is_in());
    }

    #[test]
    fn the_defservice_status_shape_is_in_scope_whatever_its_namespace() {
        let six = v(&STATUS_SHAPE);
        assert_eq!(
            classify(":tf::svc::Status", &six),
            Verdict::In("defservice Status shape")
        );
        assert_eq!(
            classify(":some::other::service::Status", &six),
            Verdict::In("defservice Status shape")
        );
    }

    #[test]
    fn a_per_surface_reply_enum_is_out_by_construction() {
        assert!(!classify(":tf::svc::Reply", &v(&["Go", "Ping"])).is_in());
        assert!(!classify(":tf::svc::Admin", &v(&["Init", "Stop"])).is_in());
    }
}

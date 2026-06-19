//! Arc 278 Stone P1 — native `WorkingMemory` + the transient/freeze boundary.
//!
//! The mutable mirror of a `:wat::rete::Session` that the fire kernel (P2–P5) mutates
//! during a fire pass. `to_transient` converts a frozen `Session` value into a native
//! `WorkingMemory`; `to_persistent` rebuilds the frozen `Session` from it. The boundary
//! is lossless: `to_persistent(to_transient(s)) == s` for every compiled / fired session.
//!
//! Both functions are `pub(crate)` — the transient mutation is sealed in Rust; no
//! mutation primitive is exposed to the wat language surface. The user calls `fire`
//! (P5), never the transient.
//!
//! ## Session record (7 fields, declaration order — `wat/rete.wat:124-131`)
//! ```text
//! network           <- :wat::core::PersistentMap
//! rules             <- :wat::core::PersistentVector<wat::rete::Rule>
//! alpha-memory      <- :wat::core::PersistentMap
//! beta-memory       <- :wat::core::PersistentMap
//! production-memory <- :wat::core::PersistentMap
//! facts             <- :wat::core::PersistentVector
//! next-id           <- :wat::core::i64
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};
use crate::span::Span;

/// The mutable mirror of a `:wat::rete::Session` — used during the fire pass (P2–P5).
///
/// The three memory maps (`alpha`, `beta`, `production`) are hot, mutated-during-fire
/// structures: native `HashMap<i64, Vec<Value>>` gives O(1) `entry().or_default().push`.
/// `network`/`rules`/`facts`/`next_id` are inputs the fire phase reads but does not
/// restructure — held as-is (passthroughs).
// P1 ships the seam; callers arrive in P2–P5 (the fire kernel). Suppress dead-code
// warnings for this stone so the project warning count stays at the known baseline.
#[allow(dead_code)]
pub(crate) struct WorkingMemory {
    /// Passthrough — immutable input: node-id → Node network.
    pub(crate) network:    Value,
    /// Passthrough — immutable input: ordered rule vector.
    pub(crate) rules:      Value,
    /// Mutable mirror of `alpha-memory`  (node-id → [Element]).
    pub(crate) alpha:      HashMap<i64, Vec<Value>>,
    /// Mutable mirror of `beta-memory`   (node-id → [Token]).
    pub(crate) beta:       HashMap<i64, Vec<Value>>,
    /// Mutable mirror of `production-memory` (node-id → [Record]).
    pub(crate) production: HashMap<i64, Vec<Value>>,
    /// Passthrough — the asserted fact PersistentVector.
    pub(crate) facts:      Value,
    /// Passthrough — monotonically increasing fact/node id counter.
    pub(crate) next_id:    i64,
}

// ─── Memory conversion helpers ────────────────────────────────────────────────

// P1 ships the seam; callers arrive in P2–P5. Suppress dead-code lints for this stone.
#[allow(dead_code)]
/// Convert a `Value::wat__core__PersistentMap` whose keys are `Value::i64` and whose
/// values are `Value::wat__core__PersistentVector` into a `HashMap<i64, Vec<Value>>`.
///
/// A malformed key (not `Value::i64`) or a malformed value (not
/// `Value::wat__core__PersistentVector`) → `RuntimeError::TypeMismatch`; entries are
/// never silently dropped.
fn pm_to_hashmap(op: &'static str, pm: &Value) -> Result<HashMap<i64, Vec<Value>>, EvalBreak> {
    match pm {
        Value::wat__core__PersistentMap(m) => {
            let mut out: HashMap<i64, Vec<Value>> = HashMap::with_capacity(m.size());
            for (k, v) in m.iter() {
                let node_id = match k {
                    Value::i64(n) => *n,
                    other => {
                        return Err(RuntimeError {
                            span: Span::unknown(),
                            kind: RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "node-id key :wat::core::i64",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        }
                        .into());
                    }
                };
                let vec = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        pv.iter().cloned().collect::<Vec<Value>>()
                    }
                    other => {
                        return Err(RuntimeError {
                            span: Span::unknown(),
                            kind: RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "memory value :wat::core::PersistentVector",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        }
                        .into());
                    }
                };
                out.insert(node_id, vec);
            }
            Ok(out)
        }
        other => Err(RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::PersistentMap (a session memory)",
                got: Box::new(ValueSnapshot::of(other)),
            },
        }
        .into()),
    }
}

// P1 ships the seam; callers arrive in P2–P5. Suppress dead-code lints for this stone.
#[allow(dead_code)]
/// Convert a `HashMap<i64, Vec<Value>>` back into a
/// `Value::wat__core__PersistentMap<i64, PersistentVector<Value>>`.
fn hashmap_to_pm(map: HashMap<i64, Vec<Value>>) -> Value {
    let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (node_id, vec) in map {
        let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        for v in vec {
            pv = pv.push_back(v);
        }
        pm = pm.insert(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
    }
    Value::wat__core__PersistentMap(pm)
}

// ─── Public boundary ──────────────────────────────────────────────────────────

// P1 ships the seam; callers arrive in P2–P5. Suppress dead-code lints for this stone.
#[allow(dead_code)]
/// Convert a frozen `:wat::rete::Session` `Value` into a mutable `WorkingMemory`.
///
/// Reads `struct_form` positions 0..7 in declaration order:
/// `network, rules, alpha-memory, beta-memory, production-memory, facts, next-id`.
///
/// Returns `RuntimeError::TypeMismatch` if:
/// - the value is not a `Value::wat__Record` with `class_fqdn == "wat::rete::Session"`,
/// - any of the three memory fields is not a `Value::wat__core__PersistentMap`,
/// - any memory key is not `Value::i64`, or
/// - any memory value is not a `Value::wat__core__PersistentVector`.
///
/// Never panics.
pub(crate) fn to_transient(session: &Value) -> Result<WorkingMemory, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient";
    let (class_fqdn, struct_form) = match session {
        Value::wat__Record { class_fqdn, struct_form } => (class_fqdn, struct_form),
        other => {
            return Err(RuntimeError {
                span: Span::unknown(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::rete::Session (a wat::Record)",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            }
            .into());
        }
    };
    if class_fqdn.as_str() != "wat::rete::Session" {
        return Err(RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Session",
                got: Box::new(ValueSnapshot::of(session)),
            },
        }
        .into());
    }
    let sf = struct_form.as_slice();
    // Declaration order: network(0) rules(1) alpha-memory(2) beta-memory(3)
    //                    production-memory(4) facts(5) next-id(6)
    let network    = sf[0].clone();
    let rules      = sf[1].clone();
    let alpha_pm   = &sf[2];
    let beta_pm    = &sf[3];
    let prod_pm    = &sf[4];
    let facts      = sf[5].clone();
    let next_id    = match &sf[6] {
        Value::i64(n) => *n,
        other => {
            return Err(RuntimeError {
                span: Span::unknown(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "next-id :wat::core::i64",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            }
            .into());
        }
    };

    let alpha      = pm_to_hashmap(OP, alpha_pm)?;
    let beta       = pm_to_hashmap(OP, beta_pm)?;
    let production = pm_to_hashmap(OP, prod_pm)?;

    Ok(WorkingMemory { network, rules, alpha, beta, production, facts, next_id })
}

// P1 ships the seam; callers arrive in P2–P5. Suppress dead-code lints for this stone.
#[allow(dead_code)]
/// Convert a `WorkingMemory` back into a frozen `:wat::rete::Session` `Value`.
///
/// Rebuilds each memory `HashMap<i64,Vec<Value>>` into a `PersistentMap<i64,PersistentVector<Value>>`,
/// then constructs a `Value::wat__Record` with `struct_form` in declaration order:
/// `[network, rules, alpha-memory, beta-memory, production-memory, facts, next-id]`.
///
/// An empty memory map → an empty `PersistentMap` (never `nil`; the field is always present).
pub(crate) fn to_persistent(wm: WorkingMemory) -> Value {
    let alpha_pm   = hashmap_to_pm(wm.alpha);
    let beta_pm    = hashmap_to_pm(wm.beta);
    let prod_pm    = hashmap_to_pm(wm.production);

    Value::wat__Record {
        class_fqdn: Arc::new("wat::rete::Session".into()),
        struct_form: Arc::new(vec![
            wm.network,
            wm.rules,
            alpha_pm,
            beta_pm,
            prod_pm,
            wm.facts,
            Value::i64(wm.next_id),
        ]),
    }
}

// ─── Round-trip unit tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{to_persistent, to_transient};
    use std::sync::Arc;
    use crate::freeze::{eval_in_frozen, startup_from_source};
    use crate::load::InMemoryLoader;
    use crate::runtime::{Environment, Value};

    /// The cold-and-windy world: Temperature + WindSpeed + ColdAndWindy records + the rule.
    const WORLD: &str = "\
(:wat::Record::def :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :weather::ColdAndWindy [location <- :wat::core::String])\n\
\n\
(:wat::rete::defrule :weather::cold-and-windy\n\
  :when\n\
  [(:weather::Temperature\n\
     (?loc <- :location)\n\
     (?c   <- :celsius)\n\
     (:wat::core::< ?c 20))\n\
   (:weather::WindSpeed\n\
     (?loc <- :location)\n\
     (?k   <- :kph)\n\
     (:wat::core::> ?k 30))]\n\
  :then\n\
  (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\n\
\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

    /// Eval a `src` expression in the cold-and-windy frozen world; panics on error.
    fn ev(src: &str) -> Value {
        let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let ast = crate::parse_one!(src).expect("parse");
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
            .value_owned()
    }

    /// Round-trip a fired `Session` (populated alpha/beta/production memories).
    /// `to_persistent(to_transient(fired)) == fired`.
    #[test]
    fn round_trip_fired_session() {
        // Build a fired session through the oracle: collect → compile → insert × 2 → fire-rules.
        let fired = ev(
            "(:wat::core::let \
               [rules   (:wat::rete::collect-rules :weather)\
                s0      (:wat::rete::compile rules)\
                s1      (:wat::rete::insert s0 (:weather::Temperature 15 \"Oslo\"))\
                s2      (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))]\
              (:wat::rete::fire-rules s2))",
        );

        let wm = to_transient(&fired).expect("to_transient should succeed on a valid Session");
        let back = to_persistent(wm);
        assert_eq!(back, fired, "round-trip identity: to_persistent(to_transient(fired)) == fired");
    }

    /// Round-trip a freshly-compiled (empty-memory) `Session`.
    /// `to_persistent(to_transient(compiled)) == compiled`.
    #[test]
    fn round_trip_empty_session() {
        let compiled = ev(
            "(:wat::rete::compile (:wat::rete::collect-rules :weather))",
        );

        let wm = to_transient(&compiled).expect("to_transient should succeed on a compiled Session");
        let back = to_persistent(wm);
        assert_eq!(back, compiled, "round-trip identity: to_persistent(to_transient(compiled)) == compiled");
    }

    /// `to_transient` on a non-Session value → TypeMismatch, not panic.
    #[test]
    fn type_mismatch_not_panic() {
        let not_a_session = Value::i64(42);
        let result = to_transient(&not_a_session);
        assert!(result.is_err(), "to_transient on a non-Session value must return Err");
    }

    /// `to_transient` on a wrong record class → TypeMismatch.
    #[test]
    fn wrong_record_class_type_mismatch() {
        let wrong = Value::wat__Record {
            class_fqdn: Arc::new("weather::Temperature".into()),
            struct_form: Arc::new(vec![Value::i64(15), Value::String(Arc::new("Oslo".into()))]),
        };
        let result = to_transient(&wrong);
        assert!(result.is_err(), "to_transient on a non-Session record must return Err");
    }
}

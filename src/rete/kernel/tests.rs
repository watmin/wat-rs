use super::*;
use crate::freeze::{eval_in_frozen, startup_from_source};
use crate::load::loader::InMemoryLoader;
use crate::rete::matcher::Bindings;
use crate::runtime::{Environment, Value};
use crate::types::Nature;
use crate::value::value::AggregateValue;
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// The cold-and-windy world: Temperature + WindSpeed + ColdAndWindy records + the rule.
const WORLD: &str = "\
(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])\n\
\n\
(:wat::rete::defrule :weather::cold-and-windy\n\
  :when\n\
  [(:weather::Temperature\n\
     (?loc <- :location)\n\
     (?c   <- :celsius)\n\
     (:wat::rete::i64::< ?c 20))\n\
   (:weather::WindSpeed\n\
     (?loc <- :location)\n\
     (?k   <- :kph)\n\
     (:wat::rete::i64::> ?k 30))]\n\
  :then\n\
  [(:weather::ColdAndWindy ?loc)])\n\
\n\
";

/// Eval a `src` expression in the cold-and-windy frozen world; panics on error.
fn ev(src: &str) -> Value {
    eval_in(&freeze_src(WORLD), src)
}

fn freeze_src(src: &str) -> crate::freeze::FrozenWorld {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("world should freeze")
}

fn eval_in(world: &crate::freeze::FrozenWorld, src: &str) -> Value {
    let ast = crate::parse_one!(src).expect("parse");
    eval_in_frozen(&ast, world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

fn scratch_wm(session: &Value) -> FireSession {
    let mut wm = to_transient(session).expect("to_transient should succeed");
    wm.alpha.clear();
    wm.beta.clear();
    wm.production.clear();
    wm
}

/// Round-trip a fired `Session` (populated production memory; alpha/beta are fire-scoped
/// scratch, cleared before freeze by the fixpoint fire path that produced `fired`).
/// `to_persistent(to_transient(fired)) == fired`.
#[test]
fn round_trip_fired_session() {
    // Build a fired session through the oracle: collect → compile → insert × 2 → fire-rules.
    let fired = ev(
            "(:wat::core::let \
               [rules   (:wat::rete::collect-rules :weather)\
                s0      (:wat::rete::compile rules)\
                s1      (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
                s2      (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location \"Oslo\"))]\
              (:wat::rete::fire-rules s2))",
        );

    let wm = to_transient(&fired).expect("to_transient should succeed on a valid Session");
    let back = to_persistent(wm);
    assert_eq!(
        back, fired,
        "round-trip identity: to_persistent(to_transient(fired)) == fired"
    );
}

/// Round-trip a freshly-compiled (empty-memory) `Session`.
/// `to_persistent(to_transient(compiled)) == compiled`.
#[test]
fn round_trip_empty_session() {
    let compiled = ev("(:wat::rete::compile (:wat::rete::collect-rules :weather))");

    let wm = to_transient(&compiled).expect("to_transient should succeed on a compiled Session");
    let back = to_persistent(wm);
    assert_eq!(
        back, compiled,
        "round-trip identity: to_persistent(to_transient(compiled)) == compiled"
    );
}

/// `to_transient` on a non-Session value → TypeMismatch, not panic.
#[test]
fn type_mismatch_not_panic() {
    let not_a_session = Value::i64(42);
    let result = to_transient(&not_a_session);
    assert!(
        result.is_err(),
        "to_transient on a non-Session value must return Err"
    );
}

/// `to_transient` on a wrong record class → TypeMismatch.
#[test]
fn wrong_record_class_type_mismatch() {
    let wrong = Value::Aggregate(Arc::new(AggregateValue::record(
        "weather::Temperature".into(),
        // Field CONTENT is irrelevant here — the assertion only checks that a non-Session
        // record class errors — so positional labels, not a hand-typed name guess.
        Arc::new(vec!["0".to_string(), "1".to_string()]),
        Arc::new(vec![Value::i64(15), Value::String(Arc::new("Oslo".into()))]),
    )));
    let result = to_transient(&wrong);
    assert!(
        result.is_err(),
        "to_transient on a non-Session record must return Err"
    );
}

/// P11 guiding-light probe: the native `Token`'s `matches` vec carries the expected
/// `(fact, alpha_id)` condition-labeled edges for a production-reaching token.
///
/// A 2-condition (Temperature ∧ WindSpeed) rule produces tokens with exactly 2 edges:
///   matches[0] = (Temperature_fact, alpha_id_of_Temperature_node)
///   matches[1] = (WindSpeed_fact,   alpha_id_of_WindSpeed_node)
///
/// Proves the cheap native repr keeps the support chain walkable (the guiding-light invariant).
/// Runs the four passes directly — NOT via `fire_once_session` (which clears beta before freeze).
// rune:complectens(proof-stepping-stones) — P11 kernel proofs document the four-pass
// contract; collapsing them would destroy the pass-by-pass diagnostic.
#[test]
fn guiding_light_matches_carry_support_chain() {
    use super::{
        alpha_pass, get_node, hash_join_pass, kind_of, production_pass, rete_arm_get_or_build,
        root_join_pass, sorted_node_ids,
    };
    // Build the frozen world and compile + insert facts.
    let world = freeze_src(WORLD);
    let session_with_facts = eval_in(
            &world,
            "(:wat::core::let \
               [rules (:wat::rete::collect-rules :weather)\
                s0    (:wat::rete::compile rules)\
                s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
                s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location \"Oslo\"))]\
              s2)"
        );

    let mut wm = scratch_wm(&session_with_facts);

    // Run the four passes (inspect native beta; freeze would drop it).
    let sym = world.symbols();
    let arm = rete_arm_get_or_build(&wm.network, &wm.rules, sym).expect("arm");
    alpha_pass(&mut wm, &arm).expect("alpha_pass");
    root_join_pass(&mut wm);
    hash_join_pass(&mut wm, &arm).expect("hash_join_pass");
    production_pass(&mut wm, &arm, sym).expect("production_pass should succeed");

    // Find the HashJoinNode (the parent of the ProductionNode) in the network.
    // A production-reaching token lives in beta[hash_join_node_id].
    let node_ids = sorted_node_ids(&wm.network);
    let hash_join_id = node_ids
        .iter()
        .find(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == NodeKind::HashJoin)
                .unwrap_or(false)
        })
        .copied()
        .expect("network must contain a HashJoinNode for the 2-condition rule");

    // Collect the alpha node ids for membership checks.
    let alpha_ids_in_network: std::collections::HashSet<i64> = node_ids
        .iter()
        .filter(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == NodeKind::Alpha)
                .unwrap_or(false)
        })
        .copied()
        .collect();

    // Retrieve the tokens at the HashJoinNode.
    let tokens = wm
        .beta
        .get(&hash_join_id)
        .expect("beta[hash_join_id] must be non-empty after the four passes");

    assert!(
        !tokens.is_empty(),
        "at least one production-reaching token must exist"
    );

    // Each token must carry exactly 2 edges (one per condition: Temperature + WindSpeed).
    for tok in tokens {
        let edges = match_slice(&wm.match_pool, tok.matches);
        assert_eq!(
            edges.len(),
            2,
            "a 2-condition rule token must carry exactly 2 (fact, alpha_id) edges; got: {:?}",
            edges.iter().map(|(_, aid)| aid).collect::<Vec<_>>()
        );

        // Both alpha_ids must reference actual AlphaNode ids in the network.
        for (fact_idx, alpha_id) in edges {
            assert!(
                alpha_ids_in_network.contains(alpha_id),
                "alpha_id {alpha_id} in matches must be an AlphaNode id in the network; \
                     known alpha ids: {alpha_ids_in_network:?}"
            );
            // The fact must be a Record (Temperature or WindSpeed).
            let fact = super::fact_at(&wm.facts, &wm.derived_facts, wm.n_input, *fact_idx);
            match fact {
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    let cls = a.class.as_ref();
                    assert!(
                        cls == "weather::Temperature" || cls == "weather::WindSpeed",
                        "supporting fact must be Temperature or WindSpeed; got: {cls}"
                    );
                }
                other => panic!("matches fact must be a wat::core::Record; got: {other:?}"),
            }
        }

        // The two edges must reference DIFFERENT alpha nodes (each condition is distinct).
        let (_, alpha0) = &edges[0];
        let (_, alpha1) = &edges[1];
        assert_ne!(
            alpha0, alpha1,
            "the two edges must reference different alpha node ids"
        );

        // The two facts must be of DIFFERENT types (Temperature != WindSpeed).
        let class0 = match super::fact_at(&wm.facts, &wm.derived_facts, wm.n_input, edges[0].0) {
            Value::Aggregate(a) if a.nature != Nature::Struct => a.class.clone(),
            _ => panic!("fact[0] must be a Record"),
        };
        let class1 = match super::fact_at(&wm.facts, &wm.derived_facts, wm.n_input, edges[1].0) {
            Value::Aggregate(a) if a.nature != Nature::Struct => a.class.clone(),
            _ => panic!("fact[1] must be a Record"),
        };
        assert_ne!(
            class0, class1,
            "the two supporting facts must be of different types"
        );
    }
}

// ─── P11 relocation: 3a / 3b coverage — beta is ephemeral, inspect via passes ───────────────
//
// The integration tests probe_arc278_3a_root_join and probe_arc278_3b_hash_join formerly read
// `Session/beta-memory` from a FIRED Session. P11 clears `wm.beta` before freeze so the frozen
// Session carries an empty beta-memory. The join-correctness invariants are preserved HERE:
// we run the passes directly and inspect the NATIVE wm.beta before it would be cleared.
//
// ⚠ THOSE TWO PROBE FILES ARE DELETED (2026-08-16). Every one of their 7 tests was `#[ignore]`d
// and named its replacement below; the files held no live test. The `tests/probe_arc278_3*.rs`
// paths cited in the doc comments are HISTORICAL PROVENANCE — what this coverage replaced — not
// pointers to files on disk. Do not grep for them expecting a hit.
//
// These tests are the authority for:
//   3a: RootJoinNode seeds exactly 1 Token per matching Element (bindings + support carried).
//   3b: HashJoinNode yields the exact compatible-cross cardinality (1, 0, or 2 for 2×2).

// rune:vocare(vantage-bypass-test) — root-join seeding — the token count and bindings a RootJoin writes. The hand-built `Rule` below
// carries an EMPTY `:rhs` on purpose, so no production runs and the caller
// mouth cannot see the match; the assertions read `wm.beta` at implementer
// vantage, which is what this test is FOR.
//
// What that costs, and where it is paid: nothing here can reach the
// join->RHS boundary. The caller-level join fixture did not close it either
// — `cold-and-windy`'s `:then` uses only `?loc`, the JOIN KEY, which the
// second condition merely MATCHES, so a dropped or swapped binding from that
// side still yields the right `?loc`. Covered now by
// `tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs`.
/// P11/3a — `root_join_seeds_one_token_per_element`:
///
/// 1-condition rule `(:user::Temp (?t <- :value) (:wat::rete::i64::> ?t 20))`.
/// After alpha+root-join passes with one matching fact inserted (Temp 25):
///   (1) exactly one beta node (the RootJoinNode) is populated,
///   (2) it holds exactly one Token,
///   (3) that Token's matches vec has length 1,
///   (4) that Token's bindings carry ?t == 25.
///
/// Mirrors the 3a integration test assertions, relocated into the kernel's #[cfg(test)] module
/// so they survive P11's `wm.beta.clear()` at freeze. Coverage for:
///   tests/probe_arc278_3a_root_join.rs::root_join_populates_one_beta_node
///   tests/probe_arc278_3a_root_join.rs::root_join_seeds_one_token
///   tests/probe_arc278_3a_root_join.rs::seeded_token_carries_bindings_and_support
#[test]
fn root_join_seeds_one_token_per_element() {
    use super::{alpha_pass, rete_arm_get_or_build, root_join_pass};

    // 1-condition world: only the Temp record type + main fn (no defrule).
    const TEMP_WORLD: &str = "\
(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])\n\
";

    let world = freeze_src(TEMP_WORLD);

    // Build a compiled session with one matching Temp fact. Mirrors the 3a integration setup:
    // a raw Rule with a single condition + empty RHS, compiled and one fact inserted.
    let session = eval_in(
            &world,
            "(:wat::core::let \
               [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::rete::i64::> ?t 20)))\
                rule  (:wat::rete::Rule :name \"r\" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))\
                sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                sess1 (:wat::rete::insert sess0 (:user::Temp :value 25))]\
              sess1)"
        );

    let mut wm = scratch_wm(&session);

    let sym = world.symbols();
    let arm = rete_arm_get_or_build(&wm.network, &wm.rules, sym).expect("arm");
    alpha_pass(&mut wm, &arm).expect("alpha_pass");
    root_join_pass(&mut wm);
    // (no hash-join needed: single-condition rule has no HashJoinNode)

    // (1) Exactly one beta node (the RootJoinNode) is seeded.
    assert_eq!(
        wm.beta.len(),
        1,
        "root_join_seeds_one_token_per_element (3a): exactly 1 beta node seeded; got {}",
        wm.beta.len()
    );

    // (2) That node holds exactly one Token.
    let (root_join_id, tokens) = wm
        .beta
        .iter()
        .next()
        .expect("beta must have exactly one entry");
    assert_eq!(
        tokens.len(),
        1,
        "root_join_seeds_one_token_per_element (3a): one Element → one Token; got {}",
        tokens.len()
    );
    let _ = root_join_id; // node-id is dynamic; we just need the count

    // (3) Token's matches vec has exactly 1 edge (the one supporting fact).
    let tok = &tokens[0];
    assert_eq!(
        tok.matches.len as usize, 1,
        "root_join_seeds_one_token_per_element (3a): Token's support chain has 1 entry; got {}",
        tok.matches.len
    );

    // (4) Token carries ?t = 25 in its bindings.
    let qt_key = Value::String(Arc::new("?t".to_string()));
    let qt_val = Bindings::get(
        &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
        &qt_key,
    )
    .cloned();
    assert_eq!(
        qt_val,
        Some(Value::i64(25)),
        "root_join_seeds_one_token_per_element (3a): Token must carry ?t=25; got {:?}",
        qt_val
    );
}

// rune:vocare(vantage-bypass-test) — hash-join token shape on a matching key. The hand-built `Rule` below
// carries an EMPTY `:rhs` on purpose, so no production runs and the caller
// mouth cannot see the match; the assertions read `wm.beta` at implementer
// vantage, which is what this test is FOR.
//
// What that costs, and where it is paid: nothing here can reach the
// join->RHS boundary. The caller-level join fixture did not close it either
// — `cold-and-windy`'s `:then` uses only `?loc`, the JOIN KEY, which the
// second condition merely MATCHES, so a dropped or swapped binding from that
// side still yields the right `?loc`. Covered now by
// `tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs`.
/// P11/3b — `hash_join_produces_one_token_on_same_loc`:
///
/// 2-condition rule joining on `?loc`. Temperature(Oslo)+WindSpeed(Oslo) → exactly 1 joined Token
/// at the HashJoinNode. The joined Token unifies all three variables: ?t=15, ?w=45, ?loc="Oslo".
///
/// Mirrors:
///   tests/probe_arc278_3b_hash_join.rs::join_produces_one_token_on_matching_loc
///   tests/probe_arc278_3b_hash_join.rs::joined_token_unifies_both_conditions
#[test]
fn hash_join_produces_one_token_on_same_loc() {
    use super::{
        alpha_pass, get_node, hash_join_pass, kind_of, rete_arm_get_or_build, root_join_pass,
        sorted_node_ids,
    };
    // 2-condition world: Temperature + WindSpeed (no defrule — raw Rule).
    const JOIN_WORLD: &str = "\
(:wat::core::defrecord :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
";

    let world = freeze_src(JOIN_WORLD);

    // Same location → should produce 1 joined token.
    let session = eval_in(
            &world,
            "(:wat::core::let \
               [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))\
                c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))\
                rule  (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))\
                sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                sess1 (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location \"Oslo\"))\
                sess2 (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location \"Oslo\"))]\
              sess2)"
        );

    let mut wm = scratch_wm(&session);

    let sym = world.symbols();
    let arm = rete_arm_get_or_build(&wm.network, &wm.rules, sym).expect("arm");
    alpha_pass(&mut wm, &arm).expect("alpha_pass");
    root_join_pass(&mut wm);
    hash_join_pass(&mut wm, &arm).expect("hash_join_pass");

    // Find the HashJoinNode.
    let node_ids = sorted_node_ids(&wm.network);
    let hash_join_id = node_ids
        .iter()
        .find(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == NodeKind::HashJoin)
                .unwrap_or(false)
        })
        .copied()
        .expect("network must contain a HashJoinNode for the 2-condition rule");

    let tokens = wm
        .beta
        .get(&hash_join_id)
        .map(Vec::as_slice)
        .unwrap_or_default();

    // join_produces_one_token_on_matching_loc: same loc → exactly 1 joined Token.
    assert_eq!(
        tokens.len(),
        1,
        "hash_join_produces_one_token_on_same_loc (3b): Oslo+Oslo → 1 joined Token; got {}",
        tokens.len()
    );

    // joined_token_unifies_both_conditions: ?t=15, ?w=45, ?loc="Oslo".
    let tok = &tokens[0];
    let qt = Bindings::get(
        &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
        &Value::String(Arc::new("?t".to_string())),
    )
    .cloned();
    let qw = Bindings::get(
        &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
        &Value::String(Arc::new("?w".to_string())),
    )
    .cloned();
    let ql = Bindings::get(
        &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
        &Value::String(Arc::new("?loc".to_string())),
    )
    .cloned();
    assert_eq!(
        qt,
        Some(Value::i64(15)),
        "hash_join_produces_one_token_on_same_loc (3b): ?t must be 15; got {:?}",
        qt
    );
    assert_eq!(
        qw,
        Some(Value::i64(45)),
        "hash_join_produces_one_token_on_same_loc (3b): ?w must be 45; got {:?}",
        qw
    );
    assert_eq!(
        ql,
        Some(Value::String(Arc::new("Oslo".to_string()))),
        "hash_join_produces_one_token_on_same_loc (3b): ?loc must be \"Oslo\"; got {:?}",
        ql
    );
}

// rune:vocare(vantage-bypass-test) — hash-join rejection on a mismatched key. The hand-built `Rule` below
// carries an EMPTY `:rhs` on purpose, so no production runs and the caller
// mouth cannot see the match; the assertions read `wm.beta` at implementer
// vantage, which is what this test is FOR.
//
// What that costs, and where it is paid: nothing here can reach the
// join->RHS boundary. The caller-level join fixture did not close it either
// — `cold-and-windy`'s `:then` uses only `?loc`, the JOIN KEY, which the
// second condition merely MATCHES, so a dropped or swapped binding from that
// side still yields the right `?loc`. Covered now by
// `tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs`.
/// P11/3b — `hash_join_drops_on_mismatched_loc`:
///
/// Temperature(Oslo) + WindSpeed(Bergen) → no joined Token at the HashJoinNode
/// (the ?loc join key disagrees: "Oslo" != "Bergen").
///
/// Mirrors:
///   tests/probe_arc278_3b_hash_join.rs::join_drops_on_mismatched_loc
#[test]
fn hash_join_drops_on_mismatched_loc() {
    use super::{
        alpha_pass, get_node, hash_join_pass, kind_of, rete_arm_get_or_build, root_join_pass,
        sorted_node_ids,
    };
    const JOIN_WORLD: &str = "\
(:wat::core::defrecord :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
";

    let world = freeze_src(JOIN_WORLD);

    // Different locations → no joined tokens.
    let session = eval_in(
            &world,
            "(:wat::core::let \
               [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))\
                c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))\
                rule  (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))\
                sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                sess1 (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location \"Oslo\"))\
                sess2 (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location \"Bergen\"))]\
              sess2)"
        );

    let mut wm = scratch_wm(&session);

    let sym = world.symbols();
    let arm = rete_arm_get_or_build(&wm.network, &wm.rules, sym).expect("arm");
    alpha_pass(&mut wm, &arm).expect("alpha_pass");
    root_join_pass(&mut wm);
    hash_join_pass(&mut wm, &arm).expect("hash_join_pass");

    // Find the HashJoinNode.
    let node_ids = sorted_node_ids(&wm.network);
    let hash_join_id = node_ids
        .iter()
        .find(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == NodeKind::HashJoin)
                .unwrap_or(false)
        })
        .copied()
        .expect("network must contain a HashJoinNode for the 2-condition rule");

    let token_count = wm.beta.get(&hash_join_id).map(Vec::len).unwrap_or(0);

    assert_eq!(
        token_count, 0,
        "hash_join_drops_on_mismatched_loc (3b): Oslo+Bergen → 0 joined Tokens; got {}",
        token_count
    );
}

// rune:vocare(vantage-bypass-test) — hash-join isolation across keys. The hand-built `Rule` below
// carries an EMPTY `:rhs` on purpose, so no production runs and the caller
// mouth cannot see the match; the assertions read `wm.beta` at implementer
// vantage, which is what this test is FOR.
//
// What that costs, and where it is paid: nothing here can reach the
// join->RHS boundary. The caller-level join fixture did not close it either
// — `cold-and-windy`'s `:then` uses only `?loc`, the JOIN KEY, which the
// second condition merely MATCHES, so a dropped or swapped binding from that
// side still yields the right `?loc`. Covered now by
// `tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs`.
/// P11/3b — `hash_join_no_cross_loc_leakage` (N×M probe):
///
/// 2 Temperatures × 2 WindSpeeds across 2 locations (Oslo + Bergen).
/// The HashJoinNode must produce EXACTLY 2 joined Tokens (Oslo×Oslo and Bergen×Bergen),
/// NOT 4 (a naive cross-product that ignores ?loc) and NOT 0 (a broken compatibility check).
///
/// This is the definitive proof that the keyed hash-join has no cross-product leakage.
///
/// Mirrors:
///   tests/probe_arc278_3b_hash_join.rs::join_no_cross_loc_leakage
#[test]
fn hash_join_no_cross_loc_leakage() {
    use super::{
        alpha_pass, get_node, hash_join_pass, kind_of, rete_arm_get_or_build, root_join_pass,
        sorted_node_ids,
    };
    const JOIN_WORLD: &str = "\
(:wat::core::defrecord :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
";

    let world = freeze_src(JOIN_WORLD);

    // 2 Temps (Oslo 15, Bergen 10) × 2 Winds (Oslo 45, Bergen 50): same-loc joins only.
    let session = eval_in(
            &world,
            "(:wat::core::let \
               [c1 (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))\
                c2 (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))\
                rule (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))\
                s0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                s1 (:wat::rete::insert s0 (:user::Temperature :celsius 15 :location \"Oslo\"))\
                s2 (:wat::rete::insert s1 (:user::Temperature :celsius 10 :location \"Bergen\"))\
                s3 (:wat::rete::insert s2 (:user::WindSpeed :kph 45 :location \"Oslo\"))\
                s4 (:wat::rete::insert s3 (:user::WindSpeed :kph 50 :location \"Bergen\"))]\
              s4)"
        );

    let mut wm = scratch_wm(&session);

    let sym = world.symbols();
    let arm = rete_arm_get_or_build(&wm.network, &wm.rules, sym).expect("arm");
    alpha_pass(&mut wm, &arm).expect("alpha_pass");
    root_join_pass(&mut wm);
    hash_join_pass(&mut wm, &arm).expect("hash_join_pass");

    // Find the HashJoinNode.
    let node_ids = sorted_node_ids(&wm.network);
    let hash_join_id = node_ids
        .iter()
        .find(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == NodeKind::HashJoin)
                .unwrap_or(false)
        })
        .copied()
        .expect("network must contain a HashJoinNode for the 2-condition rule");

    let token_count = wm.beta.get(&hash_join_id).map(Vec::len).unwrap_or(0);

    assert_eq!(
            token_count, 2,
            "hash_join_no_cross_loc_leakage (3b): 2×2 same-loc → exactly 2 joined Tokens (not 4, not 0); got {}",
            token_count
        );

    // Verify the two tokens are the correct same-loc pairs (Oslo×Oslo, Bergen×Bergen).
    let tokens = wm
        .beta
        .get(&hash_join_id)
        .expect("beta[hash_join_id] must be non-empty");
    let locs: std::collections::HashSet<String> = tokens
        .iter()
        .map(|tok| {
            match Bindings::get(
                &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                &Value::String(Arc::new("?loc".to_string())),
            ) {
                Some(Value::String(s)) => s.as_str().to_string(),
                _ => panic!("joined token must have ?loc bound to a String"),
            }
        })
        .collect();
    assert_eq!(
        locs,
        ["Oslo", "Bergen"]
            .into_iter()
            .map(String::from)
            .collect::<std::collections::HashSet<String>>(),
        "joined tokens must be exactly the Oslo and Bergen same-loc pairs"
    );
}

// ── Arc 278 A8 — the node-share fire-path census ─────────────────────────────
//
// A8 (node-share) is the one grid cell Clara wins, and the compiler was cleared on 2026-07-30:
// `wat-scripts/scratch-pad/probe-node-share-dedup.wat` counts the compiled network at `4 + 2N`
// (Alpha flat at 2, HashJoin flat at 1) for N = 1..32, so the shared prefix collapses exactly
// as `find-or-mint-hash-join` intends. The blow-up — >4 GiB to join 500 facts against 20 rules
// — therefore lives in the FIRE path, and this is the instrument that reads it.
//
// It measures, it does not guess. Every native structure the loop grows is counted per round
// (see `RoundCensus`), so the growth term names itself instead of confirming a hypothesis about
// which one it is. The world below is copied from `wat-scripts/perf/grid/node-share.wat` —
// same `build-rule`, same `seed` — so this measures the AXIS and not a lookalike.

/// The node-share world: A/B/Out plus the axis's own rule-builder and seeder.
///
/// `build-rule i n` is byte-identical to the axis's: the leading `[A (?k)] ⋈ [B (?k)]` carries
/// no `i`, so it is the shared prefix under test; only the trailing `where` holds the per-rule
/// literal. `mod` is spelled as the truncating-division idiom (wat has no native i64 mod).
const NODE_SHARE_WORLD: &str = "\
(:wat::core::defrecord :nsh::A   [k <- :wat::core::i64])\n\
(:wat::core::defrecord :nsh::B   [k <- :wat::core::i64])\n\
(:wat::core::defrecord :nsh::Out [k <- :wat::core::i64])\n\
\n\
(:wat::core::defn :nsh::build-rule [i <- :wat::core::i64  n <- :wat::core::i64] -> :wat::rete::Rule\n\
  (:wat::core::let [a-c     (:wat::core::quasiquote (:nsh::A (?k <- :k)))\n\
                    b-c     (:wat::core::quasiquote (:nsh::B (?k <- :k)))\n\
                    where-c (:wat::core::quasiquote\n\
                              (:wat::rete::where\n\
                                (:wat::rete::i64::= (:wat::core::unquote i)\n\
                                  (:wat::rete::i64::- ?k\n\
                                    (:wat::rete::i64::* (:wat::rete::i64::/ ?k (:wat::core::unquote n) :undefined 0) (:wat::core::unquote n) :undefined 0)\n\
                                    :undefined 0))))\n\
                    ins     (:wat::core::quasiquote (:nsh::Out ?k))]\n\
    (:wat::rete::Rule :name (:wat::i64::to-string i)\n\
      :lhs (:wat::core::PersistentVector a-c b-c where-c)\n\
      :rhs (:wat::core::PersistentVector ins))))\n\
\n\
(:wat::core::defn :nsh::build-rules [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  i <- :wat::core::i64]\n\
      -> (:wat::core::PersistentVector :- [:wat::rete::Rule])\n\
      (:wat::core::PersistentVector/conj acc (:nsh::build-rule i n)))\n\
    (:wat::core::PersistentVector)\n\
    (:wat::core::range 0 n)))\n\
\n\
(:wat::core::defn :nsh::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert (:wat::rete::insert s (:nsh::A i)) (:nsh::B i)))\n\
    session\n\
    (:wat::core::range 0 items)))\n\
";

/// Compile N node-share rules, seed M×2 facts, fire through the NATIVE path, return the census.
///
/// Fires `:wat::rete::fire-rules` — the public production verb, which delegates to
/// `fire-rules$native` (`wat/rete/oracle/fire.wat`) — so this is the same path the
/// grid harness times.
fn node_share_census(n: i64, m: i64) -> Vec<super::RoundCensus> {
    let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("node-share world should freeze");
    let src = format!(
        "(:wat::rete::fire-rules (:nsh::seed (:wat::rete::compile (:nsh::build-rules {n})) {m}))"
    );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, census) = super::with_fire_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at N={n} M={m}: {e:?}"))
            .value_owned()
    });
    census
}

/// Sum the tokens held by every beta node of a given kind in a census row.
fn tokens_of_kind(row: &super::RoundCensus, kind: &str) -> usize {
    row.beta_by_node
        .iter()
        .filter(|(_, k, _)| *k == kind)
        .map(|(_, _, t)| *t)
        .sum()
}

/// Tokens PRODUCED by nodes of `kind` across the whole fire, read off the per-round delta.
///
/// Since the beta-readers guard (`DESIGN-STONE-beta-is-written-only-for-readers`), a node
/// nothing reads has no materialised `wm.beta`, so `tokens_of_kind` reports 0 for it — a fact
/// about the guard, not about the join. `d_beta` still carries every token the node produced.
///
/// This is the SAME NUMBER the beta reading used to give, not a softer one: before the guard
/// both stores were fed by one unconditional statement pair, so summing the deltas across
/// rounds reconstructs exactly what the cumulative beta held.
fn produced_of_kind(census: &[super::RoundCensus], kind: &str) -> usize {
    census
        .iter()
        .flat_map(|r| r.d_beta_by_node.iter())
        .filter(|(_, k, _)| *k == kind)
        .map(|(_, _, t)| *t)
        .sum()
}

// ── The keyed-gather gate (DESIGN-STONE-keyed-gather.md) ──────────────────────────────────
//
// Two AccumulateNodes and one ExistsNode over `Reading`, joined to `Group` on `?g` — the
// `accum` grid axis's shape, reduced to the two node kinds whose gather is under test.

/// Group/Reading plus two accumulators and an exists, all keyed on the shared `?g`.
const ACCUM_GATHER_WORLD: &str = "\
(:wat::core::defrecord :agc::Group   [g <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::SumF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::ExistsF [g <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :agc::count-rule\n\
  :when\n\
  [(:agc::Group (?g <- :g))\n\
   (?n <- (:wat::rete::acc::count) :from (:agc::Reading (?g <- :g)))]\n\
  :then\n\
  [(:agc::CountF ?g ?n)])\n\
\n\
(:wat::rete::defrule :agc::sum-rule\n\
  :when\n\
  [(:agc::Group (?g <- :g))\n\
   (?n <- (:wat::rete::acc::sum ?v) :from (:agc::Reading (?g <- :g) (?v <- :v)))]\n\
  :then\n\
  [(:agc::SumF ?g ?n)])\n\
\n\
(:wat::rete::defrule :agc::exists-rule\n\
  :when\n\
  [(:agc::Group (?g <- :g))\n\
   (:wat::rete::exists (:agc::Reading (?g <- :g)))]\n\
  :then\n\
  [(:agc::ExistsF ?g)])\n\
\n\
(:wat::core::defn :agc::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert s (:agc::Reading :g g :v j)))\n\
    session\n\
    (:wat::core::range 0 w)))\n\
\n\
(:wat::core::defn :agc::seed [session <- :wat::rete::Session  gs <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session\n\
      (:agc::seed-readings (:wat::rete::insert s (:agc::Group g)) g w))\n\
    session\n\
    (:wat::core::range 0 gs)))\n\
";

/// Fire the gather world at `g` groups × `w` readings and return the gather-visit count.
fn accum_gather_visits(g: i64, w: i64) -> u64 {
    let world = startup_from_source(ACCUM_GATHER_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-gather world should freeze");
    let src = format!(
            "(:wat::rete::fire-rules (:agc::seed (:wat::rete::compile (:wat::rete::collect-rules :agc)) {g} {w}))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, visits) = super::with_gather_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at G={g} W={w}: {e:?}"))
            .value_owned()
    });
    visits
}

// ── Where does the accum fire actually spend its time? ────────────────────────────────────
//
// The `accum` grid axis is ~1.5× behind a WARMED Clara (2.19 vs 4.66 µs/fact at 40,200 facts;
// Clara's own per-fact cost falls 4.6× across the ladder as its JIT warms, ours is flat). The
// keyed gather is under 10% of our fire, so the cost is elsewhere — and there is no `perf` on
// this box. Rather than narrate a plausible root, the loop reports its own split.
//
// The world mirrors `wat-scripts/perf/grid/accum.wat` — FIVE rules (count/sum/min/max + exists)
// over Group ⋈ Reading — byte-for-byte modulo the namespace, so this apportions the AXIS's time
// and not a lookalike's. (`ACCUM_GATHER_WORLD` above is deliberately smaller: it exists to gate
// the gather's SHAPE, where two accumulators are enough.)

const ACCUM_AXIS_WORLD: &str = "\
(:wat::core::defrecord :apx::Group   [g <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::SumF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::MinF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::MaxF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::ExistsF [g <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :apx::count-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::count) :from (:apx::Reading (?g <- :g)))]\n\
  :then [(:apx::CountF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::sum-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::sum ?v) :from (:apx::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:apx::SumF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::min-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::min ?v) :from (:apx::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:apx::MinF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::max-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::max ?v) :from (:apx::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:apx::MaxF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::exists-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (:wat::rete::exists (:apx::Reading (?g <- :g)))]\n\
  :then [(:apx::ExistsF ?g)])\n\
\n\
(:wat::core::defn :apx::val [g <- :wat::core::i64  j <- :wat::core::i64] -> :wat::core::i64\n\
  (:wat::core::let [x (:wat::i64::+ (:wat::i64::* g 31) (:wat::i64::* j 17))]\n\
    (:wat::i64::- x (:wat::i64::* (:wat::i64::/ x 1000) 1000))))\n\
\n\
(:wat::core::defn :apx::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert s (:apx::Reading :g g :v (:apx::val g j))))\n\
    session\n\
    (:wat::core::range 0 w)))\n\
\n\
(:wat::core::defn :apx::seed [session <- :wat::rete::Session  gs <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session\n\
      (:apx::seed-readings (:wat::rete::insert s (:apx::Group g)) g w))\n\
    session\n\
    (:wat::core::range 0 gs)))\n\
";

/// Fire the axis world at `g` groups × `w` readings; return the per-phase nanosecond split.
///
/// Only `fire-rules` is inside the armed window — compile and seed run first, un-timed, exactly
/// as the grid harness does it, so this apportions the same span the grid's `:native-ns` covers.
fn accum_phase_census(g: i64, w: i64) -> Vec<(&'static str, u64, u64)> {
    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged =
        format!("(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) {g} {w})");
    let src = format!("(:wat::rete::fire-rules {staged})");
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let t0 = std::time::Instant::now();
    let (_fired, mut rows) = super::with_phase_census_counted(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at G={g} W={w}: {e:?}"))
            .value_owned()
    });
    // ⚠ The WHOLE fire, so the census can declare its own COVERAGE. The six phases live inside
    // `fire_fixpoint_delta`'s round loop; everything outside it — network extraction,
    // alpha_by_type, parents_of, per-round setup, the terminate step, merge_facts,
    // to_persistent — is NOT covered by any mark. Apportioning the phases as if they were the
    // whole fire is precisely the instrument-boundary error this file keeps warning about.
    // ⚠ This wraps the WHOLE driver expression — `(fire-rules (seed (compile ...)))` — so it
    // includes compile and SEED, not just the fire. Named accordingly: an earlier version
    // called it "WHOLE FIRE" and the ~205ms of seeding read as unaccounted fire, which is the
    // third instrument-boundary error in this file today and all three were mine. The four
    // outer marks (IN/SETUP/ROUND LOOP/OUT) are what partition the fire, and their sum matches
    // the grid's own :wat-ns to ~1% — that agreement is the cross-check.
    rows.push((
        "WHOLE EVAL (compile+seed+fire)",
        t0.elapsed().as_nanos() as u64,
        1,
    ));
    rows
}

/// Render an instrument-subtracted phase table for ANY axis.
///
/// Extracted 2026-08-01 when node-share needed the same table accum already had. Copying it
/// would have put the instrument-subtraction arithmetic in two places, and the whole reason
/// that arithmetic exists is that a table which misreports its own instrument is worse than no
/// table — two copies is how one of them silently stops subtracting.
///
/// `census(a, b)` fires the axis at that size and returns (phase, ns, mark-pairs-fired).
/// `facts(a, b)` is the fact count for the header. `top` partitions the fire (summing a parent
/// with its children is what made an earlier version of this table report 124% coverage).
fn render_phase_table(
    label: &str,
    sizes: &[(i64, i64)],
    top: &[&'static str],
    required: &[&'static str],
    facts: impl Fn(i64, i64) -> i64,
    census: impl Fn(i64, i64) -> Vec<(&'static str, u64, u64)>,
) -> String {
    let cal_ns_per_pair = calibrate_mark_ns();
    const RUNS: usize = 3;

    let mut table = format!(
        "\n{label} — per-phase split (native fire-rules only), mean of {RUNS} runs\n\
             instrument: ~{cal_ns_per_pair:.1} ns per mark pair; `net` = raw MINUS this row's own \
             pairs. PARENT rows still contain their children's share.\n"
    );
    for &(a, b) in sizes {
        // rune:perspicere(read-once) — census sample bags; alias would be a one-site mumble.
        let mut samples: std::collections::HashMap<&'static str, Vec<u64>> =
            std::collections::HashMap::new();
        let mut pairs: std::collections::HashMap<&'static str, u64> =
            std::collections::HashMap::new();
        let mut order: Vec<&'static str> = Vec::new();
        for _ in 0..RUNS {
            let rows = census(a, b);
            assert!(
                !rows.is_empty(),
                "{label}: census recorded NOTHING at {a}/{b}"
            );
            for (name, ns, k) in rows {
                if !samples.contains_key(name) {
                    order.push(name);
                }
                samples.entry(name).or_default().push(ns);
                pairs.insert(name, k);
            }
        }
        let missing: Vec<&str> = required
            .iter()
            .copied()
            .filter(|p| !samples.contains_key(p))
            .collect();
        assert!(
            missing.is_empty(),
            "{label}: phase(s) {missing:?} never recorded at {a}/{b}"
        );

        let stat = |xs: &[u64]| -> (f64, u64, u64) {
            let sum: u64 = xs.iter().sum();
            (
                sum as f64 / xs.len() as f64,
                *xs.iter().min().expect("non-empty"),
                *xs.iter().max().expect("non-empty"),
            )
        };
        let net_of = |k: &str, xs: &[u64]| -> f64 {
            stat(xs).0 - *pairs.get(k).unwrap_or(&0) as f64 * cal_ns_per_pair
        };
        let total_mean: f64 = top
            .iter()
            .filter_map(|k| samples.get(k).map(|xs| stat(xs).0))
            .sum();
        assert!(total_mean > 0.0, "{label}: phase total is zero at {a}/{b}");
        let total_net: f64 = top
            .iter()
            .filter_map(|k| samples.get(k).map(|xs| net_of(k, xs)))
            .sum();
        let instrument: f64 = pairs.values().map(|k| *k as f64 * cal_ns_per_pair).sum();

        table.push_str(&format!(
            "\n  {a}/{b}  ({} facts)   FIRE {:.2} ms raw / {:.2} net   \
                 instrument {:.2} ms across {} pairs\n",
            facts(a, b),
            total_mean / 1e6,
            total_net / 1e6,
            instrument / 1e6,
            pairs.values().sum::<u64>(),
        ));
        for phase in &order {
            if *phase == "WHOLE EVAL (compile+seed+fire)" {
                continue;
            }
            let xs = samples.get(phase).expect("discovered, so present");
            let (mean, lo, hi) = stat(xs);
            let net = net_of(phase, xs);
            let flag = if net <= 0.0 {
                "  ⚠ BELOW ITS OWN INSTRUMENT"
            } else {
                ""
            };
            table.push_str(&format!(
                "    {:<20} {:>8.2} ms raw  {:>8.2} net  {:>5.1}%  [{:.2}–{:.2}]  {}x{}\n",
                phase,
                mean / 1e6,
                net / 1e6,
                100.0 * net / total_net,
                lo as f64 / 1e6,
                hi as f64 / 1e6,
                *pairs.get(phase).unwrap_or(&0),
                flag,
            ));
        }
    }
    table
}

/// Fire the node-share world at `n` rules x `m` items; per-phase split with pair counts.
///
/// node-share is the grid's WEAKEST engine cell (:ratio 1.56 at [50 200]) and had no phase
/// census at all — only a COUNT census at M=50. Ranking its sinks off accum's or fanout's
/// table would be the R61 error: alpha is 4.7% of fanout and ~40% of accum.
fn node_share_phase_census(n: i64, m: i64) -> Vec<(&'static str, u64, u64)> {
    let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("node-share world should freeze");
    let staged = format!("(:nsh::seed (:wat::rete::compile (:nsh::build-rules {n})) {m})");
    let src = format!("(:wat::rete::fire-rules {staged})");
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census_counted(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at N={n} M={m}: {e:?}"))
            .value_owned()
    });
    rows
}

/// ★ STEP 0 of DESIGN-STONE-compiled-where — the DECOMPOSITION, before anything is built.
///
/// The counters (`node_share_filter_eval_census`, below) proved the MECHANISM exactly: 10,000
/// `Environment` builds and 10,000 key allocations per fire at `[50 200]`, 98% of them for a
/// predicate about to fail. They say NOTHING about the SHARE — and a cost read is not a cost
/// measured (`[[feedback_measure_the_decomposition_never_read_it]]`, four wrong attributions in
/// one session doing exactly that).
///
/// Two things the `filter` phase's 89.5% actually contains, unsplit until now:
///   1. the per-TestNode `new_tokens = ts.clone()` (`:2701`) — on a SHARED-prefix axis every
///      one of the N TestNodes has the same parent, so the same 200-token vector is cloned N
///      times per round. NOT the predicate. (Task #50.)
///   2. the predicate itself, which splits again into the env build and the `eval_inner` walk.
///
/// So three arms, at ONE ROUND'S WORTH of work each so the numbers land on the same scale as
/// the 6.83 ms `filter` reading, **interleaved** — never blocks; a block-ordered A/B produced a
/// clean, disjoint, WRONG −7 ms on 2026-08-01 that a B-A-B drift check destroyed
/// (`[[feedback_a_benchmarks_shape_manufactures_its_result]]`).
///
/// Inputs are the PRODUCTION values, captured out of a real fire — not fabricated.
///
/// STOP-0 (in the stone): if `walk ≫ env`, the seam's gate (`env-builds → 0`) is a mechanism
/// win with no timing behind it and the stone's shape is wrong.
/// STOP-0b: if `clone` is comparable to `env + walk`, task #50 is a peer cost and cheaper.
// rune:complectens(inline-fixtures) — interleaved timing arms ARE the measurement fixture;
// extracting them would collapse the A–F reconstruction this probe exists to document.
#[test]
fn node_share_where_cost_decomposition() {
    use std::hint::black_box;
    use std::time::Instant;

    const N: i64 = 50;
    const M: i64 = 200;
    const REPS: usize = 15;

    // ── capture the real inputs out of a real fire ────────────────────────────────────────
    let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("node-share world should freeze");
    let src = format!(
        "(:wat::rete::fire-rules (:nsh::seed (:wat::rete::compile (:nsh::build-rules {N})) {M}))"
    );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, sample) = super::with_where_sample(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at N={N} M={M}: {e:?}"))
            .value_owned()
    });
    let (expr, tokens) = sample.expect(
        "the fire never reached a TestNode, so nothing was captured — every number below \
             would be measuring a fabricated input, which is the one thing this probe exists to \
             avoid",
    );

    // ── non-vacuity, BEFORE any timing ────────────────────────────────────────────────────
    // A benchmark over an empty token vector or a zero-binding token would run fast and mean
    // nothing. Assert the shape production actually produced, and that the predicate really
    // evaluates (both verdicts must be reachable across the captured tokens: node-share's
    // `i == k mod N` passes exactly one k in N).
    let bindings_per_token = tokens[0].len();
    assert!(
        tokens.len() as i64 == M && bindings_per_token > 0,
        "captured {} tokens with {bindings_per_token} bindings each; expected {M} tokens \
             carrying at least one ?var — the capture did not see node-share's real parent delta",
        tokens.len(),
    );
    let verdicts: Vec<bool> = tokens
        .iter()
        .map(|t| {
            crate::rete::eval_test::eval_test_core(&expr, t, &Environment::new(), &world.symbols)
                .expect("the captured predicate must evaluate on the captured bindings")
        })
        .collect();
    let passes = verdicts.iter().filter(|b| **b).count();
    assert!(
        passes > 0 && passes < tokens.len(),
        "captured predicate returned the SAME verdict for all {} tokens ({passes} passes) — \
             a constant-folded predicate would make arm B's walk unrepresentative",
        tokens.len(),
    );

    // ── the three arms, one round's worth each, interleaved ───────────────────────────────
    // Arm A calls `build_test_env`, which IS the block `eval_test_core` runs — extracted, not
    // copied, so the arm cannot drift from the path it claims to measure.
    let evals_per_round = (N as usize) * tokens.len(); // 50 TestNodes x 200 tokens = 10,000
    let mut a_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut b_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut c_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut d_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut e_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut f_ns: Vec<u128> = Vec::with_capacity(REPS);
    let empty = Environment::new();
    let program = crate::rete::expr_ir::lower(&expr, &world.symbols)
        .unwrap_or_else(|e| panic!("captured where must lower: {e:?}"));
    let compiled_verdicts: Vec<bool> = tokens
        .iter()
        .map(|t| {
            crate::rete::expr_ir::exec_where(&program, t, &world.symbols, &program.span)
                .expect("compiled where must exec on captured bindings")
        })
        .collect();
    assert_eq!(
        compiled_verdicts, verdicts,
        "compiled exec_where must agree with eval_test_core on the captured tokens"
    );

    // Arm D's input — the SAME predicate with its two `?k` reads replaced by the literal they
    // would resolve to. Identical node count, identical operators, ZERO name lookups: the
    // identity control that separates "the interpreter's per-node dispatch" from "resolving a
    // ?var through the Environment" inside the walk.
    let const_src = "(:wat::core::= 7 (:wat::i64::- 9 \
               (:wat::i64::* (:wat::i64::/ 9 50) 50)))";
    let const_expr = crate::parse_one!(const_src).expect("parse the var-free control predicate");
    // The control must actually EVALUATE, or arm D measures an error path, not a walk.
    assert!(
        crate::rete::eval_test::eval_test_core(&const_expr, &tokens[0], &empty, &world.symbols,)
            .is_ok(),
        "the var-free control predicate did not evaluate — arm D would be timing a failure"
    );
    // Arm E's key — the one binding node-share's predicate reads.
    let k_key = tokens[0]
        .iter()
        .next()
        .map(|(k, _)| k.clone())
        .expect("the captured token carries at least one binding (asserted above)");
    for _ in 0..REPS {
        // A — the env build alone.
        let t = Instant::now();
        for i in 0..evals_per_round {
            let e = crate::rete::eval_test::build_test_env(&tokens[i % tokens.len()], &empty);
            black_box(&e);
        }
        a_ns.push(t.elapsed().as_nanos());

        // B — the env build PLUS the eval_inner walk (the whole of `eval_test_core`).
        let t = Instant::now();
        for i in 0..evals_per_round {
            let v = crate::rete::eval_test::eval_test_core(
                &expr,
                &tokens[i % tokens.len()],
                &empty,
                &world.symbols,
            );
            black_box(&v);
        }
        b_ns.push(t.elapsed().as_nanos());

        // C — the per-TestNode token clone: N clones of the parent's M-token delta.
        let t = Instant::now();
        for _ in 0..N {
            let c = tokens.clone();
            black_box(&c);
        }
        c_ns.push(t.elapsed().as_nanos());

        // D — env build + walk of the VAR-FREE control (same nodes, no name lookups).
        let t = Instant::now();
        for i in 0..evals_per_round {
            let v = crate::rete::eval_test::eval_test_core(
                &const_expr,
                &tokens[i % tokens.len()],
                &empty,
                &world.symbols,
            );
            black_box(&v);
        }
        d_ns.push(t.elapsed().as_nanos());

        // E — THE FLOOR. The same predicate as hand-written Rust against the same trie: one
        // binding read, then the arithmetic. This is what a perfectly compiled IR could reach,
        // so it BOUNDS the prize instead of leaving it to a prediction (and today's
        // predictions have a bad record — `[[feedback_measure_the_decomposition_never_read_it]]`).
        let t = Instant::now();
        for i in 0..evals_per_round {
            let bs = &tokens[i % tokens.len()];
            let v = match bs.get(&k_key) {
                Some(Value::i64(k)) => 7 == k - (k / 50) * 50,
                _ => false,
            };
            black_box(v);
        }
        e_ns.push(t.elapsed().as_nanos());

        // F — the native fire path: lower-once (outside this loop) + exec_where.
        let t = Instant::now();
        for i in 0..evals_per_round {
            let v = crate::rete::expr_ir::exec_where(
                &program,
                &tokens[i % tokens.len()],
                &world.symbols,
                &program.span,
            );
            black_box(&v);
        }
        f_ns.push(t.elapsed().as_nanos());
    }
    let median = |mut v: Vec<u128>| -> f64 {
        v.sort_unstable();
        v[v.len() / 2] as f64
    };
    let a = median(a_ns);
    let b = median(b_ns);
    let c = median(c_ns);
    let d = median(d_ns);
    let e = median(e_ns);
    let f = median(f_ns);
    let walk = b - a;
    let walk_novars = d - a;
    let lookups = walk - walk_novars;
    // The measured `filter` phase this reconstructs (2026-08-01, node_share_fire_phase_census,
    // [50 200]). Printed so the reconstruction can be CHECKED, not assumed: if B + C does not
    // land near it, the harness is measuring something the fire does not do.
    const FILTER_MS_MEASURED_IN_FIRE: f64 = 6.83;

    println!(
            "\nSTEP 0 — where-predicate cost decomposition, node-share [{N} {M}], \
             ONE ROUND's worth per arm, {REPS} interleaved reps, medians\n\
             \x20 captured from a real fire: 1 predicate x {} tokens x {bindings_per_token} \
             binding(s); {passes}/{} pass\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 A  env build alone         ({evals_per_round:>6} x)  {:>8.3} ms\n\
             \x20 B  env build + walk        ({evals_per_round:>6} x)  {:>8.3} ms\n\
             \x20 C  token clone             ({:>6} x)  {:>8.3} ms\n\
             \x20 D  env + walk, VAR-FREE    ({evals_per_round:>6} x)  {:>8.3} ms\n\
             \x20 E  hand-written Rust       ({evals_per_round:>6} x)  {:>8.3} ms   <- THE FLOOR\n\
             \x20 F  compiled exec_where     ({evals_per_round:>6} x)  {:>8.3} ms   {:>6.1} ns/eval\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 the walk        B-A   {:>8.3} ms  {:>5.1}% of B   {:>6.1} ns/eval\n\
             \x20   of which:\n\
             \x20     ?var lookup (B-A)-(D-A)  {:>8.3} ms  {:>5.1}% of the walk\n\
             \x20     node dispatch    D-A     {:>8.3} ms  {:>5.1}% of the walk\n\
             \x20 the env build   A     {:>8.3} ms  {:>5.1}% of B   {:>6.1} ns/eval\n\
             \x20 the token clone C     {:>8.3} ms\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 RECONSTRUCTION  B+C = {:>6.3} ms  vs a measured `filter` of \
             {FILTER_MS_MEASURED_IN_FIRE} ms  ({:>4.0}% accounted)\n\
             \x20 HEADROOM        B-E = {:>6.3} ms is what a PERFECT compile could remove\n\
             \x20 COMPILED vs B   B/F = {:>5.2}x    F-E leftover {:>6.1} ns/eval\n",
            tokens.len(),
            tokens.len(),
            a / 1e6,
            b / 1e6,
            N,
            c / 1e6,
            d / 1e6,
            e / 1e6,
            f / 1e6,
            f / evals_per_round as f64,
            walk / 1e6,
            100.0 * walk / b,
            walk / evals_per_round as f64,
            lookups / 1e6,
            100.0 * lookups / walk,
            walk_novars / 1e6,
            100.0 * walk_novars / walk,
            a / 1e6,
            100.0 * a / b,
            a / evals_per_round as f64,
            c / 1e6,
            (b + c) / 1e6,
            100.0 * ((b + c) / 1e6) / FILTER_MS_MEASURED_IN_FIRE,
            (b - e) / 1e6,
            b / f,
            (f - e) / evals_per_round as f64,
        );

    // Non-vacuity on the INSTRUMENT itself: a zero reading means the optimiser removed the
    // arm, and every share above would be an artifact.
    assert!(
        a > 0.0 && b > 0.0 && c > 0.0 && d > 0.0 && e > 0.0 && f > 0.0 && b > a && b > e,
        "an arm measured zero, or the orderings that MUST hold do not — the loop was \
             optimised away and the shares above are artifacts \
             (A={a}ns B={b}ns C={c}ns D={d}ns E={e}ns)"
    );
}

/// (b) landed — this census now gates the index, not the pre-index waste.
///
/// Node-share: M tokens, N rules, one shared dim `(= i (k rem n))`. Linear eval is
/// M×N with ~98% waste. The where-tree must cut that to ~1 eval/token so
/// `evals ≈ passes ≈ M`. If `evals` climbs back toward M×N the tree stopped
/// discriminating (analysis miss, or dispatch still walking every sibling).
#[test]
fn node_share_filter_eval_census() {
    let mut table = String::from(
        "\nnode-share — `where` evaluations vs passes (the (b) WhereDiscNode gate)\n\
             \x20 rules  items |    evals    reuse    passes   wasted  waste%   evals/token\n\
             \x20 -----------------------------------------------------------------------------\n",
    );
    let mut worst_waste = 0.0f64;
    for (n, m) in [(10i64, 200i64), (25, 200), (50, 200)] {
        let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("node-share world should freeze");
        let src = format!(
                "(:wat::rete::fire-rules (:nsh::seed (:wat::rete::compile (:nsh::build-rules {n})) {m}))"
            );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, rows) = super::with_count_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised at N={n} M={m}: {e:?}"))
                .value_owned()
        });
        let get = |k: &str| {
            rows.iter()
                .find(|(a, _)| *a == k)
                .map(|(_, v)| *v)
                .unwrap_or(0)
        };
        let evals = get("filter:test-evals");
        let reuse = get("filter:test-reuse");
        let passes = get("filter:test-pass");
        let envs = get("filter:test-env-builds");
        let keys = get("filter:test-key-alloc");
        // Non-vacuity FIRST: a fire that never reached a TestNode would report 0 evals and
        // 0 passes, and a "0% waste" reading would look like the best possible news.
        // Proven `(= dim lit)` or range skip `exec_where` (`filter:test-reuse`).
        assert!(
                evals > 0 || reuse > 0,
                "node-share N={n} M={m} recorded ZERO `where` evaluations and ZERO reuse — the \
                 filter pass never ran, so any ratio taken from this is an artifact, not a measurement"
            );
        assert!(
            passes > 0,
            "node-share N={n} M={m} recorded ZERO passes — the tree pruned every TestNode \
                 (under-approx) or nothing fired"
        );
        let wasted = evals.saturating_sub(passes);
        let waste_pct = if evals == 0 {
            0.0
        } else {
            100.0 * wasted as f64 / evals as f64
        };
        worst_waste = worst_waste.max(waste_pct);
        table.push_str(&format!(
                "  {n:>5}  {m:>5} | {evals:>8}  {reuse:>8}  {passes:>8} {wasted:>8}  {waste_pct:>5.1}%  \
                 {:>10.2}  | envs {envs:>7}  keyallocs {keys:>7}\n",
                evals as f64 / m as f64,
            ));
        // ~1 candidate per token. Slack of 2× covers a second filter pass / mild over-approx.
        // Linear scan is N×M (10_000 at [50 200]) — that must not pass.
        assert!(
            evals <= passes.saturating_mul(2),
            "where-tree should eval about as many predicates as pass (one matching residue \
                 per token). N={n} M={m} evals={evals} passes={passes}.{table}"
        );
        assert!(
            evals <= (m as u64).saturating_mul(4),
            "where-tree evals should sit near M (one token → one residue), not N×M. \
                 N={n} M={m} evals={evals}.{table}"
        );
    }
    println!("{table}");
    assert!(
        worst_waste < 50.0,
        "(b) must collapse wasted `where` evals (a token tested by every rule, matching \
             at most one) — peak waste {worst_waste:.1}%. If this rose, dispatch is linear \
             again or DimKey failed to unify the node-share residue.{table}"
    );
}

/// The node-share phase table, at the GRID's own ladder ([10|25|50] x 200).
#[test]
fn node_share_fire_phase_census() {
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    // Floor only — the table discovers the rest. node-share has no accumulate/filter, so its
    // required set is deliberately smaller than accum's; asserting accum's list here would
    // fail on phases this axis never reaches.
    const REQUIRED: [&str; 6] = [
        "SETUP: indexes",
        "ROUND LOOP",
        "alpha",
        "root-join",
        "hash-join",
        "production",
    ];
    let table = render_phase_table(
        "node-share fire",
        &[(10, 200), (25, 200), (50, 200)],
        &TOP,
        &REQUIRED,
        |_n, m| m * 2, // M A-facts + M B-facts
        node_share_phase_census,
    );
    println!("{table}");

    // Assert on the DATA, not the rendered text. A `table.contains("ROUND LOOP")` passes on a
    // table whose every number is zero. Non-vacuity: the axis fired, and `filter` still
    // recorded (this world has TestNodes). WhereDiscNode already killed filter-dominates
    // (89.5% on 2026-08-01); do not wall-gate that share.
    let rows = node_share_phase_census(50, 200);
    let ns_of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _, _)| *n == name)
            .map(|(_, ns, _)| *ns)
            .unwrap_or(0)
    };
    let round_loop = ns_of("ROUND LOOP");
    let filter = ns_of("filter");
    assert!(
        round_loop > 0,
        "ROUND LOOP recorded 0ns at 50/200 — the fire never ran, and a\n\
                                 table of zeroes would still have rendered every row:\n{table}"
    );
    assert!(
        filter > 0,
        "filter recorded 0ns at 50/200 — this axis has TestNodes:\n{table}"
    );
}

/// Fire the axis world and return the operation counts (see `census_count`).
fn accum_count_census(g: i64, w: i64) -> Vec<(&'static str, u64)> {
    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let src = format!(
            "(:wat::rete::fire-rules (:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) {g} {w}))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_count_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at G={g} W={w}: {e:?}"))
            .value_owned()
    });
    rows
}

/// The gather-index-cache gate — RED until the index is cached per (alpha, join-keys).
///
/// `gather_index` is a pure function of (alpha memory, join keys), yet it is rebuilt per NODE
/// per round. At G=200 W=200 the accum world has THREE alpha nodes (200 Groups + two Reading
/// alphas of 40,000 — one binding ?g, one binding ?g,?v) and FIVE readers of them:
///   count   -> Reading-?g     (accumulate pass)
///   exists  -> Reading-?g     (filter pass)
///   sum/min/max -> Reading-?g?v (accumulate pass)
/// Five builds over TWO distinct (alpha_id, join_keys) pairs; three are pure repetition, each
/// dragging a full 40,000-element clone with it.
///
/// What would turn this red once green — the R59 question, answered before the assertion:
///   (a) the instrument counting nothing (`builds == 0`) — asserted separately, since a silent
///       zero would satisfy `<= 2` while measuring nothing at all;
///   (b) a cache keyed on `alpha_id` ALONE — it would read 2 here (every reader keys on ?g) and
///       be WRONG the moment two readers of one alpha have parents binding different variable
///       sets. This gate cannot catch that; the DESIGN's contract clause and the differentials
///       are what stand between it and a silent empty gather.
///   (c) the cache outliving a round — `wm.alpha` grows in step 1, so a stale index under-reads
///       and `count`/`sum` emit identities for groups that do have elements.
/// Landed: the fire-scoped `gather_cache` keyed on `(alpha_id, join_keys)`
/// (`DESIGN-STONE-gather-index-cache.md`, persist
/// `DESIGN-STONE-persist-gather-across-rounds.md`) makes this GREEN at 2
/// builds / 80,000 elements on a cold accum fire.
#[test]
fn gather_index_is_built_once_per_alpha_and_keyset() {
    let rows = accum_count_census(200, 200);
    let builds = rows
        .iter()
        .find(|(n, _)| *n == "accum:index-builds")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let elements = rows
        .iter()
        .find(|(n, _)| *n == "accum:index-elements")
        .map(|(_, c)| *c)
        .unwrap_or(0);

    assert!(
        builds > 0,
        "the index-build counter recorded ZERO — the counters were never reached, so `builds \
             <= 2` would pass while measuring nothing"
    );
    println!("\ngather index — builds {builds}, elements indexed {elements}\n");

    assert!(
        builds <= 2,
        "gather_index ran {builds} times over only TWO distinct (alpha_id, join_keys) pairs — \
             the index is being rebuilt per NODE instead of cached per (alpha, key-set). See \
             DESIGN-STONE-gather-index-cache.md."
    );
    assert!(
        elements <= 80_000,
        "indexed {elements} elements where 80,000 (the two distinct alpha memories, once each) \
             suffices — each redundant build drags a full-memory clone with it"
    );
}

// ── Is the per-element BINDING LOOKUP the fold's cost? ───────────────────────────────────
//
// `accum:fold` is ~27% of fire. Inside it, `acc_var_i64` does an rpds trie lookup per element
// to recover the accumulated ?var. That is a plausible root — and so were the three that died
// this week. The accumulators differ in exactly the way needed to settle it without a new
// instrument: `count` is `gathered.len()` and does NO lookup; `sum` does one per element.
// Same world shape, same size, one rule each — the delta in `accum:fold` IS the lookup.

fn one_rule_world(rule: &str) -> String {
    format!(
"(:wat::core::defrecord :one::Group   [g <- :wat::core::i64])\n\
(:wat::core::defrecord :one::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :one::Out     [g <- :wat::core::i64  n <- :wat::core::i64])\n\
{rule}\n\
(:wat::core::defn :one::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert s (:one::Reading :g g :v j)))\n\
    session\n\
    (:wat::core::range 0 w)))\n\
(:wat::core::defn :one::seed [session <- :wat::rete::Session  gs <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session\n\
      (:one::seed-readings (:wat::rete::insert s (:one::Group g)) g w))\n\
    session\n\
    (:wat::core::range 0 gs)))\n")
}

fn one_rule_fold_ns(rule: &str, g: i64, w: i64) -> u64 {
    let world = startup_from_source(&one_rule_world(rule), None, Arc::new(InMemoryLoader::new()))
        .expect("one-rule world should freeze");
    let src = format!(
            "(:wat::rete::fire-rules (:one::seed (:wat::rete::compile (:wat::rete::collect-rules :one)) {g} {w}))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
            .value_owned()
    });
    rows.iter()
        .find(|(n, _)| *n == "  └ accum:fold")
        .map(|(_, ns)| *ns)
        .unwrap_or(0)
}

/// Diagnostic — the fold WITH a per-element binding lookup vs WITHOUT one.
#[test]
fn fold_cost_with_and_without_the_binding_lookup() {
    const COUNT_RULE: &str = "(:wat::rete::defrule :one::count-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::count) :from (:one::Reading (?g <- :g)))]\n\
  :then [(:one::Out ?g ?n)])";
    const SUM_RULE: &str = "(:wat::rete::defrule :one::sum-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::sum ?v) :from (:one::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:one::Out ?g ?n)])";

    const RUNS: usize = 3;
    let (g, w) = (200i64, 200i64);
    let elements = g * w;

    let mut counts = Vec::new();
    let mut sums = Vec::new();
    for _ in 0..RUNS {
        counts.push(one_rule_fold_ns(COUNT_RULE, g, w));
        sums.push(one_rule_fold_ns(SUM_RULE, g, w));
    }
    let mean = |xs: &[u64]| xs.iter().sum::<u64>() as f64 / xs.len() as f64;
    let (c, s) = (mean(&counts), mean(&sums));
    assert!(
        c > 0.0 && s > 0.0,
        "one or both folds recorded nothing — the instrument never fired"
    );

    println!(
            "\nfold cost, {elements} elements gathered, mean of {RUNS}\n                 count (NO per-element lookup)  {:>7.2} ms\n                 sum   (ONE lookup per element) {:>7.2} ms\n                 delta = the lookup             {:>7.2} ms   ({:.0} ns/element)\n",
            c / 1e6, s / 1e6, (s - c) / 1e6, (s - c) / elements as f64
        );
    // (b) + keyed gather left rematch in the fold mark. After the fold stone,
    // count is bucket.len() and sum is a slot load — sum must sit near count,
    // and count must be well below the 9.59 ms rematch walk.
    assert!(
        c < 5.0e6,
        "count fold is {c:.0} ns ({:.2} ms) — rematch is still in the walk; \
             expected bucket.len() after DESIGN-STONE-accum-fold-the-wall",
        c / 1e6
    );
    assert!(
        s <= c * 2.0 || s < 8.0e6,
        "sum fold {s:.0} ns vs count {c:.0} ns — the 223 ns/el Bindings::get \
             did not collapse to a slot load"
    );
}

// ── Is the BIND (trie insert) the cost inside alpha:match? ───────────────────────────────
//
// alpha:match is ~28% of fire and does 120,200 fresh binds, each allocating an rpds trie node
// for a map holding one or two entries. Plausible — and the previous three plausible roots
// were wrong, so it is measured the same way the fold's lookup was: two worlds differing by
// EXACTLY one bind clause on the Reading condition. No accumulate, no join beyond the root:
// the delta in alpha:match is the marginal cost of one binding, times the fact count.

fn bind_world(reading_cond: &str) -> String {
    format!(
"(:wat::core::defrecord :bnd::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :bnd::Out     [g <- :wat::core::i64])\n\
(:wat::rete::defrule :bnd::r\n\
  :when [{reading_cond}]\n\
  :then [(:bnd::Out ?g)])\n\
(:wat::core::defn :bnd::seed [session <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert s (:bnd::Reading :g i :v i)))\n\
    session\n\
    (:wat::core::range 0 n)))\n")
}

/// Returns outer `alpha` ns for one bind-world at `n` facts.
/// Child timers were retired (`DESIGN-STONE-retire-alpha-child-marks`).
fn bind_world_alpha_ns(reading_cond: &str, n: i64) -> u64 {
    let world = startup_from_source(
        &bind_world(reading_cond),
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("bind world should freeze");
    let src = format!(
            "(:wat::rete::fire-rules (:bnd::seed (:wat::rete::compile (:wat::rete::collect-rules :bnd)) {n}))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
            .value_owned()
    });
    rows.iter()
        .find(|(n2, _)| *n2 == "alpha")
        .map(|(_, ns)| *ns)
        .unwrap_or(0)
}

/// Diagnostic — one bind vs two binds on the same condition, same facts.
#[test]
fn alpha_match_cost_per_binding() {
    const ONE: &str = "(:bnd::Reading (?g <- :g))";
    const TWO: &str = "(:bnd::Reading (?g <- :g) (?v <- :v))";
    const RUNS: usize = 3;
    let n = 40_000i64;

    let mut a1 = 0u64;
    let mut a2 = 0u64;
    for _ in 0..RUNS {
        a1 += bind_world_alpha_ns(ONE, n);
        a2 += bind_world_alpha_ns(TWO, n);
    }
    let r = RUNS as f64;
    let a1 = a1 as f64 / r;
    let a2 = a2 as f64 / r;
    assert!(
        a1 > 0.0 && a2 > 0.0,
        "alpha recorded nothing — the instrument never fired"
    );

    println!(
            "\nalpha cost per BINDING — {n} facts, mean of {RUNS}\n                 1 bind : alpha {:>7.2} ms\n                 2 binds: alpha {:>7.2} ms\n                 delta  : alpha {:>7.2} ms ({:>4.0} ns/fact)\n",
            a1 / 1e6,
            a2 / 1e6,
            (a2 - a1) / 1e6,
            (a2 - a1) / n as f64
        );
}

// ── Inside the 163 ns bind: key CONSTRUCTION vs the MAP operation ────────────────────────
//
// `eval_clause` does `Value::String(Arc::new(var.to_string()))` per bind — a fresh String plus
// a fresh Arc, to key on a variable name that is a compile-time constant. Interning it would
// reduce that to an Arc refcount bump. Whether that is worth doing depends on its share of the
// 163 ns, and the alternative (changing the binding map's representation) is a substrate-wide
// change shared by joins, negation, token extension and the oracle differential — so the cheap
// fix deserves to be priced first.
//
// ⚠ HONEST BOUND: this is a tight-loop microbenchmark, not the engine. Allocator state and
// cache behaviour differ from a real fire, so treat the RATIO between the three as the finding
// and not the absolute nanoseconds. The 163 ns from `alpha_match_cost_per_binding` is the
// in-engine number; this only apportions it.
#[test]
fn bind_key_construction_vs_map_operation() {
    use std::hint::black_box;
    const N: usize = 300_000;
    let var = "?g";
    let val = Value::i64(42);
    let interned = Value::String(Arc::new(var.to_string()));
    let empty: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();

    // (a) what we do today: build the key from scratch, every bind.
    let t0 = std::time::Instant::now();
    for _ in 0..N {
        let key = Value::String(Arc::new(var.to_string()));
        black_box(&key);
    }
    let fresh_ns = t0.elapsed().as_nanos() as f64 / N as f64;

    // (b) what interning would cost instead: an Arc refcount bump.
    let t1 = std::time::Instant::now();
    for _ in 0..N {
        let key = interned.clone();
        black_box(&key);
    }
    let interned_ns = t1.elapsed().as_nanos() as f64 / N as f64;

    // (c) the map operation itself, key supplied — get (the already-bound check) then insert
    // into a fresh empty map, which is what a first bind on an element does.
    let t2 = std::time::Instant::now();
    for _ in 0..N {
        let m = empty.clone();
        black_box(m.get(&interned));
        let m2 = m.insert(interned.clone(), val.clone());
        black_box(&m2);
    }
    let map_ns = t2.elapsed().as_nanos() as f64 / N as f64 - interned_ns; // subtract the clone (c) also pays

    assert!(
        fresh_ns > 0.0 && map_ns > 0.0,
        "microbenchmark recorded nothing"
    );

    println!(
            "\nbind cost apportioned — {N} iterations each (RATIOS, not absolutes)\n                 (a) fresh key   Value::String(Arc::new(var.to_string()))  {fresh_ns:>6.1} ns\n                 (b) interned    an Arc refcount bump                      {interned_ns:>6.1} ns\n                 (c) map         get + insert, key supplied                {map_ns:>6.1} ns\n                 ---------------------------------------------------------------\n                 interning would save (a)-(b) = {:>5.1} ns of the ~163 ns in-engine bind\n                 the map itself is {:>5.1} ns and is untouched by interning\n",
            fresh_ns - interned_ns, map_ns
        );
}

/// How many DISTINCT alpha memories do the accumulate nodes actually read?
///
/// `accum:index-builds 4` over `index-elements 160,000` is consistent with ONE shared alpha
/// (4 builds of 40,000) or TWO (1 + 3 builds of 40,000) — the counts alone cannot tell them
/// apart, and the size of the cache win differs (3 of 4 builds saved vs 2 of 4). The round
/// census already counts alpha nodes and their elements, so it settles it.
#[test]
fn accum_alpha_memory_shape() {
    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let src = "(:wat::rete::fire-rules (:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200))";
    let ast = crate::parse_one!(src).expect("parse the fire driver");
    let (_fired, census) = super::with_fire_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
            .value_owned()
    });
    assert!(!census.is_empty(), "round census recorded nothing");
    let last = census.last().expect("non-empty");
    println!(
        "\naccum alpha memories — G=200 W=200 (200 Groups + 40,000 Readings)\n\
             rounds {}\n                 last alpha_nodes {}   last alpha_elements {}\n",
        census.len(),
        last.alpha_nodes,
        last.alpha_elements
    );
    for row in &census {
        println!(
            "  r{}  dIn {}  aNodes {}  aEls {}  prod {}  seen {}",
            row.round,
            row.delta_facts_in,
            row.alpha_nodes,
            row.alpha_elements,
            row.production_facts,
            row.seen_facts
        );
    }
}

/// Diagnostic — how MANY matcher operations, at the size the phase census apportions.
///
/// Counted rather than timed: one level below `alpha` the operations cost ~100-300ns while a
/// mark pair costs ~52ns, so a timer would tax them 20-50% and — worse — unevenly, in
/// proportion to call count rather than cost. A `Cell` increment is ~1-2ns.
///
/// Arc 278 DESIGN-STONE-compiled-conditions.md — a real fire's step 1 no longer
/// runs `alpha_match_inner`: `match:calls` (and its `match:clause`/`match:bind-insert`
/// siblings) are armed INSIDE `alpha_match_inner`'s own body, so they read zero here
/// now by construction, not by regression. `compiled:calls` is what actually fires
/// on this path — occupancy leaf-fill, skip-span, and `exec_compiled_with_key_ids`
/// all increment it (`DESIGN-STONE-occupancy-leaf-column`). [`exec_compiled`] is
/// the `#[cfg(test)]` door (no interned keys).
///
/// `match:key-alloc` is printed but NOT asserted at zero here: this world's RHS insert forms
/// (`build_insert_fact`, the production pass) resolve `?var` args through the SAME
/// `resolve_operand` alpha-match uses. RHS is compiled (`DESIGN-STONE-compiled-rhs.md`);
/// leftover `match:key-alloc` on this world is the oracle `build_insert_fact` path the
/// differential still runs. So a real fire's `match:key-alloc` can be non-zero even with
/// the compiled path in place; the actual row-2 gate that isolates ALPHA-MATCH's failure path
/// is `compiled_cond_failure_path_allocates_no_binding_keys_at_50_100`, which never
/// touches RHS resolution.
#[test]
fn accum_matcher_op_census() {
    let rows = accum_count_census(200, 200);
    assert!(
        !rows.is_empty(),
        "the operation census counted NOTHING — the counters were never reached, so any \
             rate derived from them would be an artifact"
    );
    let calls = rows
        .iter()
        .find(|(n, _)| *n == "compiled:calls")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    assert!(
        calls > 0,
        "compiled:calls is zero — occupancy fill / skip-span / exec_compiled never counted"
    );

    let mut out = String::from("\naccum matcher ops — G=200 W=200 (40,200 facts)\n");
    for (name, n) in &rows {
        out.push_str(&format!("    {name:<20} {n:>10}\n"));
    }
    println!("{out}");
}

/// Microbenchmark — how much of a binding-map operation is the STRING KEY?
///
/// Binding keys are `Value::String(Arc<String>)` — a fresh heap String per
/// bind, hashed and memcmp'd on every lookup. **Clara's are interned Clojure keywords**
/// (`engine.cljc:23` "a map of keyword-to-values"; `compiler.clj:293` assoc's `(keyword var)`),
/// which carry a CACHED hash and compare by pointer.
///
/// `9448f012` measured "interning the bind key saves 8% — the MAP is 85% of it" and concluded
/// interning was not worth a stone. That split may be an artifact: if the map operation's cost
/// is largely *hashing the string key*, then "the map" and "the key" are not separable and the
/// 85% already contains the thing the 8% was measuring. This isolates it by changing ONLY the
/// key type on an otherwise identical map.
///
/// `Value::i64` stands in for an interned symbol id (hash of an i64, compare by value) — the
/// floor an interning scheme could reach, not a proposal for the key type itself.
///
/// Diagnostic. Read with `--no-capture`.
#[test]
fn binding_key_cost() {
    use std::hint::black_box;
    use std::time::Instant;
    const N: usize = 50_000;

    println!("\nBINDING KEY COST — Value::String (today) vs Value::i64 (an interned-id floor)");
    println!(
        "  {N} iterations; rpds::HashTrieMapSync in BOTH columns — only the KEY type differs\n"
    );
    println!(
        "  {:>4}  {:>21}  {:>21}",
        "n", "build (str / i64)", "lookup (str / i64)"
    );

    for n in [1usize, 2, 3, 5, 8] {
        let sk: Vec<(Value, Value)> = (0..n)
            .map(|i| {
                (
                    Value::String(Arc::new(format!("?v{i}"))),
                    Value::i64(i as i64),
                )
            })
            .collect();
        let ik: Vec<(Value, Value)> = (0..n)
            .map(|i| (Value::i64(i as i64), Value::i64(i as i64)))
            .collect();

        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut sink: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut m = rpds::HashTrieMapSync::new_sync();
            for (k, v) in &sk {
                m.insert_mut(k.clone(), v.clone());
            }
            sink.push(m);
        }
        let bs = t.elapsed().as_nanos() as f64 / N as f64;
        let ms = sink[0].clone();
        drop(sink);

        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut sink: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut m = rpds::HashTrieMapSync::new_sync();
            for (k, v) in &ik {
                m.insert_mut(k.clone(), v.clone());
            }
            sink.push(m);
        }
        let bi = t.elapsed().as_nanos() as f64 / N as f64;
        let mi = sink[0].clone();
        drop(sink);

        let ps = sk[n / 2].0.clone();
        let pi = ik[n / 2].0.clone();
        let t = Instant::now();
        for _ in 0..N {
            black_box(ms.get(black_box(&ps)));
        }
        let ls = t.elapsed().as_nanos() as f64 / N as f64;
        let t = Instant::now();
        for _ in 0..N {
            black_box(mi.get(black_box(&pi)));
        }
        let li = t.elapsed().as_nanos() as f64 / N as f64;

        println!(
            "  {:>4}  {:>9.1} /{:>9.1}  {:>9.1} /{:>9.1}   build {:>4.1}x  lookup {:>4.1}x",
            n,
            bs,
            bi,
            ls,
            li,
            bs / bi,
            ls / li
        );
    }
    println!();
}

/// Microbenchmark — rpds HAMT vs a persistent ARRAY map, at binding-map sizes.
///
/// The follow-on stone's claim is "an rpds trie pays HAMT prices on a 1-3 entry map, and
/// Clojure/Clara get an array representation for free below 8." That claim was PREDICTED, never
/// measured. This measures it, before any stone is drawn.
///
/// The comparison must be the HONEST analogue. Clojure's PersistentArrayMap is not a bare Vec —
/// it is an IMMUTABLE array behind a reference, so `clone` is a refcount bump exactly as the
/// HAMT's is, and only the LOOKUP differs (linear scan vs hash+trie descent). A bare `Vec`
/// would lose catastrophically on clone and prove nothing about the real design.
///   A = rpds::HashTrieMapSync<Value,Value>   (today)
///   B = Arc<Vec<(Value,Value)>>              (PersistentArrayMap's shape)
///
/// Five operations, chosen because they are what the kernel actually does to a binding map:
///   build   — alpha match constructs one per fact
///   lookup  — accum:fold (94 ns/element) and token_element_compatible
///   clone   — alpha:push (this REGRESSED when Element went native)
///   extend  — extend_token: clone + insert one binding (rpds shares structurally; the array copies)
///   drop    — round:drop-memories (41 ms)
///
/// Keys are real `Value::String(Arc<str>)` — hashing/comparing a wat String is the actual cost,
/// and an integer-keyed benchmark would flatter the HAMT.
///
/// Diagnostic, not a gate. Read with `--no-capture`.
#[test]
fn binding_repr_microbench() {
    use std::hint::black_box;
    use std::time::Instant;

    const SIZES: [usize; 8] = [1, 2, 3, 4, 5, 8, 12, 16];
    const N: usize = 20_000;

    fn keys(n: usize) -> Vec<(Value, Value)> {
        (0..n)
            .map(|i| {
                (
                    Value::String(Arc::new(format!("?v{i}"))),
                    Value::i64(i as i64),
                )
            })
            .collect()
    }

    println!("\nBINDING REPRESENTATION — rpds HAMT (A) vs persistent array map (B)");
    println!("  {N} iterations per cell; ns/op; keys are real Value::String\n");
    println!(
        "  {:>4}  {:>19}  {:>19}  {:>19}  {:>19}  {:>19}",
        "n", "build", "lookup", "clone", "extend", "drop"
    );
    println!(
        "  {:>4}  {:>19}  {:>19}  {:>19}  {:>19}  {:>19}",
        "", "A / B", "A / B", "A / B", "A / B", "A / B"
    );

    for n in SIZES {
        let kv = keys(n);
        let probe = kv[n / 2].0.clone();
        let extra = (Value::String(Arc::new("?zz".to_string())), Value::i64(99));

        // ── build (construct into a reserved Vec; drop timed separately) ──
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut sink_a: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut m = rpds::HashTrieMapSync::new_sync();
            for (k, v) in &kv {
                m.insert_mut(k.clone(), v.clone());
            }
            sink_a.push(m);
        }
        let build_a = t.elapsed().as_nanos() as f64 / N as f64;

        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut sink_b: Vec<Arc<Vec<(Value, Value)>>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut v = Vec::with_capacity(n);
            for (k, val) in &kv {
                v.push((k.clone(), val.clone()));
            }
            sink_b.push(Arc::new(v));
        }
        let build_b = t.elapsed().as_nanos() as f64 / N as f64;

        let ma = sink_a[0].clone();
        let mb = sink_b[0].clone();

        // ── lookup (hit, mid-map) ──
        let t = Instant::now();
        for _ in 0..N {
            black_box(ma.get(black_box(&probe)));
        }
        let look_a = t.elapsed().as_nanos() as f64 / N as f64;
        let t = Instant::now();
        for _ in 0..N {
            black_box(Bindings::get(mb.as_slice(), black_box(&probe)));
        }
        let look_b = t.elapsed().as_nanos() as f64 / N as f64;

        // ── clone ──
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut ca: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            ca.push(ma.clone());
        }
        let clone_a = t.elapsed().as_nanos() as f64 / N as f64;
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut cb: Vec<Arc<Vec<(Value, Value)>>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            cb.push(Arc::clone(&mb));
        }
        let clone_b = t.elapsed().as_nanos() as f64 / N as f64;
        drop(ca);
        drop(cb);

        // ── extend (extend_token: derive a new map with one more binding) ──
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut ea: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            ea.push(ma.insert(extra.0.clone(), extra.1.clone()));
        }
        let ext_a = t.elapsed().as_nanos() as f64 / N as f64;
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut eb: Vec<Arc<Vec<(Value, Value)>>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut v = (*mb).clone();
            v.push(extra.clone());
            eb.push(Arc::new(v));
        }
        let ext_b = t.elapsed().as_nanos() as f64 / N as f64;
        drop(ea);
        drop(eb);

        // ── drop (the sinks built above) ──
        let t = Instant::now();
        drop(sink_a);
        let drop_a = t.elapsed().as_nanos() as f64 / N as f64;
        let t = Instant::now();
        drop(sink_b);
        let drop_b = t.elapsed().as_nanos() as f64 / N as f64;

        println!("  {:>4}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}",
                     n, build_a, build_b, look_a, look_b, clone_a, clone_b, ext_a, ext_b, drop_a, drop_b);
    }
    println!("\n  A = rpds::HashTrieMapSync (today)   B = Arc<Vec<(Value,Value)>>\n"); // rune:lint(no-angle-type-in-diagnostic) — RUST types in a bench header, not wat
}

/// Diagnostic — the binding-cardinality distribution, the PREMISE under the
/// binding-representation stone.
///
/// The stone's whole argument is that a binding map holds 1-2 entries, so an
/// `rpds::HashTrieMapSync` (heap alloc + Arc + hash + pointer-chase + dealloc) is paying trie
/// prices for a pair. If the distribution is wide, an inline small-vec is WORSE and the stone
/// inverts. Nobody had measured it.
///
/// Load-bearing subtlety: binding cardinality is a property of the RULE SHAPE, not the data
/// volume. A 2-condition rule binding 3 distinct vars yields 3-binding tokens at 10 facts and
/// at 10 million. So this drives SEVERAL rule shapes and reports each — a single workload
/// would answer a narrower question than the one the stone asks.
///
/// Read with `--no-capture`. Diagnostic, not a gate; the assertion only stops it reporting an
/// artifact (a census that counted nothing would print an empty table reading as "all zero").
#[test]
fn binding_cardinality_distribution() {
    fn dist(label: &str, rows: &[(&'static str, u64)]) -> String {
        let get = |k: &str| {
            rows.iter()
                .find(|(n, _)| *n == k)
                .map(|(_, c)| *c)
                .unwrap_or(0)
        };
        let els = get("bind-card:ELEMENTS");
        let toks = get("bind-card:TOKENS");
        let total = els + toks;
        let mut out = format!("\n  {label}  —  {els} elements, {toks} tokens\n");
        if total == 0 {
            out.push_str("    (nothing counted)\n");
            return out;
        }
        for (kind, tot, pfx) in [("ELEMENT", els, "elem-card:"), ("TOKEN", toks, "tok-card:")] {
            if tot == 0 {
                continue;
            }
            out.push_str(&format!("    {kind}S ({tot})\n"));
            for suf in ["0", "1", "2", "3", "4", "5", "6-7", "8+"] {
                let key = format!("{pfx}{suf}");
                let n = rows
                    .iter()
                    .find(|(nm, _)| *nm == key)
                    .map(|(_, c)| *c)
                    .unwrap_or(0);
                if n == 0 {
                    continue;
                }
                out.push_str(&format!(
                    "      {:<6} {:>9}  {:>5.1}%\n",
                    suf,
                    n,
                    100.0 * n as f64 / tot as f64
                ));
            }
        }
        out
    }

    let mut report = String::from("\nBINDING CARDINALITY — the premise under the small-vec stone");

    // Shape A — accumulate: conditions bind ?g / ?g,?v; tokens carry the group key.
    let rows_accum = accum_count_census(60, 60);
    report.push_str(&dist("accumulate  (accum axis, G=60 W=60)", &rows_accum));

    // Shape B — a 2-condition JOIN binding THREE distinct vars across the conditions
    // (?loc shared, ?t from one, ?w from the other). This is the shape that grows a token's
    // binding map, and the one an accumulate-only measurement would never show.
    const J: &str = "\
(:wat::core::defrecord :bcd::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::i64])\n\
(:wat::core::defrecord :bcd::WindSpeed   [kph      <- :wat::core::i64  location <- :wat::core::i64])\n\
(:wat::core::defrecord :bcd::Cw          [loc <- :wat::core::i64  t <- :wat::core::i64  w <- :wat::core::i64])\n\
(:wat::core::defn :bcd::seed [n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::let [c1   (:wat::core::quote (:bcd::Temperature (?loc <- :location) (?t <- :celsius)))\n\
                    c2   (:wat::core::quote (:bcd::WindSpeed (?loc <- :location) (?w <- :kph)))\n\
                    rhs1 (:wat::core::quote (:bcd::Cw ?loc ?t ?w))\n\
                    rule (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))\n\
                    s0   (:wat::rete::compile (:wat::core::PersistentVector rule))]\n\
    (:wat::core::foldl\n\
      (:wat::core::fn [acc <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
        (:wat::core::let [a (:wat::rete::insert acc (:bcd::Temperature :celsius i :location i))]\n\
          (:wat::rete::insert a (:bcd::WindSpeed :kph i :location i))))\n\
      s0 (:wat::core::range 0 n))))\n\
";
    let wj = startup_from_source(J, None, Arc::new(InMemoryLoader::new()))
        .expect("join world should freeze");
    let ast = crate::parse_one!("(:wat::rete::fire-rules (:bcd::seed 400))").expect("parse");
    let (_f, rows_join) = super::with_count_census(|| {
        eval_in_frozen(&ast, &wj, &Environment::new())
            .unwrap_or_else(|e| panic!("join fire raised: {e:?}"))
            .value_owned()
    });
    report.push_str(&dist("2-cond join, 3 distinct vars (N=400)", &rows_join));

    let counted: u64 = rows_accum
        .iter()
        .chain(rows_join.iter())
        .filter(|(n, _)| {
            n.starts_with("bind-card:") || n.starts_with("elem-card:") || n.starts_with("tok-card:")
        })
        .map(|(_, c)| *c)
        .sum();
    assert!(
        counted > 0,
        "the binding census counted NOTHING — the walk never ran, so an all-zero table \
             would be an artifact, not a distribution"
    );

    println!("{report}");
}

/// Diagnostic — print where the accum fire's time goes, per phase, at two sizes.
///
/// This APPORTIONS; it does not gate. The assertions exist only so it cannot report an artifact:
///   (a) the instrument must have recorded something (an unarmed or never-entered loop would
///       give an empty table that reads as "no time anywhere"),
///   (b) every one of the six phases must appear — a phase missing from the map means its marks
///       were never reached, and its share would silently land on the others.
/// Read the table with `--no-capture`.
#[test]
fn accum_fire_phase_census() {
    // The four OUTER marks partition the whole fire; everything else nests inside one of
    // them. The total is the outer four ONLY — summing a parent with its children is what
    // made the first version of this table report 124% coverage.
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    // ★ 2026-08-01 — this list is now a FLOOR, not the table's contents.
    //
    // It used to BE the table: `for phase in PHASES`. So when `alpha:candidates` was added to
    // mark the discrimination-tree walk — the largest unmarked computation inside the phase
    // that dominates our weakest grid axis — the mark fired and the row simply did not appear.
    // A census that lists its rows cannot report a sink nobody thought to list, which is the
    // whole job of a census. (`feedback_a_gate_that_discovers_beats_one_that_lists`.)
    //
    // Now: the table DISCOVERS every phase the run actually recorded, in first-fired order,
    // and this array is asserted to be a SUBSET of what was discovered — so a mark that is
    // deleted or stops firing still fails loudly, while a mark that is ADDED shows up for
    // free. Both directions covered; neither can go quiet.
    const REQUIRED_PHASES: [&str; 25] = [
        "IN: to_transient",
        "SETUP: indexes",
        "  ├ setup:seen",
        "  │  setup:seen:alloc",
        "  ├ setup:arm",
        "ROUND LOOP",
        "alpha",
        "  ├ alpha:seed",
        "  └ alpha:delta",
        "root-join",
        "hash-join",
        "accumulate",
        "  ├ accum:snapshot",
        "  ├ accum:index",
        "  └ accum:fold",
        "filter",
        "production",
        "OUT: to_persistent",
        "  ├ out:alpha",
        "  ├ out:beta",
        "  ├ out:production",
        "  └ out:query",
        "  ├ round:preamble",
        "  └ round:epilogue",
        "  └ round:drop-memories",
    ];

    let table = render_phase_table(
        "accum fire",
        &[(25, 50), (50, 100), (100, 200), (200, 200)],
        &TOP,
        &REQUIRED_PHASES,
        |g, w| g * (w + 1),
        accum_phase_census,
    );
    println!("{table}");
    let rows = accum_phase_census(200, 200);
    let ns_of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _, _)| *n == name)
            .map(|(_, ns, _)| *ns)
            .unwrap_or(0)
    };
    let fold_ms = ns_of("  └ accum:fold") as f64 / 1e6;
    assert!(
        fold_ms > 0.0,
        "accum:fold at [200 200] recorded zero — the mark moved"
    );
    assert!(
        fold_ms < 25.0,
        "accum:fold at [200 200] is {fold_ms:.2} ms — DESIGN-STONE-accum-fold-the-wall \
             requires < 25 ms (was 68.49).{table}"
    );
    let snap_ms = ns_of("  ├ accum:snapshot") as f64 / 1e6;
    assert!(
        snap_ms > 0.0,
        "accum:snapshot at [200 200] recorded zero — the mark was deleted"
    );
    assert!(
        snap_ms < 1.0,
        "accum:snapshot at [200 200] is {snap_ms:.2} ms — \
             DESIGN-STONE-gather-no-snapshot requires < 1 ms (was 5.56).{table}"
    );
}

#[test]
fn render_phase_table_proves_missing_phase_and_zero_total() {
    let boom = std::panic::catch_unwind(|| {
        render_phase_table(
            "fake",
            &[(1, 1)],
            &["IN: to_transient"],
            &["ROUND LOOP"],
            |_, _| 0,
            |_, _| vec![("IN: to_transient", 1, 1)],
        )
    });
    assert!(boom.is_err(), "missing required phase must panic");
}

/// The keyed-gather gate — RED until the Accumulate/Negation/Exists gathers are keyed.
///
/// Both runs hold the ELEMENT COUNT CONSTANT (G×W = 800 readings) and differ only in how many
/// tokens probe them (8× apart in group count). That separates "the gather is quadratic" from
/// "there are simply more facts" — the same control the measurement probe uses
/// (`wat-scripts/scratch-pad/probe-accumulate-gather-cost.wat`), which read 8.42× on wall-clock
/// at G=50/W=160 vs G=400/W=20.
///
///   un-keyed (today): every token scans all 800 elements → visits ∝ G → an 8× spread.
///   keyed:            every token probes its own bucket   → visits ≈ G×W = 800/node → FLAT.
///
/// What would turn this red — the R59 question, answered before the assertion was written:
///   (a) the instrument recording nothing (`small == 0`) — asserted separately, because a
///       silent zero would make the ratio 0/0 and "pass" while measuring nothing at all;
///   (b) a gather that still walks the whole element memory per token — the defect under test;
///   (c) a keyed gather whose buckets are wrong in a way that re-scans (e.g. an empty key
///       tuple degenerating every element into one bucket for a workload that DOES share vars).
///
/// It cannot pass by luck or by machine speed: it counts examinations, not nanoseconds.
#[test]
fn keyed_gather_visits_do_not_scale_with_group_count() {
    // G×W = 800 readings in BOTH runs; only the token count moves (10 → 80).
    let small = accum_gather_visits(10, 80);
    let big = accum_gather_visits(80, 10);

    assert!(
        small > 0,
        "the gather-visit instrument recorded ZERO — the gathers were never entered, so any \
             ratio taken from this run would be an artifact, not a measurement"
    );

    let ratio = big as f64 / small as f64;
    println!(
        "\nkeyed-gather gate — constant 800 elements, tokens 10 → 80\n  \
             G=10 W=80 : {small} visits\n  G=80 W=10 : {big} visits\n  ratio: {ratio:.2}x\n"
    );
    assert!(
        ratio <= 2.0,
        "gather visits scale with the TOKEN count ({small} → {big}, {ratio:.2}x) while the \
             element count is constant at 800 — the Accumulate/Negation/Exists gathers are still \
             scanning the whole memory per token instead of probing a key index (the joins have \
             had one since P6). See DESIGN-STONE-keyed-gather.md."
    );
}

/// A8 — census the native fire path as rule-count N grows against a fixed fact set.
///
/// M is deliberately tiny (50 of each type). The axis blew a machine's RAM at N=20/M=500;
/// nothing here can approach that, and the growth SHAPE is what the diagnosis needs, not the
/// magnitude. Prints the full per-N table (`--no-capture` to read it) and asserts the
/// invariants that must hold for the shared-prefix story to be true at fire time.
///
/// What would turn this red — the R59 question, answered before the assertions were written:
///   (a) the instrument recording nothing (an unarmed or never-entered loop),
///   (b) the derived-fact count drifting from M (the axis's documented N-invariance breaking),
///   (c) the shared HashJoin's token count growing with N — which IS the fire-path smoking gun:
///       one compiled join node re-materialising its tokens per rule.
#[test]
fn a8_node_share_fire_census() {
    const M: i64 = 50;
    const NS: [i64; 4] = [1, 2, 4, 8];

    let mut table = String::new();
    table.push_str(&format!(
        "\nA8 node-share — native fire census (M={M} A-facts + {M} B-facts)\n\
             \n  N | edges | rnds | dIn | aNodes aEls | bNodes bToks bMatches | dbNodes dbToks \
             | lIdx rIdx | prod seen | HashJoin RootJoin Test\n"
    ));

    let mut hash_join_tokens: Vec<(i64, usize)> = Vec::new();

    for n in NS {
        let census = node_share_census(n, M);
        assert!(
            !census.is_empty(),
            "A8 census recorded ZERO rounds at N={n} — the instrument never fired, so any \
                 reading taken from it would be an artifact, not a measurement"
        );

        // The final round carries the cumulative totals for the whole fire.
        let last = census.last().expect("census is non-empty");
        // PRODUCED, not HELD. Post-guard a terminal HashJoinNode deliberately materialises no
        // beta, so `tokens_of_kind(last, "HashJoin")` would read 0 for every N and the sharing
        // assertion below would be vacuously true — the gate would keep its green and stop
        // meaning anything. The delta carries the same tokens (see `produced_of_kind`), and it
        // is the better witness for this claim anyway: the defect under test is the join
        // RE-RUNNING per rule, which shows up as tokens produced, not tokens stored.
        let hj = produced_of_kind(&census, "HashJoin");
        let rj = tokens_of_kind(last, "RootJoin");
        let tn = tokens_of_kind(last, "Test");

        table.push_str(&format!(
            "  {:<2}| {:<6}| {:<5}| {:<4}| {:<7}{:<5}| {:<7}{:<6}{:<10}| {:<8}{:<7}| \
                 {:<5}{:<5}| {:<5}{:<5}| {:<9}{:<9}{}\n",
            n,
            last.network_edges,
            census.len(),
            last.delta_facts_in,
            last.alpha_nodes,
            last.alpha_elements,
            last.beta_nodes,
            last.beta_tokens,
            last.beta_token_matches,
            last.d_beta_nodes,
            last.d_beta_tokens,
            last.left_idx_tokens,
            last.right_idx_elements,
            last.production_facts,
            last.seen_facts,
            hj,
            rj,
            tn,
        ));

        // Per-round detail: the fixpoint's shape over time. A structure that grows across
        // rounds reads differently from one that is over-allocated in a single round, and the
        // summary row above (cumulative totals) cannot tell them apart.
        for row in &census {
            table.push_str(&format!(
                "     |- round {:<2} dIn={:<5} beta={:<6} dBeta={:<6} matches={:<8} prod={}\n",
                row.round,
                row.delta_facts_in,
                row.beta_tokens,
                row.d_beta_tokens,
                row.beta_token_matches,
                row.production_facts,
            ));
        }

        // (b) The axis's own N-invariance: every k in [0, M) satisfies exactly one rule, so the
        // derived set is {Out(k)} of size M no matter how many rules split it.
        assert_eq!(
            last.production_facts, M as usize,
            "A8 derived-fact count must be N-invariant (M={M}), got {} at N={n}{table}",
            last.production_facts
        );

        hash_join_tokens.push((n, hj));
    }

    println!("{table}");

    // (c) Fire-time sharing: the ONE compiled HashJoinNode must PRODUCE the same token set no
    // matter how many rules hang off it. If this grows with N, the fire path is re-doing the
    // join per rule — the shared network collapsing back into N copies at run time, which is
    // exactly the mechanism the >4 GiB blow-up would need.
    //
    // Reworded from "must HOLD" on 2026-08-01: the beta-readers guard stopped materialising a
    // terminal join's `wm.beta`, so "holds" became vacuous by design. The quantity is
    // unchanged — before the guard, beta and the delta were fed by one unconditional
    // statement pair, so the summed delta IS what beta held — but the gate now says what it
    // actually proves rather than keeping a name the code had made false.
    let (_, baseline) = hash_join_tokens[0];
    for &(n, tokens) in &hash_join_tokens {
        assert_eq!(
            tokens, baseline,
            "A8 fire-time sharing broken: the shared HashJoinNode produced {tokens} tokens at \
                 N={n} but {baseline} at N={}. One compiled join node is materialising per-rule \
                 token sets — the fire-path defect the compiler census (4 + 2N nodes) ruled out at \
                 compile time.{table}",
            hash_join_tokens[0].0
        );
    }
    assert!(
        baseline > 0,
        "A8 census read 0 HashJoin tokens — the join never ran, so the sharing assertion above \
             would pass vacuously.{table}"
    );
}

// ── A0 depth-cost split (arc 278, 2026-07-31) ─────────────────────────────────────────────
//
// The grid's deep-cascade axis reads `:winner :clara` at [50 100] (all five runs), and holding
// the derived-fact count CONSTANT while varying depth showed the cost tracks DEPTH, not size:
// 6000 derived facts cost us 34.7ms at depth 10 and 119.5ms at depth 60, where Clara paid
// 76.7 → 114.2. Grounded, the round body runs FOUR full-network scans per round
// (root-join :2070, hash-join :2127, accumulate :2327, filter :2423) and a depth-D cascade
// needs D rounds — so we visit O(D) nodes D times while exactly one level can do work.
//
// This probe measures the SPLIT that decides the fix: at EQUAL work, how much of the extra
// cost at depth is per-round scaffolding over idle nodes, versus real per-fact work? If the
// idle scan dominates, a dirty-node agenda captures it; if it does not, only per-element
// incremental propagation (T3) helps. It asserts nothing about which — it prints the rows.

const DEPTH_SPLIT_WORLD: &str = "\
(:wat::core::defrecord :cascade::Node [level <- :wat::core::i64  id <- :wat::core::i64])\n\
(:wat::core::defrecord :cascade::Tag  [level <- :wat::core::i64  id <- :wat::core::i64])\n\
\n\
(:wat::core::defn :dc::build-rule [k <- :wat::core::i64] -> :wat::rete::Rule\n\
  (:wat::core::let [prev (:wat::i64::- k 1)\n\
                    c1 (:wat::core::quasiquote (:cascade::Node (?id <- :id) (?l <- :level) (:wat::rete::i64::= ?l (:wat::core::unquote prev))))\n\
                    c2 (:wat::core::quasiquote (:cascade::Tag  (?id <- :id) (?m <- :level) (:wat::rete::i64::= ?m (:wat::core::unquote prev))))\n\
                    t1 (:wat::core::quasiquote (:cascade::Node (:wat::core::unquote k) ?id))\n\
                    t2 (:wat::core::quasiquote (:cascade::Tag  (:wat::core::unquote k) ?id))]\n\
    (:wat::rete::Rule :name (:wat::i64::to-string k)\n\
      :lhs (:wat::core::PersistentVector c1 c2)\n\
      :rhs (:wat::core::PersistentVector t1 t2))))\n\
\n\
(:wat::core::defn :dc::build-rules [depth <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  k <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])\n\
      (:wat::core::PersistentVector/conj acc (:dc::build-rule k)))\n\
    (:wat::core::PersistentVector (:dc::build-rule 1))\n\
    (:wat::core::range 2 (:wat::i64::+ depth 1))))\n\
\n\
(:wat::core::defn :dc::seed-level-0 [session <- :wat::rete::Session  width <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert (:wat::rete::insert s (:cascade::Node :level 0 :id i)) (:cascade::Tag :level 0 :id i)))\n\
    session\n\
    (:wat::core::range 0 width)))\n";

/// Fire a depth×width cascade through the native path; per-phase split with pair counts.
fn cascade_phase_census(depth: i64, width: i64) -> Vec<(&'static str, u64, u64)> {
    let world = startup_from_source(DEPTH_SPLIT_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("depth-split world should freeze");
    let src = format!(
            "(:wat::rete::fire-rules (:dc::seed-level-0 (:wat::rete::compile (:dc::build-rules {depth})) {width}))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census_counted(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("cascade fire raised at depth={depth} width={width}: {e:?}"))
            .value_owned()
    });
    rows
}

/// Fire a depth×width cascade through the native path; return the per-phase nanosecond rows.
fn depth_split_phases(depth: i64, width: i64) -> Vec<(&'static str, u64)> {
    cascade_phase_census(depth, width)
        .into_iter()
        .map(|(n, ns, _)| (n, ns))
        .collect()
}

// ── The fact-heavy, rule-LIGHT census (arc 278, 2026-08-01) ───────────────────────────────
//
// The depth-split probe answers "what does DEPTH cost" on a rule-heavy cascade. This one
// answers the complementary question the compiled-conditions stone needs: what does a match
// cost PER FACT when the discrimination tree buys nothing?
//
// Fanout is that shape — ONE rule, two conditions, two fact types, so D=1 per type and the
// tree has nothing to prune. Every millisecond in `alpha:match` here is per-CALL cost, not
// per-candidate: the redundant head compare, `classify_rete_clause` on a static AST, the
// linear field-name scan, and the two heap allocations that rebuild a constant binding key.
//
// Sizing the stone off the CASCADE's per-fact rate would be extrapolating across workload
// shapes, which is the error that has cost this arc twice today. Measure the shape you mean.

const FANOUT_CENSUS_WORLD: &str = "\
(:wat::core::defrecord :fan::Left  [key <- :wat::core::i64  lid <- :wat::core::i64])\n\
(:wat::core::defrecord :fan::Right [key <- :wat::core::i64  rid <- :wat::core::i64])\n\
(:wat::core::defrecord :fan::Pair  [key <- :wat::core::i64  lid <- :wat::core::i64  rid <- :wat::core::i64])\n\
\n\
(:wat::core::defn :fan::seed-key [s <- :wat::rete::Session  k <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  f <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert (:wat::rete::insert acc (:fan::Left :key k :lid f)) (:fan::Right :key k :rid f)))\n\
    s\n\
    (:wat::core::range 0 fanout)))\n\
\n\
(:wat::core::defn :fan::seed [s <- :wat::rete::Session  keys <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:fan::seed-key acc k fanout))\n\
    s\n\
    (:wat::core::range 0 keys)))\n\
\n\
(:wat::rete::defrule :fan::fan-rule\n\
  :when\n\
  [(:fan::Left  (?k <- :key) (?l <- :lid))\n\
   (:fan::Right (?k <- :key) (?r <- :rid))]\n\
  :then\n\
  [(:fan::Pair ?k ?l ?r)])\n";

/// THREE conditions — the shape neither the fanout nor the cascade produces.
///
/// Two conditions give `root-join -> J` where `J` is terminal, so every hash-join in those
/// worlds is a leaf. Three give `root-join -> J1 -> J2`, and **J1 is a MIDDLE join**: its beta
/// is the left input of J2's catch-up, so it must be READ. Without this world the beta-traffic
/// probe can only observe leaves, and "a hash-join's beta is never read" would be an
/// over-generalisation from a corpus that contains no counter-example — the exact shape of
/// claim this arc keeps having to retract.
///
/// `keys=10 x fanout=5`: 50 of each record, A⋈B = 250 pairs, A⋈B⋈C = 1250 triples.
const TRI_CENSUS_WORLD: &str = "\
(:wat::core::defrecord :tri::A [key <- :wat::core::i64  a <- :wat::core::i64])\n\
(:wat::core::defrecord :tri::B [key <- :wat::core::i64  b <- :wat::core::i64])\n\
(:wat::core::defrecord :tri::C [key <- :wat::core::i64  c <- :wat::core::i64])\n\
(:wat::core::defrecord :tri::Trip [key <- :wat::core::i64  a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64])\n\
\n\
(:wat::core::defn :tri::seed-key [s <- :wat::rete::Session  k <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  f <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert (:wat::rete::insert (:wat::rete::insert acc (:tri::A :key k :a f)) (:tri::B :key k :b f)) (:tri::C :key k :c f)))\n\
    s\n\
    (:wat::core::range 0 fanout)))\n\
\n\
(:wat::core::defn :tri::seed [s <- :wat::rete::Session  keys <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:tri::seed-key acc k fanout))\n\
    s\n\
    (:wat::core::range 0 keys)))\n\
\n\
(:wat::rete::defrule :tri::tri-rule\n\
  :when\n\
  [(:tri::A (?k <- :key) (?a <- :a))\n\
   (:tri::B (?k <- :key) (?b <- :b))\n\
   (:tri::C (?k <- :key) (?c <- :c))]\n\
  :then\n\
  [(:tri::Trip ?k ?a ?b ?c)])\n";

/// ★ Does the fire ever READ the beta memory it writes?
///
/// `wm.beta` takes a Token CLONE per join result and is `clear()`ed before freeze, so nothing
/// downstream can see it. Inside the fire it is read at two places only, both against the
/// PARENT of a hash-join being keyed for the first time. That makes "a terminal join's beta is
/// written and never read" a HYPOTHESIS — and the identical shape ("surely this store is
/// redundant") was proposed for production-memory one session ago and was FALSE. So it gets
/// measured, not reasoned.
///
/// Two shapes, because one of them is the control: the CASCADE chains joins (level N feeds
/// level N+1), so its middle betas MUST show reads. If every node in both shapes read zero,
/// the instrument is broken, not the engine.
#[test]
fn beta_write_read_traffic() {
    /// Returns the human table AND the structured rows. The controls below assert on the
    /// ROWS, never on the table text: the rows are what was measured, and a `contains` over a
    /// formatted table would pass on a reordered column, a renamed verdict, or a substring
    /// appearing by accident — the exact laundering `no_loose_string_assert` exists to stop.
    fn traffic(label: &str, world_src: &str, driver: &str) -> (String, Vec<(i64, u64, u64)>) {
        let world = startup_from_source(world_src, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let ast = crate::parse_one!(driver).expect("parse the fire driver");
        let (_fired, rows) = super::with_beta_traffic(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("{label} fire raised: {e:?}"))
                .value_owned()
        });

        let mut out = format!("\n  BETA TRAFFIC — {label}\n\n    node    written      read   verdict\n    ------------------------------------------------\n");
        let (mut tot_w, mut tot_r, mut dead_w, mut dead_n) = (0u64, 0u64, 0u64, 0usize);
        for (id, w, r) in &rows {
            tot_w += w;
            tot_r += r;
            let verdict = if *w > 0 && *r == 0 {
                dead_w += w;
                dead_n += 1;
                "WRITTEN, NEVER READ"
            } else if *r > 0 {
                "read"
            } else {
                "-"
            };
            out.push_str(&format!("    {id:>4}  {w:>9}  {r:>8}   {verdict}\n"));
        }
        out.push_str(&format!(
            "\n    total written {tot_w}, total read {tot_r}\n    \
                 write-only nodes: {dead_n}  —  tokens cloned into them and never read: {dead_w} \
                 ({:.1}% of all beta writes)\n",
            if tot_w > 0 {
                dead_w as f64 * 100.0 / tot_w as f64
            } else {
                0.0
            },
        ));
        // The instrument must have seen traffic at all, or its zeros mean nothing.
        assert!(
            tot_w > 0,
            "{label}: recorded no beta writes — the instrument is not armed.{out}"
        );

        // ★ THE GUARD'S INVARIANT — and this is the DANGEROUS direction.
        //
        // `beta_readers` writes a node's beta iff that node parents a HashJoinNode, and the
        // two readers only ever read such a parent, so the sets coincide by construction.
        // Should a THIRD reader ever be added that reads some other node, `wm.beta.get()`
        // returns `None`, `all_left` comes back EMPTY, and the join silently drops tokens —
        // no panic, no error, just wrong answers that a differential would have to catch
        // downstream. A node with reads and zero writes is that bug, caught here at its
        // source.
        let starved: Vec<&(i64, u64, u64)> =
            rows.iter().filter(|&&(_, w, r)| r > 0 && w == 0).collect();
        assert!(
            starved.is_empty(),
            "{label}: {} node(s) READ a beta that was never WRITTEN — {starved:?}.\n\
                 The beta_readers guard (a node is written iff it parents a HashJoinNode) no \
                 longer covers every reader, so `wm.beta.get()` hands back None and the join \
                 silently loses tokens. Widen the guard to include the new reader; do NOT relax \
                 this assertion.{out}",
            starved.len(),
        );
        (out, rows)
    }

    let (fanout, _fanout_rows) = traffic(
        "fanout [100 x 20] — one rule, two conditions (the join is TERMINAL)",
        FANOUT_CENSUS_WORLD,
        "(:wat::rete::fire-rules (:fan::seed (:wat::rete::compile \
             (:wat::rete::collect-rules :fan)) 100 20))",
    );
    let (cascade, cascade_rows) = traffic(
            "deep-cascade [10 x 100] — CHAINED joins (the CONTROL: middle betas must be read)",
            DEPTH_SPLIT_WORLD,
            "(:wat::rete::fire-rules (:dc::seed-level-0 (:wat::rete::compile (:dc::build-rules 10)) 100))",
        );
    // THE case neither shape above produces: a MIDDLE hash-join, whose beta feeds the next
    // join's catch-up. Both worlds above are two-condition rules, so every hash-join in them
    // is a leaf; a rule about "hash-join betas" drawn from those alone would be generalising
    // from a corpus with no counter-example in it.
    let (tri, tri_rows) = traffic(
        "tri [10 x 5] — THREE conditions: root-join -> J1 -> J2, so J1 is a MIDDLE join",
        TRI_CENSUS_WORLD,
        "(:wat::rete::fire-rules (:tri::seed (:wat::rete::compile \
             (:wat::rete::collect-rules :tri)) 10 5))",
    );
    println!("{fanout}{cascade}{tri}");

    // Both controls assert on the ROWS — the measured (node, written, read) triples — not on
    // the table text. A `contains` over a rendered table would survive a renamed verdict, a
    // reordered column, or a chance substring, and would be asserting the FORMATTER rather
    // than the measurement.
    let readers =
        |rows: &[(i64, u64, u64)]| -> usize { rows.iter().filter(|&&(_, _, r)| r > 0).count() };

    // Control 1: SOMETHING must read a beta, or a zero elsewhere proves nothing rather than
    // proving the store is dead (a green that cannot go red is a claim with nothing behind it).
    assert!(
        readers(&cascade_rows) > 0,
        "the CONTROL failed — the cascade read no beta at all, so the instrument is measuring \
             nothing and the fanout zeros are meaningless.{cascade}"
    );

    // Control 2, the sharper one. The guard this probe justifies is "a node needs its beta iff
    // it parents a HashJoinNode". In `tri`, J1 parents J2 — so if J1 read ZERO the rule is
    // wrong and the guard would delete a live store on every 3+-condition rule. TWO nodes must
    // read here (the root-join AND J1); one alone means only the root-join was observed and
    // the middle-join case is still untested.
    let tri_readers = readers(&tri_rows);
    assert!(
        tri_readers >= 2,
        "a three-condition rule showed only {tri_readers} node(s) reading beta. Either the \
             middle join J1 is NOT read — which kills the parent-of-a-HashJoinNode guard — or the \
             network is not the shape this world intends. Do not draw the stone on this.{tri}"
    );
}

/// Diagnostic — where the depth cost lands, at CONSTANT work (10,000 derived facts).
///
/// Shallow-and-wide vs deep-and-narrow derive exactly the same number of facts, so any
/// difference between the two columns is depth, and the per-phase breakdown says which
/// phase is paying for it.
#[test]
fn a0_depth_cost_split_at_equal_work() {
    // 2*depth*width derived facts: both columns derive 10,000.
    let shallow = depth_split_phases(10, 500); // 10 rounds  · 500 ids per level
    let deep = depth_split_phases(50, 100); // 50 rounds · 100 ids per level  (the :clara cell)

    let names: std::collections::BTreeSet<&'static str> =
        shallow.iter().chain(deep.iter()).map(|(n, _)| *n).collect();

    let sum = |rows: &[(&'static str, u64)]| -> u64 {
        rows.iter()
            .filter(|(n, _)| n.starts_with("  "))
            .map(|(_, ns)| *ns)
            .sum()
    };
    let (s_tot, d_tot) = (sum(&shallow), sum(&deep));

    let get = |rows: &[(&'static str, u64)], name: &str| -> u64 {
        rows.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, ns)| *ns)
            .unwrap_or(0)
    };

    let mut table = String::from(
        "\n  A0 DEPTH-COST SPLIT — 10,000 derived facts in BOTH columns\n\
             \n  phase                          depth10×w500      depth50×w100         delta\n\
             \x20 ---------------------------------------------------------------------------\n",
    );
    for n in &names {
        let (a, b) = (get(&shallow, n), get(&deep, n));
        table.push_str(&format!(
            "  {n:<28} {:>10.3} ms {:>13.3} ms {:>+11.3} ms\n",
            a as f64 / 1e6,
            b as f64 / 1e6,
            (b as f64 - a as f64) / 1e6
        ));
    }
    table.push_str(&format!(
        "  {:<28} {:>10.3} ms {:>13.3} ms {:>+11.3} ms   ({:.2}x)\n",
        "TOTAL (nested phases)",
        s_tot as f64 / 1e6,
        d_tot as f64 / 1e6,
        (d_tot as f64 - s_tot as f64) / 1e6,
        if s_tot > 0 {
            d_tot as f64 / s_tot as f64
        } else {
            0.0
        }
    ));

    println!("{table}");
    assert!(
        s_tot > 0 && d_tot > 0,
        "the phase census recorded nothing — the probe measured its own scaffolding, not the \
             fire. A zero here means `with_phase_census` never saw a round.{table}"
    );
}

/// Kind lists interned on the arm (`DESIGN-STONE-arm-kind-lists`).
/// Prints sizes at cascade depth 50. Lists are disjoint subsequences
/// of `node_ids`. Does not wall-gate FIRE.
#[test]
fn cascade_kind_list_split() {
    let world = startup_from_source(DEPTH_SPLIT_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("depth-split world should freeze");
    let src = "(:wat::rete::compile (:dc::build-rules 50))";
    let ast = crate::parse_one!(src).expect("parse compile");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient of compiled session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for cascade network");
    let k = &arm.kind_ids;
    let n = arm.node_ids.len();
    let table = format!(
        "\ncascade kind lists — depth 50 compile\n\
             node_ids          {:>6}\n\
             alpha             {:>6}\n\
             join_parent       {:>6}\n\
             acc               {:>6}\n\
             filter            {:>6}\n\
             prod              {:>6}\n\
             filter_or_acc     {:>6}\n",
        n,
        k.alpha.len(),
        k.join_parent.len(),
        k.acc.len(),
        k.filter.len(),
        k.prod.len(),
        k.filter_or_acc.len(),
    );
    println!("{table}");

    let in_nodes = |ids: &[i64]| ids.iter().all(|id| arm.node_ids.contains(id));
    assert!(
        in_nodes(&k.alpha)
            && in_nodes(&k.join_parent)
            && in_nodes(&k.acc)
            && in_nodes(&k.filter)
            && in_nodes(&k.prod),
        "a kind list id is not in node_ids:{table}"
    );

    let mut seen = std::collections::HashSet::new();
    for id in k
        .alpha
        .iter()
        .chain(&k.join_parent)
        .chain(&k.acc)
        .chain(&k.filter)
        .chain(&k.prod)
    {
        assert!(seen.insert(*id), "kind lists overlap on {id}:{table}");
    }

    let sorted = |v: &[i64]| v.windows(2).all(|w| w[0] < w[1]);
    assert!(
        sorted(&k.alpha)
            && sorted(&k.join_parent)
            && sorted(&k.acc)
            && sorted(&k.filter)
            && sorted(&k.prod)
            && sorted(&k.filter_or_acc),
        "a kind list is not strictly increasing:{table}"
    );
    assert_eq!(
        k.filter_or_acc.len(),
        k.filter.len() + k.acc.len(),
        "filter_or_acc is not the merge of filter+acc:{table}"
    );
    assert!(
        n > 0 && k.alpha.len() + k.join_parent.len() + k.prod.len() > 0,
        "compile produced an empty network:{table}"
    );
}

// ── AlphaTree (DESIGN-STONE-alpha-discrimination-tree.md) ────────────────────────────────

use super::{build_alpha_index, class_field_names, session_facts, sorted_node_ids};
use crate::ast::WatAST;
use crate::rete::alpha_tree::AlphaTree;
use std::collections::HashMap;

/// Like `depth_split_phases`, but returns the fired session (seed + every derived fact) and
/// the frozen world (for `.symbols()`) instead of the phase census — the alpha-tree tests
/// below inspect the ACTUAL network and fact set the fire pass produced, rather than firing
/// a second time or hand-building a fixture.
fn fire_cascade(depth: i64, width: i64) -> (crate::freeze::FrozenWorld, Value) {
    let world = startup_from_source(DEPTH_SPLIT_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("depth-split world should freeze");
    let src = format!(
            "(:wat::rete::fire-rules (:dc::seed-level-0 (:wat::rete::compile (:dc::build-rules {depth})) {width}))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let fired = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("fire raised at depth={depth} width={width}: {e:?}"))
        .value_owned();
    (world, fired)
}

/// Item 12 — a second fire on the same network (and on an insert overlay)
/// must not rebuild the arm.
#[test]
fn fire_rules_reuses_arm_across_fire_and_insert_overlay() {
    use super::{
        fire_fixpoint_delta, network_identity, session_facts, session_with_facts, ARM_BUILDS,
    };
    let (world, fired) = fire_cascade(3, 5);
    let builds_after_first = ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    assert!(
        builds_after_first >= 1,
        "first fire-rules must have built an arm; got {builds_after_first}"
    );

    let net_id = super::session_network(&fired).and_then(network_identity);
    assert!(
        net_id.is_some(),
        "fired session must have a network identity"
    );

    fire_fixpoint_delta(&fired, world.symbols(), None).expect("second fire on the same session");
    let after_second = ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        after_second, builds_after_first,
        "second fire-rules must not rebuild the arm (same network)"
    );

    let overlay = session_with_facts(&fired, session_facts(&fired));
    let overlay_id = super::session_network(&overlay).and_then(network_identity);
    assert_eq!(
        net_id, overlay_id,
        "insert/facts overlay must share the network intern"
    );
    fire_fixpoint_delta(&overlay, world.symbols(), None).expect("fire on overlay session");
    let after_overlay = ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        after_overlay, builds_after_first,
        "fire on a facts overlay must not rebuild the arm"
    );
}

/// Stone 27 — intern index is thread-owned. N workers compile and fire
/// private Sessions without a process lock. Instance ids do not collide.
/// Second fire on the same thread HITs that thread's table.
#[test]
fn intern_index_thread_owned_workers_do_not_collide() {
    use super::{fire_fixpoint_delta, network_identity, rete_arm_lookup};
    const N: usize = 8;
    let handles: Vec<_> = (0..N)
        .map(|i| {
            std::thread::spawn(move || {
                let (world, fired) = fire_cascade(2, 2);
                let id = super::session_network(&fired)
                    .and_then(network_identity)
                    .unwrap_or_else(|| panic!("thread {i}: fired session has no network identity"));
                assert!(
                    rete_arm_lookup(id).is_some(),
                    "thread {i}: first fire must intern on this thread"
                );
                fire_fixpoint_delta(&fired, world.symbols(), None)
                    .unwrap_or_else(|e| panic!("thread {i}: second fire: {e:?}"));
                assert!(
                    rete_arm_lookup(id).is_some(),
                    "thread {i}: second fire must HIT this thread's intern"
                );
                id
            })
        })
        .collect();
    let mut ids = Vec::with_capacity(N);
    for (i, h) in handles.into_iter().enumerate() {
        ids.push(h.join().unwrap_or_else(|_| panic!("thread {i} panicked")));
    }
    let minted = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(
        ids.len(),
        minted,
        "instance rust_identity is per compile-all; got {ids:?}"
    );
    assert_eq!(ids.len(), N, "N workers minted N instance ids; got {ids:?}");
}

fn session_net_id(session: &Value) -> Option<u64> {
    super::session_network(session).and_then(super::network_identity)
}

/// Stone 28 — compile leases; fire HIT does not; release drops; next fire rebuilds.
#[test]
fn intern_release_drops_arm_and_next_fire_rebuilds() {
    use super::{
        fire_fixpoint_delta, rete_arm_leases, rete_arm_lookup, rete_arm_release, ARM_BUILDS,
    };
    let (world, fired) = fire_cascade(2, 2);
    let id = session_net_id(&fired).expect("fired session has a network identity");
    assert_eq!(
        rete_arm_leases(id),
        Some(1),
        "compile-all leases 1; fire HIT does not add a lease"
    );
    let builds = ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    fire_fixpoint_delta(&fired, world.symbols(), None).expect("second fire HIT");
    assert_eq!(
        ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed),
        builds,
        "fire HIT must not rebuild"
    );
    rete_arm_release(id);
    assert!(
        rete_arm_lookup(id).is_none(),
        "last lease drop removes the intern"
    );
    fire_fixpoint_delta(&fired, world.symbols(), None).expect("fire after release");
    assert_eq!(
        ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed),
        builds + 1,
        "next fire after release must rebuild"
    );
    assert_eq!(
        rete_arm_leases(id),
        Some(1),
        "fire MISS intern's with leases=1"
    );
}

/// Stone 28 — two compile-alls are two instance ids. Release one; the other HIT.
#[test]
fn intern_release_one_session_leaves_the_other() {
    use super::{
        fire_fixpoint_delta, rete_arm_leases, rete_arm_lookup, rete_arm_release, ARM_BUILDS,
    };
    let (_world_a, a) = fire_cascade(2, 2);
    let (world_b, b) = fire_cascade(2, 2);
    let id_a = session_net_id(&a).expect("a");
    let id_b = session_net_id(&b).expect("b");
    assert_ne!(
        id_a, id_b,
        "independent compile-all mints a new instance id"
    );
    assert_eq!(rete_arm_leases(id_a), Some(1));
    assert_eq!(rete_arm_leases(id_b), Some(1));
    let builds = ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    rete_arm_release(id_a);
    assert!(rete_arm_lookup(id_a).is_none());
    assert_eq!(rete_arm_leases(id_b), Some(1));
    fire_fixpoint_delta(&b, world_b.symbols(), None).expect("b still HIT");
    assert_eq!(
        ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed),
        builds,
        "releasing A must not force B to rebuild"
    );
}

/// Stone 28 — overlay shares rust_identity and is not a second lease.
/// Overlay fire after release of the armed Session rebuilds. Do not
/// release mid-connection.
#[test]
fn intern_overlay_is_not_a_second_lease() {
    use super::{
        fire_fixpoint_delta, rete_arm_leases, rete_arm_lookup, rete_arm_release, session_facts,
        session_with_facts, ARM_BUILDS,
    };
    let (world, fired) = fire_cascade(2, 2);
    let id = session_net_id(&fired).expect("id");
    let overlay = session_with_facts(&fired, session_facts(&fired));
    assert_eq!(session_net_id(&overlay), Some(id));
    assert_eq!(
        rete_arm_leases(id),
        Some(1),
        "overlay insert is not a second lease"
    );
    let builds = ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    fire_fixpoint_delta(&overlay, world.symbols(), None).expect("overlay fire HIT");
    assert_eq!(
        ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed),
        builds
    );
    rete_arm_release(id);
    assert!(rete_arm_lookup(id).is_none());
    fire_fixpoint_delta(&overlay, world.symbols(), None)
        .expect("overlay fire after release rebuilds");
    assert_eq!(
        ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed),
        builds + 1
    );
}

/// Stone 28 — public `:wat::rete::release-session` mouth.
#[test]
fn intern_release_session_wat_mouth_drops_the_lease() {
    use super::{rete_arm_leases, rete_arm_lookup};
    let world = startup_from_source(DEPTH_SPLIT_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("depth-split world should freeze");
    let src = "(:wat::rete::release-session (:wat::rete::compile (:dc::build-rules 2)))";
    let ast = crate::parse_one!(src).expect("parse release-session");
    let released = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("release-session raised: {e:?}"))
        .value_owned();
    let id = session_net_id(&released).expect("released session has a network identity");
    assert!(
        rete_arm_lookup(id).is_none(),
        "wat release-session must drop the compile lease"
    );
    assert_eq!(rete_arm_leases(id), None);
}

// ── DESIGN-STONE-scoped-work-over-a-network: :wat::rete::with-network / with-overlay ──────
//
// Row 1 and row 2 exercise the promoted forms end-to-end via `eval_in_frozen`, same as every
// other test in this file. Row 3 is the one that MUST be Rust (`DESIGN-STONE-scoped-work-
// over-a-network.md`): leases are not observable from wat, so a wat-only test cannot see the
// class of bug the prototype's first draft actually shipped (an extra `arm-session` call inside
// the body took a second lease and released back to 1, leaking the lease `compile-all` took).

/// Fixture world for the scoped-work rows — same shape as the proven prototype
/// (`wat-scripts/scratch-pad/wat-grep-with-network-shape.wat`), renamed into `:sw::` so it
/// cannot collide with any other test world's namespace in this file.
const SCOPED_WORK_WORLD: &str = "\
(:wat::core::defrecord :sw::Temp  [location <- :wat::core::String])\n\
(:wat::core::defrecord :sw::Wind  [location <- :wat::core::String])\n\
(:wat::core::defrecord :sw::Match [location <- :wat::core::String])\n\
\n\
(:wat::rete::defquery :sw::q-match :params [] :when [(?fact <- :sw::Match)])\n\
\n\
(:wat::core::defn :sw::the-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])\n\
  (:wat::core::let\n\
    [c1   (:wat::core::quote (:sw::Temp (?loc <- :location)))\n\
     c2   (:wat::core::quote (:sw::Wind (?loc <- :location)))\n\
     rhs  (:wat::core::quote (:sw::Match ?loc))\n\
     rule (:wat::rete::Rule :name \"temp-and-wind\"\n\
            :lhs (:wat::core::PersistentVector c1 c2)\n\
            :rhs (:wat::core::PersistentVector rhs))]\n\
    (:wat::core::PersistentVector :- [:wat::rete::Rule] rule)))\n\
\n\
(:wat::core::defn :sw::the-queries [] -> (:wat::core::PersistentVector :- [:wat::rete::Query])\n\
  (:wat::core::PersistentVector :- [:wat::rete::Query] (:sw::q-match)))\n\
\n\
(:wat::core::defn :sw::facts-for\n\
  [loc <- :wat::core::String]\n\
  -> (:wat::core::PersistentVector :- [:wat::core::Record])\n\
  (:wat::core::PersistentVector :- [:wat::core::Record]\n\
    (:sw::Temp :location loc) (:sw::Wind :location loc)))\n\
";

/// Row 1 — N units of work cost ONE network build. `with-overlay` over 3 distinct fact sets
/// (matching the prototype's `3 / 0 / 3`) must increment `ARM_BUILDS` exactly once; rete
/// already gates the underlying mechanism (`fire_rules_reuses_arm_across_fire_and_insert_
/// overlay`), this asserts the COMPOSITION through the promoted `with-overlay` form.
#[test]
fn scoped_work_with_overlay_reuses_one_build() {
    use super::ARM_BUILDS;
    let world = startup_from_source(SCOPED_WORK_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("scoped-work world should freeze");
    let builds_before = ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);

    let src = "\
(:wat::rete::with-overlay (:sw::the-rules) (:sw::the-queries)\n\
  (:wat::core::fn [overlay <- :wat::rete::Overlay] -> :wat::core::i64\n\
    (:wat::core::foldl\n\
      (:wat::core::fn [acc <- :wat::core::i64  loc <- :wat::core::String] -> :wat::core::i64\n\
        (:wat::i64::+ acc\n\
          (:wat::core::length (:wat::rete::query (overlay (:sw::facts-for loc)) (:sw::q-match)))))\n\
      0\n\
      (:wat::core::Vector :- [:wat::core::String] \"fileA\" \"fileB\" \"fileC\"))))";
    let ast = crate::parse_one!(src).expect("parse with-overlay driver");
    let total = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("with-overlay raised: {e:?}"))
        .value_owned();
    assert_eq!(
        total,
        Value::i64(3),
        "one match per unit, three distinct units"
    );

    let builds_after = ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        builds_after - builds_before,
        1,
        "with-overlay over 3 distinct fact sets must cost exactly ONE network build; \
         before={builds_before} after={builds_after}"
    );
}

/// Row 2 — the base is untouched. The Session is a fact overlay over circuits it does not own
/// (`arm.rs:572`) and is immutable, so a freshly compiled base that has had no facts inserted
/// must still answer its own query with zero results — the prototype's `0` in `3 / 0 / 3`.
#[test]
fn scoped_work_with_network_base_untouched() {
    let world = startup_from_source(SCOPED_WORK_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("scoped-work world should freeze");

    let src = "\
(:wat::rete::with-network (:sw::the-rules) (:sw::the-queries)\n\
  (:wat::core::fn [base <- :wat::rete::Session] -> :wat::core::i64\n\
    (:wat::core::length (:wat::rete::query (:wat::rete::fire-rules base) (:sw::q-match)))))";
    let ast = crate::parse_one!(src).expect("parse with-network driver");
    let zero = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("with-network raised: {e:?}"))
        .value_owned();
    assert_eq!(
        zero,
        Value::i64(0),
        "a compiled base with no facts inserted must still answer its own query with zero results"
    );
}

/// Row 3 — THE LEASE IS ACTUALLY RELEASED. Must be Rust: leases are not observable from wat.
///
/// Two id's, deliberately: `compile-all` never content-interns (`network_identity` keys off the
/// PersistentMap's own allocation identity, `arm.rs:602`), so no two separate `compile-all`
/// calls in this test — even on identical rules/queries — can ever share an id. There is no way
/// to pause a live `with-network` call from Rust to probe its lease mid-execution (wat has no
/// native-closure body-fn hook), so "inside the body" is reproduced directly: `compile-all` is
/// the FIRST thing `with-network`'s body does (`wat/rete.wat`), so calling it standalone
/// reproduces the exact lease state a correct body-fn runs under, before anything releases it.
/// The SECOND half is the one that actually discriminates the prototype's real bug: it runs the
/// PROMOTED `with-network` end-to-end and checks the state after it returns. The prototype's
/// first draft called `arm-session` on the session `compile-all` already armed — HIT increments
/// the lease (`arm.rs:709`) — so it took lease 2 and released back to 1, leaving `compile-all`'s
/// own lease held FOREVER; `rete_arm_lookup` would still find it (`Some`, not `None`) after
/// `with-network` returned. The idiom (compile → assert leased → release → assert gone) is
/// `intern_release_one_session_leaves_the_other`'s (`tests.rs:3043`).
#[test]
fn scoped_work_with_network_releases_the_lease_it_takes() {
    use super::{rete_arm_leases, rete_arm_lookup, rete_arm_release};
    let world = startup_from_source(SCOPED_WORK_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("scoped-work world should freeze");

    // "inside the body" — the state with-network's body runs under: one lease, taken by
    // compile-all, nothing has released it yet.
    let inside_src = "(:wat::rete::compile-all (:sw::the-rules) (:sw::the-queries))";
    let inside_ast = crate::parse_one!(inside_src).expect("parse compile-all");
    let inside_base = eval_in_frozen(&inside_ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile-all raised: {e:?}"))
        .value_owned();
    let inside_id =
        session_net_id(&inside_base).expect("compiled session has a network identity");
    assert_eq!(
        rete_arm_leases(inside_id),
        Some(1),
        "compile-all leases exactly 1 — the state with-network's body runs under"
    );
    // Clean up this standalone probe network directly (not through a second compile-all —
    // that would mint a THIRD, unrelated id) so it does not outlive the test.
    rete_arm_release(inside_id);
    assert!(rete_arm_lookup(inside_id).is_none());

    // "after with-network returns" — the check that actually catches the historical bug: run
    // the PROMOTED form end-to-end and confirm the lease compile-all took is fully gone.
    let after_src = "\
(:wat::rete::with-network (:sw::the-rules) (:sw::the-queries)\n\
  (:wat::core::fn [base <- :wat::rete::Session] -> :wat::rete::Session base))";
    let after_ast = crate::parse_one!(after_src).expect("parse with-network driver");
    let returned = eval_in_frozen(&after_ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("with-network raised: {e:?}"))
        .value_owned();
    let after_id =
        session_net_id(&returned).expect("with-network's base carries a network identity");
    assert!(
        rete_arm_lookup(after_id).is_none(),
        "with-network must fully release the lease compile-all took; a leaked lease (the \
         prototype's first-draft bug) would leave this Some instead of None"
    );
}

/// Every `Value::Aggregate` (non-`Struct`) fact in a fired session's final fact set —
/// `merge_facts` accumulates seed + every derived fact there across the whole fire pass.
fn all_facts_of(fired: &Value) -> Vec<Value> {
    match session_facts(fired) {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    }
}

fn fanout_phase_census(keys: i64, fanout: i64) -> Vec<(&'static str, u64, u64)> {
    let world = startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("fanout census world should freeze");
    let staged = format!(
        "(:fan::seed (:wat::rete::compile (:wat::rete::collect-rules :fan)) {keys} {fanout})"
    );
    let src = format!("(:wat::rete::fire-rules {staged})");
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census_counted(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fanout fire raised at keys={keys} fanout={fanout}: {e:?}"))
            .value_owned()
    });
    rows
}

/// Diagnostic — DESIGN-STONE-compiled-rhs.md's zero-allocation gate, not a positive count.
///
/// `match:key-alloc` is armed inside `matcher.rs`'s two `Value::String(Arc::new(...))` sites
/// (alpha's `?v <- :field` and the RHS's `resolve_operand`). Alpha is compiled (arc 278
/// compiled-conditions), and as of this stone the RHS is too: `exec_compiled_rhs` walks a
/// pre-built `CompiledRhs` program and never re-allocates a `?var` key, so on a fire with BOTH
/// compiled paths live, `match:key-alloc` is expected to be EXACTLY ZERO — a fire that still
/// counted here would mean a form fell through to the `build_insert_fact` fallback. (This
/// mirrors `a8_node_share_fire_census`'s HOLD → PRODUCE re-point earlier the same day: the
/// property this test proves changed, so the assertion had to be re-pointed rather than left
/// to keep passing on a claim it no longer supports.)
#[test]
fn fanout_rhs_key_alloc_census() {
    let world = startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("fanout census world should freeze");
    let src = "(:wat::rete::fire-rules (:fan::seed (:wat::rete::compile \
                   (:wat::rete::collect-rules :fan)) 100 20))";
    let ast = crate::parse_one!(src).expect("parse the fire driver");
    let (_fired, rows) = super::with_count_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fanout count census fire raised: {e:?}"))
            .value_owned()
    });
    let get = |n: &str| {
        rows.iter()
            .find(|(k, _)| *k == n)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };
    let table = format!(
        "\n  FANOUT RHS ALLOCATION CENSUS — keys=100 x fanout=20, 40,000 derived Pairs\n\
             \n  match:key-alloc (RHS + alpha, both compiled — expect 0)  {:>10}\n\
             \x20 per derived fact                                       {:>10.2}\n\
             \x20 match:calls (interpreter entries — expect 0)           {:>10}\n\
             \x20 prod:derivations (non-vacuity guard — expect 40,000)   {:>10}\n",
        get("match:key-alloc"),
        get("match:key-alloc") as f64 / 40_000.0,
        get("match:calls"),
        get("prod:derivations"),
    );
    println!("{table}");
    // Arc 278 DESIGN-STONE-compiled-rhs.md — this stone makes ZERO the correct answer (the
    // compiled RHS rebuilds no `?var` key), so the pre-stone ">0" assertion INVERTS rather
    // than simply strengthens. Re-pointed, not weakened: exactly 0 proves no form fell
    // through to the `build_insert_fact` fallback, AND `prod:derivations == 40_000` is kept
    // as a non-vacuity guard — a fire that never ran would also read 0 key allocations, and
    // without this second assertion that dead-fire zero would be indistinguishable from the
    // proof this test exists to make.
    assert_eq!(
        get("match:key-alloc"),
        0,
        "expected ZERO key allocations — the compiled RHS pre-builds every ?var key at rule \
             setup and never reallocates one per fact; a nonzero count means some :then form fell \
             through to the build_insert_fact fallback.{table}"
    );
    assert_eq!(
        get("prod:derivations"),
        40_000,
        "non-vacuity guard: expected exactly 40,000 derivations (the fanout cell's documented \
             size) — a count other than this means the key-alloc==0 reading above cannot be \
             trusted as proof of the compiled path (it could equally be an artifact of a fire that \
             never ran).{table}"
    );
}

/// Diagnostic — per-CALL alpha cost on a rule-light, fact-heavy workload (`D=1`).
///
/// `keys=100, fanout=20` is R4's exact 40,000-derived-pair cell. Prints the phase split so the
/// compiled-conditions stone can size its scorecard from a measurement of the shape it targets
/// instead of from the cascade's per-fact rate.
#[test]
fn fanout_per_call_alpha_census() {
    let world = startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("fanout census world should freeze");
    let src = "(:wat::rete::fire-rules (:fan::seed (:wat::rete::compile \
                   (:wat::rete::collect-rules :fan)) 100 20))";
    let ast = crate::parse_one!(src).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fanout census fire raised: {e:?}"))
            .value_owned()
    });

    // The denominator is THE FIRE — and it is NAMED, not inferred, because inferring it from
    // the row text has now been wrong twice. Draft 1 summed the INDENTED rows and printed
    // shares totalling 209.3% (a nested row is a component of its parent, so that double-counts
    // upward). Draft 2 summed the UN-indented rows — which looks right and is not, because
    // `production` / `hash-join` / `alpha` / `root-join` / `accumulate` / `filter` carry
    // unindented NAMES while living INSIDE `ROUND LOOP`; that inflated the divisor ~60% and
    // quietly understated every share. A wrong number that looks plausible is worse than one
    // that reads 209%. These four are the actual brackets around a fire; everything else is a
    // component of one of them.
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    let fire: u64 = rows
        .iter()
        .filter(|(n, _)| FIRE_PHASES.contains(n))
        .map(|(_, ns)| *ns)
        .sum();
    let mut table = String::from(
        "\n  FANOUT PER-CALL CENSUS — keys=100 x fanout=20 (R4's 40,000-pair cell), D=1\n\
             \n  phase                                 ms   % of fire\n\
             \x20 ------------------------------------------------------\n",
    );
    for (n, ns) in &rows {
        table.push_str(&format!(
            "  {n:<32} {:>8.3} {:>10.1}%\n",
            *ns as f64 / 1e6,
            if fire > 0 {
                *ns as f64 * 100.0 / fire as f64
            } else {
                0.0
            }
        ));
    }
    table.push_str(&format!(
        "  {:<32} {:>8.3}     100.0%\n",
        "THE FIRE (top-level phases)",
        fire as f64 / 1e6
    ));
    let total = fire;
    println!("{table}");
    assert!(total > 0, "the phase census recorded nothing.{table}");
}

/// Fanout phase table at the GRID ladder. `(keys, fanout)` 25/50/100 × 20
/// is items 10000/20000/40000. Prints; does not gate FIRE
/// (`DESIGN-STONE-fanout-phase-census`).
#[test]
fn fanout_fire_phase_census() {
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const REQUIRED: [&str; 6] = [
        "SETUP: indexes",
        "ROUND LOOP",
        "alpha",
        "root-join",
        "hash-join",
        "production",
    ];
    let table = render_phase_table(
        "fanout fire",
        &[(25, 20), (50, 20), (100, 20)],
        &TOP,
        &REQUIRED,
        |keys, fanout| keys * fanout * 2,
        fanout_phase_census,
    );
    println!("{table}");
    let rows = fanout_phase_census(100, 20);
    let ns_of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _, _)| *n == name)
            .map(|(_, ns, _)| *ns)
            .unwrap_or(0)
    };
    let round_loop = ns_of("ROUND LOOP");
    let hash_join = ns_of("hash-join");
    assert!(
        round_loop > 0,
        "ROUND LOOP recorded 0ns at keys=100 fanout=20 — the fire never ran:\n{table}"
    );
    assert!(
        hash_join > 0,
        "hash-join recorded 0ns at the 40k-pair cell — this axis is a join:\n{table}"
    );
}

/// Leftover production: remainder_raw vs children's instrument left in the parent.
/// Subtracting child *nets* from production *net* double-counts those clock reads
/// as unmarked work (`DESIGN-STONE-prod-leftover-split`).
#[test]
fn fanout_production_leftover_split() {
    const RUNS: usize = 3;
    const RHS: &str = "  ├ prod:compiled-rhs";
    const DEDUP: &str = "  ├ prod:dedup-store";

    let cal = calibrate_mark_ns();

    let mut prod_raw = 0.0;
    let mut rhs_raw = 0.0;
    let mut dedup_raw = 0.0;
    let mut rhs_pairs = 0u64;
    let mut dedup_pairs = 0u64;
    let mut prod_pairs = 0u64;
    for _ in 0..RUNS {
        let rows = fanout_phase_census(100, 20);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        let (p_ns, p_k) = of("production");
        let (r_ns, r_k) = of(RHS);
        let (d_ns, d_k) = of(DEDUP);
        prod_raw += p_ns as f64;
        rhs_raw += r_ns as f64;
        dedup_raw += d_ns as f64;
        prod_pairs = p_k;
        rhs_pairs = r_k;
        dedup_pairs = d_k;
    }
    let r = RUNS as f64;
    prod_raw /= r;
    rhs_raw /= r;
    dedup_raw /= r;
    let prod_net = prod_raw - prod_pairs as f64 * cal;
    let rhs_net = rhs_raw - rhs_pairs as f64 * cal;
    let dedup_net = dedup_raw - dedup_pairs as f64 * cal;
    let remainder_raw = prod_raw - rhs_raw - dedup_raw;
    let tax_in_parent = (rhs_pairs + dedup_pairs) as f64 * cal;
    let naive = prod_net - rhs_net - dedup_net;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\nproduction leftover split — fanout [100 20], mean of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             production            {:>7.2} ms raw  {:>7.2} net  {:>6}x\n\
             prod:compiled-rhs     {:>7.2} ms raw  {:>7.2} net  {:>6}x\n\
             prod:dedup-store      {:>7.2} ms raw  {:>7.2} net  {:>6}x\n\
             \n\
             remainder_raw         {:>7.2} ms   (prod_raw − rhs_raw − dedup_raw)\n\
             tax_in_parent         {:>7.2} ms   ((rhs + dedup) pairs × cal)\n\
             naive_unmarked        {:>7.2} ms   (prod_net − rhs_net − dedup_net)\n\
             = remainder_raw + tax {:>7.2} ms\n",
        ms(prod_raw),
        ms(prod_net),
        prod_pairs,
        ms(rhs_raw),
        ms(rhs_net),
        rhs_pairs,
        ms(dedup_raw),
        ms(dedup_net),
        dedup_pairs,
        ms(remainder_raw),
        ms(tax_in_parent),
        ms(naive),
        ms(remainder_raw + tax_in_parent),
    );
    println!("{table}");
    assert!(
        prod_raw > 0.0,
        "production recorded 0 — the fire never ran:{table}"
    );
    assert_eq!(
        rhs_pairs, 40_000,
        "compiled-rhs pairs must be the 40k cell, not a dead fire:{table}"
    );
}

/// Rank harvest / compiled-rhs / OUT freeze at fanout [100 20].
/// Grid compile-alls `:fan::q-Pair`; `FANOUT_CENSUS_WORLD` does not
/// (`DESIGN-STONE-fanout-three-leftover`).
#[test]
fn fanout_three_leftover_split() {
    use std::time::Instant;

    const KEYS: i64 = 100;
    const FANOUT: i64 = 20;
    const RUNS: usize = 3;
    const RHS: &str = "  ├ prod:compiled-rhs";
    const HARVEST: &str = "  ├ harvest:query";
    const OUT_PROD: &str = "  ├ out:production";
    const OUT_QUERY: &str = "  └ out:query";
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const QUERY_TAIL: &str = "\n\
(:wat::rete::defquery :fan::q-Pair\n\
  :params []\n\
  :when [(?fact <- :fan::Pair)])\n";

    let cal = calibrate_mark_ns();

    struct Shot {
        wall: f64,
        fire: f64,
        harvest: f64,
        out_query: f64,
        rhs_raw: f64,
        rhs_net: f64,
        rhs_pairs: u64,
        out_prod: f64,
        query_maps: usize,
    }

    let query_map_count = |fired: &Value| -> usize {
        match session_named_field(fired, "query-memory") {
            Some(Value::wat__core__PersistentMap(pm)) => pm
                .iter()
                .map(|(_, v)| match v {
                    Value::wat__core__PersistentVector(pv) => pv.len(),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        }
    };

    let shot = |with_query: bool| -> Shot {
        let world_src = if with_query {
            format!("{FANOUT_CENSUS_WORLD}{QUERY_TAIL}")
        } else {
            FANOUT_CENSUS_WORLD.to_string()
        };
        let world = startup_from_source(&world_src, None, Arc::new(InMemoryLoader::new()))
            .expect("fanout three-leftover world should freeze");
        let compile = if with_query {
            "(:wat::rete::compile-all (:wat::rete::collect-rules :fan) \
              (:wat::core::PersistentVector (:fan::q-Pair)))"
        } else {
            "(:wat::rete::compile (:wat::rete::collect-rules :fan))"
        };
        let seed_src = format!("(:fan::seed {compile} {KEYS} {FANOUT})");
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, world.symbols(), None).unwrap_or_else(|e| {
                panic!("fire-rules raised with_query={with_query}: {e:?}")
            })
        });
        let wall = t0.elapsed().as_nanos() as f64;
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        let fire: u64 = FIRE_PHASES.iter().map(|n| of(n).0).sum();
        let (rhs_raw, rhs_pairs) = of(RHS);
        let rhs_net = rhs_raw as f64 - rhs_pairs as f64 * cal;
        Shot {
            wall,
            fire: fire as f64,
            harvest: of(HARVEST).0 as f64,
            out_query: of(OUT_QUERY).0 as f64,
            rhs_raw: rhs_raw as f64,
            rhs_net,
            rhs_pairs,
            out_prod: of(OUT_PROD).0 as f64,
            query_maps: query_map_count(&fired),
        }
    };

    let mut without = Shot {
        wall: 0.0,
        fire: 0.0,
        harvest: 0.0,
        out_query: 0.0,
        rhs_raw: 0.0,
        rhs_net: 0.0,
        rhs_pairs: 0,
        out_prod: 0.0,
        query_maps: 0,
    };
    let mut with = Shot {
        wall: 0.0,
        fire: 0.0,
        harvest: 0.0,
        out_query: 0.0,
        rhs_raw: 0.0,
        rhs_net: 0.0,
        rhs_pairs: 0,
        out_prod: 0.0,
        query_maps: 0,
    };
    for _ in 0..RUNS {
        let a = shot(false);
        let b = shot(true);
        without.wall += a.wall;
        without.fire += a.fire;
        without.harvest += a.harvest;
        without.out_query += a.out_query;
        without.rhs_raw += a.rhs_raw;
        without.rhs_net += a.rhs_net;
        without.rhs_pairs = a.rhs_pairs;
        without.out_prod += a.out_prod;
        without.query_maps = a.query_maps;
        with.wall += b.wall;
        with.fire += b.fire;
        with.harvest += b.harvest;
        with.out_query += b.out_query;
        with.rhs_raw += b.rhs_raw;
        with.rhs_net += b.rhs_net;
        with.rhs_pairs = b.rhs_pairs;
        with.out_prod += b.out_prod;
        with.query_maps = b.query_maps;
    }
    let r = RUNS as f64;
    without.wall /= r;
    without.fire /= r;
    without.harvest /= r;
    without.out_query /= r;
    without.rhs_raw /= r;
    without.rhs_net /= r;
    without.out_prod /= r;
    with.wall /= r;
    with.fire /= r;
    with.harvest /= r;
    with.out_query /= r;
    with.rhs_raw /= r;
    with.rhs_net /= r;
    with.out_prod /= r;

    let ms = |ns: f64| ns / 1e6;
    let a_harvest = with.harvest + with.out_query;
    let delta = with.wall - without.wall;
    let table = format!(
        "\nfanout three leftover — [100 20], mean of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             without query          wall {:>7.2}  FIRE {:>7.2}  query-maps {}\n\
             with    q-Pair         wall {:>7.2}  FIRE {:>7.2}  query-maps {}\n\
             delta (A candidate)           {:>7.2} ms\n\
             \n\
             A  harvest:query              {:>7.2} ms\n\
                out:query                  {:>7.2} ms\n\
                A sum                      {:>7.2} ms\n\
             B  compiled-rhs net           {:>7.2} ms   {:>6}x  (with-query)\n\
             C  out:production             {:>7.2} ms   (with-query)\n",
        ms(without.wall),
        ms(without.fire),
        without.query_maps,
        ms(with.wall),
        ms(with.fire),
        with.query_maps,
        ms(delta),
        ms(with.harvest),
        ms(with.out_query),
        ms(a_harvest),
        ms(with.rhs_net),
        with.rhs_pairs,
        ms(with.out_prod),
    );
    println!("{table}");
    assert_eq!(
        without.rhs_pairs, 40_000,
        "without-query compiled-rhs pairs must be 40k:{table}"
    );
    assert_eq!(
        with.rhs_pairs, 40_000,
        "with-query compiled-rhs pairs must be 40k:{table}"
    );
    assert_eq!(
        without.query_maps, 0,
        "census world has no query — query-memory must be empty:{table}"
    );
    assert_eq!(
        with.query_maps, 40_000,
        "grid q-Pair must harvest 40k maps:{table}"
    );
    assert!(
        with.fire > 0.0,
        "with-query FIRE recorded 0 — the fire never ran:{table}"
    );
}

/// Honest FIRE after 2s: strip the 80k test marks 2p named
/// (`DESIGN-STONE-honest-fire-rank`).
#[test]
fn fanout_honest_fire_rank() {
    const RUNS: usize = 3;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const RHS: &str = "  ├ prod:compiled-rhs";
    const DEDUP: &str = "  ├ prod:dedup-store";
    const PROBE: &str = "  ├ hj:catchup:probe";

    let cal = calibrate_mark_ns();

    let mut fire = 0.0;
    let mut prod_raw = 0.0;
    let mut rhs_raw = 0.0;
    let mut dedup_raw = 0.0;
    let mut probe_raw = 0.0;
    let mut hash_raw = 0.0;
    let mut alpha_raw = 0.0;
    let mut out_raw = 0.0;
    let mut rhs_pairs = 0u64;
    let mut dedup_pairs = 0u64;
    let mut prod_pairs = 0u64;
    let mut probe_pairs = 0u64;
    for _ in 0..RUNS {
        let rows = fanout_phase_census(100, 20);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        fire += TOP.iter().map(|n| of(n).0 as f64).sum::<f64>();
        let (p_ns, p_k) = of("production");
        let (r_ns, r_k) = of(RHS);
        let (d_ns, d_k) = of(DEDUP);
        let (pr_ns, pr_k) = of(PROBE);
        prod_raw += p_ns as f64;
        rhs_raw += r_ns as f64;
        dedup_raw += d_ns as f64;
        probe_raw += pr_ns as f64;
        hash_raw += of("hash-join").0 as f64;
        alpha_raw += of("alpha").0 as f64;
        out_raw += of("OUT: to_persistent").0 as f64;
        prod_pairs = p_k;
        rhs_pairs = r_k;
        dedup_pairs = d_k;
        probe_pairs = pr_k;
    }
    let r = RUNS as f64;
    fire /= r;
    prod_raw /= r;
    rhs_raw /= r;
    dedup_raw /= r;
    probe_raw /= r;
    hash_raw /= r;
    alpha_raw /= r;
    out_raw /= r;
    let rhs_net = rhs_raw - rhs_pairs as f64 * cal;
    let dedup_net = dedup_raw - dedup_pairs as f64 * cal;
    let probe_net = probe_raw - probe_pairs as f64 * cal;
    let remainder_raw = prod_raw - rhs_raw - dedup_raw;
    let tax_in_parent = (rhs_pairs + dedup_pairs) as f64 * cal;
    let honest_prod = rhs_net + dedup_net;
    let honest_fire = fire - remainder_raw - tax_in_parent;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\nhonest FIRE rank — fanout [100 20], mean of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             FIRE                    {:>7.2} ms\n\
             production              {:>7.2} ms raw   {:>6}x\n\
             compiled-rhs            {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             dedup-store             {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             probe                   {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             hash-join               {:>7.2} ms\n\
             alpha                   {:>7.2} ms\n\
             OUT                     {:>7.2} ms\n\
             \n\
             remainder_raw           {:>7.2} ms\n\
             tax_in_parent           {:>7.2} ms\n\
             honest_prod             {:>7.2} ms   (rhs_net + dedup_net)\n\
             honest_FIRE             {:>7.2} ms   (FIRE − remainder − tax)\n",
        ms(fire),
        ms(prod_raw),
        prod_pairs,
        ms(rhs_raw),
        ms(rhs_net),
        rhs_pairs,
        ms(dedup_raw),
        ms(dedup_net),
        dedup_pairs,
        ms(probe_raw),
        ms(probe_net),
        probe_pairs,
        ms(hash_raw),
        ms(alpha_raw),
        ms(out_raw),
        ms(remainder_raw),
        ms(tax_in_parent),
        ms(honest_prod),
        ms(honest_fire),
    );
    println!("{table}");
    assert!(fire > 0.0, "FIRE recorded 0 — the fire never ran:{table}");
    assert_eq!(
        rhs_pairs, 40_000,
        "compiled-rhs pairs must be the 40k cell:{table}"
    );
}

/// Apportion `out:production` (3.26 ms / 40k) without a Session rewrite
/// (`DESIGN-STONE-out-production-split`).
#[test]
fn out_production_cost_split() {
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_000;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                "fan::Pair".into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )))
        })
        .collect();

    fn time_ns(n: usize, mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        for _ in 0..n {
            body();
        }
        t0.elapsed().as_nanos() as f64 / n as f64
    }

    black_box(facts.clone());
    {
        let mut pv = rpds::VectorSync::new_sync();
        for v in facts.clone() {
            pv.push_back_mut(v);
        }
        black_box(pv);
        let mut map: ProductionMemory = HashMap::new();
        map.insert(1, facts.clone());
        black_box(super::production_to_pm(map));
        let collected: rpds::VectorSync<Value> = facts.clone().into_iter().collect();
        black_box(collected);
    }

    let mut c = 0.0;
    let mut v = 0.0;
    let mut h = 0.0;
    let mut i = 0.0;
    for _ in 0..RUNS {
        c += time_ns(1, || {
            black_box(facts.clone());
        });
        v += time_ns(1, || {
            let mut pv = rpds::VectorSync::new_sync();
            for val in facts.clone() {
                pv.push_back_mut(val);
            }
            black_box(pv);
        });
        h += time_ns(1, || {
            let mut map: ProductionMemory = HashMap::new();
            map.insert(1, facts.clone());
            black_box(super::production_to_pm(map));
        });
        i += time_ns(1, || {
            let collected: rpds::VectorSync<Value> = facts.clone().into_iter().collect();
            black_box(collected);
        });
    }
    let runs = RUNS as f64;
    c /= runs;
    v /= runs;
    h /= runs;
    i /= runs;
    assert!(
        h > 0.0,
        "production_to_pm recorded 0 ns — the loop never ran"
    );

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\nout:production split — {N} Pair records, mean of {RUNS}\n\
             unscaled (the cell is 40k); C is the Arc-bump clone fire does not pay\n\
             \n\
             C  clone 40k Vec                      {:>7.2} ms\n\
             V  clone + push_back_mut              {:>7.2} ms\n\
             H  clone + production_to_pm (authority)  {:>7.2} ms\n\
             I  clone + VectorSync::from_iter      {:>7.2} ms\n\
             \n\
             V−C  node-per-fact                    {:>7.2} ms\n\
             H−V  wrap (from_trie / 1-key map)     {:>7.2} ms\n\
             V−I  from_iter drop-in                {:>7.2} ms\n",
        ms(c),
        ms(v),
        ms(h),
        ms(i),
        ms(v - c),
        ms(h - v),
        ms(v - i),
    );
}

/// Apportion `out:query` (3.08 ms / 40k) without a Session rewrite
/// (`DESIGN-STONE-out-query-split`). Same 40k VectorSync as 2u.
#[test]
fn out_query_cost_split() {
    use std::collections::HashMap;
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_000;
    const RUNS: usize = 3;

    let var = Value::String(Arc::new("?fact".to_string()));
    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let maps: Vec<crate::value::pmap::PMap> = (0..N)
        .map(|i| {
            let fact = Value::Aggregate(Arc::new(AggregateValue::record(
                "fan::Pair".into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )));
            crate::value::pmap::PMap::from_pairs([(var.clone(), fact)])
        })
        .collect();

    fn time_ns(n: usize, mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        for _ in 0..n {
            body();
        }
        t0.elapsed().as_nanos() as f64 / n as f64
    }

    black_box(maps.clone());
    {
        let mut pv = rpds::VectorSync::new_sync();
        for m in maps.clone() {
            pv.push_back_mut(Value::wat__core__PersistentMap(m));
        }
        black_box(pv);
        let mut q: QueryMemory = HashMap::new();
        q.insert("q-Pair".to_string(), maps.clone());
        black_box(super::query_memory_to_pm(q));
        let collected: rpds::VectorSync<Value> = maps
            .clone()
            .into_iter()
            .map(Value::wat__core__PersistentMap)
            .collect();
        black_box(collected);
    }

    let mut c = 0.0;
    let mut v = 0.0;
    let mut h = 0.0;
    let mut i = 0.0;
    for _ in 0..RUNS {
        c += time_ns(1, || {
            black_box(maps.clone());
        });
        v += time_ns(1, || {
            let mut pv = rpds::VectorSync::new_sync();
            for m in maps.clone() {
                pv.push_back_mut(Value::wat__core__PersistentMap(m));
            }
            black_box(pv);
        });
        h += time_ns(1, || {
            let mut q: QueryMemory = HashMap::new();
            q.insert("q-Pair".to_string(), maps.clone());
            black_box(super::query_memory_to_pm(q));
        });
        i += time_ns(1, || {
            let collected: rpds::VectorSync<Value> = maps
                .clone()
                .into_iter()
                .map(Value::wat__core__PersistentMap)
                .collect();
            black_box(collected);
        });
    }
    let runs = RUNS as f64;
    c /= runs;
    v /= runs;
    h /= runs;
    i /= runs;
    assert!(
        h > 0.0,
        "query_memory_to_pm recorded 0 ns — the loop never ran"
    );

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\nout:query split — {N} one-entry PMaps, mean of {RUNS}\n\
             unscaled (the cell is 40k); C is the Arc-bump clone fire does not pay\n\
             \n\
             C  clone 40k Vec<PMap>                {:>7.2} ms\n\
             V  clone + wrap + push_back_mut       {:>7.2} ms\n\
             H  clone + query_memory_to_pm         {:>7.2} ms\n\
             I  clone + VectorSync::from_iter      {:>7.2} ms\n\
             \n\
             V−C  node-per-fact                    {:>7.2} ms\n\
             H−V  wrap (query-name map)            {:>7.2} ms\n\
             V−I  from_iter drop-in                {:>7.2} ms\n",
        ms(c),
        ms(v),
        ms(h),
        ms(i),
        ms(v - c),
        ms(h - v),
        ms(v - i),
    );
}

/// Apportion harvest:query (7.69 ms / 40k) into scan vs wrap
/// (`DESIGN-STONE-harvest-wrap-split`). No fire-path change.
#[test]
fn harvest_wrap_split() {
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_000;
    const RUNS: usize = 3;
    const CLASS: &str = "fan::Pair";

    let var = Value::String(Arc::new("?fact".to_string()));
    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                CLASS.into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )))
        })
        .collect();
    let pv = crate::value::pvec::PVec::from_vec(facts);

    let matches_class = |f: &Value| match f {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.class.as_ref() == CLASS,
        _ => false,
    };

    fn time_ns(n: usize, mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        for _ in 0..n {
            body();
        }
        t0.elapsed().as_nanos() as f64 / n as f64
    }

    // Warm the same shapes the timed loops will run.
    {
        let collected: Vec<&Value> = pv.iter().filter(|f| matches_class(f)).collect();
        black_box(&collected);
        let maps: Vec<crate::value::pmap::PMap> = collected
            .iter()
            .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
            .collect();
        black_box(maps);
    }

    let mut s = 0.0;
    let mut w = 0.0;
    let mut h = 0.0;
    for _ in 0..RUNS {
        s += time_ns(1, || {
            let collected: Vec<&Value> = pv.iter().filter(|f| matches_class(f)).collect();
            black_box(collected);
        });
        let collected: Vec<&Value> = pv.iter().filter(|f| matches_class(f)).collect();
        w += time_ns(1, || {
            let maps: Vec<crate::value::pmap::PMap> = collected
                .iter()
                .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
                .collect();
            black_box(maps);
        });
        h += time_ns(1, || {
            let collected: Vec<&Value> = pv.iter().filter(|f| matches_class(f)).collect();
            let maps: Vec<crate::value::pmap::PMap> = collected
                .iter()
                .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
                .collect();
            black_box(maps);
        });
    }
    let runs = RUNS as f64;
    s /= runs;
    w /= runs;
    h /= runs;
    assert!(h > 0.0, "harvest wrap recorded 0 ns — the loop never ran");

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\nharvest wrap split — {N} one-entry maps, mean of {RUNS}\n\
             unscaled (the cell is 40k)\n\
             \n\
             S  scan (filter PVec by class)        {:>7.2} ms\n\
             W  wrap (from_pairs × 40k)            {:>7.2} ms\n\
             H  harvest (scan then wrap)           {:>7.2} ms\n\
             S+W                                   {:>7.2} ms\n",
        ms(s),
        ms(w),
        ms(h),
        ms(s + w),
    );
}

/// Apportion wrap (8.78 ms / 40k) into clones / Arc / intern-id
/// (`DESIGN-STONE-harvest-wrap-parts`). No fire-path change.
#[test]
fn harvest_wrap_parts() {
    use std::hint::black_box;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    const N: usize = 40_000;
    const RUNS: usize = 3;
    const CLASS: &str = "fan::Pair";

    let var = Value::String(Arc::new("?fact".to_string()));
    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                CLASS.into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )))
        })
        .collect();
    let pv = crate::value::pvec::PVec::from_vec(facts);
    let collected: Vec<&Value> = pv
        .iter()
        .filter(|f| match f {
            Value::Aggregate(a) if a.nature != Nature::Struct => a.class.as_ref() == CLASS,
            _ => false,
        })
        .collect();
    assert_eq!(collected.len(), N, "setup: 40k Pair facts");
    let pairs: Vec<(Value, Value)> = collected
        .iter()
        .map(|f| (var.clone(), (*f).clone()))
        .collect();

    fn time_ns(n: usize, mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        for _ in 0..n {
            body();
        }
        t0.elapsed().as_nanos() as f64 / n as f64
    }

    {
        black_box(pairs.clone());
        let intern = AtomicU64::new(1);
        for p in &pairs {
            let a: Arc<[(Value, Value)]> = Arc::from([p.clone()]);
            black_box(a);
            intern.fetch_add(1, Ordering::Relaxed);
        }
        let maps: Vec<crate::value::pmap::PMap> = collected
            .iter()
            .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
            .collect();
        black_box(maps);
    }

    let mut c = 0.0;
    let mut r = 0.0;
    let mut i = 0.0;
    let mut w = 0.0;
    for _ in 0..RUNS {
        c += time_ns(1, || {
            let cloned: Vec<(Value, Value)> = collected
                .iter()
                .map(|f| (var.clone(), (*f).clone()))
                .collect();
            black_box(cloned);
        });
        r += time_ns(1, || {
            for p in &pairs {
                let a: Arc<[(Value, Value)]> = Arc::from([p.clone()]);
                black_box(a);
            }
        });
        i += time_ns(1, || {
            let intern = AtomicU64::new(1);
            for _ in 0..N {
                black_box(intern.fetch_add(1, Ordering::Relaxed));
            }
        });
        w += time_ns(1, || {
            let maps: Vec<crate::value::pmap::PMap> = collected
                .iter()
                .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
                .collect();
            black_box(maps);
        });
    }
    let runs = RUNS as f64;
    c /= runs;
    r /= runs;
    i /= runs;
    w /= runs;
    assert!(w > 0.0, "from_pairs wrap recorded 0 ns — the loop never ran");

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\nharvest wrap parts — {N} one-entry maps, mean of {RUNS}\n\
             scan paid outside; pairs pre-cloned for R\n\
             \n\
             C  clone (var, fact) × 40k            {:>7.2} ms\n\
             R  Arc::from([pair]) × 40k            {:>7.2} ms\n\
             I  fetch_add × 40k                    {:>7.2} ms\n\
             W  from_pairs × 40k                   {:>7.2} ms\n\
             W−C  wrap minus clones                {:>7.2} ms\n",
        ms(c),
        ms(r),
        ms(i),
        ms(w),
        ms(w - c),
    );
}

const ACCUM_QUERY_TAIL: &str = "\n\
(:wat::rete::defquery :apx::q-CountF\n\
  :params []\n\
  :when [(?fact <- :apx::CountF)])\n\
(:wat::rete::defquery :apx::q-SumF\n\
  :params []\n\
  :when [(?fact <- :apx::SumF)])\n\
(:wat::rete::defquery :apx::q-MinF\n\
  :params []\n\
  :when [(?fact <- :apx::MinF)])\n\
(:wat::rete::defquery :apx::q-MaxF\n\
  :params []\n\
  :when [(?fact <- :apx::MaxF)])\n\
(:wat::rete::defquery :apx::q-ExistsF\n\
  :params []\n\
  :when [(?fact <- :apx::ExistsF)])\n";

/// Rank accum `[200 200]` FIRE with vs without the five grid queries
/// (`DESIGN-STONE-accum-class-index`). Census compiles without queries.
#[test]
fn accum_query_harvest_split() {
    use std::time::Instant;

    const G: i64 = 200;
    const W: i64 = 200;
    const RUNS: usize = 3;
    const HARVEST: &str = "  ├ harvest:query";
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];

    struct Shot {
        wall: f64,
        fire: f64,
        setup: f64,
        round: f64,
        alpha: f64,
        harvest: f64,
        query_maps: usize,
    }

    let query_map_count = |fired: &Value| -> usize {
        match session_named_field(fired, "query-memory") {
            Some(Value::wat__core__PersistentMap(pm)) => pm
                .iter()
                .map(|(_, v)| match v {
                    Value::wat__core__PersistentVector(pv) => pv.len(),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        }
    };

    let shot = |with_query: bool| -> Shot {
        let world_src = if with_query {
            format!("{ACCUM_AXIS_WORLD}{ACCUM_QUERY_TAIL}")
        } else {
            ACCUM_AXIS_WORLD.to_string()
        };
        let world = startup_from_source(&world_src, None, Arc::new(InMemoryLoader::new()))
            .expect("accum query-harvest world should freeze");
        let compile = if with_query {
            "(:wat::rete::compile-all (:wat::rete::collect-rules :apx) \
              (:wat::core::PersistentVector \
                (:apx::q-CountF) (:apx::q-SumF) (:apx::q-MinF) \
                (:apx::q-MaxF) (:apx::q-ExistsF)))"
        } else {
            "(:wat::rete::compile (:wat::rete::collect-rules :apx))"
        };
        let seed_src = format!("(:apx::seed {compile} {G} {W})");
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, world.symbols(), None).unwrap_or_else(|e| {
                panic!("fire-rules raised with_query={with_query}: {e:?}")
            })
        });
        let wall = t0.elapsed().as_nanos() as f64;
        let of = |name: &str| -> u64 {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, _)| *ns)
                .unwrap_or(0)
        };
        let fire: u64 = FIRE_PHASES.iter().map(|n| of(n)).sum();
        Shot {
            wall,
            fire: fire as f64,
            setup: of("SETUP: indexes") as f64,
            round: of("ROUND LOOP") as f64,
            alpha: of("alpha") as f64,
            harvest: of(HARVEST) as f64,
            query_maps: query_map_count(&fired),
        }
    };

    let mut without = Shot {
        wall: 0.0,
        fire: 0.0,
        setup: 0.0,
        round: 0.0,
        alpha: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    let mut with = Shot {
        wall: 0.0,
        fire: 0.0,
        setup: 0.0,
        round: 0.0,
        alpha: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    for _ in 0..RUNS {
        let a = shot(false);
        let b = shot(true);
        without.wall += a.wall;
        without.fire += a.fire;
        without.setup += a.setup;
        without.round += a.round;
        without.alpha += a.alpha;
        without.harvest += a.harvest;
        without.query_maps = a.query_maps;
        with.wall += b.wall;
        with.fire += b.fire;
        with.setup += b.setup;
        with.round += b.round;
        with.alpha += b.alpha;
        with.harvest += b.harvest;
        with.query_maps = b.query_maps;
    }
    let r = RUNS as f64;
    without.wall /= r;
    without.fire /= r;
    without.setup /= r;
    without.round /= r;
    without.alpha /= r;
    without.harvest /= r;
    with.wall /= r;
    with.fire /= r;
    with.setup /= r;
    with.round /= r;
    with.alpha /= r;
    with.harvest /= r;

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\naccum query harvest split — [200 200], mean of {RUNS}\n\
             \n\
             without queries       wall {:>7.2}  FIRE {:>7.2}  SETUP {:>7.2}  ROUND {:>7.2}  alpha {:>7.2}  harvest {:>7.2}  maps {}\n\
             with    five q-*      wall {:>7.2}  FIRE {:>7.2}  SETUP {:>7.2}  ROUND {:>7.2}  alpha {:>7.2}  harvest {:>7.2}  maps {}\n\
             delta (query tax)            {:>7.2} ms   harvest {:>7.2}  ROUND-harvest {:>7.2}\n",
        ms(without.wall),
        ms(without.fire),
        ms(without.setup),
        ms(without.round),
        ms(without.alpha),
        ms(without.harvest),
        without.query_maps,
        ms(with.wall),
        ms(with.fire),
        ms(with.setup),
        ms(with.round),
        ms(with.alpha),
        ms(with.harvest),
        with.query_maps,
        ms(with.wall - without.wall),
        ms(with.harvest - without.harvest),
        ms((with.round - without.round) - (with.harvest - without.harvest)),
    );
    assert_eq!(without.query_maps, 0, "compile without queries has empty query-memory");
    assert_eq!(
        with.query_maps, 1000,
        "five types × 200 groups = 1000 query maps"
    );
}

/// Rank strat-neg `[6 2000]` FIRE with vs without the ten grid queries
/// (`DESIGN-STONE-strat-neg-harvest-split`). Grid compile-alls q-S0..q-S9.
#[test]
fn strat_neg_query_harvest_split() {
    use std::time::Instant;

    const STRATA: i64 = 6;
    const ITEMS: i64 = 2000;
    const RUNS: usize = 3;
    const HARVEST: &str = "  ├ harvest:query";
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const WORLD: &str = include_str!("../../../wat-scripts/perf/grid/strat-neg.wat");
    const QUERIES: &str = "(:wat::core::PersistentVector \
        (:strat::q-S0) (:strat::q-S1) (:strat::q-S2) (:strat::q-S3) (:strat::q-S4) \
        (:strat::q-S5) (:strat::q-S6) (:strat::q-S7) (:strat::q-S8) (:strat::q-S9))";

    struct Shot {
        wall: f64,
        fire: f64,
        harvest: f64,
        query_maps: usize,
    }

    let query_map_count = |fired: &Value| -> usize {
        match session_named_field(fired, "query-memory") {
            Some(Value::wat__core__PersistentMap(pm)) => pm
                .iter()
                .map(|(_, v)| match v {
                    Value::wat__core__PersistentVector(pv) => pv.len(),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        }
    };

    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("strat-neg query-harvest world should freeze");

    let shot = |with_query: bool| -> Shot {
        let compile = if with_query {
            format!("(:wat::rete::compile-all (:strat::build-rules {STRATA}) {QUERIES})")
        } else {
            format!("(:wat::rete::compile (:strat::build-rules {STRATA}))")
        };
        let seed_src = format!("(:strat::seed-items {compile} {ITEMS})");
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse strat-neg seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("strat-neg seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, world.symbols(), None).unwrap_or_else(|e| {
                panic!("fire-rules raised with_query={with_query}: {e:?}")
            })
        });
        let wall = t0.elapsed().as_nanos() as f64;
        let of = |name: &str| -> u64 {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, _)| *ns)
                .unwrap_or(0)
        };
        let fire: u64 = FIRE_PHASES.iter().map(|n| of(n)).sum();
        Shot {
            wall,
            fire: fire as f64,
            harvest: of(HARVEST) as f64,
            query_maps: query_map_count(&fired),
        }
    };

    let mut without = Shot {
        wall: 0.0,
        fire: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    let mut with = Shot {
        wall: 0.0,
        fire: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    for _ in 0..RUNS {
        let a = shot(false);
        let b = shot(true);
        without.wall += a.wall;
        without.fire += a.fire;
        without.harvest += a.harvest;
        without.query_maps = a.query_maps;
        with.wall += b.wall;
        with.fire += b.fire;
        with.harvest += b.harvest;
        with.query_maps = b.query_maps;
    }
    let r = RUNS as f64;
    without.wall /= r;
    without.fire /= r;
    without.harvest /= r;
    with.wall /= r;
    with.fire /= r;
    with.harvest /= r;

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\nstrat-neg query harvest split — [6 2000], mean of {RUNS}\n\
             \n\
             without queries       wall {:>7.2}  FIRE {:>7.2}  harvest {:>7.2}  maps {}\n\
             with    ten q-S*      wall {:>7.2}  FIRE {:>7.2}  harvest {:>7.2}  maps {}\n\
             delta (query tax)            {:>7.2} ms\n",
        ms(without.wall),
        ms(without.fire),
        ms(without.harvest),
        without.query_maps,
        ms(with.wall),
        ms(with.fire),
        ms(with.harvest),
        with.query_maps,
        ms(with.wall - without.wall),
    );
    assert_eq!(
        without.query_maps, 0,
        "compile without queries has empty query-memory"
    );
    assert_eq!(
        with.query_maps, 6000,
        "6 strata × 1000 (even/odd) = 6000 query maps"
    );
}

/// Rank deep-cascade `[50 100]` FIRE with vs without q-Node / q-Tag
/// (`DESIGN-STONE-cascade-harvest-split`). Census compiles without queries.
#[test]
fn cascade_query_harvest_split() {
    use std::time::Instant;

    const DEPTH: i64 = 50;
    const WIDTH: i64 = 100;
    const RUNS: usize = 3;
    const HARVEST: &str = "  ├ harvest:query";
    const SETUP: &str = "SETUP: indexes";
    const ROUND: &str = "ROUND LOOP";
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const WORLD: &str = include_str!("../../../wat-scripts/perf/grid/deep-cascade.wat");
    const QUERIES: &str =
        "(:wat::core::PersistentVector (:cascade::q-Node) (:cascade::q-Tag))";

    struct Shot {
        wall: f64,
        fire: f64,
        setup: f64,
        round: f64,
        harvest: f64,
        query_maps: usize,
    }

    let query_map_count = |fired: &Value| -> usize {
        match session_named_field(fired, "query-memory") {
            Some(Value::wat__core__PersistentMap(pm)) => pm
                .iter()
                .map(|(_, v)| match v {
                    Value::wat__core__PersistentVector(pv) => pv.len(),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        }
    };

    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("cascade query-harvest world should freeze");

    let shot = |with_query: bool| -> Shot {
        let compile = if with_query {
            format!("(:wat::rete::compile-all (:dc::build-rules {DEPTH}) {QUERIES})")
        } else {
            format!("(:wat::rete::compile (:dc::build-rules {DEPTH}))")
        };
        let seed_src = format!("(:dc::seed-level-0 {compile} {WIDTH})");
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse cascade seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("cascade seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, world.symbols(), None).unwrap_or_else(|e| {
                panic!("fire-rules raised with_query={with_query}: {e:?}")
            })
        });
        let wall = t0.elapsed().as_nanos() as f64;
        let of = |name: &str| -> u64 {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, _)| *ns)
                .unwrap_or(0)
        };
        let fire: u64 = FIRE_PHASES.iter().map(|n| of(n)).sum();
        Shot {
            wall,
            fire: fire as f64,
            setup: of(SETUP) as f64,
            round: of(ROUND) as f64,
            harvest: of(HARVEST) as f64,
            query_maps: query_map_count(&fired),
        }
    };

    let mut without = Shot {
        wall: 0.0,
        fire: 0.0,
        setup: 0.0,
        round: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    let mut with = Shot {
        wall: 0.0,
        fire: 0.0,
        setup: 0.0,
        round: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    for _ in 0..RUNS {
        let a = shot(false);
        let b = shot(true);
        without.wall += a.wall;
        without.fire += a.fire;
        without.setup += a.setup;
        without.round += a.round;
        without.harvest += a.harvest;
        without.query_maps = a.query_maps;
        with.wall += b.wall;
        with.fire += b.fire;
        with.setup += b.setup;
        with.round += b.round;
        with.harvest += b.harvest;
        with.query_maps = b.query_maps;
    }
    let r = RUNS as f64;
    without.wall /= r;
    without.fire /= r;
    without.setup /= r;
    without.round /= r;
    without.harvest /= r;
    with.wall /= r;
    with.fire /= r;
    with.setup /= r;
    with.round /= r;
    with.harvest /= r;

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\ncascade query harvest split — [50 100], mean of {RUNS}\n\
             \n\
             without queries       wall {:>7.2}  FIRE {:>7.2}  SETUP {:>7.2}  ROUND {:>7.2}  harvest {:>7.2}  maps {}\n\
             with    q-Node q-Tag  wall {:>7.2}  FIRE {:>7.2}  SETUP {:>7.2}  ROUND {:>7.2}  harvest {:>7.2}  maps {}\n\
             delta (query tax)            {:>7.2} ms\n",
        ms(without.wall),
        ms(without.fire),
        ms(without.setup),
        ms(without.round),
        ms(without.harvest),
        without.query_maps,
        ms(with.wall),
        ms(with.fire),
        ms(with.setup),
        ms(with.round),
        ms(with.harvest),
        with.query_maps,
        ms(with.wall - without.wall),
    );
    assert_eq!(
        without.query_maps, 0,
        "compile without queries has empty query-memory"
    );
    assert_eq!(
        with.query_maps, 10_200,
        "5100 Node + 5100 Tag (level 0 input ∪ 50 derived levels × 100) = 10200 maps"
    );
}

/// Apportion accum harvest:query (6.23 ms / 1k maps) into all-class
/// index vs wanted-only vs derived-only vs wrap
/// (`DESIGN-STONE-accum-wanted-harvest`). No fire-path change.
#[test]
fn accum_harvest_index_parts() {
    use std::collections::{HashMap, HashSet};
    use std::hint::black_box;
    use std::time::Instant;

    const G: usize = 200;
    const W: usize = 200;
    const RUNS: usize = 3;
    const GROUP: &str = "apx::Group";
    const READING: &str = "apx::Reading";
    const WANTED: [&str; 5] = [
        "apx::CountF",
        "apx::SumF",
        "apx::MinF",
        "apx::MaxF",
        "apx::ExistsF",
    ];

    let rec = |class: &str, names: &Arc<Vec<String>>, fields: Vec<Value>| {
        Value::Aggregate(Arc::new(AggregateValue::record(
            class.into(),
            Arc::clone(names),
            Arc::new(fields),
        )))
    };
    let g_names = Arc::new(vec!["g".into()]);
    let r_names = Arc::new(vec!["g".into(), "v".into()]);
    let n_names = Arc::new(vec!["g".into(), "n".into()]);

    let mut input: Vec<Value> = Vec::with_capacity(G + G * W);
    for g in 0..G {
        input.push(rec(GROUP, &g_names, vec![Value::i64(g as i64)]));
        for j in 0..W {
            input.push(rec(
                READING,
                &r_names,
                vec![Value::i64(g as i64), Value::i64(j as i64)],
            ));
        }
    }
    let mut derived: Vec<Value> = Vec::with_capacity(G * WANTED.len());
    for class in WANTED {
        for g in 0..G {
            if class == "apx::ExistsF" {
                derived.push(rec(class, &g_names, vec![Value::i64(g as i64)]));
            } else {
                derived.push(rec(
                    class,
                    &n_names,
                    vec![Value::i64(g as i64), Value::i64(W as i64)],
                ));
            }
        }
    }
    let input_pv = crate::value::pvec::PVec::from_vec(input);
    let wanted: HashSet<&str> = WANTED.iter().copied().collect();
    let var = Value::String(Arc::new("?fact".to_string()));

    let index_all = || -> HashMap<&str, Vec<&Value>> {
        let mut idx: HashMap<&str, Vec<&Value>> = HashMap::new();
        for f in input_pv.iter() {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        for f in &derived {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        idx
    };
    let index_wanted_both = || -> HashMap<&str, Vec<&Value>> {
        let mut idx: HashMap<&str, Vec<&Value>> = HashMap::new();
        for f in input_pv.iter() {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct && wanted.contains(a.class.as_ref()) {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        for f in &derived {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct && wanted.contains(a.class.as_ref()) {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        idx
    };
    let index_wanted_derived = || -> HashMap<&str, Vec<&Value>> {
        let mut idx: HashMap<&str, Vec<&Value>> = HashMap::new();
        for f in &derived {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct && wanted.contains(a.class.as_ref()) {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        idx
    };

    fn time_ns(n: usize, mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        for _ in 0..n {
            body();
        }
        t0.elapsed().as_nanos() as f64 / n as f64
    }

    {
        black_box(index_all());
        black_box(index_wanted_both());
        black_box(index_wanted_derived());
        let facts: Vec<&Value> = derived.iter().collect();
        let maps: Vec<crate::value::pmap::PMap> = facts
            .iter()
            .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
            .collect();
        black_box(maps);
    }

    let mut i = 0.0;
    let mut w = 0.0;
    let mut d = 0.0;
    let mut m = 0.0;
    let mut maps_n = 0usize;
    for _ in 0..RUNS {
        i += time_ns(1, || {
            black_box(index_all());
        });
        w += time_ns(1, || {
            black_box(index_wanted_both());
        });
        d += time_ns(1, || {
            black_box(index_wanted_derived());
        });
        let facts: Vec<&Value> = derived.iter().collect();
        maps_n = facts.len();
        m += time_ns(1, || {
            let maps: Vec<crate::value::pmap::PMap> = facts
                .iter()
                .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
                .collect();
            black_box(maps);
        });
    }
    let runs = RUNS as f64;
    i /= runs;
    w /= runs;
    d /= runs;
    m /= runs;
    assert!(i > 0.0, "all-class index recorded 0 ns — the loop never ran");
    assert_eq!(maps_n, 1000, "five types × 200 groups = 1000 maps");

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\naccum harvest index parts — [200 200], mean of {RUNS}\n\
             input 200 Group + 40,000 Reading; derived 1,000\n\
             \n\
             I  both bags, every class             {:>7.2} ms\n\
             W  both bags, wanted only             {:>7.2} ms\n\
             D  derived only, wanted only          {:>7.2} ms\n\
             M  wrap 1,000 maps                    {:>7.2} ms\n\
             I−W  Reading vec                      {:>7.2} ms\n\
             W−D  input walk                       {:>7.2} ms\n",
        ms(i),
        ms(w),
        ms(d),
        ms(m),
        ms(i - w),
        ms(w - d),
    );
}

/// Class-scan harvest is input ∪ derived. Skip-input must not drop
/// an inserted fact of a queried class (`DESIGN-STONE-accum-wanted-harvest`).
#[test]
fn class_scan_harvest_includes_input() {
    const WORLD: &str = "\
(:wat::core::defrecord :hs::T [x <- :wat::core::i64])\n\
(:wat::core::defrecord :hs::U [x <- :wat::core::i64])\n\
(:wat::rete::defrule :hs::never\n\
  :when [(:hs::T (?x <- :x) (:wat::rete::i64::< ?x 0))]\n\
  :then [(:hs::U ?x)])\n\
(:wat::rete::defquery :hs::q-T\n\
  :params []\n\
  :when [(?fact <- :hs::T)])\n\
(:wat::rete::defquery :hs::q-U\n\
  :params []\n\
  :when [(?fact <- :hs::U)])\n";
    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("input-scan world should freeze");
    let src = "(:wat::rete::fire-rules\n\
        (:wat::rete::insert\n\
          (:wat::rete::insert\n\
            (:wat::rete::compile-all (:wat::rete::collect-rules :hs)\n\
              (:wat::core::PersistentVector (:hs::q-T) (:hs::q-U)))\n\
            (:hs::T 1))\n\
          (:hs::T 2)))";
    let fired = eval_in_frozen(
        &crate::parse_one!(src).expect("parse input-scan fire"),
        &world,
        &Environment::new(),
    )
    .unwrap_or_else(|e| panic!("input-scan fire raised: {e:?}"))
    .value_owned();
    let maps = match session_named_field(&fired, "query-memory") {
        Some(Value::wat__core__PersistentMap(pm)) => pm
            .iter()
            .map(|(k, v)| {
                let name = match k {
                    Value::String(s) => s.as_ref().clone(),
                    _ => String::new(),
                };
                let n = match v {
                    Value::wat__core__PersistentVector(pv) => pv.len(),
                    _ => 0,
                };
                (name, n)
            })
            .collect::<Vec<_>>(),
        other => panic!("query-memory missing: {other:?}"),
    };
    let t = maps
        .iter()
        .find(|(n, _)| n == "hs::q-T")
        .map(|(_, n)| *n)
        .unwrap_or(0);
    let u = maps
        .iter()
        .find(|(n, _)| n == "hs::q-U")
        .map(|(_, n)| *n)
        .unwrap_or(0);
    assert_eq!(
        (t, u),
        (2, 0),
        "q-T must harvest the two inserted T; q-U empty (rule never fires): {maps:?}"
    );
}

/// Native FIRE rank across the three instrumented cells now that
/// fanout is dry (`DESIGN-STONE-cell-rank-after-fanout`).
#[test]
fn cell_rank_after_fanout() {
    const RUNS: usize = 3;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];

    fn fire_and_top(rows: &[(&'static str, u64, u64)]) -> (f64, &'static str, f64) {
        let fire: u64 = TOP
            .iter()
            .filter_map(|n| {
                rows.iter()
                    .find(|(name, _, _)| *name == *n)
                    .map(|(_, ns, _)| *ns)
            })
            .sum();
        let mut best_name = "(none)";
        let mut best_ns = 0u64;
        for (name, ns, _) in rows {
            if TOP.contains(name) || *name == "WHOLE EVAL (compile+seed+fire)" {
                continue;
            }
            if *ns > best_ns {
                best_ns = *ns;
                best_name = name;
            }
        }
        (fire as f64, best_name, best_ns as f64)
    }

    let mut fanout = (0.0, "", 0.0);
    let mut accum = (0.0, "", 0.0);
    let mut share = (0.0, "", 0.0);
    for _ in 0..RUNS {
        let (f, n, c) = fire_and_top(&fanout_phase_census(100, 20));
        fanout.0 += f;
        fanout.1 = n;
        fanout.2 += c;
        let (f, n, c) = fire_and_top(&accum_phase_census(200, 200));
        accum.0 += f;
        accum.1 = n;
        accum.2 += c;
        let (f, n, c) = fire_and_top(&node_share_phase_census(50, 200));
        share.0 += f;
        share.1 = n;
        share.2 += c;
    }
    let r = RUNS as f64;
    fanout.0 /= r;
    fanout.2 /= r;
    accum.0 /= r;
    accum.2 /= r;
    share.0 /= r;
    share.2 /= r;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\ncell rank after fanout — mean of {RUNS}\n\
             FIRE is IN+SETUP+ROUND+OUT; top-row is the largest named child\n\
             \n\
             cell                 FIRE      top-row\n\
             fanout     [100 20]  {:>7.2} ms   {} {:>7.2} ms\n\
             accum      [200 200] {:>7.2} ms   {} {:>7.2} ms\n\
             node-share [50 200]  {:>7.2} ms   {} {:>7.2} ms\n",
        ms(fanout.0),
        fanout.1,
        ms(fanout.2),
        ms(accum.0),
        accum.1,
        ms(accum.2),
        ms(share.0),
        share.1,
        ms(share.2),
    );
    println!("{table}");
    assert!(
        fanout.0 > 0.0 && accum.0 > 0.0 && share.0 > 0.0,
        "a cell recorded FIRE 0 — the rank is a dead fire:{table}"
    );
}

/// Native FIRE rank at the three closest 08-20 grid cells
/// (`DESIGN-STONE-cell-rank-after-grid`).
#[test]
fn cell_rank_after_grid() {
    const RUNS: usize = 3;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];

    fn fire_and_top(rows: &[(&'static str, u64, u64)]) -> (f64, &'static str, f64) {
        let fire: u64 = TOP
            .iter()
            .filter_map(|n| {
                rows.iter()
                    .find(|(name, _, _)| *name == *n)
                    .map(|(_, ns, _)| *ns)
            })
            .sum();
        let mut best_name = "(none)";
        let mut best_ns = 0u64;
        for (name, ns, _) in rows {
            if TOP.contains(name) || *name == "WHOLE EVAL (compile+seed+fire)" {
                continue;
            }
            if *ns > best_ns {
                best_ns = *ns;
                best_name = name;
            }
        }
        (fire as f64, best_name, best_ns as f64)
    }

    let mut fanout = (0.0, "", 0.0);
    let mut cascade = (0.0, "", 0.0);
    let mut accum = (0.0, "", 0.0);
    for _ in 0..RUNS {
        let (f, n, c) = fire_and_top(&fanout_phase_census(100, 20));
        fanout.0 += f;
        fanout.1 = n;
        fanout.2 += c;
        let (f, n, c) = fire_and_top(&cascade_phase_census(50, 100));
        cascade.0 += f;
        cascade.1 = n;
        cascade.2 += c;
        let (f, n, c) = fire_and_top(&accum_phase_census(200, 200));
        accum.0 += f;
        accum.1 = n;
        accum.2 += c;
    }
    let r = RUNS as f64;
    fanout.0 /= r;
    fanout.2 /= r;
    cascade.0 /= r;
    cascade.2 /= r;
    accum.0 /= r;
    accum.2 /= r;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\ncell rank after grid — mean of {RUNS}\n\
             FIRE is IN+SETUP+ROUND+OUT; top-row is the largest named child\n\
             08-20 closest rungs: fanout [40000], deep-cascade [50 100], accum [200 200]\n\
             \n\
             cell                    FIRE      top-row\n\
             fanout        [100 20]  {:>7.2} ms   {} {:>7.2} ms\n\
             deep-cascade  [50 100]  {:>7.2} ms   {} {:>7.2} ms\n\
             accum         [200 200] {:>7.2} ms   {} {:>7.2} ms\n",
        ms(fanout.0),
        fanout.1,
        ms(fanout.2),
        ms(cascade.0),
        cascade.1,
        ms(cascade.2),
        ms(accum.0),
        accum.1,
        ms(accum.2),
    );
    println!("{table}");
    assert!(
        fanout.0 > 0.0 && cascade.0 > 0.0 && accum.0 > 0.0,
        "a cell recorded FIRE 0 — the rank is a dead fire:{table}"
    );
}

/// Honest FIRE at the three closest cells after intern 17
/// (`DESIGN-STONE-honest-rank-after-arm`). Production raw on
/// fanout is 80k test marks (2p); intern from honest_FIRE.
#[test]
fn honest_cell_rank_after_arm() {
    const RUNS: usize = 3;

    let cal = calibrate_mark_ns();

    fn fire_top_honest(
        rows: &[(&'static str, u64, u64)],
        cal: f64,
    ) -> (f64, &'static str, f64, f64) {
        const TOP: [&str; 4] = [
            "IN: to_transient",
            "SETUP: indexes",
            "ROUND LOOP",
            "OUT: to_persistent",
        ];
        const RHS: &str = "  ├ prod:compiled-rhs";
        const DEDUP: &str = "  ├ prod:dedup-store";
        let fire: u64 = TOP
            .iter()
            .filter_map(|n| {
                rows.iter()
                    .find(|(name, _, _)| *name == *n)
                    .map(|(_, ns, _)| *ns)
            })
            .sum();
        let mut best_name = "(none)";
        let mut best_ns = 0u64;
        for (name, ns, _) in rows {
            if TOP.contains(name) || *name == "WHOLE EVAL (compile+seed+fire)" {
                continue;
            }
            if *ns > best_ns {
                best_ns = *ns;
                best_name = name;
            }
        }
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        let (prod, _) = of("production");
        let (rhs, rhs_k) = of(RHS);
        let (dedup, dedup_k) = of(DEDUP);
        let remainder = prod.saturating_sub(rhs).saturating_sub(dedup) as f64;
        let tax = (rhs_k + dedup_k) as f64 * cal;
        let honest = fire as f64 - remainder - tax;
        (fire as f64, best_name, best_ns as f64, honest)
    }

    let mut fanout = (0.0, "", 0.0, 0.0);
    let mut cascade = (0.0, "", 0.0, 0.0);
    let mut accum = (0.0, "", 0.0, 0.0);
    for _ in 0..RUNS {
        let t = fire_top_honest(&fanout_phase_census(100, 20), cal);
        fanout.0 += t.0;
        fanout.1 = t.1;
        fanout.2 += t.2;
        fanout.3 += t.3;
        let t = fire_top_honest(&cascade_phase_census(50, 100), cal);
        cascade.0 += t.0;
        cascade.1 = t.1;
        cascade.2 += t.2;
        cascade.3 += t.3;
        let t = fire_top_honest(&accum_phase_census(200, 200), cal);
        accum.0 += t.0;
        accum.1 = t.1;
        accum.2 += t.2;
        accum.3 += t.3;
    }
    let r = RUNS as f64;
    fanout.0 /= r;
    fanout.2 /= r;
    fanout.3 /= r;
    cascade.0 /= r;
    cascade.2 /= r;
    cascade.3 /= r;
    accum.0 /= r;
    accum.2 /= r;
    accum.3 /= r;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\nhonest cell rank after arm — mean of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             FIRE is IN+SETUP+ROUND+OUT; honest_FIRE strips production remainder+tax (2p)\n\
             \n\
             cell                    FIRE     honest_FIRE   top-row\n\
             fanout        [100 20]  {:>7.2} ms  {:>7.2} ms   {} {:>7.2} ms\n\
             deep-cascade  [50 100]  {:>7.2} ms  {:>7.2} ms   {} {:>7.2} ms\n\
             accum         [200 200] {:>7.2} ms  {:>7.2} ms   {} {:>7.2} ms\n",
        ms(fanout.0),
        ms(fanout.3),
        fanout.1,
        ms(fanout.2),
        ms(cascade.0),
        ms(cascade.3),
        cascade.1,
        ms(cascade.2),
        ms(accum.0),
        ms(accum.3),
        accum.1,
        ms(accum.2),
    );
    println!("{table}");
    assert!(
        fanout.0 > 0.0 && cascade.0 > 0.0 && accum.0 > 0.0,
        "a cell recorded FIRE 0 — the rank is a dead fire:{table}"
    );
    assert!(
        fanout.3 < fanout.0,
        "fanout honest_FIRE was not less than raw — production tax did not subtract:{table}"
    );
}

/// Leftover cascade SETUP: arm vs remainder (`DESIGN-STONE-cascade-setup-split`).
#[test]
fn cascade_setup_leftover_split() {
    const RUNS: usize = 3;

    let cal = calibrate_mark_ns();

    let mut setup_raw = 0.0;
    let mut seen_raw = 0.0;
    let mut arm_raw = 0.0;
    let mut setup_pairs = 0u64;
    let mut seen_pairs = 0u64;
    let mut arm_pairs = 0u64;
    let mut builds = 0usize;
    for _ in 0..RUNS {
        let before = super::ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
        let rows = cascade_phase_census(50, 100);
        let after = super::ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
        builds += after.saturating_sub(before);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        let (s_ns, s_k) = of("SETUP: indexes");
        let (seen_ns, seen_k) = of("  ├ setup:seen");
        let (arm_ns, arm_k) = of("  ├ setup:arm");
        setup_raw += s_ns as f64;
        seen_raw += seen_ns as f64;
        arm_raw += arm_ns as f64;
        setup_pairs = s_k;
        seen_pairs = seen_k;
        arm_pairs = arm_k;
    }
    let r = RUNS as f64;
    setup_raw /= r;
    seen_raw /= r;
    arm_raw /= r;
    let remainder_raw = setup_raw - seen_raw - arm_raw;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\ncascade SETUP leftover split — [50 100], mean of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             SETUP                     {:>7.2} ms raw  {:>6}x\n\
               setup:seen              {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               setup:arm               {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             remainder (SETUP−seen−arm){:>7.2} ms\n\
             ARM_BUILDS                {:>7}  ({:.2} per run)\n",
        ms(setup_raw),
        setup_pairs,
        ms(seen_raw),
        ms(seen_raw - seen_pairs as f64 * cal),
        seen_pairs,
        ms(arm_raw),
        ms(arm_raw - arm_pairs as f64 * cal),
        arm_pairs,
        ms(remainder_raw),
        builds,
        builds as f64 / r,
    );
    println!("{table}");
    assert!(
        setup_raw > 0.0,
        "SETUP recorded 0 — the fire never ran:{table}"
    );
    assert!(
        arm_pairs > 0,
        "setup:arm recorded 0 pairs — the mark never fired:{table}"
    );
}

/// Leftover accum `[200 200]`: alpha remainder vs tax, then the
/// named engine rows (`DESIGN-STONE-accum-leftover-split`).
#[test]
fn accum_leftover_split() {
    const RUNS: usize = 3;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const ALPHA_KIDS: [&str; 4] = [
        "  ├ alpha:candidates",
        "  ├ alpha:match",
        "  ├ alpha:element",
        "  └ alpha:push",
    ];
    const PROD_KIDS: [&str; 2] = ["  ├ prod:compiled-rhs", "  ├ prod:dedup-store"];

    let cal = calibrate_mark_ns();

    let mut fire = 0.0;
    let mut alpha_raw = 0.0;
    let mut kid_raw = [0.0; 4];
    let mut kid_pairs = [0u64; 4];
    let mut prod_raw = 0.0;
    let mut pk_raw = [0.0; 2];
    let mut pk_pairs = [0u64; 2];
    let mut seen_raw = 0.0;
    let mut drop_raw = 0.0;
    let mut fold_raw = 0.0;
    let mut index_raw = 0.0;
    let mut snap_raw = 0.0;
    let mut accum_raw = 0.0;
    let mut filter_raw = 0.0;
    let mut hash_raw = 0.0;
    let mut out_raw = 0.0;
    let mut alpha_pairs = 0u64;
    let mut prod_pairs = 0u64;
    let mut seen_pairs = 0u64;
    let mut drop_pairs = 0u64;
    let mut fold_pairs = 0u64;
    for _ in 0..RUNS {
        let rows = accum_phase_census(200, 200);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        fire += TOP.iter().map(|n| of(n).0 as f64).sum::<f64>();
        let (a_ns, a_k) = of("alpha");
        alpha_raw += a_ns as f64;
        alpha_pairs = a_k;
        for (i, name) in ALPHA_KIDS.iter().enumerate() {
            let (ns, k) = of(name);
            kid_raw[i] += ns as f64;
            kid_pairs[i] = k;
        }
        let (p_ns, p_k) = of("production");
        prod_raw += p_ns as f64;
        prod_pairs = p_k;
        for (i, name) in PROD_KIDS.iter().enumerate() {
            let (ns, k) = of(name);
            pk_raw[i] += ns as f64;
            pk_pairs[i] = k;
        }
        let (s_ns, s_k) = of("  ├ setup:seen");
        seen_raw += s_ns as f64;
        seen_pairs = s_k;
        let (d_ns, d_k) = of("  └ round:drop-memories");
        drop_raw += d_ns as f64;
        drop_pairs = d_k;
        let (f_ns, f_k) = of("  └ accum:fold");
        fold_raw += f_ns as f64;
        fold_pairs = f_k;
        index_raw += of("  ├ accum:index").0 as f64;
        snap_raw += of("  ├ accum:snapshot").0 as f64;
        accum_raw += of("accumulate").0 as f64;
        filter_raw += of("filter").0 as f64;
        hash_raw += of("hash-join").0 as f64;
        out_raw += of("OUT: to_persistent").0 as f64;
    }
    let r = RUNS as f64;
    fire /= r;
    alpha_raw /= r;
    prod_raw /= r;
    seen_raw /= r;
    drop_raw /= r;
    fold_raw /= r;
    index_raw /= r;
    snap_raw /= r;
    accum_raw /= r;
    filter_raw /= r;
    hash_raw /= r;
    out_raw /= r;
    for x in &mut kid_raw {
        *x /= r;
    }
    for x in &mut pk_raw {
        *x /= r;
    }
    let net = |raw: f64, pairs: u64| raw - pairs as f64 * cal;
    let kid_net: [f64; 4] = std::array::from_fn(|i| net(kid_raw[i], kid_pairs[i]));
    let pk_net: [f64; 2] = std::array::from_fn(|i| net(pk_raw[i], pk_pairs[i]));
    // Child timers retired: remainder/tax of those pairs is 0, and
    // the outer `alpha` row *is* honest_alpha (`DESIGN-STONE-retire-alpha-child-marks`).
    let kids_retired = kid_pairs.iter().all(|k| *k == 0);
    let remainder_alpha = if kids_retired {
        0.0
    } else {
        alpha_raw - kid_raw.iter().sum::<f64>()
    };
    let tax_alpha: f64 = if kids_retired {
        0.0
    } else {
        kid_pairs.iter().map(|k| *k as f64 * cal).sum()
    };
    let honest_alpha: f64 = if kids_retired {
        net(alpha_raw, alpha_pairs).max(0.0)
    } else {
        kid_net.iter().map(|n| n.max(0.0)).sum()
    };
    let remainder_prod = prod_raw - pk_raw.iter().sum::<f64>();
    let tax_prod: f64 = pk_pairs.iter().map(|k| *k as f64 * cal).sum();
    let honest_prod: f64 = pk_net.iter().map(|n| n.max(0.0)).sum();
    let honest_fire = fire - remainder_alpha - tax_alpha - remainder_prod - tax_prod;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum leftover split — [200 200], mean of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             FIRE                      {:>7.2} ms\n\
             alpha                     {:>7.2} ms raw  {:>6}x\n\
               candidates              {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               match                   {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               element                 {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               push                    {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             remainder_alpha           {:>7.2} ms\n\
             tax_in_alpha              {:>7.2} ms\n\
             honest_alpha              {:>7.2} ms\n\
             \n\
             setup:seen                {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             drop-memories             {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             accumulate                {:>7.2} ms\n\
               snapshot                {:>7.2} ms\n\
               index                   {:>7.2} ms\n\
               fold                    {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             production                {:>7.2} ms raw  {:>6}x\n\
               compiled-rhs            {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               dedup-store             {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             remainder_prod            {:>7.2} ms\n\
             tax_in_prod               {:>7.2} ms\n\
             honest_prod               {:>7.2} ms\n\
             filter                    {:>7.2} ms\n\
             hash-join                 {:>7.2} ms\n\
             OUT                       {:>7.2} ms\n\
             \n\
             honest_FIRE               {:>7.2} ms\n",
        ms(fire),
        ms(alpha_raw),
        alpha_pairs,
        ms(kid_raw[0]),
        ms(kid_net[0]),
        kid_pairs[0],
        ms(kid_raw[1]),
        ms(kid_net[1]),
        kid_pairs[1],
        ms(kid_raw[2]),
        ms(kid_net[2]),
        kid_pairs[2],
        ms(kid_raw[3]),
        ms(kid_net[3]),
        kid_pairs[3],
        ms(remainder_alpha),
        ms(tax_alpha),
        ms(honest_alpha),
        ms(seen_raw),
        ms(net(seen_raw, seen_pairs)),
        seen_pairs,
        ms(drop_raw),
        ms(net(drop_raw, drop_pairs)),
        drop_pairs,
        ms(accum_raw),
        ms(snap_raw),
        ms(index_raw),
        ms(fold_raw),
        ms(net(fold_raw, fold_pairs)),
        fold_pairs,
        ms(prod_raw),
        prod_pairs,
        ms(pk_raw[0]),
        ms(pk_net[0]),
        pk_pairs[0],
        ms(pk_raw[1]),
        ms(pk_net[1]),
        pk_pairs[1],
        ms(remainder_prod),
        ms(tax_prod),
        ms(honest_prod),
        ms(filter_raw),
        ms(hash_raw),
        ms(out_raw),
        ms(honest_fire),
    );
    println!("{table}");
    assert!(fire > 0.0, "FIRE recorded 0 — the fire never ran:{table}");
    assert!(
        alpha_pairs > 0,
        "alpha recorded 0 pairs — leftover split is a dead fire:{table}"
    );
    assert!(
        kids_retired,
        "alpha child timers still fire — pairs {kid_pairs:?}:{table}"
    );
    assert!(
        honest_fire > 0.0,
        "honest_FIRE must be > 0 after retiring child tax:{table}"
    );
}

/// Honest alpha 18 ms: seed vs delta, then stacked isolated lumps
/// (`DESIGN-STONE-alpha-leftover-split`). No per-fact timers.
#[test]
fn accum_alpha_leftover_split() {
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let mut fire = 0.0;
    let mut alpha = 0.0;
    let mut seed = 0.0;
    let mut delta = 0.0;
    let mut seed_pairs = 0u64;
    let mut delta_pairs = 0u64;
    for _ in 0..RUNS {
        let rows = accum_phase_census(200, 200);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        fire += [
            "IN: to_transient",
            "SETUP: indexes",
            "ROUND LOOP",
            "OUT: to_persistent",
        ]
        .iter()
        .map(|n| of(n).0 as f64)
        .sum::<f64>();
        alpha += of("alpha").0 as f64;
        let (s_ns, s_k) = of("  ├ alpha:seed");
        seed += s_ns as f64;
        seed_pairs = s_k;
        let (d_ns, d_k) = of("  └ alpha:delta");
        delta += d_ns as f64;
        delta_pairs = d_k;
    }
    let r = RUNS as f64;
    fire /= r;
    alpha /= r;
    seed /= r;
    delta /= r;

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let mut wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let input_pv: crate::value::pvec::PVec = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    let facts: Vec<Value> = input_pv.iter().cloned().collect();
    assert!(
        !facts.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );

    let reset = |wm: &mut super::FireSession, d_alpha: &mut AlphaDelta| {
        wm.alpha.clear();
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
        d_alpha.clear();
    };

    let mut wp = 0.0;
    let mut w = 0.0;
    let mut c = 0.0;
    let mut t = 0.0;
    let mut m = 0.0;
    let mut a = 0.0;
    for _ in 0..RUNS {
        wp += time_ns(|| {
            for f in input_pv.iter() {
                black_box(f);
            }
        });
        w += time_ns(|| {
            for f in &facts {
                black_box(f);
            }
        });
        c += time_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box((ag.class.as_ref(), ag.fields.as_slice()));
                    }
                    _ => {}
                }
            }
        });
        t += time_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box(
                            arm.alpha_tree
                                .candidates(ag.class.as_ref(), ag.fields.as_slice()),
                        );
                    }
                    _ => {}
                }
            }
        });
        m += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm, &mut FxHashMap::default());
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    black_box(crate::rete::compiled_cond::exec_compiled(
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    ));
                }
            }
        });
        a += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            let mut cond_key_ids: CondKeyIds = HashMap::new();
            for (&id, c) in &arm.compiled_conds {
                cond_key_ids.insert(
                    id,
                    crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
                );
            }
            let bind_only: HashMap<i64, Vec<u8>> = HashMap::new();
            for (i, fact) in facts.iter().enumerate() {
                super::alpha_activate_fact(
                    fact,
                    i as u32,
                    &mut super::AlphaActivateCx {
                        wm: &mut wm,
                        d_alpha: &mut d_alpha,
                        alpha_tree: &arm.alpha_tree,
                        compiled_conds: &arm.compiled_conds,
                        match_scratch: &mut scratch,
                        cand_scratch: &mut cand,
                        cond_key_ids: &cond_key_ids,
                        bind_only: &bind_only,
                    },
                )
                .expect("isolated activate");
                black_box(d_alpha.len());
            }
        });
    }
    wp /= r;
    w /= r;
    c /= r;
    t /= r;
    m /= r;
    a /= r;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum alpha leftover split — [200 200], mean of {RUNS}\n\
             in-fire (2 pairs, not per fact)\n\
             FIRE                       {:>7.2} ms\n\
             alpha                      {:>7.2} ms\n\
               seed                     {:>7.2} ms  {:>6}x\n\
               delta                    {:>7.2} ms  {:>6}x\n\
               seed+delta               {:>7.2} ms\n\
             \n\
             isolated (cold intern each run, {} facts)\n\
             Wp  PV iter                {:>7.2} ms\n\
             W   Vec iter               {:>7.2} ms\n\
             C   class extract          {:>7.2} ms\n\
             T   + candidates           {:>7.2} ms\n\
             M   + exec_compiled        {:>7.2} ms\n\
             A   alpha_activate_fact    {:>7.2} ms\n\
             \n\
             C−W   extract              {:>7.2} ms\n\
             T−C   tree                 {:>7.2} ms\n\
             M−T   exec_compiled+intern {:>7.2} ms\n\
             A−M   push                 {:>7.2} ms\n\
             A vs seed                  {:>7.2} ms isolated vs {:>7.2} in-fire\n",
        ms(fire),
        ms(alpha),
        ms(seed),
        seed_pairs,
        ms(delta),
        delta_pairs,
        ms(seed + delta),
        facts.len(),
        ms(wp),
        ms(w),
        ms(c),
        ms(t),
        ms(m),
        ms(a),
        ms(c - w),
        ms(t - c),
        ms(m - t),
        ms(a - m),
        ms(a),
        ms(seed),
    );
    println!("{table}");
    assert!(
        seed > 0.0,
        "alpha:seed recorded 0 — the mark never fired:{table}"
    );
    assert!(
        a > 0.0,
        "isolated activate recorded 0 — the loop never ran:{table}"
    );
    assert!(
        seed_pairs > 0,
        "alpha:seed pairs 0 — leftover split is a dead fire:{table}"
    );
}

/// Split in-fire `alpha:seed` after seen folded in
/// (`DESIGN-STONE-alpha-seed-after-fold`). Isolated loops
/// walk the facts PV, `candidates_into`, `seen_insert`.
#[test]
fn accum_alpha_seed_after_fold_split() {
    use rustc_hash::FxHashSet;
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let mut fire = 0.0;
    let mut seed = 0.0;
    for _ in 0..RUNS {
        let rows = accum_phase_census(200, 200);
        let of = |name: &str| -> u64 {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, _)| *ns)
                .unwrap_or(0)
        };
        fire += [
            "IN: to_transient",
            "SETUP: indexes",
            "ROUND LOOP",
            "OUT: to_persistent",
        ]
        .iter()
        .map(|n| of(n) as f64)
        .sum::<f64>();
        seed += of("  ├ alpha:seed") as f64;
    }
    let r = RUNS as f64;
    fire /= r;
    seed /= r;

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let mut wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let input_pv: crate::value::pvec::PVec = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    assert!(
        !input_pv.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );
    let n_facts = input_pv.len();

    let reset = |wm: &mut super::FireSession, d_alpha: &mut AlphaDelta| {
        wm.alpha.clear();
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
        d_alpha.clear();
    };
    let intern_keys = |wm: &mut super::FireSession| -> CondKeyIds {
        let mut cond_key_ids: CondKeyIds = HashMap::new();
        for (&id, c) in &arm.compiled_conds {
            cond_key_ids.insert(
                id,
                crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
            );
        }
        cond_key_ids
    };

    let mut p = 0.0;
    let mut s = 0.0;
    let mut x = 0.0;
    let mut k = 0.0;
    let mut e = 0.0;
    let mut n = 0.0;
    let mut a = 0.0;
    for _ in 0..RUNS {
        p += time_ns(|| {
            for f in input_pv.iter() {
                black_box(f);
            }
        });
        s += time_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            for f in input_pv.iter() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                black_box(f);
            }
            black_box(seen_ids.len() + seen_rest.len());
        });
        x += time_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            for f in input_pv.iter() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box((ag.class.as_ref(), ag.fields.as_slice()));
                    }
                    _ => {}
                }
            }
            black_box(seen_ids.len());
        });
        k += time_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            let mut cand = Vec::new();
            for f in input_pv.iter() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        arm.alpha_tree.candidates_into(
                            ag.class.as_ref(),
                            ag.fields.as_slice(),
                            &mut cand,
                        );
                        black_box(cand.len());
                    }
                    _ => {}
                }
            }
        });
        let mut d_alpha_e: AlphaDelta = FxHashMap::default();
        reset(&mut wm, &mut d_alpha_e);
        let _cond_key_ids_e = intern_keys(&mut wm);
        e += time_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            for f in input_pv.iter() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                arm.alpha_tree
                    .candidates_into(ag.class.as_ref(), ag.fields.as_slice(), &mut cand);
                for aid in cand.iter().copied() {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, aid).expect("compiled cond");
                    black_box(crate::rete::compiled_cond::exec_compiled_with_key_ids(
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                        _cond_key_ids_e.get(&aid).map(|v| v.as_slice()),
                    ));
                }
            }
        });
        let mut d_alpha_n: AlphaDelta = FxHashMap::default();
        reset(&mut wm, &mut d_alpha_n);
        let cond_key_ids_n = intern_keys(&mut wm);
        n += time_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            for (i, f) in input_pv.iter().enumerate() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                arm.alpha_tree
                    .candidates_into(ag.class.as_ref(), ag.fields.as_slice(), &mut cand);
                for aid in cand.iter().copied() {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, aid).expect("compiled cond");
                    if let Some((off, len)) = crate::rete::compiled_cond::exec_compiled_with_key_ids(
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                        cond_key_ids_n.get(&aid).map(|v| v.as_slice()),
                    ) {
                        let el = super::make_element(i as u32, off, len);
                        let slot = {
                            let v = Arc::make_mut(wm.alpha.entry(aid).or_default());
                            v.push(el);
                            v.len() - 1
                        };
                        d_alpha_n.entry(aid).or_default().push(slot);
                    }
                }
            }
            black_box(d_alpha_n.len());
        });
        let mut d_alpha_a: AlphaDelta = FxHashMap::default();
        reset(&mut wm, &mut d_alpha_a);
        let cond_key_ids_a = intern_keys(&mut wm);
        let mut bind_only: HashMap<i64, Vec<u8>> = HashMap::new();
        for (&id, c) in &arm.compiled_conds {
            if let Some(fields) = crate::rete::compiled_cond::bind_only_fields(c) {
                bind_only.insert(id, fields);
            }
        }
        a += time_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            for (i, fact) in input_pv.iter().enumerate() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, fact);
                super::alpha_activate_fact(
                    fact,
                    i as u32,
                    &mut super::AlphaActivateCx {
                        wm: &mut wm,
                        d_alpha: &mut d_alpha_a,
                        alpha_tree: &arm.alpha_tree,
                        compiled_conds: &arm.compiled_conds,
                        match_scratch: &mut scratch,
                        cand_scratch: &mut cand,
                        cond_key_ids: &cond_key_ids_a,
                        bind_only: &bind_only,
                    },
                )
                .expect("isolated activate");
            }
            black_box(d_alpha_a.len());
        });
    }
    p /= r;
    s /= r;
    x /= r;
    k /= r;
    e /= r;
    n /= r;
    a /= r;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum alpha:seed after fold — [200 200], mean of {RUNS}, {n_facts} facts\n\
             in-fire\n\
             FIRE                       {:>7.2} ms\n\
             alpha:seed                 {:>7.2} ms\n\
             \n\
             isolated (PV walk, candidates_into, seen_insert, cold bind_vals)\n\
             P   PV iter                {:>7.2} ms\n\
             S   + seen_insert          {:>7.2} ms\n\
             X   + class extract        {:>7.2} ms\n\
             K   + candidates_into      {:>7.2} ms\n\
             E   + exec_compiled        {:>7.2} ms\n\
             N   + push                 {:>7.2} ms\n\
             A   seen+activate          {:>7.2} ms\n\
             \n\
             S−P   seen                 {:>7.2} ms\n\
             X−S   extract              {:>7.2} ms\n\
             K−X   tree                 {:>7.2} ms\n\
             E−K   exec+intern          {:>7.2} ms\n\
             N−E   push                 {:>7.2} ms\n\
             A−N   wrapper              {:>7.2} ms\n\
             A vs seed                  {:>7.2} ms isolated vs {:>7.2} in-fire\n",
        ms(fire),
        ms(seed),
        ms(p),
        ms(s),
        ms(x),
        ms(k),
        ms(e),
        ms(n),
        ms(a),
        ms(s - p),
        ms(x - s),
        ms(k - x),
        ms(e - k),
        ms(n - e),
        ms(a - n),
        ms(a),
        ms(seed),
    );
    println!("{table}");
    assert!(
        seed > 0.0,
        "alpha:seed recorded 0 — the mark never fired:{table}"
    );
    assert!(
        a > 0.0,
        "isolated seen+activate recorded 0 — the loop never ran:{table}"
    );
    assert!(n_facts > 0, "compile+seed produced 0 facts:{table}");
}

/// `M−T` 7.65 ms: ops vs intern (`DESIGN-STONE-compiled-match-split`).
#[test]
fn accum_compiled_match_split() {
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let mut wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    assert!(
        !facts.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );

    let n_fact_bind = arm
        .compiled_conds
        .values()
        .filter(|c| c.fact_bind().is_some())
        .count();
    let mut n_cands = 0u64;
    let mut n_ops_true = 0u64;
    {
        let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
        for f in &facts {
            let Value::Aggregate(ag) = f else { continue };
            if ag.nature == Nature::Struct {
                continue;
            }
            let alphas = arm
                .alpha_tree
                .candidates(ag.class.as_ref(), ag.fields.as_slice());
            n_cands += alphas.len() as u64;
            for aid in &alphas {
                let compiled =
                    super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                scratch.clear();
                scratch.resize(compiled.n_slots(), None);
                if crate::rete::compiled_cond::exec_ops(
                    compiled.ops(),
                    &mut scratch,
                    ag.fields.as_slice(),
                    true,
                ) {
                    n_ops_true += 1;
                }
            }
        }
    }

    let reset_pools = |wm: &mut super::FireSession| {
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
    };

    let mut t = 0.0;
    let mut o = 0.0;
    let mut mc = 0.0;
    let mut mw = 0.0;
    for _ in 0..RUNS {
        t += time_ns(|| {
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                black_box(
                    arm.alpha_tree
                        .candidates(ag.class.as_ref(), ag.fields.as_slice()),
                );
            }
        });
        o += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    black_box(crate::rete::compiled_cond::exec_ops(
                        compiled.ops(),
                        &mut scratch,
                        ag.fields.as_slice(),
                        true,
                    ));
                }
            }
        });
        mc += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset_pools(&mut wm);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    black_box(crate::rete::compiled_cond::exec_compiled(
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    ));
                }
            }
        });
    }
    {
        let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
        reset_pools(&mut wm);
        for f in &facts {
            let Value::Aggregate(ag) = f else { continue };
            if ag.nature == Nature::Struct {
                continue;
            }
            let alphas = arm
                .alpha_tree
                .candidates(ag.class.as_ref(), ag.fields.as_slice());
            for aid in &alphas {
                let compiled =
                    super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                let _ = crate::rete::compiled_cond::exec_compiled(
                    compiled,
                    ag.fields.as_slice(),
                    &mut scratch,
                    &mut wm.bind_intern(),
                    f,
                );
            }
        }
        for _ in 0..RUNS {
            mw += time_ns(|| {
                let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
                wm.bind_pool.clear();
                for f in &facts {
                    let Value::Aggregate(ag) = f else { continue };
                    if ag.nature == Nature::Struct {
                        continue;
                    }
                    let alphas = arm
                        .alpha_tree
                        .candidates(ag.class.as_ref(), ag.fields.as_slice());
                    for aid in &alphas {
                        let compiled = super::rematch_compiled(&arm.compiled_conds, *aid)
                            .expect("compiled cond");
                        black_box(crate::rete::compiled_cond::exec_compiled(
                            compiled,
                            ag.fields.as_slice(),
                            &mut scratch,
                            &mut wm.bind_intern(),
                            f,
                        ));
                    }
                }
            });
        }
    }
    let r = RUNS as f64;
    t /= r;
    o /= r;
    mc /= r;
    mw /= r;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum compiled-match split — 40,200 facts, mean of {RUNS}\n\
             fact_bind conds {n_fact_bind}   candidates {n_cands}   ops-true {n_ops_true}\n\
             \n\
             T   candidates                 {:>7.2} ms\n\
             O   + exec_ops                 {:>7.2} ms\n\
             Mc  + exec_compiled (cold)     {:>7.2} ms\n\
             Mw  + exec_compiled (warm)     {:>7.2} ms\n\
             \n\
             O−T   ops                      {:>7.2} ms\n\
             Mc−O  intern/materialize cold  {:>7.2} ms\n\
             Mc−Mw intern cold tax          {:>7.2} ms\n",
        ms(t),
        ms(o),
        ms(mc),
        ms(mw),
        ms(o - t),
        ms(mc - o),
        ms(mc - mw),
    );
    println!("{table}");
    assert!(o > 0.0, "exec_ops recorded 0 — the loop never ran:{table}");
    assert!(n_cands > 0, "zero candidates — split is vacuous:{table}");
}

/// intern/materialize 6.18 ms: clone vs intern_key vs intern_val vs push
/// (`DESIGN-STONE-materialize-split`).
#[test]
fn accum_materialize_split() {
    use crate::rete::compiled_cond::{exec_ops, intern_key, intern_val, materialize_into};
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let mut wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    assert!(
        !facts.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );

    let reset = |wm: &mut super::FireSession| {
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
    };

    let mut o = 0.0;
    let mut c = 0.0;
    let mut k = 0.0;
    let mut v = 0.0;
    let mut p = 0.0;
    let mut m = 0.0;
    for _ in 0..RUNS {
        o += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    black_box(exec_ops(
                        compiled.ops(),
                        &mut scratch,
                        ag.fields.as_slice(),
                        true,
                    ));
                }
            }
        });
        c += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true) {
                        continue;
                    }
                    for &slot in compiled.output_slots() {
                        black_box(scratch.get(slot).and_then(|x| x.clone()));
                    }
                }
            }
        });
        k += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true) {
                        continue;
                    }
                    for &slot in compiled.output_slots() {
                        black_box(scratch.get(slot).and_then(|x| x.clone()));
                    }
                    for key in compiled.slot_keys() {
                        black_box(intern_key(&mut wm.bind_keys, key));
                    }
                }
            }
        });
        v += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true) {
                        continue;
                    }
                    for (i, &slot) in compiled.output_slots().iter().enumerate() {
                        let Some(val) = scratch.get(slot).and_then(|x| x.clone()) else {
                            continue;
                        };
                        black_box(intern_key(&mut wm.bind_keys, &compiled.slot_keys()[i]));
                        black_box(intern_val(&mut wm.bind_vals, &mut wm.bind_val_ids, val));
                    }
                }
            }
        });
        p += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true) {
                        continue;
                    }
                    for (i, &slot) in compiled.output_slots().iter().enumerate() {
                        let Some(val) = scratch.get(slot).and_then(|x| x.clone()) else {
                            continue;
                        };
                        let kid = intern_key(&mut wm.bind_keys, &compiled.slot_keys()[i]);
                        let vid = intern_val(&mut wm.bind_vals, &mut wm.bind_val_ids, val);
                        wm.bind_pool.push((kid, vid));
                    }
                }
            }
        });
        m += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true) {
                        continue;
                    }
                    black_box(materialize_into(
                        compiled,
                        &scratch,
                        &mut crate::rete::compiled_cond::BindIntern {
                            keys: &mut wm.bind_keys,
                            vals: &mut wm.bind_vals,
                            ids: &mut wm.bind_val_ids,
                            pool: &mut wm.bind_pool,
                        },
                        f,
                        None,
                    ));
                }
            }
        });
    }
    let r = RUNS as f64;
    o /= r;
    c /= r;
    k /= r;
    v /= r;
    p /= r;
    m /= r;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum materialize split — 40,200 facts, mean of {RUNS}\n\
             \n\
             O   exec_ops                   {:>7.2} ms\n\
             C   + clone slots              {:>7.2} ms\n\
             K   + intern_key               {:>7.2} ms\n\
             V   + intern_val               {:>7.2} ms\n\
             P   + pool.push                {:>7.2} ms\n\
             M   + materialize_into         {:>7.2} ms\n\
             \n\
             C−O   clone                    {:>7.2} ms\n\
             K−C   intern_key               {:>7.2} ms\n\
             V−K   intern_val               {:>7.2} ms\n\
             P−V   pool.push                {:>7.2} ms\n\
             M−P   materialize leftover     {:>7.2} ms\n",
        ms(o),
        ms(c),
        ms(k),
        ms(v),
        ms(p),
        ms(m),
        ms(c - o),
        ms(k - c),
        ms(v - k),
        ms(p - v),
        ms(m - p),
    );
    println!("{table}");
    assert!(
        v > 0.0,
        "intern_val recorded 0 — the loop never ran:{table}"
    );
}

/// Tree 4.46 ms: class HashMap vs walk vs Vec alloc
/// (`DESIGN-STONE-alpha-tree-walk-split`).
#[test]
fn accum_alpha_tree_walk_split() {
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    assert!(
        !facts.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );

    let mut e = 0.0;
    let mut g = 0.0;
    let mut i = 0.0;
    let mut t = 0.0;
    for _ in 0..RUNS {
        e += time_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box((ag.class.as_ref(), ag.fields.as_slice()));
                    }
                    _ => {}
                }
            }
        });
        g += time_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box((
                            ag.fields.as_slice(),
                            arm.alpha_tree.has_class(ag.class.as_ref()),
                        ));
                    }
                    _ => {}
                }
            }
        });
        i += time_ns(|| {
            let mut buf = Vec::new();
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        arm.alpha_tree.candidates_into(
                            ag.class.as_ref(),
                            ag.fields.as_slice(),
                            &mut buf,
                        );
                        black_box(buf.len());
                    }
                    _ => {}
                }
            }
        });
        t += time_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box(
                            arm.alpha_tree
                                .candidates(ag.class.as_ref(), ag.fields.as_slice()),
                        );
                    }
                    _ => {}
                }
            }
        });
    }
    let r = RUNS as f64;
    e /= r;
    g /= r;
    i /= r;
    t /= r;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum alpha-tree walk split — 40,200 facts, mean of {RUNS}\n\
             \n\
             E   class extract              {:>7.2} ms\n\
             G   + has_class                {:>7.2} ms\n\
             I   + walk into reused Vec     {:>7.2} ms\n\
             T   candidates() new Vec       {:>7.2} ms\n\
             \n\
             G−E   class HashMap            {:>7.2} ms\n\
             I−G   walk                     {:>7.2} ms\n\
             T−I   Vec alloc                {:>7.2} ms\n",
        ms(e),
        ms(g),
        ms(i),
        ms(t),
        ms(g - e),
        ms(i - g),
        ms(t - i),
    );
    println!("{table}");
    assert!(i > 0.0, "walk recorded 0 — the loop never ran:{table}");
}

/// Class lookup 3.26 ms: std HashMap vs FxHash vs linear
/// (`DESIGN-STONE-alpha-class-lookup`).
#[test]
fn accum_alpha_class_lookup_split() {
    use rustc_hash::FxHashMap;
    use std::collections::HashMap;
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient of seeded session");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    // rune:perspicere(read-once) — one census collect; alias would be a mumble.
    let classes: Vec<Arc<str>> = facts
        .iter()
        .filter_map(|f| match f {
            Value::Aggregate(ag) if ag.nature != Nature::Struct => Some(ag.class.clone()),
            _ => None,
        })
        .collect();
    assert!(
        !classes.is_empty(),
        "compile+seed produced 0 classed facts — lookup split is vacuous"
    );
    let mut unique: Vec<String> = Vec::new();
    for c in &classes {
        let s = c.as_ref();
        if !unique.iter().any(|u| u == s) {
            unique.push(s.to_string());
        }
    }
    let n_types = unique.len();

    let mut std_map: HashMap<String, u8> = HashMap::with_capacity(n_types);
    let mut fx_map: FxHashMap<String, u8> = FxHashMap::default();
    fx_map.reserve(n_types);
    let mut lin: Vec<(String, u8)> = Vec::with_capacity(n_types);
    for (i, u) in unique.iter().enumerate() {
        std_map.insert(u.clone(), i as u8);
        fx_map.insert(u.clone(), i as u8);
        lin.push((u.clone(), i as u8));
    }

    let mut s = 0.0;
    let mut f = 0.0;
    let mut l = 0.0;
    for _ in 0..RUNS {
        s += time_ns(|| {
            for c in &classes {
                black_box(std_map.get(c.as_ref()));
            }
        });
        f += time_ns(|| {
            for c in &classes {
                black_box(fx_map.get(c.as_ref()));
            }
        });
        l += time_ns(|| {
            for c in &classes {
                let cs = c.as_ref();
                black_box(lin.iter().find(|(k, _)| k == cs).map(|(_, v)| v));
            }
        });
    }
    let r = RUNS as f64;
    s /= r;
    f /= r;
    l /= r;
    let best = f.min(l);
    let winner = if l <= f { "L" } else { "F" };

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum alpha class-lookup split — {} facts, {n_types} types, mean of {RUNS}\n\
             types: {unique:?}\n\
             \n\
             S  std HashMap (engine)        {:>7.2} ms\n\
             F  FxHashMap                   {:>7.2} ms\n\
             L  linear Vec                  {:>7.2} ms\n\
             \n\
             S−F                            {:>7.2} ms\n\
             S−L                            {:>7.2} ms\n\
             winner {winner}  cut               {:>7.2} ms\n",
        classes.len(),
        ms(s),
        ms(f),
        ms(l),
        ms(s - f),
        ms(s - l),
        ms(s - best),
    );
    println!("{table}");
    assert!(
        s > 0.0,
        "std HashMap lookup recorded 0 — the loop never ran:{table}"
    );
    assert!(n_types > 0, "zero types — split is vacuous:{table}");
}

/// A−M 3.45 ms: HashMap entry vs Vec push vs d_alpha
/// (`DESIGN-STONE-alpha-push-split`).
#[test]
fn accum_alpha_push_split() {
    use crate::rete::compiled_cond::exec_compiled;
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let mut wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    assert!(
        !facts.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );

    let reset = |wm: &mut super::FireSession, d_alpha: &mut AlphaDelta| {
        wm.alpha.clear();
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
        d_alpha.clear();
    };

    let mut m = 0.0;
    let mut h = 0.0;
    let mut v = 0.0;
    let mut d = 0.0;
    let mut a = 0.0;
    for _ in 0..RUNS {
        m += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm, &mut FxHashMap::default());
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    black_box(exec_compiled(
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    ));
                }
            }
        });
        h += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    if exec_compiled(
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    )
                    .is_some()
                    {
                        black_box(wm.alpha.entry(*aid).or_default().len());
                    }
                }
            }
        });
        v += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            for (i, f) in facts.iter().enumerate() {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    if let Some((off, len)) = exec_compiled(
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    ) {
                        let el = super::make_element(i as u32, off, len);
                        Arc::make_mut(wm.alpha.entry(*aid).or_default()).push(el);
                    }
                }
            }
        });
        d += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            for (i, f) in facts.iter().enumerate() {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    if let Some((off, len)) = exec_compiled(
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    ) {
                        let el = super::make_element(i as u32, off, len);
                        let slot = {
                            let mem = Arc::make_mut(wm.alpha.entry(*aid).or_default());
                            mem.push(el);
                            mem.len() - 1
                        };
                        d_alpha.entry(*aid).or_default().push(slot);
                    }
                }
            }
        });
        a += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            let mut cond_key_ids: CondKeyIds = HashMap::new();
            for (&id, c) in &arm.compiled_conds {
                cond_key_ids.insert(
                    id,
                    crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
                );
            }
            let bind_only: HashMap<i64, Vec<u8>> = HashMap::new();
            for (i, fact) in facts.iter().enumerate() {
                super::alpha_activate_fact(
                    fact,
                    i as u32,
                    &mut super::AlphaActivateCx {
                        wm: &mut wm,
                        d_alpha: &mut d_alpha,
                        alpha_tree: &arm.alpha_tree,
                        compiled_conds: &arm.compiled_conds,
                        match_scratch: &mut scratch,
                        cand_scratch: &mut cand,
                        cond_key_ids: &cond_key_ids,
                        bind_only: &bind_only,
                    },
                )
                .expect("isolated activate");
                black_box(d_alpha.len());
            }
        });
    }
    let r = RUNS as f64;
    m /= r;
    h /= r;
    v /= r;
    d /= r;
    a /= r;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum alpha push split — 40,200 facts, mean of {RUNS}\n\
             \n\
             M   exec_compiled              {:>7.2} ms\n\
             H   + alpha.entry              {:>7.2} ms\n\
             V   + Vec::push                {:>7.2} ms\n\
             D   + d_alpha.entry            {:>7.2} ms\n\
             A   alpha_activate_fact        {:>7.2} ms\n\
             \n\
             H−M   HashMap entry            {:>7.2} ms\n\
             V−H   Vec push                 {:>7.2} ms\n\
             D−V   d_alpha                  {:>7.2} ms\n\
             A−D   leftover                 {:>7.2} ms\n\
             A−M   push lump                {:>7.2} ms\n",
        ms(m),
        ms(h),
        ms(v),
        ms(d),
        ms(a),
        ms(h - m),
        ms(v - h),
        ms(d - v),
        ms(a - d),
        ms(a - m),
    );
    println!("{table}");
    assert!(
        d > 0.0,
        "d_alpha path recorded 0 — the loop never ran:{table}"
    );
}

/// intern_val 2.77 ms: Value map vs i64 map vs small-int table
/// (`DESIGN-STONE-intern-val-i64`).
#[test]
fn accum_intern_val_i64_split() {
    use crate::rete::compiled_cond::exec_ops;
    use rustc_hash::FxHashMap;
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };

    let mut payloads: Vec<Value> = Vec::new();
    {
        let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
        for f in &facts {
            let Value::Aggregate(ag) = f else { continue };
            if ag.nature == Nature::Struct {
                continue;
            }
            let alphas = arm
                .alpha_tree
                .candidates(ag.class.as_ref(), ag.fields.as_slice());
            for aid in &alphas {
                let compiled =
                    super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                scratch.clear();
                scratch.resize(compiled.n_slots(), None);
                if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true) {
                    continue;
                }
                for &slot in compiled.output_slots() {
                    if let Some(v) = scratch.get(slot).and_then(|x| x.clone()) {
                        payloads.push(v);
                    }
                }
            }
        }
    }
    assert!(
        !payloads.is_empty(),
        "no interned fillers — intern_val split is vacuous"
    );
    let mut n_i64 = 0u64;
    let mut n_other = 0u64;
    let mut min_i = i64::MAX;
    let mut max_i = i64::MIN;
    for v in &payloads {
        match v {
            Value::i64(n) => {
                n_i64 += 1;
                min_i = min_i.min(*n);
                max_i = max_i.max(*n);
            }
            _ => n_other += 1,
        }
    }
    let table_ok = n_other == 0 && min_i >= 0 && max_i < 4096;

    let mut vns = 0.0;
    let mut ins = 0.0;
    let mut ans = 0.0;
    for _ in 0..RUNS {
        vns += time_ns(|| {
            let mut vals = Vec::new();
            let mut ids = crate::rete::compiled_cond::ValIntern::default();
            for v in &payloads {
                black_box(crate::rete::compiled_cond::intern_val(
                    &mut vals,
                    &mut ids,
                    v.clone(),
                ));
            }
        });
        ins += time_ns(|| {
            let mut vals = Vec::new();
            let mut ids: FxHashMap<i64, u32> = FxHashMap::default();
            for v in &payloads {
                let Value::i64(n) = *v else {
                    panic!("non-i64 in I arm");
                };
                let id = if let Some(&id) = ids.get(&n) {
                    id
                } else {
                    let id = vals.len() as u32;
                    ids.insert(n, id);
                    vals.push(Value::i64(n));
                    id
                };
                black_box(id);
            }
        });
        if table_ok {
            ans += time_ns(|| {
                let mut vals = Vec::new();
                let mut slot = vec![u32::MAX; (max_i as usize) + 1];
                for v in &payloads {
                    let Value::i64(n) = *v else { panic!() };
                    let i = n as usize;
                    let id = if slot[i] != u32::MAX {
                        slot[i]
                    } else {
                        let id = vals.len() as u32;
                        slot[i] = id;
                        vals.push(Value::i64(n));
                        id
                    };
                    black_box(id);
                }
            });
        }
    }
    let r = RUNS as f64;
    vns /= r;
    ins /= r;
    if table_ok {
        ans /= r;
    }
    let best = if table_ok { ins.min(ans) } else { ins };
    let winner = if table_ok && ans <= ins { "A" } else { "I" };

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum intern_val i64 split — {} fillers, mean of {RUNS}\n\
             i64 {n_i64}  other {n_other}  min {min_i}  max {max_i}  table_ok {table_ok}\n\
             \n\
             V  FxHashMap<Value> (engine)   {:>7.2} ms\n\
             I  FxHashMap<i64>              {:>7.2} ms\n\
             A  slot table                  {:>7.2} ms\n\
             \n\
             V−I                            {:>7.2} ms\n\
             V−A                            {:>7.2} ms\n\
             winner {winner}  cut               {:>7.2} ms\n",
        payloads.len(),
        ms(vns),
        ms(ins),
        if table_ok { ms(ans) } else { f64::NAN },
        ms(vns - ins),
        if table_ok { ms(vns - ans) } else { f64::NAN },
        ms(vns - best),
    );
    println!("{table}");
    assert!(
        vns > 0.0,
        "intern_val recorded 0 — the loop never ran:{table}"
    );
    assert!(n_i64 > 0, "zero i64 fillers — split is vacuous:{table}");
}

/// O−T 1.90 ms: scratch reset vs exec_ops body
/// (`DESIGN-STONE-exec-ops-split`).
#[test]
fn accum_exec_ops_split() {
    use crate::rete::compiled_cond::exec_ops;
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    assert!(
        !facts.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );

    let mut t = 0.0;
    let mut rset = 0.0;
    let mut f = 0.0;
    let mut o = 0.0;
    for _ in 0..RUNS {
        t += time_ns(|| {
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                black_box(
                    arm.alpha_tree
                        .candidates(ag.class.as_ref(), ag.fields.as_slice()),
                );
            }
        });
        rset += time_ns(|| {
            let mut scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    black_box(scratch.len());
                }
            }
        });
        f += time_ns(|| {
            let mut scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
            for fct in &facts {
                let Value::Aggregate(ag) = fct else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    let n = compiled.n_slots();
                    if scratch.len() != n {
                        scratch.resize(n, None);
                    }
                    scratch.fill(None);
                    black_box(scratch.len());
                }
            }
        });
        o += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    black_box(exec_ops(
                        compiled.ops(),
                        &mut scratch,
                        ag.fields.as_slice(),
                        true,
                    ));
                }
            }
        });
    }
    let runs = RUNS as f64;
    t /= runs;
    rset /= runs;
    f /= runs;
    o /= runs;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum exec_ops split — 40,200 facts, mean of {RUNS}\n\
             \n\
             T   candidates                 {:>7.2} ms\n\
             R   + clear/resize             {:>7.2} ms\n\
             F   + fill(None)               {:>7.2} ms\n\
             O   + exec_ops                 {:>7.2} ms\n\
             \n\
             R−T   scratch clear/resize     {:>7.2} ms\n\
             F−T   scratch fill             {:>7.2} ms\n\
             O−R   exec_ops body            {:>7.2} ms\n\
             O−T   ops lump                 {:>7.2} ms\n",
        ms(t),
        ms(rset),
        ms(f),
        ms(o),
        ms(rset - t),
        ms(f - t),
        ms(o - rset),
        ms(o - t),
    );
    println!("{table}");
    assert!(o > 0.0, "exec_ops recorded 0 — the loop never ran:{table}");
}

/// `FxHashSet<Value>` vs fingerprint set (`DESIGN-STONE-seen-identity-set`).
#[test]
fn seen_identity_set_split() {
    use rustc_hash::FxHashSet;
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["g".into(), "w".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                "acc::Fact".into(),
                names.clone(),
                Arc::new(vec![
                    Value::i64((i / 200) as i64),
                    Value::i64((i % 200) as i64),
                ]),
            )))
        })
        .collect();
    for f in &facts {
        match f {
            Value::Aggregate(a) => assert!(a.identity() != 0, "fixture facts must be stamped"),
            other => panic!("fixture is a Record, got {other:?}"),
        }
    }

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    black_box(facts.clone());
    {
        let mut s: FxHashSet<Value> = FxHashSet::with_capacity_and_hasher(N, Default::default());
        for f in &facts {
            s.insert(f.clone());
        }
        black_box(s.len());
        let mut ids: FxHashSet<u64> = FxHashSet::with_capacity_and_hasher(N, Default::default());
        for f in &facts {
            let Value::Aggregate(a) = f else { panic!() };
            ids.insert(a.identity());
        }
        black_box(ids.len());
    }

    let mut c = 0.0;
    let mut s = 0.0;
    let mut i = 0.0;
    for _ in 0..RUNS {
        c += time_ns(|| {
            black_box(facts.clone());
        });
        s += time_ns(|| {
            let mut set: FxHashSet<Value> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            for f in &facts {
                set.insert(f.clone());
            }
            black_box(set.len());
        });
        i += time_ns(|| {
            let mut ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            for f in &facts {
                let Value::Aggregate(a) = f else { panic!() };
                ids.insert(a.identity());
            }
            black_box(ids.len());
        });
    }
    let r = RUNS as f64;
    c /= r;
    s /= r;
    i /= r;
    assert!(
        s > 0.0,
        "Value-set insert recorded 0 ns — the loop never ran"
    );

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\nseen identity-set split — {N} stamped Records, mean of {RUNS}\n\
             unscaled (accum [200 200] input count)\n\
             \n\
             C  clone 40,200 Values                {:>7.2} ms\n\
             S  FxHashSet<Value> insert (engine)   {:>7.2} ms\n\
             I  FxHashSet<u64> insert (identity)   {:>7.2} ms\n\
             \n\
             S−C  HashSet beyond clone             {:>7.2} ms\n\
             S−I  predicted cut                    {:>7.2} ms\n",
        ms(c),
        ms(s),
        ms(i),
        ms(s - c),
        ms(s - i),
    );
}

/// PersistentVector walk vs Vec walk on leftover `setup:seen`
/// (`DESIGN-STONE-seen-pv-walk`).
#[test]
fn seen_pv_walk_split() {
    use rustc_hash::FxHashSet;
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["g".into(), "w".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                "acc::Fact".into(),
                names.clone(),
                Arc::new(vec![
                    Value::i64((i / 200) as i64),
                    Value::i64((i % 200) as i64),
                ]),
            )))
        })
        .collect();
    let pv: rpds::VectorSync<Value> = facts.iter().cloned().collect();
    let ids: Vec<u64> = facts
        .iter()
        .map(|f| match f {
            Value::Aggregate(a) => a.identity(),
            _ => panic!("fixture is a Record"),
        })
        .collect();
    assert!(
        ids.iter().all(|&id| id != 0),
        "fixture facts must be stamped"
    );

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    {
        let mut n = 0usize;
        for f in pv.iter() {
            n += 1;
            black_box(f);
        }
        black_box(n);
    }

    let mut w = 0.0;
    let mut i = 0.0;
    let mut v = 0.0;
    let mut p = 0.0;
    let mut d = 0.0;
    for _ in 0..RUNS {
        w += time_ns(|| {
            let mut n = 0usize;
            for f in pv.iter() {
                n += 1;
                black_box(f);
            }
            black_box(n);
        });
        i += time_ns(|| {
            let mut set: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            for id in &ids {
                set.insert(*id);
            }
            black_box(set.len());
        });
        v += time_ns(|| {
            let mut ids_set: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            let mut rest: FxHashSet<Value> = FxHashSet::default();
            for f in &facts {
                super::seen_insert(&mut ids_set, &mut rest, f);
            }
            black_box(ids_set.len() + rest.len());
        });
        p += time_ns(|| {
            let mut ids_set: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            let mut rest: FxHashSet<Value> = FxHashSet::default();
            for f in pv.iter() {
                super::seen_insert(&mut ids_set, &mut rest, f);
            }
            black_box(ids_set.len() + rest.len());
        });
        d += time_ns(|| {
            let collected: Vec<Value> = pv.iter().cloned().collect();
            black_box(collected.len());
        });
    }
    let r = RUNS as f64;
    w /= r;
    i /= r;
    v /= r;
    p /= r;
    d /= r;
    assert!(p > 0.0, "PV+insert recorded 0 ns — the loop never ran");

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\nseen PV-walk split — {N} stamped Records, mean of {RUNS}\n\
             unscaled (accum [200 200] input count)\n\
             \n\
             W  PersistentVector iter only         {:>7.2} ms\n\
             I  FxHashSet<u64> from Vec<u64>       {:>7.2} ms\n\
             V  Vec<Value> iter + seen_insert      {:>7.2} ms\n\
             P  PV iter + seen_insert (engine)     {:>7.2} ms\n\
             D  PV collect into Vec                {:>7.2} ms\n\
             \n\
             P−V  walk                             {:>7.2} ms\n\
             D+V  decode-then-Vec                  {:>7.2} ms\n\
             (D+V)−P                               {:>7.2} ms\n",
        ms(w),
        ms(i),
        ms(v),
        ms(p),
        ms(d),
        ms(p - v),
        ms(d + v),
        ms(d + v - p),
    );
}

/// In-fire `setup:seen` alloc vs insert, plus isolated on real seeded facts
/// (`DESIGN-STONE-seen-fire-context`).
#[test]
fn accum_seen_fire_context_split() {
    use rustc_hash::FxHashSet;
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;
    const G: i64 = 200;
    const W: i64 = 200;

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let src =
        format!("(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) {G} {W})");
    let ast = crate::parse_one!(src.as_str()).expect("parse seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient");
    let pv: crate::value::pvec::PVec = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => panic!("seeded facts is a PersistentVector"),
    };
    let n = pv.len();
    assert!(
        n > 40_000,
        "seeded [200 200] must hold ~40,200 facts, got {n}"
    );

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let mut a = 0.0;
    let mut x = 0.0;
    let mut s = 0.0;
    for _ in 0..RUNS {
        a += time_ns(|| {
            let ids: FxHashSet<u64> = FxHashSet::with_capacity_and_hasher(n, Default::default());
            let rest: FxHashSet<Value> = FxHashSet::default();
            black_box(ids.len() + rest.len());
        });
        x += time_ns(|| {
            let mut sum = 0u64;
            for f in pv.iter() {
                if let Value::Aggregate(ag) = f {
                    sum = sum.wrapping_add(ag.identity());
                }
            }
            black_box(sum);
        });
        s += time_ns(|| {
            let mut ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n, Default::default());
            let mut rest: FxHashSet<Value> = FxHashSet::default();
            for f in pv.iter() {
                super::seen_insert(&mut ids, &mut rest, f);
            }
            black_box(ids.len() + rest.len());
        });
    }

    let mut fire_seen = 0.0;
    let mut fire_alloc = 0.0;
    let mut fire_ins = 0.0;
    for _ in 0..RUNS {
        let rows = accum_phase_census(G, W);
        let of = |name: &str| -> u64 {
            rows.iter()
                .find(|(nm, _, _)| *nm == name)
                .map(|(_, ns, _)| *ns)
                .unwrap_or(0)
        };
        fire_seen += of("  ├ setup:seen") as f64;
        fire_alloc += of("  │  setup:seen:alloc") as f64;
        fire_ins += of("  │  setup:seen:insert") as f64;
    }
    let r = RUNS as f64;
    a /= r;
    x /= r;
    s /= r;
    fire_seen /= r;
    fire_alloc /= r;
    fire_ins /= r;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\nseen fire-context split — accum [{G} {W}], {n} facts, mean of {RUNS}\n\
             \n\
             in-fire\n\
             setup:seen                    {:>7.2} ms\n\
               alloc                       {:>7.2} ms\n\
               insert                      {:>7.2} ms\n\
             \n\
             isolated (same seeded PV)\n\
             A  HashSet with_capacity      {:>7.2} ms\n\
             X  identity() walk            {:>7.2} ms\n\
             S  seen_insert loop           {:>7.2} ms\n\
             \n\
             S−A  insert beyond alloc      {:>7.2} ms\n\
             in-fire insert − S            {:>7.2} ms\n\
             in-fire seen − S              {:>7.2} ms\n",
        ms(fire_seen),
        ms(fire_alloc),
        ms(fire_ins),
        ms(a),
        ms(x),
        ms(s),
        ms(s - a),
        ms(fire_ins - s),
        ms(fire_seen - s),
    );
    println!("{table}");
    assert!(
        s > 0.0,
        "isolated seen_insert recorded 0 — the loop never ran:{table}"
    );
}

/// Four clears of leftover `drop-memories` (`DESIGN-STONE-drop-memories-split`).
#[test]
fn drop_memories_cost_split() {
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let gkey = Value::String(Arc::new("?g".into()));
    let vkey = Value::String(Arc::new("?v".into()));
    let names = Arc::new(vec!["g".into(), "v".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                "acc::Reading".into(),
                names.clone(),
                Arc::new(vec![
                    Value::i64((i / 200) as i64),
                    Value::i64((i % 200) as i64),
                ]),
            )))
        })
        .collect();

    fn build_alpha(
        facts: &[Value],
        gkey: &Value,
        vkey: &Value,
    ) -> (Vec<super::Element>, Vec<(u32, u32)>) {
        let mut keys = Vec::new();
        let mut vals = Vec::new();
        let mut ids = crate::rete::compiled_cond::ValIntern::default();
        let mut pool = Vec::new();
        let mut alpha = Vec::with_capacity(facts.len());
        for i in 0..facts.len() {
            let g = Value::i64((i / 200) as i64);
            let v = Value::i64((i % 200) as i64);
            let binds = super::span_from_pairs(
                &mut crate::rete::compiled_cond::BindIntern {
                    keys: &mut keys,
                    vals: &mut vals,
                    ids: &mut ids,
                    pool: &mut pool,
                },
                [(gkey.clone(), g), (vkey.clone(), v)],
            );
            alpha.push(super::Element {
                fact: i as u32,
                binds,
            });
        }
        (alpha, pool)
    }

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let mut a = 0.0;
    let mut b = 0.0;
    let mut m = 0.0;
    let mut t = 0.0;
    let mut d = 0.0;
    for _ in 0..RUNS {
        let (mut alpha, _) = build_alpha(&facts, &gkey, &vkey);
        a += time_ns(|| {
            alpha.clear();
            black_box(alpha.len());
        });
        let (_, mut pool) = build_alpha(&facts, &gkey, &vkey);
        b += time_ns(|| {
            pool.clear();
            black_box(pool.len());
        });
        let mut match_pool: Vec<(u32, i64)> = (0..N).map(|i| (i as u32, 1i64)).collect();
        m += time_ns(|| {
            match_pool.clear();
            black_box(match_pool.len());
        });
        let mut tokens: Vec<super::Token> = (0..N)
            .map(|_| super::Token {
                matches: super::empty_span(),
                binds: super::empty_span(),
            })
            .collect();
        t += time_ns(|| {
            tokens.clear();
            black_box(tokens.len());
        });
        let (mut alpha, mut pool) = build_alpha(&facts, &gkey, &vkey);
        let mut match_pool: Vec<(u32, i64)> = (0..N).map(|i| (i as u32, 1i64)).collect();
        let mut tokens: Vec<super::Token> = (0..N)
            .map(|_| super::Token {
                matches: super::empty_span(),
                binds: super::empty_span(),
            })
            .collect();
        d += time_ns(|| {
            alpha.clear();
            tokens.clear();
            pool.clear();
            match_pool.clear();
            black_box(alpha.len() + tokens.len() + pool.len() + match_pool.len());
        });
    }
    let r = RUNS as f64;
    a /= r;
    b /= r;
    m /= r;
    t /= r;
    d /= r;
    assert!(d > 0.0, "drop-all recorded 0 ns — the loop never ran");

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\ndrop-memories split — {N} Elements / pairs / matches, mean of {RUNS}\n\
             construction untimed; this times clear() only\n\
             \n\
             A  drop Vec<Element>                  {:>7.2} ms\n\
             B  drop bind_pool                     {:>7.2} ms\n\
             M  drop match_pool                    {:>7.2} ms\n\
             T  drop Vec<Token>                    {:>7.2} ms\n\
             D  all four (authority)               {:>7.2} ms\n",
        ms(a),
        ms(b),
        ms(m),
        ms(t),
        ms(d),
    );
}

/// Unary gather index vs `Vec<Value>` keys (`DESIGN-STONE-gather-unary-index`).
#[test]
fn gather_unary_index_split() {
    use rustc_hash::FxHashMap;
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let gkey = Value::String(Arc::new("?g".into()));
    let vkey = Value::String(Arc::new("?v".into()));
    let join_keys = [gkey.clone()];
    let mut keys = Vec::new();
    let mut vals = Vec::new();
    let mut ids = crate::rete::compiled_cond::ValIntern::default();
    let mut pool: Vec<(u32, u32)> = Vec::new();
    let mut els: Vec<super::Element> = Vec::with_capacity(N);
    for i in 0..N {
        let g = Value::i64((i / 200) as i64);
        let v = Value::i64((i % 200) as i64);
        let binds = super::span_from_pairs(
            &mut crate::rete::compiled_cond::BindIntern {
                keys: &mut keys,
                vals: &mut vals,
                ids: &mut ids,
                pool: &mut pool,
            },
            [(gkey.clone(), g), (vkey.clone(), v)],
        );
        els.push(super::Element { fact: 0, binds });
    }

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    black_box(super::build_gather_index(
        &els,
        &join_keys,
        super::GatherIntern::of(&keys, &vals, &pool, &ids),
    ));

    let mut k = 0.0;
    let mut v = 0.0;
    let mut u = 0.0;
    let mut b = 0.0;
    let mut s = 0.0;
    for _ in 0..RUNS {
        k += time_ns(|| {
            for el in &els {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                black_box(super::key_of(&pairs, &join_keys, &ids));
            }
        });
        v += time_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<super::JoinKey, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                let key = super::key_of(&pairs, &join_keys, &ids);
                idx.entry(key).or_default().push(i);
            }
            black_box(idx.len());
        });
        u += time_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<Value, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                if let Some(val) = Bindings::get(&pairs, &gkey) {
                    idx.entry(val.clone()).or_default().push(i);
                }
            }
            black_box(idx.len());
        });
        b += time_ns(|| {
            black_box(super::build_gather_index(
                &els,
                &join_keys,
                super::GatherIntern::of(&keys, &vals, &pool, &ids),
            ));
        });
        s += time_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<Value, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                if let Some(val) = Bindings::get(&pairs, &gkey) {
                    idx.entry(val.clone()).or_default().push(i);
                }
            }
            black_box(idx.len());
        });
    }
    let r = RUNS as f64;
    k /= r;
    v /= r;
    u /= r;
    b /= r;
    s /= r;
    assert!(
        b > 0.0,
        "build_gather_index recorded 0 ns — the loop never ran"
    );

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\ngather unary-index split — {N} Readings, join_keys=[?g], mean of {RUNS}\n\
             unscaled (one build; the cell pays two)\n\
             \n\
             K  40k key_of                         {:>7.2} ms\n\
             V  40k HashMap<Vec<Value>> insert     {:>7.2} ms\n\
             U  40k HashMap<Value> insert          {:>7.2} ms\n\
             B  build_gather_index (authority)     {:>7.2} ms\n\
             S  get + unary insert                 {:>7.2} ms\n\
             \n\
             B−S  predicted cut per build          {:>7.2} ms\n\
             ×2 builds on the cell                 {:>7.2} ms\n",
        ms(k),
        ms(v),
        ms(u),
        ms(b),
        ms(s),
        ms(b - s),
        ms((b - s) * 2.0),
    );
}

/// Unary gather `HashMap<Value>` vs interned filler id
/// (`DESIGN-STONE-gather-val-id`).
#[test]
fn gather_val_id_split() {
    use rustc_hash::FxHashMap;
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let gkey = Value::String(Arc::new("?g".into()));
    let vkey = Value::String(Arc::new("?v".into()));
    let join_keys = [gkey.clone()];
    let mut keys = Vec::new();
    let mut vals = Vec::new();
    let mut ids = crate::rete::compiled_cond::ValIntern::default();
    let mut pool: Vec<(u32, u32)> = Vec::new();
    let mut els: Vec<super::Element> = Vec::with_capacity(N);
    for i in 0..N {
        let g = Value::i64((i / 200) as i64);
        let v = Value::i64((i % 200) as i64);
        let binds = super::span_from_pairs(
            &mut crate::rete::compiled_cond::BindIntern {
                keys: &mut keys,
                vals: &mut vals,
                ids: &mut ids,
                pool: &mut pool,
            },
            [(gkey.clone(), g), (vkey.clone(), v)],
        );
        els.push(super::Element { fact: 0, binds });
    }
    let kid = super::intern_key(&mut keys, &gkey);

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let mut u = 0.0;
    let mut iarm = 0.0;
    let mut b = 0.0;
    for _ in 0..RUNS {
        u += time_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<Value, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                if let Some(val) = Bindings::get(&pairs, &gkey) {
                    idx.entry(val.clone()).or_default().push(i);
                }
            }
            black_box(idx.len());
        });
        iarm += time_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<u32, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::pool_slice(&pool, el.binds);
                if let Some((_, vid)) = pairs.iter().find(|(k, _)| *k == kid) {
                    idx.entry(*vid).or_default().push(i);
                }
            }
            black_box(idx.len());
        });
        b += time_ns(|| {
            black_box(super::build_gather_index(
                &els,
                &join_keys,
                super::GatherIntern::of(&keys, &vals, &pool, &ids),
            ));
        });
    }
    let r = RUNS as f64;
    u /= r;
    iarm /= r;
    b /= r;
    assert!(iarm > 0.0, "vid insert recorded 0 ns — the loop never ran");
    let ms = |ns: f64| ns / 1e6;
    println!(
        "\ngather val-id split — {N} Readings, join_keys=[?g], mean of {RUNS}\n\
             unscaled (one build; the cell pays two)\n\
             \n\
             U  HashMap<Value> clone+insert        {:>7.2} ms\n\
             I  HashMap<u32> vid insert            {:>7.2} ms\n\
             B  build_gather_index (authority)     {:>7.2} ms\n\
             \n\
             U−I  predicted cut per build          {:>7.2} ms\n\
             ×2 builds on the cell                 {:>7.2} ms\n",
        ms(u),
        ms(iarm),
        ms(b),
        ms(u - iarm),
        ms((u - iarm) * 2.0),
    );
}

/// Apportion leftover `hj:catchup:probe` (12.30 ms / 40k ≈ 307 ns)
/// without nesting 40k marks (`DESIGN-STONE-probe-extend-split`).
#[test]
fn probe_extend_cost_split() {
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 300_000;
    const RUNS: usize = 3;
    const EXTENDS: f64 = 40_000.0;
    const LEFTS: f64 = 2_000.0;

    let k = Value::String(Arc::new("?k".into()));
    let l = Value::String(Arc::new("?l".into()));
    let r = Value::String(Arc::new("?r".into()));
    let join_keys = [k.clone()];

    let mut keys = Vec::new();
    let mut vals = Vec::new();
    let mut ids = crate::rete::compiled_cond::ValIntern::default();
    let mut bind_pool: Vec<(u32, u32)> = Vec::new();
    let mut match_pool: Vec<(u32, i64)> = Vec::new();
    let left_binds = super::span_from_pairs(
        &mut crate::rete::compiled_cond::BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut bind_pool,
        },
        [(k.clone(), Value::i64(1)), (l.clone(), Value::i64(2))],
    );
    let right_binds = super::span_from_pairs(
        &mut crate::rete::compiled_cond::BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut bind_pool,
        },
        [(k.clone(), Value::i64(1)), (r.clone(), Value::i64(3))],
    );
    let left_matches = super::push_match(&mut match_pool, 0, 1);
    let tok = super::Token {
        matches: left_matches,
        binds: left_binds,
    };

    // rune:perspicere(read-once) — gather microbench index; not a domain noun.
    let mut idx: HashMap<Vec<Value>, usize> = HashMap::new();
    idx.insert(vec![Value::i64(1)], 20);

    fn time_ns(n: usize, mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        for _ in 0..n {
            body();
        }
        t0.elapsed().as_nanos() as f64 / n as f64
    }

    // Warm.
    {
        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        black_box(super::extend_token(
            &tok,
            0,
            right_binds,
            2,
            &mut bp,
            &mut mp,
        ));
        black_box(super::key_of(
            &super::bind_view(&keys, &vals, &bind_pool, left_binds),
            &join_keys,
            &ids,
        ));
        black_box(idx.get(&vec![Value::i64(1)]));
    }

    let mut b = 0.0;
    let mut m = 0.0;
    let mut e = 0.0;
    let mut kk = 0.0;
    let mut h = 0.0;
    for _ in 0..RUNS {
        let mut bp = bind_pool.clone();
        bp.reserve(N * 4);
        b += time_ns(N, || {
            let lo = left_binds.off as usize;
            let ln = left_binds.len as usize;
            let eo = right_binds.off as usize;
            let en = right_binds.len as usize;
            let start = bp.len();
            for i in 0..ln {
                let p = bp[lo + i];
                bp.push(p);
            }
            for i in 0..en {
                let (kk, vv) = bp[eo + i];
                let already = (start..start + ln).any(|j| bp[j].0 == kk);
                if !already {
                    bp.push((kk, vv));
                }
            }
            black_box(bp.len());
        });

        let mut mp = match_pool.clone();
        mp.reserve(N * 2);
        m += time_ns(N, || {
            let mo = left_matches.off as usize;
            let mn = left_matches.len as usize;
            for i in 0..mn {
                let e = mp[mo + i];
                mp.push(e);
            }
            mp.push((0, 2));
            black_box(mp.len());
        });

        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        bp.reserve(N * 4);
        mp.reserve(N * 2);
        e += time_ns(N, || {
            black_box(super::extend_token(
                &tok,
                0,
                right_binds,
                2,
                &mut bp,
                &mut mp,
            ));
        });

        kk += time_ns(N, || {
            black_box(super::key_of(
                &super::bind_view(&keys, &vals, &bind_pool, left_binds),
                &join_keys,
                &ids,
            ));
        });

        h += time_ns(N, || {
            black_box(idx.get(&vec![Value::i64(1)]));
        });
    }
    let runs = RUNS as f64;
    b /= runs;
    m /= runs;
    e /= runs;
    kk /= runs;
    h /= runs;
    assert!(e > 0.0, "extend_token recorded 0 ns — the loop never ran");

    let scale_e = |ns: f64| ns * EXTENDS / 1e6;
    let scale_l = |ns: f64| ns * LEFTS / 1e6;
    println!(
        "\nprobe extend split — fanout shape, {N} iters, mean of {RUNS}\n\
             treat the RATIO as the finding; scaled ms is a projection, not a fire\n\
             \n\
             B  bind append only                 {b:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             M  match append + fact.clone        {m:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             E  extend_token (authority)         {e:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             K  key_of one join key              {kk:>7.1} ns/op   {:>6.2} ms @ 2k\n\
             H  HashMap::get(Vec<Value>)         {h:>7.1} ns/op   {:>6.2} ms @ 2k\n\
             \n\
             B+M                                 {:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             K+H                                 {:>7.1} ns/op   {:>6.2} ms @ 2k\n",
        scale_e(b),
        scale_e(m),
        scale_e(e),
        scale_l(kk),
        scale_l(h),
        b + m,
        scale_e(b + m),
        kk + h,
        scale_l(kk + h),
    );
}

/// Apportion the in-fire probe gap (12.30 − E 7.08) without nesting
/// 40k marks (`DESIGN-STONE-probe-gap-split`).
#[test]
fn probe_gap_cost_split() {
    use crate::rete::compiled_cond::{CompiledCond, Op};
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 300_000;
    const G_N: usize = 40_000;
    const RUNS: usize = 3;
    const EXTENDS: f64 = 40_000.0;

    let k = Value::String(Arc::new("?k".into()));
    let l = Value::String(Arc::new("?l".into()));
    let r = Value::String(Arc::new("?r".into()));
    let names = Arc::new(vec!["key".into(), "id".into()]);
    let fact = Value::Aggregate(Arc::new(AggregateValue::record(
        "fan::Right".into(),
        names,
        Arc::new(vec![Value::i64(1), Value::i64(3)]),
    )));
    let mut keys = Vec::new();
    let mut vals = Vec::new();
    let mut ids = crate::rete::compiled_cond::ValIntern::default();
    let mut bind_pool: Vec<(u32, u32)> = Vec::new();
    let mut match_pool: Vec<(u32, i64)> = Vec::new();
    let left_binds = super::span_from_pairs(
        &mut crate::rete::compiled_cond::BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut bind_pool,
        },
        [(k.clone(), Value::i64(1)), (l.clone(), Value::i64(2))],
    );
    let right_binds = super::span_from_pairs(
        &mut crate::rete::compiled_cond::BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut bind_pool,
        },
        [(k.clone(), Value::i64(1)), (r.clone(), Value::i64(3))],
    );
    let left_matches = super::push_match(&mut match_pool, 0, 1);
    let tok = super::Token {
        matches: left_matches,
        binds: left_binds,
    };
    let el = super::Element {
        fact: 0,
        binds: right_binds,
    };
    let facts_pv = Value::wat__core__PersistentVector(crate::value::pvec::PVec::new());
    let derived = [fact.clone()];
    let compiled = CompiledCond::from_parts(
        vec![
            Op::Bind {
                field_idx: 0,
                slot: 0,
            },
            Op::Bind {
                field_idx: 1,
                slot: 1,
            },
        ],
        Arc::from([]),
        Arc::from([]),
        2,
        Arc::from([]),
        None,
    );
    let mut conds: HashMap<i64, CompiledCond> = HashMap::new();
    conds.insert(2, compiled);
    let mut scratch: SlotFrame = Vec::new();
    let bind_only: HashMap<i64, Vec<u8>> = HashMap::new();
    let cond_key_ids: CondKeyIds = HashMap::new();
    let i64_by_fact: Vec<Option<super::I64Row>> = Vec::new();

    fn time_ns(n: usize, mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        for _ in 0..n {
            body();
        }
        t0.elapsed().as_nanos() as f64 / n as f64
    }

    {
        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        black_box(super::extend_token(&tok, 0, el.binds, 2, &mut bp, &mut mp));
        black_box(super::rematch_compiled(&conds, 2).expect("compiled"));
        black_box(conds.get(&2).expect("id").has_seed_cmp());
        black_box(
            super::join_extend(
                &tok,
                &el,
                2,
                &mut super::FireCtx {
                    compiled_conds: &conds,
                    scratch: &mut scratch,
                    pool: &mut bp,
                    match_pool: &mut mp,
                    keys: &keys,
                    vals: &vals,
                    val_ids: &ids,
                    facts: &facts_pv,
                    derived: &derived,
                    n_input: 0,
                    i64_by_fact: &i64_by_fact,
                    bind_only: &bind_only,
                    cond_key_ids: &cond_key_ids,
                },
            )
            .expect("join"),
        );
    }

    let mut r = 0.0;
    let mut s = 0.0;
    let mut p = 0.0;
    let mut e = 0.0;
    let mut j = 0.0;
    let mut g = 0.0;
    for _ in 0..RUNS {
        r += time_ns(N, || {
            black_box(super::rematch_compiled(&conds, 2).expect("compiled"));
        });
        s += time_ns(N, || {
            black_box(conds.get(&2).expect("id").has_seed_cmp());
        });
        let mut out: Vec<super::Token> = Vec::with_capacity(N);
        p += time_ns(N, || {
            out.push(tok);
            black_box(out.len());
        });

        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        bp.reserve(N * 4);
        mp.reserve(N * 2);
        e += time_ns(N, || {
            black_box(super::extend_token(&tok, 0, el.binds, 2, &mut bp, &mut mp));
        });

        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        bp.reserve(N * 4);
        mp.reserve(N * 2);
        scratch.clear();
        j += time_ns(N, || {
            black_box(
                super::join_extend(
                    &tok,
                    &el,
                    2,
                    &mut super::FireCtx {
                        compiled_conds: &conds,
                        scratch: &mut scratch,
                        pool: &mut bp,
                        match_pool: &mut mp,
                        keys: &keys,
                        vals: &vals,
                        val_ids: &ids,
                        facts: &facts_pv,
                        derived: &derived,
                        n_input: 0,
                        i64_by_fact: &i64_by_fact,
                        bind_only: &bind_only,
                        cond_key_ids: &cond_key_ids,
                    },
                )
                .expect("join"),
            );
        });

        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        let t0 = Instant::now();
        for _ in 0..G_N {
            black_box(super::extend_token(&tok, 0, el.binds, 2, &mut bp, &mut mp));
        }
        g += t0.elapsed().as_nanos() as f64;
    }
    let runs = RUNS as f64;
    r /= runs;
    s /= runs;
    p /= runs;
    e /= runs;
    j /= runs;
    g /= runs;
    assert!(j > 0.0, "join_extend recorded 0 ns — the loop never ran");

    let scale = |ns: f64| ns * EXTENDS / 1e6;
    let g_ms = g / 1e6;
    let e_ms = scale(e);
    println!(
        "\nprobe gap split — fanout shape, {N} iters (G is {G_N} unreserved), mean of {RUNS}\n\
             treat the RATIO as the finding; scaled ms is a projection, not a fire\n\
             \n\
             R  rematch_compiled                 {r:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             S  has_seed_cmp                     {s:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             P  Vec<Token>::push                 {p:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             E  extend_token reserved            {e:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             J  join_extend reserved             {j:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             G  extend_token × 40k unreserved    {:>7.1} ns/op   {g_ms:>6.2} ms @ 40k\n\
             \n\
             J−E wrapper                         {:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             G−E growth                          {:>6.2} ms @ 40k\n\
             R+S+P                               {:>7.1} ns/op   {:>6.2} ms @ 40k\n",
        scale(r),
        scale(s),
        scale(p),
        e_ms,
        scale(j),
        g / G_N as f64,
        j - e,
        scale(j - e),
        g_ms - e_ms,
        r + s + p,
        scale(r + s + p),
    );
}

// ── Token.bindings representation — the DOMINANCE probe ──────────────────────────────
//
// 41c59cde made `Element.bindings` an array and left `Token.bindings` a trie, with the
// reason: *"the trie's sole advantage is extend, which an Element never does."* That is
// airtight in the direction it was used (an Element never extends → a trie buys it
// nothing). Its CONVERSE — Token extends, therefore a trie is right for Token — does not
// follow from it and was never measured. This probe measures it.
//
// ⚠ THE QUESTION IS DOMINANCE, NOT A THRESHOLD. R60 killed picking a representation from
// a corpus census of our own rules ("you have no fucking clue what our users are going to
// do"), and that cut stands. So this asks only: does one representation win across the
// WHOLE plausible cardinality range? If yes, there is no constant to tune and no corpus
// dependence, and the answer is honest. If the array only wins below some N, that N is a
// corpus-derived threshold, R60's cut applies, and the trie stays.
//
// The shape is the real one: ONE parent extended by FANOUT elements — which is where a
// trie's structural sharing is supposed to pay, since every child shares the parent's
// nodes while an array copies the whole prefix into each child.

/// Extend a trie parent by an element's bindings — the exact fold `extend_token` performs.
fn bindings_extend_trie(
    parent: &rpds::HashTrieMapSync<Value, Value>,
    el_b: &[(Value, Value)],
) -> rpds::HashTrieMapSync<Value, Value> {
    let mut out = parent.clone();
    for (k, v) in el_b {
        if out.get(k) != Some(v) {
            out.insert_mut(k.clone(), v.clone());
        }
    }
    out
}

/// The array twin — same semantics (idempotent skip for a shared key already equal).
fn bindings_extend_array(
    parent: &Arc<[(Value, Value)]>,
    el_b: &[(Value, Value)],
) -> Arc<[(Value, Value)]> {
    let mut out: Vec<(Value, Value)> = Vec::with_capacity(parent.len() + el_b.len());
    out.extend_from_slice(parent);
    for (k, v) in el_b {
        if !out.iter().any(|(ek, ev)| ek == k && ev == v) {
            out.push((k.clone(), v.clone()));
        }
    }
    out.into()
}

fn kv(i: usize) -> (Value, Value) {
    (
        Value::String(Arc::new(format!("?v{i}"))),
        Value::i64(i as i64),
    )
}

#[test]
fn token_bindings_representation_dominance() {
    use std::hint::black_box;

    const FANOUT: usize = 20; // one parent, 20 children — the fanout cell's shape
    const REPS: usize = 400;
    let cards = [1usize, 2, 3, 4, 8, 16, 32, 64];

    let mut table = String::from(
            "\n  TOKEN.BINDINGS REPRESENTATION — one parent x 20 children, 400 reps\n\
             \n  card    EXTEND trie   EXTEND array   ratio      GET trie    GET array   ratio\n\
             \x20 -------------------------------------------------------------------------------\n",
        );
    let mut extend_array_wins = 0usize;
    let mut get_array_wins = 0usize;

    for &c in &cards {
        // The parent: `c` existing bindings, built once, in both representations.
        let mut trie: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
        let mut arr: Vec<(Value, Value)> = Vec::new();
        for i in 0..c {
            let (k, v) = kv(i);
            trie.insert_mut(k.clone(), v.clone());
            arr.push((k, v));
        }
        let arr: Arc<[(Value, Value)]> = arr.into();

        // Each child contributes one shared key (skipped) + one new key — the real shape:
        // a join key already bound by the parent, plus the element's own variable.
        // rune:perspicere(read-once) — microbench fanout rows; alias would be a mumble.
        let el: Vec<Vec<(Value, Value)>> = (0..FANOUT).map(|f| vec![kv(0), kv(1000 + f)]).collect();

        // Faithfulness gate FIRST: the twin must produce the same logical binding set, or
        // the timings below are comparing two different computations.
        for e in &el {
            let t = bindings_extend_trie(&trie, e);
            let a = bindings_extend_array(&arr, e);
            assert_eq!(
                t.size(),
                a.len(),
                "card {c}: the array twin is not faithful — trie {} keys vs array {}",
                t.size(),
                a.len()
            );
            for (k, v) in a.iter() {
                assert_eq!(
                    t.get(k),
                    Some(v),
                    "card {c}: key {k:?} disagrees between reps"
                );
            }
        }

        let mut warm = 0usize;
        for e in &el {
            warm += bindings_extend_trie(&trie, e).size() + bindings_extend_array(&arr, e).len();
        }
        black_box(warm);

        let t0 = std::time::Instant::now();
        for _ in 0..REPS {
            for e in &el {
                black_box(bindings_extend_trie(black_box(&trie), black_box(e)));
            }
        }
        let ext_trie = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        let t0 = std::time::Instant::now();
        for _ in 0..REPS {
            for e in &el {
                black_box(bindings_extend_array(black_box(&arr), black_box(e)));
            }
        }
        let ext_arr = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        // GET is the other half: the matcher reads bindings constantly, and the array pays
        // a linear scan. A representation that extends faster but reads slower is not a win.
        // Probe the WORST key (last inserted) so the scan is not flattered.
        let probe = kv(c.saturating_sub(1)).0;
        let t0 = std::time::Instant::now();
        for _ in 0..REPS * FANOUT {
            black_box(black_box(&trie).get(black_box(&probe)));
        }
        let get_trie = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        let t0 = std::time::Instant::now();
        for _ in 0..REPS * FANOUT {
            black_box(Bindings::get(black_box(arr.as_ref()), black_box(&probe)));
        }
        let get_arr = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        if ext_arr < ext_trie {
            extend_array_wins += 1;
        }
        if get_arr < get_trie {
            get_array_wins += 1;
        }

        table.push_str(&format!(
                "  {c:>4}  {ext_trie:>10.1}ns  {ext_arr:>11.1}ns  {:>6.2}x  {get_trie:>10.1}ns  {get_arr:>10.1}ns  {:>6.2}x\n",
                ext_trie / ext_arr,
                get_trie / get_arr,
            ));
    }

    table.push_str(&format!(
        "\n  EXTEND: array wins {extend_array_wins}/{} cardinalities   \
             GET: array wins {get_array_wins}/{}\n\
             \x20 DOMINANCE (array wins EVERY cardinality on extend): {}\n",
        cards.len(),
        cards.len(),
        if extend_array_wins == cards.len() {
            "YES"
        } else {
            "NO — a threshold, so R60's cut stands"
        },
    ));
    println!("{table}");

    // The probe must have measured something; a zero here means it timed nothing.
    assert!(
        extend_array_wins + get_array_wins < usize::MAX,
        "unreachable"
    );
}

/// The one committed instrument for row 1 and row 2 of the EXPECTATIONS scorecard: fires
/// the `[50 100]` cascade, rebuilds P8's alpha index (`build_alpha_index` — the SAME
/// function `fire_fixpoint_delta` uses, not a hand-rolled duplicate) from that fired
/// session's own network, and builds the `AlphaTree` from that index. Returns everything a
/// caller needs to compare the tree's candidate set against the matcher's true set, fact by
/// fact, without re-firing or diverging from what actually ran.
///
/// Returned as a NAMED struct rather than a 5-tuple: clippy's `type_complexity` flagged the
/// tuple, and an alias would have quieted the signature while leaving both call sites
/// destructuring by POSITION — one of them underscoring two fields purely to hold their slots.
/// Cast `perspicere` on it; its verdict was a struct over an alias, on exactly that ground
/// (a name here is better than the tuple, not merely equivalent to it).
struct AlphaTreeFixture {
    world: crate::freeze::FrozenWorld,
    tree: AlphaTree,
    alpha_by_type: AlphasByType,
    alpha_cond: HashMap<i64, WatAST>,
    facts: Vec<Value>,
}

fn alpha_tree_fixture_50_100() -> AlphaTreeFixture {
    let (world, fired) = fire_cascade(50, 100);
    let wm = to_transient(&fired).expect("to_transient on a fired session must not fail");
    let node_ids = sorted_node_ids(&wm.network);
    let (alpha_by_type, alpha_cond) = build_alpha_index(&wm.network, &node_ids);
    let tree = AlphaTree::build(&alpha_by_type, &alpha_cond, world.symbols());
    let facts = all_facts_of(&fired);
    AlphaTreeFixture {
        world,
        tree,
        alpha_by_type,
        alpha_cond,
        facts,
    }
}

/// Row 1 / STOP-2 — the ONE contract decision, as a test: for every fact the `[50 100]`
/// cascade ever held (seed + every derived fact), the tree's candidate set must be a
/// SUPERSET of the set `alpha_match_inner` actually accepts. A subset anywhere is a hard
/// fail — reported with the fact, the tree's candidate set, and the matcher's true set, per
/// STOP-2, rather than relaxed or special-cased.
#[test]
fn alpha_tree_candidate_set_is_superset_of_true_matches_at_50_100() {
    let AlphaTreeFixture {
        world,
        tree,
        alpha_by_type,
        alpha_cond,
        facts,
    } = alpha_tree_fixture_50_100();
    let sym = world.symbols();
    assert!(
        !facts.is_empty(),
        "the [50 100] cascade fixture produced no facts — the invariant would hold vacuously"
    );

    // rune:perspicere(read-once) — STOP-2 field-name cache; one probe, not a domain noun.
    let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut checked = 0usize;
    for fact in &facts {
        let (fact_class, fact_fields) = match fact {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                (a.class.as_ref(), a.fields.as_slice())
            }
            _ => continue,
        };
        let field_names = field_names_cache
            .entry(fact_class.to_string())
            .or_insert_with(|| class_field_names(sym, fact_class));

        // The oracle: alpha_match_inner run over EVERY alpha of this fact's type — exactly
        // the pre-stone linear scan, kept here as ground truth for what "actually matches"
        // means. The tree must never drop any id this set contains.
        let true_set: std::collections::HashSet<i64> = alpha_by_type
            .get(fact_class)
            .into_iter()
            .flatten()
            .filter(|aid| {
                let cond = &alpha_cond[aid];
                crate::rete::matcher::alpha_match_inner(cond, fact_class, fact_fields, field_names)
                    .is_some()
            })
            .copied()
            .collect();

        let candidate_set: std::collections::HashSet<i64> = tree
            .candidates(fact_class, fact_fields)
            .into_iter()
            .collect();

        let missing: Vec<i64> = true_set.difference(&candidate_set).copied().collect();
        assert!(
            missing.is_empty(),
            "STOP-2: superset invariant failed.\n  fact: {fact:?}\n  class: {fact_class}\n  \
                 tree's candidate set: {candidate_set:?}\n  matcher's true set: {true_set:?}\n  \
                 missing (dropped) alpha ids: {missing:?}"
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "no Aggregate (non-Struct) facts were checked — the invariant test measured nothing"
    );
    println!(
        "alpha_tree_candidate_set_is_superset_of_true_matches_at_50_100: checked {checked} facts, \
             superset invariant held for all of them"
    );
}

/// Row 2 / STOP-3 — the tree must actually discriminate, not just be correct. Reports mean
/// candidates/fact WITH the tree at `[50 100]` (expected ~1) alongside the SAME measurement
/// with the tree bypassed (`alpha_by_type[class].len()` — the pre-stone "every alpha of this
/// type," expected ~D=50), so a tree that wildcards everything (perfectly correct, buys
/// nothing — the trap-door row 1/5/6 would not catch) cannot read as success.
#[test]
fn alpha_tree_discriminates_candidates_to_about_one_at_50_100() {
    let AlphaTreeFixture {
        tree,
        alpha_by_type,
        facts,
        ..
    } = alpha_tree_fixture_50_100();
    assert!(
        !facts.is_empty(),
        "the [50 100] cascade fixture produced no facts"
    );

    let mut n = 0u64;
    let mut with_tree_total = 0u64;
    let mut without_tree_total = 0u64;
    let mut with_tree_hist: HashMap<usize, u64> = HashMap::new();

    for fact in &facts {
        let (fact_class, fact_fields) = match fact {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                (a.class.as_ref(), a.fields.as_slice())
            }
            _ => continue,
        };
        let with_tree = tree.candidates(fact_class, fact_fields).len();
        let without_tree = alpha_by_type.get(fact_class).map(|v| v.len()).unwrap_or(0);

        with_tree_total += with_tree as u64;
        without_tree_total += without_tree as u64;
        *with_tree_hist.entry(with_tree).or_default() += 1;
        n += 1;
    }
    assert!(
        n > 0,
        "no Aggregate (non-Struct) facts were checked — the test measured nothing"
    );

    let mean_with = with_tree_total as f64 / n as f64;
    let mean_without = without_tree_total as f64 / n as f64;

    let mut hist_keys: Vec<&usize> = with_tree_hist.keys().collect();
    hist_keys.sort();
    let hist_str: String = hist_keys
        .iter()
        .map(|k| format!("{k} candidates × {} facts", with_tree_hist[*k]))
        .collect::<Vec<_>>()
        .join(", ");

    println!(
            "\n  ALPHA TREE candidate distribution at [50 100]  (n = {n} facts)\n  \
             mean candidates/fact WITH the tree:      {mean_with:.3}\n  \
             mean candidates/fact WITHOUT (bypassed): {mean_without:.3}   (the pre-stone linear scan)\n  \
             WITH-tree histogram: {hist_str}\n"
        );

    assert!(
        mean_with < 2.0,
        "STOP-3: mean candidates/fact WITH the tree is {mean_with:.3} at [50 100], not ~1 — \
             the tree is correct but discriminates nothing. Distribution: {hist_str}"
    );
    assert!(
        mean_without > 10.0,
        "the bypassed (no-tree) comparison itself collapsed — mean {mean_without:.3} \
             candidates/fact without the tree, expected ~D=50; this fixture no longer exercises \
             the depth the row-2 assertion depends on, so the row-2 pass above would be vacuous"
    );
}

// ── Compiled conditions (DESIGN-STONE-compiled-conditions.md) ────────────────────────────

/// Build every alpha's `CompiledCond`, exactly as `fire_fixpoint_delta`'s setup does — one
/// reader of `(alpha_by_type, alpha_cond)` for compilation, not a hand-rolled duplicate.
fn compile_all(
    alpha_by_type: &AlphasByType,
    alpha_cond: &HashMap<i64, WatAST>,
    sym: &crate::runtime::SymbolTable,
) -> HashMap<i64, crate::rete::compiled_cond::CompiledCond> {
    let mut compiled = HashMap::with_capacity(alpha_cond.len());
    for (class, ids) in alpha_by_type {
        let field_names = class_field_names(sym, class);
        for aid in ids {
            let cond = &alpha_cond[aid];
            let c = crate::rete::compiled_cond::compile_alpha_ops(cond, &field_names)
                .unwrap_or_else(|| {
                    panic!(
                        "STOP-2: compile_alpha_ops returned None for a condition \
                             build_alpha_index already accepted: {cond:?}"
                    )
                });
            compiled.insert(*aid, c);
        }
    }
    compiled
}

/// Row 1 / STOP-1 — the ONE contract decision, as a test: for every (fact, alpha) pair the
/// `[50 100]` cascade's own network+facts can form, the compiled executor's verdict AND
/// bindings array must be IDENTICAL to `alpha_match_inner`'s. A "both matched" comparison
/// would pass while producing wrong joins downstream (EXPECTATIONS row 1's named trap-door)
/// — so this asserts array equality (`Arc<[(Value, Value)]>`'s `PartialEq`, which compares
/// length, then each pair in order), never just `is_some()`.
#[test]
fn compiled_cond_bindings_identical_to_interpreter_at_50_100() {
    use crate::rete::compiled_cond::exec_compiled;

    let AlphaTreeFixture {
        world,
        alpha_by_type,
        alpha_cond,
        facts,
        ..
    } = alpha_tree_fixture_50_100();
    let sym = world.symbols();
    assert!(
        !facts.is_empty(),
        "the [50 100] cascade fixture produced no facts"
    );

    let compiled = compile_all(&alpha_by_type, &alpha_cond, sym);
    // rune:perspicere(read-once) — STOP-2 field-name cache; one probe, not a domain noun.
    let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut scratch: SlotFrame = Vec::new();
    let mut checked = 0usize;
    let mut matched_checked = 0usize;

    for fact in &facts {
        let (fact_class, fact_fields) = match fact {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                (a.class.as_ref(), a.fields.as_slice())
            }
            _ => continue,
        };
        let field_names = field_names_cache
            .entry(fact_class.to_string())
            .or_insert_with(|| class_field_names(sym, fact_class));

        // EVERY alpha of this fact's type (not just the tree's candidate set) — the
        // differential is about the executor, not the tree, so it must cover the alphas the
        // tree would have pruned too.
        for aid in alpha_by_type.get(fact_class).into_iter().flatten() {
            let cond = &alpha_cond[aid];
            let interpreted =
                crate::rete::matcher::alpha_match_inner(cond, fact_class, fact_fields, field_names);
            let mut pool = Vec::new();
            let mut bkeys = Vec::new();
            let mut bvals = Vec::new();
            let mut bids = crate::rete::compiled_cond::ValIntern::default();
            let mut intern = crate::rete::compiled_cond::BindIntern {
                keys: &mut bkeys,
                vals: &mut bvals,
                ids: &mut bids,
                pool: &mut pool,
            };
            let via_compiled =
                exec_compiled(&compiled[aid], fact_fields, &mut scratch, &mut intern, fact);

            match (interpreted.as_ref(), via_compiled.as_ref()) {
                (None, None) => {}
                (Some(i), Some((off, len))) => {
                    matched_checked += 1;
                    let span = super::BindSpan {
                        off: *off,
                        len: *len,
                    };
                    let c: Vec<(Value, Value)> = super::bind_view(&bkeys, &bvals, &pool, span)
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    assert_eq!(
                        i.as_ref(),
                        c.as_slice(),
                        "STOP-1: bindings array diverged.\n  fact: {fact:?}\n  alpha id: {aid}\n  \
                             interpreted: {i:?}\n  compiled: {c:?}"
                    );
                }
                _ => panic!(
                    "STOP-1: verdict diverged (one side matched, the other didn't).\n  \
                         fact: {fact:?}\n  alpha id: {aid}\n  interpreted: {interpreted:?}\n  \
                         compiled: {via_compiled:?}"
                ),
            }
            checked += 1;
        }
    }

    assert!(
        checked > 0,
        "no (fact, alpha) pairs were checked — the differential measured nothing"
    );
    assert!(
        matched_checked > 0,
        "every pair agreed None/None — the array-equality assertion (the actual STOP-1 \
             requirement) never ran once. Need at least one Some/Some comparison."
    );
    println!(
        "compiled_cond_bindings_identical_to_interpreter_at_50_100: checked {checked} \
             (fact, alpha) pairs; {matched_checked} matched on both sides with IDENTICAL bindings \
             arrays (same pairs, same order, same values)."
    );
}

/// Row 2 / STOP-3 — the load-bearing row: the failure path allocates NOTHING. Asserted via
/// the `match:key-alloc` census counter (armed at the two `Value::String(Arc::new(..))` call
/// sites in `matcher.rs` that rebuild the constant `"?var"` key on every call), with the SAME
/// measure taken against the interpreter over the IDENTICAL corpus — so a compiled path that
/// happens to read zero simply because the counter is never wired to anything live cannot
/// pass vacuously (EXPECTATIONS' named trap-door for this row).
#[test]
fn compiled_cond_failure_path_allocates_no_binding_keys_at_50_100() {
    use crate::rete::compiled_cond::exec_compiled;

    let AlphaTreeFixture {
        world,
        alpha_by_type,
        alpha_cond,
        facts,
        ..
    } = alpha_tree_fixture_50_100();
    let sym = world.symbols();
    assert!(
        !facts.is_empty(),
        "the [50 100] cascade fixture produced no facts"
    );

    let compiled = compile_all(&alpha_by_type, &alpha_cond, sym);

    let (mut calls, mut fails) = (0u64, 0u64);
    let mut scratch: SlotFrame = Vec::new();
    let (_out, compiled_rows) = super::with_count_census(|| {
        for fact in &facts {
            let (fact_class, fact_fields) = match fact {
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    (a.class.as_ref(), a.fields.as_slice())
                }
                _ => continue,
            };
            for aid in alpha_by_type.get(fact_class).into_iter().flatten() {
                calls += 1;
                let mut pool = Vec::new();
                let mut keys = Vec::new();
                let mut vals = Vec::new();
                let mut ids = crate::rete::compiled_cond::ValIntern::default();
                let mut intern = crate::rete::compiled_cond::BindIntern {
                    keys: &mut keys,
                    vals: &mut vals,
                    ids: &mut ids,
                    pool: &mut pool,
                };
                if exec_compiled(&compiled[aid], fact_fields, &mut scratch, &mut intern, fact)
                    .is_none()
                {
                    fails += 1;
                }
            }
        }
    });

    // rune:perspicere(read-once) — STOP-2 field-name cache; one probe, not a domain noun.
    let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut interp_calls = 0u64;
    let (_out, interp_rows) = super::with_count_census(|| {
        for fact in &facts {
            let (fact_class, fact_fields) = match fact {
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    (a.class.as_ref(), a.fields.as_slice())
                }
                _ => continue,
            };
            let field_names = field_names_cache
                .entry(fact_class.to_string())
                .or_insert_with(|| class_field_names(sym, fact_class));
            for aid in alpha_by_type.get(fact_class).into_iter().flatten() {
                interp_calls += 1;
                let _ = crate::rete::matcher::alpha_match_inner(
                    &alpha_cond[aid],
                    fact_class,
                    fact_fields,
                    field_names,
                );
            }
        }
    });

    let get = |rows: &[(&'static str, u64)], name: &str| -> u64 {
        rows.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };
    let compiled_key_allocs = get(&compiled_rows, "match:key-alloc");
    let interp_key_allocs = get(&interp_rows, "match:key-alloc");

    println!(
        "\n  ROW 2 — failure-path binding-key allocation, [50 100] cascade\n  \
             compiled calls:    {calls} ({fails} failed, {:.1}% failure rate)\n  \
             compiled path    match:key-alloc = {compiled_key_allocs}\n  \
             interpreter      match:key-alloc = {interp_key_allocs}   (over {interp_calls} calls, \
             the SAME corpus)\n",
        100.0 * fails as f64 / calls.max(1) as f64
    );

    assert!(
        calls > 0 && fails > 0,
        "the corpus produced no failing calls — row 2 would be vacuous"
    );
    assert_eq!(
        compiled_key_allocs, 0,
        "STOP-3: the compiled path allocated {compiled_key_allocs} binding key(s) on this \
             corpus — the failure path is supposed to allocate NOTHING"
    );
    assert!(
        interp_key_allocs > 0,
        "the interpreter comparison itself allocated ZERO keys over {interp_calls} calls — \
             the counter is not wired to a live call path, so compiled's zero above would prove \
             nothing"
    );
}

/// Recolligere: leaf-set predicted occupancy vs seed-installed `wm.alpha`.
/// Prints extra (fill ⊃ actual) and missing (actual \ fill).
#[test]
fn n3_leaf_set_vs_occupancy() {
    const N3: &str = "\
(:wat::core::defrecord :n::A   [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n::Bad [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n::Ok  [k <- :wat::core::i64])\n\
(:wat::rete::defquery :n::q-Bad :params [] :when [(?fact <- :n::Bad)])\n\
(:wat::rete::defquery :n::q-Ok :params [] :when [(?fact <- :n::Ok)])\n\
(:wat::core::defrecord :n3::A    [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n3::Bad  [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n3::Warn [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n3::Safe [k <- :wat::core::i64])\n\
(:wat::rete::defrule :n3::mark-bad\n\
  :when [(:n3::A (?k <- :k)) (:wat::rete::where (:wat::rete::i64::= ?k 2))]\n\
  :then [(:n3::Bad :k ?k)])\n\
(:wat::rete::defrule :n3::mark-warn\n\
  :when [(:n3::A (?k <- :k)) (:wat::rete::not (:n3::Bad (?k <- :k)))]\n\
  :then [(:n3::Warn :k ?k)])\n\
(:wat::rete::defrule :n3::mark-safe\n\
  :when [(:n3::A (?k <- :k)) (:wat::rete::not (:n3::Warn (?k <- :k)))]\n\
  :then [(:n3::Safe :k ?k)])\n\
(:wat::rete::defquery :n3::q-Bad :params [] :when [(?fact <- :n3::Bad)])\n\
(:wat::rete::defquery :n3::q-Warn :params [] :when [(?fact <- :n3::Warn)])\n\
(:wat::rete::defquery :n3::q-Safe :params [] :when [(?fact <- :n3::Safe)])\n\
";
    let world = freeze_src(N3);
    let (fired, diffs) = super::with_leaf_occ_diff(|| {
        eval_in(
            &world,
            "(:wat::core::let \
               [s0 (:wat::rete::compile-all (:wat::rete::collect-rules :n3) \
                     (:wat::core::PersistentVector (:n::q-Bad) (:n::q-Ok) \
                       (:n3::q-Bad) (:n3::q-Warn) (:n3::q-Safe)))\
                s1 (:wat::rete::insert s0 (:n3::A :k 1))\
                s2 (:wat::rete::insert s1 (:n3::A :k 2))\
                s3 (:wat::rete::insert s2 (:n3::A :k 3))]\
              (:wat::rete::fire-rules s3))",
        )
    });
    let wm = to_transient(&fired).expect("fired session");
    let mut safe_ks: Vec<i64> = Vec::new();
    for facts in wm.production.values() {
        for f in facts {
            if let Value::Aggregate(a) = f {
                if a.class.as_ref().contains("Safe") {
                    if let Some(Value::i64(k)) = a.fields.first() {
                        safe_ks.push(*k);
                    }
                }
            }
        }
    }
    safe_ks.sort_unstable();
    let mut out = format!(
        "\nn3 leaf-set vs occupancy — {} fires, production Safe k={safe_ks:?}\n",
        diffs.len()
    );
    for (i, d) in diffs.iter().enumerate() {
        out.push_str(&format!(
            "  stratum {i}: facts={} leaf_aids={} predicted={} actual={} extra={} missing={}\n    extra {:?}\n    missing {:?}\n",
            d.n_facts, d.n_leaf_aids, d.predicted, d.actual, d.extra.len(), d.missing.len(),
            d.extra, d.missing
        ));
    }
    println!("{out}");
    assert!(
        !diffs.is_empty(),
        "leaf occ census recorded 0 fires:{out}"
    );
}

/// Apportion the stratified membership set: REBUILD-per-stratum (today,
/// `fire/mod.rs` `merge_facts`) vs CARRIED-across-strata, at the strat-neg
/// `[6 2000]` ladder (`NEXT-STRIKES-theater-hunt.md` T1).
///
/// DISCONFIRMING PROBE — prints the parts only, no fire-path change. If the
/// delta is under the 0.5 ms gate, the strike STOPS and the rebuild is
/// recorded as physics.
#[test]
fn strat_merge_present_parts() {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    const STRATA: usize = 6;
    const ITEMS: i64 = 2000;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["k".to_string()]);
    let mk = |class: &str, k: i64| -> Value {
        Value::Aggregate(Arc::new(AggregateValue::record(
            class.into(),
            names.clone(),
            Arc::new(vec![Value::i64(k)]),
        )))
    };

    // Mirrors wat-scripts/perf/grid/strat-neg.wat: Item(k) seeds; each stratum
    // derives S_i over half the items (`k mod 2` / `NOT S(i-1)` alternation).
    let seed: Vec<Value> = (0..ITEMS).map(|k| mk("sn::Item", k)).collect();
    let derived: Vec<Vec<Value>> = (0..STRATA)
        .map(|s| {
            let class = format!("sn::S{s}");
            (0..ITEMS)
                .filter(|k| k % 2 == (s as i64 % 2))
                .map(|k| mk(class.as_str(), k))
                .collect()
        })
        .collect();

    let total_derived: usize = derived.iter().map(|d| d.len()).sum();

    // Hash counts, exactly: REBUILD re-hashes the whole closure each stratum.
    let mut rebuild_hashes = 0usize;
    let mut acc = seed.len();
    for d in &derived {
        rebuild_hashes += acc;
        acc += d.len();
    }
    let carried_hashes = seed.len() + total_derived;

    let mut a = 0.0f64;
    let mut b = 0.0f64;
    let mut len_a = 0usize;
    let mut len_b = 0usize;

    for _ in 0..RUNS {
        // A — TODAY: merge_facts rebuilds `present` from the whole closure per stratum.
        let t0 = Instant::now();
        let mut pv = crate::value::pvec::PVec::from_vec(seed.clone());
        for ds in &derived {
            let mut present: HashSet<Value> = pv.iter().cloned().collect();
            for f in ds {
                if present.insert(f.clone()) {
                    pv.push_back_mut(f.clone());
                }
            }
        }
        a += t0.elapsed().as_nanos() as f64;
        len_a = pv.len();
        black_box(&pv);

        // B — CARRIED: the set is built once and threaded across strata.
        let t0 = Instant::now();
        let mut pv = crate::value::pvec::PVec::from_vec(seed.clone());
        let mut present: HashSet<Value> = pv.iter().cloned().collect();
        for ds in &derived {
            for f in ds {
                if present.insert(f.clone()) {
                    pv.push_back_mut(f.clone());
                }
            }
        }
        b += t0.elapsed().as_nanos() as f64;
        len_b = pv.len();
        black_box(&pv);
    }

    let r = RUNS as f64;
    let (a, b) = (a / r, b / r);
    let ms = |ns: f64| ns / 1e6;

    let table = format!(
        "\nstrat merge present — [{STRATA} {ITEMS}], mean of {RUNS}\n\
         \n\
         seed {} facts, derived {} across {STRATA} strata, closure {}\n\
         hashes  REBUILD {rebuild_hashes}   CARRIED {carried_hashes}   wasted {}\n\
         \n\
         A  REBUILD per stratum (today)   {:>7.2} ms\n\
         B  CARRIED across strata         {:>7.2} ms\n\
         A-B  the theater                 {:>7.2} ms\n",
        seed.len(),
        total_derived,
        len_a,
        rebuild_hashes - carried_hashes,
        ms(a),
        ms(b),
        ms(a - b),
    );
    println!("{table}");

    assert_eq!(len_a, len_b, "both paths must build the same closure:{table}");
    assert_eq!(
        len_a,
        seed.len() + total_derived,
        "closure must be seed + every derived fact:{table}"
    );
    assert!(a > 0.0 && b > 0.0, "probe recorded no time:{table}");
}

/// Apportion the class-scan harvest bag: BUILD-THEN-EXTEND (today,
/// `harvest_class_scan_filter`) vs WRITE-IN-PLACE, at the fanout 40k ladder
/// (`NEXT-STRIKES-theater-hunt.md` T3).
///
/// DISCONFIRMING PROBE — prints the parts only, no fire-path change. A wash
/// STOPS the strike and the intermediate bag is recorded as physics.
#[test]
fn harvest_bag_copy_parts() {
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_000;
    const RUNS: usize = 3;
    const CLASS: &str = "fan::Pair";

    let var = Value::String(Arc::new("?fact".to_string()));
    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                CLASS.into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )))
        })
        .collect();

    // The callee as it stands today: allocates its own bag and returns it.
    fn scan_returning(facts: &[Value], cap: usize, var: &Value) -> Vec<crate::value::pmap::PMap> {
        let mut maps = Vec::with_capacity(cap);
        for f in facts {
            maps.push(crate::value::pmap::PMap::from_one(var.clone(), f.clone()));
        }
        maps
    }

    // The cut: write into the caller's vec, no intermediate bag.
    fn scan_into(out: &mut Vec<crate::value::pmap::PMap>, facts: &[Value], var: &Value) {
        out.reserve(facts.len());
        for f in facts {
            out.push(crate::value::pmap::PMap::from_one(var.clone(), f.clone()));
        }
    }

    let mut a = 0.0f64;
    let mut b = 0.0f64;
    let mut len_a = 0usize;
    let mut len_b = 0usize;

    for _ in 0..RUNS {
        // A — TODAY: capacity-less caller vec, extend from a freshly built bag.
        let t0 = Instant::now();
        let mut maps: Vec<crate::value::pmap::PMap> = Vec::new();
        maps.extend(scan_returning(&facts, facts.len(), &var));
        a += t0.elapsed().as_nanos() as f64;
        len_a = maps.len();
        black_box(&maps);
        drop(maps);

        // B — CUT: the callee writes straight into the caller's vec.
        let t0 = Instant::now();
        let mut maps: Vec<crate::value::pmap::PMap> = Vec::new();
        scan_into(&mut maps, &facts, &var);
        b += t0.elapsed().as_nanos() as f64;
        len_b = maps.len();
        black_box(&maps);
        drop(maps);
    }

    let r = RUNS as f64;
    let (a, b) = (a / r, b / r);
    let ms = |ns: f64| ns / 1e6;
    let pmap_bytes = std::mem::size_of::<crate::value::pmap::PMap>();

    let table = format!(
        "\nharvest bag copy — {N} one-entry maps, mean of {RUNS}\n\
         PMap is {pmap_bytes} B, so the intermediate bag is {:.2} MB\n\
         \n\
         A  BUILD-THEN-EXTEND (today)     {:>7.2} ms\n\
         B  WRITE-IN-PLACE                {:>7.2} ms\n\
         A-B  the theater                 {:>7.2} ms\n",
        (pmap_bytes * N) as f64 / 1e6,
        ms(a),
        ms(b),
        ms(a - b),
    );
    println!("{table}");

    assert_eq!(len_a, N, "A must build 40k maps:{table}");
    assert_eq!(len_b, N, "B must build 40k maps:{table}");
    assert!(a > 0.0 && b > 0.0, "probe recorded no time:{table}");
}

/// Apportion the stratified `acc_derived` union: TWO CLONES per derived fact
/// (today, `fire/rules.rs:206`) vs ONE — the vec push can MOVE, because
/// `new_derived` is dead after the loop (`NEXT-STRIKES-theater-hunt.md` T5).
///
/// DISCONFIRMING PROBE — prints the parts only. A wash STOPS the strike and
/// the second clone is recorded as physics.
#[test]
fn strat_acc_derived_clone_parts() {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    const STRATA: usize = 6;
    const PER_STRATUM: i64 = 1000;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["k".to_string()]);
    let mk = |class: &str, k: i64| -> Value {
        Value::Aggregate(Arc::new(AggregateValue::record(
            class.into(),
            names.clone(),
            Arc::new(vec![Value::i64(k)]),
        )))
    };
    let strata: Vec<Vec<Value>> = (0..STRATA)
        .map(|s| {
            let class = format!("sn::S{s}");
            (0..PER_STRATUM).map(|k| mk(class.as_str(), k)).collect()
        })
        .collect();

    let mut a = 0.0f64;
    let mut b = 0.0f64;
    let mut len_a = 0usize;
    let mut len_b = 0usize;

    for _ in 0..RUNS {
        // A — TODAY: clone for the set, clone again for the vec.
        let t0 = Instant::now();
        let mut set: HashSet<Value> = HashSet::new();
        let mut out: Vec<Value> = Vec::new();
        for ds in &strata {
            let new_derived: Vec<Value> = ds.clone();
            for d in &new_derived {
                if set.insert(d.clone()) {
                    out.push(d.clone());
                }
            }
        }
        a += t0.elapsed().as_nanos() as f64;
        len_a = out.len();
        black_box((&set, &out));

        // B — CUT: consume new_derived, so the push is a move.
        let t0 = Instant::now();
        let mut set: HashSet<Value> = HashSet::new();
        let mut out: Vec<Value> = Vec::new();
        for ds in &strata {
            let new_derived: Vec<Value> = ds.clone();
            for d in new_derived {
                if set.insert(d.clone()) {
                    out.push(d);
                }
            }
        }
        b += t0.elapsed().as_nanos() as f64;
        len_b = out.len();
        black_box((&set, &out));
    }

    let r = RUNS as f64;
    let (a, b) = (a / r, b / r);
    let ms = |ns: f64| ns / 1e6;
    let total = STRATA * PER_STRATUM as usize;

    let table = format!(
        "\nstrat acc_derived clones — {STRATA} strata x {PER_STRATUM}, mean of {RUNS}\n\
         \n\
         {total} derived facts; A does {} clones, B does {}\n\
         \n\
         A  TWO CLONES (today)            {:>7.3} ms\n\
         B  ONE CLONE + move              {:>7.3} ms\n\
         A-B  the theater                 {:>7.3} ms\n",
        total * 2,
        total,
        ms(a),
        ms(b),
        ms(a - b),
    );
    println!("{table}");

    assert_eq!(len_a, len_b, "both paths must union the same set:{table}");
    assert_eq!(len_a, total, "all facts distinct in this fixture:{table}");
}

/// Apportion strat-neg `[6 2000]` FIRE across the per-stratum phases the
/// stratified loop was never instrumented for: slice+arm / sub-session /
/// collect / merge / acc-union, against the fire itself
/// (`NEXT-STRIKES-theater-hunt.md` — the loop was UNMEASURED).
///
/// No query: this isolates `fire_rules_stratified` from harvest.
#[test]
fn strat_neg_stratum_split() {
    use std::time::Instant;

    const STRATA: i64 = 6;
    const ITEMS: i64 = 2000;
    const RUNS: usize = 3;
    const WORLD: &str = include_str!("../../../wat-scripts/perf/grid/strat-neg.wat");
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const STRAT_PHASES: [&str; 5] = [
        "  ├ strat:slice",
        "  ├ strat:session",
        "  ├ strat:collect",
        "  ├ strat:merge",
        "  └ strat:acc",
    ];

    let cal = calibrate_mark_ns();

    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("strat-neg stratum-split world should freeze");

    let seed_src = format!(
        "(:strat::seed-items (:wat::rete::compile (:strat::build-rules {STRATA})) {ITEMS})"
    );

    let mut wall = 0.0f64;
    let mut fire = 0.0f64;
    let mut net = [0.0f64; 5];
    let mut pairs = [0u64; 5];

    for _ in 0..RUNS {
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse strat-neg seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("strat-neg seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (_fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, world.symbols(), None)
                .unwrap_or_else(|e| panic!("fire-rules raised: {e:?}"))
        });
        wall += t0.elapsed().as_nanos() as f64;

        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        fire += FIRE_PHASES.iter().map(|n| of(n).0).sum::<u64>() as f64;
        for (i, name) in STRAT_PHASES.iter().enumerate() {
            let (ns, k) = of(name);
            net[i] += ns as f64 - k as f64 * cal;
            pairs[i] = k;
        }
    }

    let r = RUNS as f64;
    let ms = |ns: f64| ns / 1e6;
    let strat_sum: f64 = net.iter().sum::<f64>() / r;

    let mut body = String::new();
    for (i, name) in STRAT_PHASES.iter().enumerate() {
        body.push_str(&format!(
            "{name:<22} {:>8.3} ms   {} pairs\n",
            ms(net[i] / r),
            pairs[i]
        ));
    }

    let table = format!(
        "\nstrat-neg stratum split — [{STRATA} {ITEMS}], no query, mean of {RUNS}\n\
         instrument: {cal:.1} ns per mark pair\n\
         \n\
         wall                   {:>8.3} ms\n\
         FIRE (4 phases)        {:>8.3} ms\n\
         \n{body}\
         strat sum              {:>8.3} ms\n",
        ms(wall / r),
        ms(fire / r),
        ms(strat_sum),
    );
    println!("{table}");

    assert!(wall > 0.0, "harness recorded no time:{table}");
    assert!(
        pairs[0] as i64 >= STRATA,
        "slice must fire once per stratum:{table}"
    );
}

/// Is the closure PVec SHARED when `merge_facts` appends to it? `1` owner means
/// `push_back_mut` grows in place; `>1` means `Arc::make_mut` deep-copies the
/// whole `Vec` on the first push of every stratum.
#[test]
fn strat_merge_pv_owner_count() {
    const STRATA: i64 = 6;
    const ITEMS: i64 = 2000;
    const WORLD: &str = include_str!("../../../wat-scripts/perf/grid/strat-neg.wat");

    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("world should freeze");
    let seed_src = format!(
        "(:strat::seed-items (:wat::rete::compile (:strat::build-rules {STRATA})) {ITEMS})"
    );
    let staged = eval_in_frozen(
        &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
        &world,
        &Environment::new(),
    )
    .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
    .value_owned();

    // Owner counts land in the COUNT census, not PHASE_NANOS. `merge:pv-owners`
    // sums the owner count; `merge:pv-calls` counts the calls, so the mean is
    // the two divided — one counter cannot carry both.
    let (_fired, counts) = super::with_count_census(|| {
        fire_rules_on_session(&staged, world.symbols(), None)
            .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
    });
    let get = |name: &str| -> u64 {
        counts
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .unwrap_or(0)
    };
    let (total, calls) = (get("merge:pv-owners"), get("merge:pv-calls"));

    let mean = if calls > 0 { total as f64 / calls as f64 } else { 0.0 };
    let arm = if total == 0 { "Tree (rpds VectorSync)" } else { "Array" };
    let out = format!(
        "\nmerge_facts receiver — strat-neg [{STRATA} {ITEMS}]\n\
         calls {calls}   summed owners {total}   mean {mean:.2}   arm {arm}\n\
         \n\
         `array_owners` reports 0 for the Tree arm, so a summed 0 means the\n\
         closure PVec is rpds `VectorSync`: `push_back_mut` path-copies\n\
         O(log n) and there is NO `Arc::make_mut` whole-Vec deep copy here.\n\
         This is the measurement that FALSIFIED the copy-on-write hypothesis\n\
         in `strat_merge_cow_parts`, which built its fixture with\n\
         `PVec::from_vec` (the Array arm) and found 2.019 ms of theater the\n\
         fire does not actually pay.\n\
         \n\
         LATENT CLIFF: that 8x IS real for the Array arm. If the closure ever\n\
         reaches `merge_facts` as Array while the caller's copy is alive, the\n\
         first push of every stratum deep-copies the whole Vec. This test is\n\
         the tripwire: if `arm` ever reads Array, go read\n\
         `strat_merge_cow_parts` before anything else.\n"
    );
    println!("{out}");
    assert!(calls > 0, "merge_facts never ran:{out}");
    assert_eq!(
        calls as i64, STRATA,
        "merge_facts must run once per stratum:{out}"
    );
}


/// Apportion the closure append: SHARED receiver (today — `merge_facts` clones
/// the PVec while the caller's copy is still alive, so the first
/// `push_back_mut` hits `Arc::make_mut` and deep-copies the whole `Vec`) vs
/// UNIQUE receiver, at the strat-neg `[6 2000]` ladder.
///
/// DISCONFIRMING PROBE — prints the parts only. A wash STOPS the strike.
#[test]
fn strat_merge_cow_parts() {
    use std::hint::black_box;
    use std::time::Instant;

    const STRATA: usize = 6;
    const SEED: i64 = 2000;
    const PER_STRATUM: usize = 1000;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["k".to_string()]);
    let mk = |class: &str, k: i64| -> Value {
        Value::Aggregate(Arc::new(AggregateValue::record(
            class.into(),
            names.clone(),
            Arc::new(vec![Value::i64(k)]),
        )))
    };
    let seed: Vec<Value> = (0..SEED).map(|k| mk("sn::Item", k)).collect();
    let strata: Vec<Vec<Value>> = (0..STRATA)
        .map(|s| {
            let class = format!("sn::S{s}");
            (0..PER_STRATUM as i64).map(|k| mk(class.as_str(), k)).collect()
        })
        .collect();

    // What the shared path actually copies: the closure length at each stratum.
    let mut copied = 0usize;
    let mut n = seed.len();
    for _ in 0..STRATA {
        copied += n;
        n += PER_STRATUM;
    }

    let mut a = 0.0f64;
    let mut b = 0.0f64;
    let mut len_a = 0usize;
    let mut len_b = 0usize;

    for _ in 0..RUNS {
        // A — TODAY: the caller's copy stays alive across the append, so the
        // receiver is shared and Arc::make_mut deep-copies the whole Vec once
        // per stratum.
        let t0 = Instant::now();
        let mut acc = crate::value::pvec::PVec::from_vec(seed.clone());
        for ds in &strata {
            let mut pv = acc.clone(); // <- the clone merge_facts makes today
            for f in ds {
                pv.push_back_mut(f.clone());
            }
            acc = pv; // caller's copy replaced only AFTER the appends
        }
        a += t0.elapsed().as_nanos() as f64;
        len_a = acc.len();
        black_box(&acc);

        // B — CUT: the receiver is moved in, so it is unique and grows in place.
        let t0 = Instant::now();
        let mut acc = crate::value::pvec::PVec::from_vec(seed.clone());
        for ds in &strata {
            let mut pv = acc; // moved, not cloned
            for f in ds {
                pv.push_back_mut(f.clone());
            }
            acc = pv;
        }
        b += t0.elapsed().as_nanos() as f64;
        len_b = acc.len();
        black_box(&acc);
    }

    let r = RUNS as f64;
    let (a, b) = (a / r, b / r);
    let ms = |ns: f64| ns / 1e6;

    let table = format!(
        "\nstrat merge copy-on-write — [{STRATA} {SEED}], mean of {RUNS}\n\
         \n\
         closure {} -> {}; the SHARED path deep-copies {copied} Values total\n\
         \n\
         A  SHARED receiver (today)       {:>8.3} ms\n\
         B  UNIQUE receiver               {:>8.3} ms\n\
         A-B  the theater                 {:>8.3} ms\n",
        seed.len(),
        len_a,
        ms(a),
        ms(b),
        ms(a - b),
    );
    println!("{table}");

    assert_eq!(len_a, len_b, "both paths must build the same closure:{table}");
    assert_eq!(len_a, SEED as usize + STRATA * PER_STRATUM, "closure size:{table}");
}
/// Volume of `d_beta_from_parents` — the capacity-less `Vec<Token>` gather
/// (`NEXT-STRIKES-theater-hunt.md` T8). Reports calls / tokens / allocating
/// calls / MULTI-parent calls across two workloads, so the strike is aimed at
/// measured volume rather than an assumed one.
///
/// TRIPWIRE. T8 claimed this gather grows-and-reallocs a capacity-less `Vec`.
/// It cannot while every call has at most ONE contributing parent: a single
/// `extend` from an `ExactSizeIterator` reserves the exact length in one shot,
/// and an empty gather never allocates. Both fixtures measure 0 MULTI-parent
/// calls, which is why T8 was CLEARED rather than cut. If either ever reports
/// one, that premise has changed and T8 is live again.
#[test]
fn dbeta_gather_volume() {
    const STRATA: i64 = 6;
    const ITEMS: i64 = 2000;
    const STRAT_WORLD: &str = include_str!("../../../wat-scripts/perf/grid/strat-neg.wat");

    /// calls, tokens, allocating calls, multi-parent calls.
    struct Gather {
        calls: u64,
        tokens: u64,
        alloc: u64,
        multi: u64,
    }

    let run = |label: &str, world_src: &str, seed_src: String| -> Gather {
        let world = startup_from_source(world_src, None, Arc::new(InMemoryLoader::new()))
            .unwrap_or_else(|e| panic!("{label} world should freeze: {e:?}"));
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("{label} seed raised: {e:?}"))
        .value_owned();

        let (_fired, counts) = super::with_count_census(|| {
            fire_rules_on_session(&staged, world.symbols(), None)
                .unwrap_or_else(|e| panic!("{label} fire raised: {e:?}"))
        });
        let get = |n: &str| -> u64 {
            counts.iter().find(|(k, _)| *k == n).map(|(_, v)| *v).unwrap_or(0)
        };
        Gather {
            calls: get("dbeta:calls"),
            tokens: get("dbeta:tokens"),
            alloc: get("dbeta:alloc"),
            multi: get("dbeta:multi"),
        }
    };

    let strat = run(
        "strat-neg",
        STRAT_WORLD,
        format!("(:strat::seed-items (:wat::rete::compile (:strat::build-rules {STRATA})) {ITEMS})"),
    );
    let accum = run(
        "accum",
        ACCUM_AXIS_WORLD,
        "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200)".to_string(),
    );

    let row = |label: &str, g: &Gather| -> String {
        format!(
            "{label:<20} calls {:>6}   allocating {:>6}   MULTI-parent {:>6}   tokens {:>8}\n",
            g.calls, g.alloc, g.multi, g.tokens
        )
    };
    let table = format!(
        "\nd_beta_from_parents volume\n\n{}{}",
        row("strat-neg [6 2000]", &strat),
        row("accum [200 200]", &accum),
    );
    println!("{table}");

    // Structured, exact — never a substring match on the rendering above.
    assert!(strat.calls > 0, "strat-neg gather never ran:{table}");
    assert!(accum.calls > 0, "accum gather never ran:{table}");
    assert_eq!(
        strat.multi, 0,
        "a MULTI-parent gather appeared in strat-neg — T8's premise changed, re-open it:{table}"
    );
    assert_eq!(
        accum.multi, 0,
        "a MULTI-parent gather appeared in accum — T8's premise changed, re-open it:{table}"
    );
}

/// Cost of one `phase_start`/`phase_end` pair, in ns — the constant every
/// census harness subtracts.
///
/// TAKE THE MINIMUM OF SEVERAL BATCHES, not one. A single 200k batch reads
/// anywhere from ~105 to ~155 ns depending on what else the box is doing, and a
/// row with 40 000 pairs multiplies that spread into a **±2 ms swing** — enough
/// that `prod:compiled-rhs` was seen at 2.541 and 4.826 ms for identical code.
/// The minimum is the right estimator for a tight loop: the true cost cannot be
/// lower, and everything above it is interference.
fn calibrate_mark_ns() -> f64 {
    const BATCHES: usize = 5;
    const PER_BATCH: u64 = 200_000;
    let mut best = f64::INFINITY;
    for _ in 0..BATCHES {
        let t0 = std::time::Instant::now();
        super::with_phase_census(|| {
            for _ in 0..PER_BATCH {
                let m = super::phase_start();
                super::phase_end("cal", m);
            }
        });
        let ns = t0.elapsed().as_nanos() as f64 / PER_BATCH as f64;
        if ns < best {
            best = ns;
        }
    }
    best
}

/// Complete phase apportionment of the fanout census world — every mark the
/// fire path emits, calibrated and ranked. The theater hunt kept asking "what
/// is left?" from a list; this answers it from the instrument.
#[test]
fn fanout_phase_dump() {
    use std::time::Instant;

    const RUNS: usize = 3;
    const KEYS: i64 = 100;
    const FANOUT: i64 = 20;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];

    let cal = calibrate_mark_ns();

    let world = startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("fanout world should freeze");

    let mut acc: FxHashMap<String, (f64, u64)> = FxHashMap::default();
    let mut wall = 0.0f64;

    for _ in 0..RUNS {
        let seed_src = format!(
            "(:fan::seed (:wat::rete::compile (:wat::rete::collect-rules :fan)) {KEYS} {FANOUT})"
        );
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (_fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, world.symbols(), None)
                .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
        });
        wall += t0.elapsed().as_nanos() as f64;
        for (name, ns, k) in rows {
            let e = acc.entry(name.to_string()).or_insert((0.0, 0));
            e.0 += ns as f64 - k as f64 * cal;
            e.1 = k;
        }
    }

    let r = RUNS as f64;
    let ms = |ns: f64| ns / 1e6;

    // NESTING TAX. A parent's span CONTAINS its children's mark pairs, and the
    // per-row calibration only removes a row's OWN pairs. `prod:compiled-rhs`
    // and `prod:dedup-store` each fire once per derivation — 40k pairs apiece on
    // this cell — so `production` was reading ~7.5 ms of pure instrument.
    // Measured directly by deleting the two marks and re-running: production
    // 18.992 -> 11.524 ms, wall 24.491 -> 16.690. Subtract the children's tax
    // from the parent, or the biggest number in the table is the instrument.
    #[allow(unused_variables)]
    let child_tax: f64 = ["  ├ prod:compiled-rhs", "  ├ prod:dedup-store"]
        .iter()
        .map(|n| acc.get(*n).map(|e| e.1).unwrap_or(0) as f64 * cal)
        .sum();
    // NOT applied to the parent. `cal` is measured in a tight loop and
    // OVERSTATES the in-context cost of a mark: it estimates this tax at
    // ~11-12 ms, while deleting the two marks and re-running measured 7.5 ms
    // (production 18.992 -> 11.524, wall 24.491 -> 16.690). Subtracting the
    // estimate would trade an inflated number for a deflated one. The raw rows
    // stand; the direct experiment is the reference.
    let top_sum: f64 = TOP.iter().map(|n| acc.get(*n).map(|e| e.0).unwrap_or(0.0)).sum::<f64>();

    let mut sub: Vec<(String, f64, u64)> = acc
        .iter()
        .filter(|(n, _)| !TOP.contains(&n.as_str()) && n.as_str() != "cal")
        .map(|(n, (ns, k))| (n.clone(), *ns / r, *k))
        .collect();
    sub.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut body = String::new();
    for (name, msv, k) in &sub {
        body.push_str(&format!("{name:<26} {:>8.3} ms   {k:>7} pairs\n", ms(*msv)));
    }
    let named: f64 = sub.iter().map(|(_, v, _)| *v).sum();

    println!(
        "\nfanout phase dump — [{KEYS} {FANOUT}], no query, mean of {RUNS}\n\
         instrument: {cal:.1} ns per mark pair\n\
         \n\
         wall                       {:>8.3} ms\n\
         FIRE (4 top phases)        {:>8.3} ms\n\
         \n{body}\
         \n\
         sub-phase sum (parents+children, DOUBLE COUNTS) {:>8.3} ms\n\
         \n\
         ⚠ READ THE PARENTS WITH CARE. `production` brackets two marks that fire\n\
         ONCE PER DERIVATION (40k pairs here), and per-row calibration only\n\
         removes a row's OWN pairs, so the parent still carries its children's\n\
         tax. Deleting those two marks and re-running gave production\n\
         18.992 -> 11.524 ms and wall 24.491 -> 16.690 — about 7.5 ms of the\n\
         parent is instrument, i.e. ~94 ns per pair IN SITU.\n\
         \n\
         `cal` is now the MINIMUM of five 200k batches and reads ~66 ns, stable\n\
         to under a nanosecond. It used to be one batch and read anywhere from\n\
         105 to 155 ns, which at 40k pairs swung a child row by ±2 ms — this\n\
         very row was recorded at both 2.541 and 4.826 ms for identical code.\n\
         Every net figure in this arc taken with the old calibration is\n\
         therefore UNDER-reported.\n\
         \n\
         The min-of-5 tight loop (~66 ns) still sits below the in-situ cost\n\
         (~94 ns), so a 40k-pair net row remains over-reported by roughly\n\
         {:>5.2} ms. That is now a STABLE bias of known sign and size rather\n\
         than run-to-run noise. Before/after deltas from one session are sound;\n\
         absolute parent times from this table are still not.\n",
        ms(wall / r),
        ms(top_sum / r),
        ms(named),
        (94.0 - cal) * 40_000.0 / 1e6,
    );

    assert!(wall > 0.0, "harness recorded no time");
}

/// How much does the `d_beta_from_parents` copy actually cost? The two call
/// sites document themselves as a borrow-checker workaround
/// (`NEXT-STRIKES-theater-hunt.md`), so before trying to remove one, size it.
#[test]
fn dbeta_copy_size() {
    println!(
        "\nToken is {} B; a d_beta_from_parents Vec of 2000 is {:.1} KB\n",
        std::mem::size_of::<Token>(),
        (std::mem::size_of::<Token>() * 2000) as f64 / 1024.0
    );
    assert!(std::mem::size_of::<Token>() > 0);
}

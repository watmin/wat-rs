//! What a fire round DOES to a session's memories — the transient shape.
//!
//! Nine tests over round-trip, type refusal, and the alpha/root/hash/production passes run
//! directly (`scratch_wm` + the `#[cfg(test)]` reference passes in `fire/mod.rs`), because a test
//! that needs to LOOK INSIDE beta cannot go through a path that freezes it away.


use super::*;

/// Eval a `src` expression in the cold-and-windy frozen world; panics on error.
fn ev(src: &str) -> Value {
    eval_in(&freeze_src(WORLD), src)
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
                s0      (:wat::core::match (:wat::rete::compile rules) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))\
                s1      (:wat::core::match (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\
                s2      (:wat::core::match (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))]\
              (:wat::core::match (:wat::rete::fire-rules s2) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None))))",
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
    let compiled = ev("(:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :weather)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))");

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
                s0    (:wat::core::match (:wat::rete::compile rules) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))\
                s1    (:wat::core::match (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\
                s2    (:wat::core::match (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))]\
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
/// 1-condition rule `(:user::Temp (?t <- :value) (:wat::rete::core::i64::> ?t 20))`.
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
               [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::rete::core::i64::> ?t 20)))\
                rule  (:wat::rete::Rule :name \"r\" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))\
                sess0 (:wat::core::match (:wat::rete::compile (:wat::core::PersistentVector rule)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))\
                sess1 (:wat::core::match (:wat::rete::insert sess0 (:user::Temp :value 25)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))]\
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
                sess0 (:wat::core::match (:wat::rete::compile (:wat::core::PersistentVector rule)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))\
                sess1 (:wat::core::match (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\
                sess2 (:wat::core::match (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))]\
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
                sess0 (:wat::core::match (:wat::rete::compile (:wat::core::PersistentVector rule)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))\
                sess1 (:wat::core::match (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\
                sess2 (:wat::core::match (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location \"Bergen\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))]\
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
                s0 (:wat::core::match (:wat::rete::compile (:wat::core::PersistentVector rule)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))\
                s1 (:wat::core::match (:wat::rete::insert s0 (:user::Temperature :celsius 15 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\
                s2 (:wat::core::match (:wat::rete::insert s1 (:user::Temperature :celsius 10 :location \"Bergen\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\
                s3 (:wat::core::match (:wat::rete::insert s2 (:user::WindSpeed :kph 45 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\
                s4 (:wat::core::match (:wat::rete::insert s3 (:user::WindSpeed :kph 50 :location \"Bergen\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))]\
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

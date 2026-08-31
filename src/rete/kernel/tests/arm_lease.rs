//! The arm intern/lease protocol — who holds a compiled network and when it is dropped.
//!
//! Reason to change: the leasing contract in `kernel/arm.rs` (`rete_arm_intern` /
//! `rete_arm_release`), not anything about what a rule computes.


use super::*;

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
    let src = "(:wat::rete::release-session (:wat::core::match (:wat::rete::compile (:dc::build-rules 2)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))))";
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
        (:wat::core::i64::+ acc\n\
          (:wat::core::length (:wat::rete::query (:wat::core::match (overlay (:sw::facts-for loc)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r) (:wat::kernel::assertion-failed! \"overlay: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __c __s) (:wat::kernel::assertion-failed! \"overlay: fixpoint round cap exceeded\" :wat::core::None :wat::core::None))) (:sw::q-match)))))\n\
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
    (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules base) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None))) (:sw::q-match)))))";
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
    let inside_src = "(:wat::core::match (:wat::rete::compile-all (:sw::the-rules) (:sw::the-queries)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))";
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

// ── PROBE (arc 278, Class B1) — THE UNWIND PATHS ────────────────────────────────────────────
//
// Row 3 above proves the lease is released when the body RETURNS. Nothing proved what happens
// when the body UNWINDS, and `wat/rete/syntax.wat:307` releases in a
// `(do (release-session base) result)` that sits AFTER the body — so an unwinding body skips it.
// The ceiling-breach path is reached from inside that body, which is what makes this the leak
// that fires exactly when memory pressure is highest.
//
// TWO unwind paths exist and a body can reach either, so each gets its OWN test rather than two
// arms of one: a first draft asserted both in sequence, arm 1 failed, and arm 2 was never
// reached — one drive cannot prove a two-arm gate.
//   - a wat runtime error (`DivisionByZero`) leaves `eval_in_frozen` as `Err`;
//   - `:wat::kernel::assertion-failed!` PANICS the host (`runtime.rs:15922` says so), which is
//     what every ceiling-outcome match arm in this file's own fixtures calls. An earlier draft
//     rode only this one and never reached its own assertion — the panic blew past it.
//
// Both measure a table-size DELTA, not an absolute: on an unwind `with-network` never hands the
// Session back, so there is no id to ask `rete_arm_leases` about. A leaked lease is a row that
// outlives the call; a released one leaves the table exactly as it was found.

/// The body raises a wat ERROR — `eval_in_frozen` returns `Err`, no panic involved.
#[test]
fn scoped_work_with_network_releases_the_lease_when_the_body_raises() {
    use super::rete_arm_table_len;
    let world = startup_from_source(SCOPED_WORK_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("scoped-work world should freeze");

    let before = rete_arm_table_len();
    let src = "\
(:wat::rete::with-network (:sw::the-rules) (:sw::the-queries)\n\
  (:wat::core::fn [base <- :wat::rete::Session] -> :wat::core::i64\n\
    (:wat::core::i64::/ 1 0)))";
    let ast = crate::parse_one!(src).expect("parse erroring with-network driver");
    let outcome = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        outcome.is_err(),
        "the body must actually raise; a body that returned would prove nothing"
    );
    let after = rete_arm_table_len();
    assert_eq!(
        after, before,
        "with-network must release the lease compile-all took when the body raises a wat ERROR; \
         table grew {before} -> {after}, so the InternedNetwork is pinned until thread end"
    );
}

/// The body PANICS the host — the shape every ceiling-outcome match arm in this file calls.
#[test]
fn scoped_work_with_network_releases_the_lease_when_the_body_panics() {
    use super::rete_arm_table_len;
    let world = startup_from_source(SCOPED_WORK_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("scoped-work world should freeze");

    let before = rete_arm_table_len();
    let src = "\
(:wat::rete::with-network (:sw::the-rules) (:sw::the-queries)\n\
  (:wat::core::fn [base <- :wat::rete::Session] -> :wat::core::i64\n\
    (:wat::kernel::assertion-failed! \"probe: the body panics\" :wat::core::None :wat::core::None)))";
    let ast = crate::parse_one!(src).expect("parse panicking with-network driver");
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
    }));
    assert!(
        caught.is_err(),
        "the body must actually panic the host; assertion-failed! is a panic, not an Err"
    );
    let after = rete_arm_table_len();
    assert_eq!(
        after, before,
        "with-network must release the lease compile-all took when the body PANICS; \
         table grew {before} -> {after}, so the InternedNetwork is pinned until thread end"
    );
}

/// Row 4 (arc 278, Class B1) — `with-overlay` INHERITS the cure, driven rather than argued.
///
/// `with-overlay` is built ON `with-network` (one release site, not two), so the call graph
/// says it must inherit. That is an argument, and an argument is not a measurement: the body
/// `with-overlay` hands to `with-network` is an INNER CLOSURE that captures `base` and mints
/// the overlay verb, and a capture is exactly the shape that could keep the guard's frame — or
/// a Session copy — alive past the scope the guard is supposed to close. Only driving it
/// answers that.
///
/// The wat-ERROR arm, not the panic arm. Its sibling above already proved the panic path
/// through `with-network`, and the thing under test here is the closure layer, which both arms
/// traverse identically. The panic arm for `with-overlay` is therefore reachable and NOT
/// driven — one probe is what the scorecard's row 4 (and the floor count it feeds) specifies.
///
/// Verified RED against the pre-fix form: with `(do (release-session base) result)` restored,
/// this fails with `table grew 0 -> 1`, exactly as its two siblings do.
#[test]
fn scoped_work_with_overlay_releases_the_lease_when_the_body_raises() {
    use super::rete_arm_table_len;
    let world = startup_from_source(SCOPED_WORK_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("scoped-work world should freeze");

    let before = rete_arm_table_len();
    let src = "\
(:wat::rete::with-overlay (:sw::the-rules) (:sw::the-queries)\n\
  (:wat::core::fn [overlay <- :wat::rete::Overlay] -> :wat::core::i64\n\
    (:wat::core::i64::/ 1 0)))";
    let ast = crate::parse_one!(src).expect("parse erroring with-overlay driver");
    let outcome = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        outcome.is_err(),
        "the body must actually raise; a body that returned would prove nothing"
    );
    let after = rete_arm_table_len();
    assert_eq!(
        after, before,
        "with-overlay must release the lease its inner with-network took when the body raises a \
         wat ERROR; table grew {before} -> {after}, so the InternedNetwork is pinned until \
         thread end"
    );
}

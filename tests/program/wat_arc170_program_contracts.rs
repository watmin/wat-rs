//! Arc 170 slice 2 — wat-level surface contracts.
//!
//! These tests prove the slice-2 surface end-to-end:
//!
//! 1. `:user::main` 4-arg signature freezes; 3-arg fires walker.
//! 2. `:user::main` returns ExitCode (u8); zero + non-zero values
//!    propagate through the substrate's exit-code pipeline.
//! 3. argv pure passthrough — wat program reads argv[i] matching
//!    what wat-cli received.
//! 4. `(:wat::kernel::spawn-process fn)` — the fn matching the
//!    `:user::process` contract spawns an OS process; typed-channel
//!    send/recv works end-to-end through EDN-over-pipes.
//! 5. spawn-process with inline-lambda fn (slice 1b's fn-form
//!    entry_form path).
//! 6. spawn-process with factory-fn (single-level capture via slice
//!    1b's prologue).
//! 7. (retired) spawn-process with impure Sender capture — the fixture
//!    minted its Sender via the now-annihilated depth-1 channel
//!    constructor; there is no way to construct the scenario without
//!    it, so the test is gone.
//!
//! 8/9. (retired) `*-program{,-ast}` callsite retirement nags —
//!    ANNIHILATED (arc 170 CULMINATION); the verbs had zero live callers
//!    and no runtime eval, so the check-time nag + its tests are gone.
//! 10. `(:wat::kernel::spawn-thread fn)` — UNCHANGED behavior;
//!     positive control verifying no regression.
//! 11. 3-arg `:user::main` — walker fires with the
//!     BareLegacyMainSignature diagnostic.

use wat::freeze::{
    expected_user_main_signature, invoke_user_main, startup_beside, startup_from_file,
    validate_user_main_signature,
};
use wat::runtime::Value;
use wat::types::TypeExpr;

// ─── helpers ───────────────────────────────────────────────────────────

fn freeze_ok(fixture: &str) -> wat::freeze::FrozenWorld {
    startup_from_file(fixture)
        .unwrap_or_else(|e| panic!("freeze should succeed for {fixture:?}; got: {e}"))
}

/// Load the primary fixture (canonical trivial main) via startup_beside.
fn freeze_trivial() -> wat::freeze::FrozenWorld {
    startup_beside(file!()).expect("trivial-main fixture must freeze")
}

// ─── T1. :user::main [] -> :wat::core::nil signature freezes; 3-arg fires walker ──

#[test]
fn t1_canonical_nil_main_freezes() {
    // Arc 170 slice 1e canonical shape: no params, nil return. Should freeze cleanly.
    let world = freeze_trivial();
    // Validator agrees — the canonical signature passes.
    validate_user_main_signature(&world).expect("[] -> nil :user::main validates");
    // expected_user_main_signature() exposes the canonical shape: 0 params, nil return.
    let (params, ret) = expected_user_main_signature();
    assert_eq!(params.len(), 0, "expected 0 params (argv is ambient), got {}", params.len());
    assert_eq!(
        ret,
        TypeExpr::Tuple(vec![]),
        "expected nil (Tuple([])) return"
    );
}

// ⊘ T1 DELETED 2026-08-16 — `t1_legacy_3arg_main_fires_walker`, an UNWRITTEN test
// (`unimplemented!()`) whose `#[ignore]` asserted a substrate defect that does not exist:
// "BareLegacyMainSignature walker no longer fires ... likely walker-disconnect".
//
// MEASURED — a 3-arg `:user::main` through `wat --check`:
//   #wat.macro/MainSignatureError ":user::main must take exactly 0 parameters; got 3.
//    ... The canonical signature is `[] -> :wat::core::nil`."
// The guard is live at src/check.rs:906-914 and fires on ANY non-canonical shape (both the
// params arm and the return-type arm were observed firing the same day). See
// docs/arc/2026/05/170-program-entry-points/NOTE-the-walker-disconnect-suspicion-was-false.md
//
// A suspicion was typed into an ignore-reason instead of a test, and sat on disk as the only
// account of a check that had been working the whole time. Nobody asked the binary.

// ─── T2. :user::main [] -> :wat::core::nil invokes cleanly ─────────────

#[test]
fn t2_canonical_main_returns_nil_value() {
    // nil IS the success exit code (arc 170 REALIZATIONS pass 10).
    // invoke_user_main on a canonical [] -> nil main returns nil.
    let world = freeze_trivial();
    let result = invoke_user_main(&world, Vec::new()).expect(":user::main should run");
    assert!(
        matches!(result, Value::Unit),
        "expected nil (Value::Unit); got {:?}", result
    );
}

#[test]
fn t2_canonical_main_with_let_body_returns_nil() {
    // A canonical main with a non-trivial body (let binding + discard)
    // still returns nil. Confirms the do-work-return-nil pattern runs.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t2_let.wat");
    let result = invoke_user_main(&world, Vec::new()).expect(":user::main should run");
    assert!(
        matches!(result, Value::Unit),
        "expected nil (Value::Unit); got {:?}", result
    );
}

// ─── T3. argv ambient reachable via (:wat::runtime::argv) ─────────────

#[test]
fn t3_argv_reachable_via_ambient() {
    // Arc 170 REALIZATIONS pass 7: argv is ambient (not a parameter).
    // A canonical main body can access (:wat::runtime::argv) — the
    // freeze should succeed (type-check validates the argv expression).
    // At runtime the ambient vector is whatever set_argv was called with
    // (empty if never set). We just confirm the program freezes and runs.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t3_argv.wat");
    let result = invoke_user_main(&world, Vec::new()).expect(":user::main runs");
    assert!(
        matches!(result, Value::Unit),
        "expected nil (Value::Unit); got {:?}", result
    );
}

// ─── T5. spawn-process(inline lambda) — slice 1b fn-form path ──────────

#[test]
fn t5_spawn_process_inline_lambda_round_trips() {
    // Arc 278 IPC de-prime — migrated off the non-prime `:wat::kernel::spawn-process`
    // onto the composed primes (`spawn-program' (process)` + `send'` + `recv'`). The
    // launcher constructs the child via an inline (:wat::core::forms (:wat::core::defn ...))
    // program, spawns it as a process peer, feeds its `readln` with `send' 21`, and returns
    // the doubled value that crossed back off the peer as a recv' RecvOutcome::Message.
    // The launcher now returns the recv'd i64 directly (== 42), so this test measures the
    // value that genuinely crossed the wire (NOT stdout-scraped). Mirrors t17's apply-and-
    // assert-i64 shape (SAME .rs file family). Fixture: t5_launch_lambda.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t5_launch_lambda.wat");
    let launcher = world.symbols().get(":my::launch").expect("launch defined");
    let result = wat::runtime::apply_function(
        launcher.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect(":my::launch runs (spawn-program' + send' + recv')");
    match &result {
        Value::i64(n) => assert_eq!(
            *n, 42,
            "expected 21*2=42 received as a recv' Message; got {}",
            n
        ),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── T6. spawn-process(factory-fn) — single-level capture ──────────────

#[test]
fn t6_spawn_process_factory_with_capture_round_trips() {
    // Arc 278 IPC de-prime — migrated off the non-prime `:wat::kernel::spawn-process`
    // onto the composed primes (`spawn-program' (process)` + `send'` + `recv'`). The
    // substrate-equivalent of closure-capture-across-fork is runtime AST construction:
    // the launcher splices the runtime `offset` value INTO the child program AST via
    // `:wat::core::quasiquote` + `:wat::core::unquote`, builds the
    // `(:wat::core::Vector :wat::WatAST main-form)` forms VALUE, and hands it to
    // `spawn-program' (process)` — the process clause accepts a forms value the same way
    // spawn-process did, so the quasiquote-factory shape is unchanged; only the DRIVER
    // flipped to the peer wire. The launcher feeds the child's `readln` with `send' 7`
    // and returns the `(n + offset)` value that crossed back as a recv' RecvOutcome::Message.
    // With offset=100 the recv'd i64 is 107 — the value that genuinely crossed the wire.
    // Fixture: t6_launch_factory.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t6_launch_factory.wat");
    let launcher = world.symbols().get(":my::launch").expect("launch defined");
    let result = wat::runtime::apply_function(
        launcher.clone(),
        vec![Value::i64(100)],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect(":my::launch runs (spawn-program' + send' + recv')");
    match &result {
        Value::i64(n) => assert_eq!(
            *n, 107,
            "expected 100+7=107 received as a recv' Message; got {}",
            n
        ),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ⊘ T11 DELETED 2026-08-16 — `t11_legacy_main_signature_fires_walker_diagnostic`, the sibling
// of the deleted T1 above and unwritten for the same reason. Same disproven suspicion, same
// NOTE. The rejection path is not uncovered: `validate_user_main_signature` /
// `expected_user_main_signature` are asserted in this file, and the return-type arm has live
// incidental coverage from `tests/wat_lang/probe_undefined_builtin_resolves_*.wat.bad`.

// ─── T17. hermetic run — happy path over the primed peer wire (arc 278 IPC de-prime)
//
// Migrated off the `:wat::test::run-hermetic` macro onto the composed primes:
// `spawn-program' (process)` spawns the peer, its `:user::main` computes 2+2 and
// `println`s it, and the parent drains that single value off the peer via `recv'` —
// the value arrives as a `RecvOutcome::Message`.
//
// Surface form exercised (fixture t17_run_hermetic.wat):
//   (:wat::kernel::spawn-program (:wat::spawn::process)
//     (:wat::core::forms (:user::main ... (:wat::kernel::println (:wat::core::i64::+ 2 2)))))
//   → (:wat::kernel::recv p) → RecvOutcome::Message m → m
//
// The function is defined at :my::test::two-plus-two; invoked with
// apply_function (zero args); the recv'd value must be i64 4.

#[test]
fn t17_run_hermetic_layer1_passing_assertion() {
    // Arc 278 IPC de-prime — migrated off :wat::test::run-hermetic onto the primed
    // peer wire. The child's :user::main computes 2+2 and println's it; the parent
    // spawn-program' (process) + recv' receives that single value off the peer as a
    // RecvOutcome::Message. A "passing" run now means the value genuinely crossed the
    // wire as 4 (NOT stdout-scraped). Fixture: t17_run_hermetic.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t17_run_hermetic.wat");
    let func = world
        .symbols()
        .get(":my::test::two-plus-two")
        .expect(":my::test::two-plus-two defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + recv' should succeed");
    // two-plus-two returns the recv'd i64 directly — the value that crossed the peer wire.
    match &result {
        wat::runtime::Value::i64(n) => assert_eq!(
            *n, 4,
            "expected 2+2=4 received as a recv' Message; got {}",
            n
        ),
        other => panic!("expected i64 result; got {:?}", other),
    }
}

#[test]
fn t17b_run_hermetic_layer1_failing_assertion_surfaces_failure() {
    // Complementary to T17 — the FAILURE path of the SAME primed wire.
    // Arc 278 IPC de-prime: migrated off :wat::test::run-hermetic onto the composed
    // primes (`spawn-program' (process)` + `recv'`). The child's assert-eq (1+0 != 2)
    // FAILS, so the child PANICS before it can send anything; the parent's recv' returns
    // RecvOutcome::Lost carrying a :wat::kernel::LociDiedError. An assert-eq failure is an
    // AssertionPayload panic, so the cause is LociDiedError::Panic whose failure field is
    // Some(Failure) carrying the structured assert-eq diagnostic. The death is SURFACED
    // (the fixture returns the raw LociDiedError), NEVER swallowed. Mirrors t18b's Lost/
    // Panic assertion shape (minus t18b's recv-all' Result unwrap — recv' hands back the
    // Lost cause directly). Fixture: t17b_run_hermetic_fail.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t17b_run_hermetic_fail.wat");
    let func = world
        .symbols()
        .get(":my::test::one-neq-two")
        .expect(":my::test::one-neq-two defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + recv' driver should not itself panic");

    // The child died on assert-eq before sending, so recv' surfaced a Lost cause;
    // the fixture returns that :wat::kernel::LociDiedError enum directly.
    let ev = match &result {
        wat::runtime::Value::Enum(ev) => ev.as_ref(),
        other => panic!("expected :wat::kernel::LociDiedError enum cause; got {:?}", other),
    };
    assert_eq!(
        ev.type_path, ":wat::kernel::LociDiedError",
        "expected the recv' Lost cause to be a LociDiedError; got type_path {}",
        ev.type_path
    );
    assert_eq!(
        ev.variant_name, "Panic",
        "child assert-eq failure must surface as LociDiedError::Panic; got variant {}",
        ev.variant_name
    );

    // Panic.fields = [message :String, failure :Option<Failure>]. An AssertionPayload panic
    // carries the structured Failure, so failure is Some(Failure).
    let failure_val = match &ev.fields[1] {
        wat::runtime::Value::Option(opt) => match opt.as_ref() {
            Some(v) => v,
            None => panic!(
                "expected LociDiedError::Panic.failure = Some(Failure) (assert-eq carries an AssertionPayload); got None"
            ),
        },
        other => panic!("expected Panic.failure :Option<Failure>; got {:?}", other),
    };

    // Failure struct must have the correct type_name.
    let failure_struct = match failure_val {
        wat::runtime::Value::Aggregate(s) if s.nature == wat::Nature::Record && s.class.as_ref() == "wat::kernel::Failure" => s,
        other => panic!("expected :wat::kernel::Failure struct; got {:?}", other),
    };
    // Arc 278 the string-wrap annihilation — Failure.fields[0] is the mandatory `error`
    // (Fault); its fields[0] is the message String. Must carry the structured assert-eq
    // diagnostic, read STRUCTURALLY off the surfaced Panic — no string re-parse.
    let message = match &failure_struct.fields[0] {
        wat::runtime::Value::Aggregate(err) => match &err.fields[0] {
            wat::runtime::Value::String(s) => s.to_string(),
            other => panic!("expected Failure.error.message :String; got {:?}", other),
        },
        other => panic!("expected Failure.error :Aggregate; got {:?}", other),
    };
    assert_eq!(
        message,
        "assert-eq failed",
        "t17b_msg: LociDiedError::Panic carries the assert-eq diagnostic golden"
    );
}

// ─── T18. BIDIRECTIONAL PRIME EXEMPLAR — spawn-program' + send' + recv' drain
//
// Arc 278 IPC de-prime. This consumer retired off `run-hermetic-with-io` (the
// non-prime Sender/from-pipe + Receiver/from-pipe over a 4-field Process) onto
// the composed primes: `spawn-program' (process)` spawns the peer, `send' 21`
// feeds the child's `readln`, and each child `println` crosses back to the
// parent as a `recv'` `RecvOutcome::Message` — drained until `RecvOutcome::Closed`
// (a genuine clean EOF) into the collected outputs.
//
// The child body (readln n → println n*2 → nil) is UNCHANGED from the old form;
// only the DRIVER flipped to the peer wire. `:my::test::echo-doubled` now returns
// the drained outputs directly as a `Vector<i64>` — this test asserts it is [42],
// i.e. the doubled value genuinely crossed the wire as a recv' Message (NOT
// stdout-scraped).
//
// Fixture: t18_echo_doubled.wat (drain is the shared primed helper
// `:wat::kernel::recv-all`, whose canonical call site this is).

#[test]
fn t18_run_hermetic_with_io_layer2_echo_doubled() {
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t18_echo_doubled.wat");
    let func = world
        .symbols()
        .get(":my::test::echo-doubled")
        .expect(":my::test::echo-doubled defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + send' + recv'-drain should succeed");

    // echo-doubled returns the drained outputs directly: Vector<i64> == [42].
    // Each element was received over the peer wire as a recv' RecvOutcome::Message.
    let outputs = match &result {
        wat::runtime::Value::Vec(v) => v.as_ref(),
        other => panic!("expected Vec outputs; got {:?}", other),
    };
    assert_eq!(
        outputs.len(),
        1,
        "expected exactly one output value drained off the peer; got {}",
        outputs.len()
    );
    match &outputs[0] {
        wat::runtime::Value::i64(n) => assert_eq!(
            *n, 42,
            "expected output 42 (21 * 2) received as a recv' Message; got {}",
            n
        ),
        other => panic!("expected i64 output; got {:?}", other),
    }
}

// ─── T18c. recv-all' HELPER GATE — drains ALL outputs, not just one
//
// Arc 278 IPC de-prime. t18 exercises `:wat::kernel::recv-all` on a single-output
// peer; this is the helper's own gate on the "ALL": a peer that emits THREE `println`
// values must be drained into the full collected Vector, in order, before the clean
// EOF (`RecvOutcome::Closed`) turns into `(Ok outputs)`. The child readln's 7 and
// println's 7, 14, 21; recv-all' returns `Ok [7 14 21]`.
//
// Fixture: t18c_recv_all_multi.wat.
#[test]
fn t18c_recv_all_drains_all_outputs() {
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t18c_recv_all_multi.wat");
    let func = world
        .symbols()
        .get(":my::test::echo-multi")
        .expect(":my::test::echo-multi defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + send' + recv-all' drain should succeed");

    // recv-all' returns Ok[outputs]; echo-multi unwraps to the Vector<i64> == [7 14 21].
    let outputs = match &result {
        wat::runtime::Value::Vec(v) => v.as_ref(),
        other => panic!("expected Vec outputs; got {:?}", other),
    };
    let got: Vec<i64> = outputs
        .iter()
        .map(|v| match v {
            wat::runtime::Value::i64(n) => *n,
            other => panic!("expected i64 output; got {:?}", other),
        })
        .collect();
    assert_eq!(
        got,
        vec![7, 14, 21],
        "recv-all' must drain ALL peer outputs in order into Ok[…]; got {:?}",
        got
    );
}

#[test]
fn t18b_run_hermetic_with_io_layer2_failing_assertion_surfaces_failure() {
    // Complementary to T18 — the FAILURE path of the SAME primed bidirectional wire.
    // Arc 278 IPC de-prime: this consumer retired off `run-hermetic-with-io` onto the
    // composed primes (`spawn-program' (process)` + `send'` + `:wat::kernel::recv-all`).
    // The child recvs 2 (fed by `send' p 2`), then `assert-eq n 3` fails (2 != 3), so the
    // child PANICS before its `println` — the peer DIES mid-exchange.
    //
    // recv-all' surfaces that death honestly: it returns `(Err cause)` where `cause` is a
    // `:wat::kernel::LociDiedError`. An assert-eq failure is an AssertionPayload panic, so
    // `cause` is the `LociDiedError::Panic` variant, whose `failure` field is `Some(Failure)`
    // carrying the structured assert-eq diagnostic (same shape the arc-278 failure exemplar
    // proves: tests/comms/probe_arc278_failure_carries_structured_error.wat). The death is
    // SURFACED in the Err, NEVER swallowed — that is the whole point of recv-all'.
    //
    // (The old form drained a `RunResultIO` and inspected its `.failure` slot; the peer's
    // own Lost cause IS the failure now, read straight off recv-all''s Err.)
    // Fixture: t18b_recv_assert_fail.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t18b_recv_assert_fail.wat");
    let func = world
        .symbols()
        .get(":my::test::recv-assert-fail")
        .expect(":my::test::recv-assert-fail defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + send' + recv-all' driver should not itself panic");

    // recv-all' returns Result<Vector<i64>, LociDiedError>. The child died mid-exchange,
    // so this MUST be Err[cause] — no outputs were drained (the child panicked before println).
    let cause = match &result {
        wat::runtime::Value::Result(r) => match &**r {
            Ok(ok) => panic!(
                "expected Err[LociDiedError] (child died on assert-eq mid-exchange); got Ok({:?})",
                ok
            ),
            Err(cause) => cause,
        },
        other => panic!("expected Value::Result from recv-all'; got {:?}", other),
    };

    // The Err cause is a :wat::kernel::LociDiedError enum. An assert-eq failure panics with
    // an AssertionPayload, so the peer's death is the Panic variant.
    let ev = match cause {
        wat::runtime::Value::Enum(ev) => ev.as_ref(),
        other => panic!("expected :wat::kernel::LociDiedError enum cause; got {:?}", other),
    };
    assert_eq!(
        ev.type_path, ":wat::kernel::LociDiedError",
        "expected the Err cause to be a LociDiedError; got type_path {}",
        ev.type_path
    );
    assert_eq!(
        ev.variant_name, "Panic",
        "child assert-eq failure must surface as LociDiedError::Panic; got variant {}",
        ev.variant_name
    );

    // Panic.fields = [message :String, failure :Option<Failure>]. An AssertionPayload panic
    // carries the structured Failure, so failure is Some(Failure).
    let failure_val = match &ev.fields[1] {
        wat::runtime::Value::Option(opt) => match opt.as_ref() {
            Some(v) => v,
            None => panic!(
                "expected LociDiedError::Panic.failure = Some(Failure) (assert-eq carries an AssertionPayload); got None"
            ),
        },
        other => panic!("expected Panic.failure :Option<Failure>; got {:?}", other),
    };

    // Failure struct must have the correct type_name.
    let failure_struct = match failure_val {
        wat::runtime::Value::Aggregate(s) if s.nature == wat::Nature::Record && s.class.as_ref() == "wat::kernel::Failure" => s,
        other => panic!("expected :wat::kernel::Failure struct; got {:?}", other),
    };

    // Failure.message (arc 278 — fields[0] is `error` (Fault); its fields[0] is the message
    // String) must carry the structured assert-eq diagnostic, read STRUCTURALLY off the
    // surfaced Panic — no string re-parse. Phase C′ emit_panics_to_stderr is active for
    // spawn_process; the child's assertion diagnostic rides the Lost cause intact.
    let message = match &failure_struct.fields[0] {
        wat::runtime::Value::Aggregate(err) => match &err.fields[0] {
            wat::runtime::Value::String(s) => s.to_string(),
            other => panic!("expected Failure.error.message :String; got {:?}", other),
        },
        other => panic!("expected Failure.error :Aggregate; got {:?}", other),
    };
    assert_eq!(
        message,
        "assert-eq failed",
        "t18b_msg: LociDiedError::Panic carries the assert-eq diagnostic golden"
    );
}

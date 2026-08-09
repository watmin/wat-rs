//! Arc 278 BRIEF-arming-is-internal-only — THE ACCEPTANCE GATE for the "arming an alarm is
//! internal-ops-only" checker rule, with its non-vacuity control.
//!
//! WHY: `wat/service.wat:56` — `Alarm<O>`'s `op` slot is typed `:O`, the service's superset
//! `Op` enum — wide enough to hold ANY op, public or internal. Nothing in the type stopped a
//! handler from arming a PUBLIC (client-facing) op. Proven by run
//! (`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-call-context.md` § "RUN 2026-08-09"):
//! `--check` accepted `:op (:probe::tick2::Op::Bump (…Request…))` clean, and running it armed
//! the timer, which fired with NO client in the `idx` slot, mutated durable state via the
//! handler's ordinary `Outcome::Reply`, and the reply vanished — a silent discard reachable by
//! writing one ordinary constructor.
//!
//! THE FIX (`alarm_op_internal_check` + `literal_enum_variant_ctor`, `src/check.rs`): at an
//! `:wat::service::Alarm` construction site, when the `op` field's value is a literal
//! `<service>::Op` variant ctor, the variant must be INTERNAL — its name begins with `-`
//! (`wat/service.wat:876-892`, the dash-preserved variant naming this rule reads as ground
//! truth). Hooked at the site that actually decides both the kwargs sugar (the exemplar form)
//! AND the positional prime-ctor form `(:wat::service::Alarm' …)` — both delegate to the SAME
//! generic call-inference path in `infer_list` (verified live: `infer_kwargs_construct_check`
//! builds a synthetic `(:T' …)` call and infers it, landing in the identical spot a
//! hand-written `Alarm'` call would) — plus the raw-builtin door
//! `(:wat::core::aggregate-new :wat::service::Alarm …)` in `infer_aggregate_new_check`, which a
//! hand-written form (not just `:T'`'s own generated body) can also reach.
//!
//! STOP-2, stated: the DYNAMIC case (an `op` value that is a variable or a call result, not a
//! literal ctor) is out of scope — `literal_enum_variant_ctor` returns `None` and the rule is
//! silent. A handler only ever receives `req`, never an `Op`, so a literal ctor is
//! realistically the only way to obtain one.

use wat::freeze::{startup_from_file, StartupError};

const BAD: &str = "tests/services/probe_arc278_arming_is_internal_only.wat.bad";
const CONTROL: &str = "tests/services/probe_arc278_arming_is_internal_only_control.wat";

/// THE WALL BITES — arming a PUBLIC op (`bump`) via `Alarm` is refused AT LOAD.
#[test]
fn public_op_armed_via_alarm_is_refused_at_load() {
    let err = startup_from_file(BAD).expect_err(
        "a defservice handler arming a PUBLIC op (`bump`, no leading dash — client-facing) via \
         `:wat::service::Alarm`'s explicit `<service>::Op::Bump` ctor must be refused at load — \
         an alarm has no client, so only an INTERNAL (`-`-prefixed) op may be armed",
    );
    let StartupError::Check(ce) = &err else {
        panic!("expected StartupError::Check(AlarmArmsPublicOp), got {err:?}");
    };
    let rendered = format!("{ce:?}");
    assert!(
        // rune:lint(loose-assert) — the rendering embeds a machine-specific span (absolute
        // source path + live line number), so a golden cannot pin it; a targeted PRESENCE
        // check for the error kind's own EDN tag is the precise claim available here.
        rendered.contains("AlarmArmsPublicOp"),
        "expected the arming-is-internal-only violation, got: {rendered}"
    );
    // Name the SUBJECT, not just the kind: a wall that fires on the wrong op would pass the
    // check above while proving nothing about which op was refused.
    assert!(
        // rune:lint(loose-assert) — same reason as above.
        rendered.contains("Bump"),
        "the diagnostic must name the offending PUBLIC variant, got: {rendered}"
    );
}

/// THE NON-VACUITY CONTROL — swap the armed op for the INTERNAL `-tick` and the very same
/// shape of file loads.
///
/// Without this, the RED above would prove "something in that fixture is bad", not "exactly
/// arming a public op is refused" (R59 `NISI FRANGAS, NIHIL PROBAS`). It also guards the
/// opposite failure: a change that made the rule deny too much (e.g. refusing internal ops
/// too) would turn this red — which is exactly what would happen to
/// `tests/services/probe_arc278_self_scheduling.wat` in production, so this is the narrow,
/// fast copy of that non-vacuity property.
#[test]
fn control_arming_the_internal_op_loads() {
    startup_from_file(CONTROL).unwrap_or_else(|e| {
        panic!(
            "the control MUST load — if it does not, the gate's RED no longer isolates the \
             public-op arm and BOTH tests are lying. Fix this first. Got: {e:?}"
        )
    });
}

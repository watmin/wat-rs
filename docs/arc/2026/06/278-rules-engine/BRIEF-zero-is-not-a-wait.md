# BRIEF — zero is not a wait

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`113bc7e97`. Read `DESIGN-zero-is-not-a-wait.md` first — it carries the evidence and the rejected
alternatives, so you never re-derive them.

## THE WORK

Mint a new time type, `:wat::time::NonZeroDuration`, stored as `NonZeroU64`, so that a zero-length
wait has **no form in the language**. The seven unit constructors (`Nanosecond` … `Day`) return it
instead of `Duration` and reject `n <= 0`. `:wat::kernel::after` and the `Alarm` record accept only
it. `:wat::time::Duration` stays exactly as it is — it is what `:wat::time::-` produces when you
measure elapsed time, and a measurement of zero is legitimate. The readout and arithmetic families
accept either type. **Call sites do not change**: `(:wat::time::Millisecond 50)` is spelled
identically before and after; only its inferred type moves. Names are ruled and recorded in the
DESIGN — build them as written.

## ROOMS — read in this order

1. **`src/value/value.rs:284-300`** — the `Duration(i64)` variant. Add `NonZeroDuration(NonZeroU64)`
   beside it. Its doc comment at `:291-294` promises "a future stone makes this type-enforced via
   `u64`" — that promise is about `Duration` and is **NOT** what you are building (see STOP-4);
   correct the comment to point at S17.
2. **`src/intrinsic/time.rs:318-372`** — the arc-097 header and `unit_constructor`. `:351` is the
   exemplar: `if n < 0 { panic!("… direction lives in the operation, not the sign of the duration") }`.
   Yours is that sentence in the other axis. Return `NonZeroDuration`.
3. **`src/intrinsic/time.rs:744-790`** — `:wat::time::-`. **Leave its `Duration` return alone.**
   `:772` guards `ns < 0` and correctly admits `0`.
4. **`src/intrinsic/time.rs:590-1050`** — the readout family (`milliseconds`, `nanoseconds`, …) and
   the `-ago`/`-from-now` builders. These must accept **either** type.
5. **`src/check.rs:20785-20815`** — where the seven constructors register `ret: duration_ty()`.
   Register the new path and switch those seven.
6. **`src/check.rs:14137-14146`** — `infer_polymorphic_time_arith`. **This is your exemplar for
   accepting two nominal types**: it already matches `TypeExpr::Path(p) if p == ":wat::time::Duration"`.
   Add the `NonZeroDuration` arms here rather than inventing a coercion.
7. **`src/rete/purity.rs:2330-2348`** — the `:wat::time::*` purity rows. The seven constructors are
   listed; keep their classification unchanged.
8. **`src/intrinsic/kernel/resource.rs:450-460`** — `@arg duration … **non-negative** delay before
   the timer fires`. That word certifies the one value that does not fire. It becomes **positive**.
9. **`src/runtime.rs:26455-26475`** — `eval_kernel_after`'s `nanos < 0` guard. Once the argument is
   `NonZeroDuration`, this is structurally unreachable for wat callers; keep or convert it to the
   type extraction, and say in the SCORE which you did and why.
10. **`wat/service.wat:67`** — `(defrecord :wat::service::Alarm :- [O] [after <- :wat::time::Duration
    op <- :O])`. **Change the TYPE only. The field stays named `after`** (the rename is Stone C).
    This is stdlib, frozen into the binary at build time — read `wat/fix.wat`'s BOOTSTRAP /
    STASH-DANCE header before you touch it.
11. **`src/comms/process.rs:1330-1395`** — read only, so you understand what you are walling off:
    `it_value = duration` with `{0,0}` is POSIX **disarm**.

## SKETCH

```rust
// value.rs — beside Duration(i64)
NonZeroDuration(std::num::NonZeroU64),

// time.rs — unit_constructor
let n = require_i64(op, eval(n, env, sym)?, list_span)?;
if n <= 0 {
    panic!("({} {}): a wait must be positive; whether you wait lives in the \
            operation, not the magnitude of the duration — zero is a legal \
            MEASUREMENT (:wat::time::- on equal Instants) and an illegal \
            COMMITMENT: it disarms the timer", op, n);
}
let nanos = n.checked_mul(unit_nanos).unwrap_or_else(|| { /* unchanged */ });
Ok(Value::NonZeroDuration(NonZeroU64::new(nanos as u64).expect("n > 0 checked above")))
```

## BLAST RADIUS

`src/value/value.rs`, `src/intrinsic/time.rs`, `src/check.rs`, `src/rete/purity.rs`,
`src/intrinsic/kernel/resource.rs`, `src/runtime.rs`, and **one line** of `wat/service.wat`.

**No `.wat` corpus migration. No codemod. No test-file edits for spelling.** If you find yourself
editing `.wat` call sites to make things compile, that is STOP-1.

## STOP TRIGGERS

1. **You need to edit `.wat` call sites to compile.** The design's central claim is that call sites
   are spelled identically. If that is false, the contract decision is wrong — STOP and report which
   sites and why.
2. **A readout or arithmetic verb cannot accept both types** through the `check.rs:14137` dispatch.
   STOP and report the exact verb and the checker error — do not add a conversion at call sites.
3. **You cannot make the wall fire in a negative control.** A wall that never fires is a deleted
   wall. STOP and report.
4. **You are about to change `Duration`'s storage to `u64`.** That is S17, not this stone. STOP.
5. **The floor's existing red changes.** `probe_async_publish::refused_subscriber_is_retried_not_dropped`
   is expected to stay red — it is Stone D's. If it goes green, or a *different* test goes red, STOP
   and report the whole arm verbatim.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it. Do not use `run_in_background`, do
not set a Monitor, do not poll and stop — three riders on this arc died exactly that way.

The floor is **`scripts/floor.sh`** (release). **Read the Summary line, never a piped exit code.** On
any red you did not intend: **do NOT re-run.** Copy the failing test's entire stdout+stderr block
**verbatim** — never a `| head` window — name the exact assertion, and report.

Leave your work uncommitted. Prior comparable result to copy for shape:
`SCORE-the-sane-circuit.md` — note especially how its row 2 proved a term load-bearing by *removing*
it and demanding a failure.

## REPORT

- each EXPECTATIONS row's real result, from your own run
- **the negative control**, both directions, with the refusal message verbatim
- what `wat/telemetry/span.wat:131`'s computed flush interval does when the field is 0
- whether `runtime.rs:26462`'s guard survived, and why
- the floor Summary line verbatim
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Every number in
  this arc's briefs has been wrong at least once.

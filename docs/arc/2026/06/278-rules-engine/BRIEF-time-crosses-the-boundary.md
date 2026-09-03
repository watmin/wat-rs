# BRIEF — time crosses the boundary

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`.
Read `DESIGN-time-crosses-the-boundary.md` first.

## THE WORK

Teach the typed EDN coercion the three time types, so a `:wat::time::*` value can be a field of a
`defsurface` message. Today none can — not even `Instant`, whose bytes survive the wire intact and
are still refused. Three arms in one `match`, beside an exemplar that is already there.

## ROOMS — read in this order

1. **`wat-scripts/scratch-pad/probe-nonzeroduration-crosses-the-wire.wat`** — run it first, before
   reading any Rust. It is the acceptance criterion and it currently fails 3 of 4 cells. Its fourth
   cell (`instant-EXEMPLAR`) is the finding: encoding was never the blocker.
2. **`src/edn/render.rs:2297`** — the exemplar arm, verbatim the shape to copy:
   ```rust
   ":wat::core::i64" => match edn {
       Edn::Integer(n) => Ok(Value::i64(*n)),
       ...
   }
   ```
3. **`src/edn/render.rs:2266`** — `edn_to_typed_value_inner`, where your three arms go.
4. **`src/edn/render.rs:~2238`** — the doc table mapping target → EDN shape → `Value`. Three rows.
5. **`src/edn/render.rs:4158-4160`** — the encode. **Read it; do not change it.**
   `Value::Instant → OwnedValue::Inst`, `Value::Duration → OwnedValue::Integer`. The `Integer`
   encoding is correct once the coercion can read it.
6. **`src/edn/render.rs:2138`** — `Edn::Inst(t) => Ok(Value::Instant(*t))` in the *untyped* decoder.
   Your `Instant` arm is this line, in the typed path, where it is missing.
7. **`src/edn/render.rs:2175-2200`** — `edn_shape_name`, which produces the `got` string. You should
   not need to edit it; if you do, say why.
8. **`src/intrinsic/time.rs:342`** — `unit_constructor`'s `n <= 0` refusal. **Your `NonZeroDuration`
   arm enforces the same rule at the boundary**, and its message should echo this one's axis.

## SKETCH

```rust
":wat::time::Instant" => match edn {
    Edn::Inst(t) => Ok(Value::Instant(*t)),
    other => Err(EdnCoerceError { expected: ..., got: edn_shape_name(other).into(), path: String::new() }),
},
":wat::time::Duration" => match edn {
    Edn::Integer(n) if *n >= 0 => Ok(Value::Duration(*n)),
    // a negative duration has no form; see time.rs:351
    other => Err(...),
},
":wat::time::NonZeroDuration" => match edn {
    Edn::Integer(n) if *n > 0 => Ok(Value::NonZeroDuration(
        std::num::NonZeroU64::new(*n as u64).expect("n > 0 checked above"))),
    // ★ zero refused HERE, as a typed error, so a peer sending UpTo(0) gets
    //   RequestMalformed and the service SURVIVES — not the LociDiedError/Panic
    //   that Stone A's rung-2 constructor raise would produce.
    other => Err(...),
},
```

## BLAST RADIUS

**`src/edn/render.rs` only.** No `.wat`. No codemod. No new `Value` variants. No encode changes.

## STOP TRIGGERS

1. **You are about to change `render.rs:4158-4160`'s encode.** `Instant` proves encoding is not the
   blocker. STOP and report what forced it.
2. **You are about to add a `Value` variant or a tagged EDN form.** Out of scope, and it breaks every
   i64-on-the-wire caller. STOP.
3. **The zero refusal kills the service** rather than returning a typed error. That is the panic
   wearing a different hat and it fails the stone's whole point. STOP and report.
4. **A `.wat` file needs editing to make the probe pass.** The probe is already correct. STOP.
5. **A floor test that passed at `5207/5207` goes red.** Capture whole, name the arm, do not re-run.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it. No `run_in_background`, no Monitor,
no poll-and-stop — three riders on this arc died that way.

Floor is `scripts/floor.sh` (release). **Read the Summary line, never a piped exit code.** On any red
you did not intend: **do NOT re-run.** Copy the whole stdout+stderr block verbatim, name the exact
assertion, report.

Leave your work uncommitted. Prior comparable result for shape: `SCORE-zero-is-not-a-wait.md`.

## REPORT

- the probe's four cells, verbatim, before and after
- **the zero-at-the-boundary proof: the `RequestMalformed` variant AND a subsequent successful call
  on the same connection**, showing the service survived
- the negative control (a `Duration` target handed a `String`)
- the floor Summary line verbatim
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Every number in
  this arc's briefs has been wrong at least once, and the orchestrator's last census was taken with
  two blind spots.

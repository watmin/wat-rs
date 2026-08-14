# BRIEF — STONE 255.1c-guard: hoist the registry check, and measure what it costs

Read `DESIGN-STONE-255.1c-guard-hoist.md` first — it carries the differential that proves the defect
and the reason the baseline must be taken before the change.

## THE WORK, in one paragraph

Build a timing harness for `dispatch_keyword_head_value`, **record the baseline**, then move the
registry check from a guard arm partway down the match to a plain `if let` **before the match is
entered**, then **record the same number again**. Report both. The code change is about five lines;
the deliverable is the pair of numbers and an honest read of the delta.

## ROOMS — read in this order

1. **`src/runtime.rs:4652`** — `dispatch_keyword_head_value`, the fn you are changing. Find where its
   `match head {` begins; that is where your `if let` goes, immediately above it.
2. **`src/runtime.rs:5607–5611`** — the guard arm as it stands today:
   ```rust
   h if crate::intrinsic::registry().lookup(h).is_some() => {
       let handler = crate::intrinsic::registry().lookup(h).unwrap();
       handler(args, list_span, env, sym)
   }
   ```
   Note it looks the name up **twice**. The hoisted form does it once via `if let Some(handler)`.
   **Delete this arm** when you hoist — leaving both is two consult points.
3. **`src/intrinsic/mod.rs:350–376`** — `IntrinsicRegistry`, `lookup`, `lookup_entry`. Read it so
   you know exactly what the hoisted call costs: a `HashMap<&'static str, _>` get.
4. **`src/runtime.rs:5036`** — `":wat::core::i64::+"`'s arm. This is the hot path the bench must
   drive, and it currently sits *above* the guard, which is why it has never paid a lookup.

## THE ORDER IS THE METHOD — do not deviate from it

1. Write the harness. Run it. **Write the baseline number down in your report.**
2. *Then* hoist.
3. Run the same harness, unchanged, on the same machine, in the same session.
4. Report both numbers and the delta.

A number captured after the change cannot be compared to anything. This ordering is the stone.

## THE HARNESS

An in-crate `#[test] #[ignore = "manual perf harness — run explicitly; see 255.1c-guard"]` in
`runtime.rs`'s existing test module. It should drive `dispatch_keyword_head_value` on
`":wat::core::i64::+"` with two constant i64 args in a tight loop (≥1_000_000 iterations), time the
loop, and print ns/op with `eprintln!`.

Take the **best of at least 5 runs**, not one run and not a mean — you are measuring a floor, and a
single sample on a shared machine measures noise as much as code. Report the spread you saw, not
only the figure you kept.

Make sure the loop's result is consumed (e.g. accumulate into a value you assert on at the end) so
the optimiser cannot delete the work. **A bench measuring nothing reports a beautiful number** —
state in your report how you ensured the work survived.

## IMPLEMENTATION SKETCH

```rust
fn dispatch_keyword_head_value(head: &str, args: &[WatAST], /* … */) -> Result<Value, EvalBreak> {
    // 255.1c-guard — the registry is consulted BEFORE the literal table.
    // Registered wins, always: a literal arm can no longer shadow a registration
    // by sitting higher in the match. (Proven shadowable at HEAD: see the DESIGN.)
    if let Some(handler) = crate::intrinsic::registry().lookup(head) {
        return handler(args, list_span, env, sym);
    }
    match head {
        // … the table, unchanged, MINUS the old guard arm at :5608 …
    }
}
```

## BLAST RADIUS

`src/runtime.rs` only — the hoisted `if let`, the deleted guard arm, and the `#[ignore]`d harness in
its test module. **No other file. No family carved. No arm deleted other than the guard itself. No
change to `IntrinsicRegistry`, the resolver, the checker, or any `.wat`.**

## STOP TRIGGERS — each means ship nothing, report the gap

**STOP-1 — the hoist changes any observable.** It must be inert: measured, no registered name has a
literal arm, so nothing should route differently. If any test's *behaviour* changes (as opposed to a
timing test), stop and report which — it means a registered name and a literal arm both exist and the
shadowing is live in the corpus, which is a finding bigger than this stone.

**STOP-2 — the harness cannot isolate dispatch.** If the loop's cost is dominated by setup (building
`WatAST` args, environment lookups) rather than the dispatch call, the number answers nothing.
Report what fraction you could not exclude rather than shipping a figure that measures the harness.

**STOP-3 — `dispatch_keyword_head_value` cannot be called from the test module** without widening
its visibility. Do NOT make it `pub` to satisfy the bench. Report the obstacle.

**STOP-4 — the floor moves.** Any test that goes red is a finding, captured whole and verbatim per
the red protocol. Do not re-run to see whether it clears.

## THE GATE

1. `cargo build --release` — exit 0.
2. `cargo clippy --release --all-targets` — zero warnings, no new `#[allow]`.
3. **The two numbers**, baseline and post-hoist, each best-of-≥5, with the spread stated, and the
   sentence explaining how you kept the optimiser from deleting the loop body.
4. **A shadowing proof**: after the hoist, add a literal arm above the match for an already-registered
   name (e.g. `":wat::core::Bytes::to-hex"`), build, run
   `(:wat::core::Bytes::to-hex (:wat::core::Vector :wat::core::u8 (:wat::core::u8 255)))` and show it
   still returns `"ff"` — i.e. the registration now wins. **Then revert that arm** and confirm the
   tree is clean. This is the stone's own `NISI FRANGAS` proof: without it, "registered wins" is a
   claim. (At HEAD the same experiment returns `"SHADOWED"`.)
5. `git diff --stat` — `src/runtime.rs` and nothing else.
6. Floor: **not yours.** The orchestrator runs `scripts/floor.sh` centrally and weighs by its own re-run.

Run everything **FOREGROUND** and block on it. **You are a rider, not the orchestrator: ending your
turn ENDS you** — nothing wakes you, no notification is coming. Your turn ends when the numbers are
in your hands, not when a command is launched.

⚠ **Rebuild before you measure.** A restored source with a stale binary reports the *previous*
build's behaviour — that exact mistake was made against this very question earlier today and produced
a false reading that survived one round. `cargo build --release` between every source change and every
run.

## A PRIOR RESULT TO COPY FOR SHAPE

`25c1f452` (255.1c-time, home #2) — for the reporting register: every scorecard row scored against
what was actually observed, the sketch overridden from the bodies and the disagreement stated, and
the two brief errors named as the brief's rather than smoothed over.

# STONE A-ii — the f64 home: 19 ops move to `:wat::f64::*`

DRAWN + BRIEFED 2026-08-25 against `b2d10158f`.
DESIGN: `DESIGN-STONE-the-numerics-get-their-homes.md`.
PRIOR ART, and it is close enough to copy: **Stone A-i** (`b2d10158f`) did i64's 17. Read
`src/intrinsic/i64.rs` end to end before you start — this stone is its mirror.

## The one thing to hold

**BOTH SPELLINGS LIVE WHEN YOU ARE DONE.** `:wat::core::f64::+` must still work exactly as today,
and `:wat::f64::+` must work too. Nothing in the corpus moves here; 331 call sites still spell the
old name and they migrate in Stone B. **`:wat::core::+`, the polymorphic generic, is not touched.**

If you find yourself deleting an old dispatch arm, stop — that is Stone C's work and it breaks the floor.

## The 19 ops, by arity shape

```
binary  (10)   + - * /   < <= > >= = not=
binary  (2)    max min
unary   (4)    abs round to-i64 to-string
ternary (1)    clamp
VARIADIC(2)    max-of min-of
```

## Your role

cwd `/home/john/work/holon/wat-rs`; run `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND, blocking. **You may not spawn sub-agents.** Do not commit, push, stash, revert, or
`git checkout`; `git stash@{0}` must never be touched.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check <f>`, `./target/release/wat <f>`, and single named tests.
**Not** the floor, **not** clippy — the orchestrator measures those centrally.

## The rooms — RE-DERIVED at `b2d10158f` (A-i moved `runtime.rs` +33; the old numbers are stale)

```
src/intrinsic/i64.rs                  THE MIRROR. Copy its structure, its preambles, its delegation.
src/intrinsic/mod.rs                  `mod f64;` MUST be added or the submissions never link —
                                      the one step whose omission fails SILENTLY.
src/runtime.rs  eval_f64_arith        10440       eval_f64_unary        10755
                eval_f64_to_string    10632       eval_f64_clamp        10798
                eval_f64_to_i64       10653       eval_f64_compare      11721
                eval_f64_round        10687       eval_f64_reduce       15893
dispatch arms   ~5993 (max-of) · ~6001 (min-of) · clamp just above them
```

## The three shapes A-i did not have to solve

1. **VARIADIC — `max-of` / `min-of`.** `#[wat_intrinsic]` supports it: a handler taking
   `args: &[WatAST]` registers as `Arity::Variadic` (`src/intrinsic/mod.rs:140-148`).
   **Two live examples to copy:** `src/intrinsic/list.rs:32-41` (`:wat::core::List`) and
   `src/intrinsic/string.rs:577-581` (`:wat::string::concat`). Both take the slice directly.
2. **TERNARY — `clamp`.** `eval_f64_clamp` already takes an `args` slice. Either three fixed params
   (`Exact(3)`) or the slice; pick whichever keeps the delegation honest and say which and why.
3. **`max` / `min` vs `max-of` / `min-of`.** Two pairs, similar names, different arities. Do not
   collapse them and do not swap them.

## How to make both spellings live — the A-i pattern, verbatim

A-i's rule and it is the load-bearing one: **do not copy the arithmetic — share it.** Where a
closure body sits inline in an old dispatch arm, lift it to a named `pub(crate)` fn, have the old
arm call it, and have the new handler call the same fn. A-i lifted seven (`i64_add_op` … ).

**Pass the op name as a PARAMETER, never a captured constant.** That is what makes an error name the
spelling the caller actually used. A-i proved it both ways; f64's error kinds must do the same.

Whatever f64's division/NaN/infinity contract is today, it must be **byte-identical** under both
spellings. Verify it, do not assume it — and note that f64 has failure modes i64 does not.

## ★ A PREDICTION THIS STONE TESTS — report the result either way

A-i's cascade had **two** failure classes. One was fixed at the root:

- **The purity gate WILL fire again**, `UNREVIEWED` +19 over `ledger 233`. Expected. It is
  default-deny and these verbs are new. **Fix it with a RULING beside the old `:wat::core::f64::*`
  twins — NOT a ledger entry.** The ledger is unreviewed *debt*; growing it to balance a count is
  the opposite of paying it. `KNOWN_UNREVIEWED` must still read 233 when you finish.
- **The five diagnostics goldens must NOT fire.** A-i pinned `src/runtime.rs:25614` into them and
  they broke on a +33 line shift — the fifth such occurrence. `assert_edn_eq!` now zeroes `:line`
  on any `wat.core/Span` whose `:file` ends `.rs`. **This stone moves `runtime.rs` again, so it is
  the first real test of that fix.**

  **If the goldens break anyway, that is a FINDING and I want it before anything else** — it means
  the normalizer is incomplete, and a class I recorded as pulled-out-by-the-root is still growing.
  Report it with the verbatim diff; do not paper over it with `UPDATE_EDN=1`.

## STOP triggers — each rejects

1. **STOP-1 — an op cannot be registered without duplicating an implementation.** Name it; ship the
   rest. Two copies of a float contract that must agree forever is a defect, not a migration.
2. **STOP-2 — an old spelling stops working.** Stone C's job, not this one.
3. **STOP-3 — the arity sniff cannot express one of the 19.** Name it and what the attribute could
   not do. **Do not hand-roll a shim that bypasses the registry** — a name registered by another
   mechanism is invisible to Stone C's membership test, which is the point of the arc.
4. **STOP-4 — a room's line number does not hold.** Written against `b2d10158f`. (A-i's brief had
   this too and every number held; A-i then moved them all, which is why these are re-derived.)

## Acceptance — every row derives its bar

```bash
# 1. nineteen registered. BAR: 19.
grep -c '#\[wat_intrinsic(":wat::f64::' src/intrinsic/f64.rs

# 2. the module is linked — the silent-failure step. BAR: non-empty.
grep -n 'mod f64' src/intrinsic/mod.rs

# 3. BOTH spellings run. A probe under wat-scripts/scratch-pad/ asserting a result for each of the
#    19 under BOTH spellings (38 assertions) — loader-gated, so it stays a live check.
#    Include max-of/min-of at MORE THAN TWO arguments, or the variadic row proves nothing.
./target/release/wat --check wat-scripts/scratch-pad/<probe>.wat; echo "EXIT=$?"   # 0
./target/release/wat        wat-scripts/scratch-pad/<probe>.wat; echo "EXIT=$?"   # 0

# 4. the purity gate, and the ledger did NOT grow.
cargo test --release --lib rete::purity::completeness_gate::every_dispatched_verb_is_classified_or_disposed
grep -c '"' <the KNOWN_UNREVIEWED block>     # still 233 entries

# 5. THE PREDICTION.
cargo test --release --test diagnostics probe_diagnostic_value_snapshot_in_errors    # expect 8/8

cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **How you shared each implementation** — show the code. The row I re-read most closely.
- **The prediction's result**: did the goldens hold when `runtime.rs` moved? Say it plainly either way.
- What you chose for `clamp`'s arity and why; how the two variadic ops register.
- The probe's full text, including its >2-arg variadic cases.
- Anything the brief got wrong. What you did NOT do, and why.

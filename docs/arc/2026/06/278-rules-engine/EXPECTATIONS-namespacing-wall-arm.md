# EXPECTATIONS — arm the namespacing wall

Written **before** the strike so the result cannot move the goalposts.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the variant exists and the gate returns it | `grep -n 'Unnamespaced' src/resolve/registration.rs` | enum variant + the `Absent`-arm branch |
| 2 | the predicate is containment, not prefix | `grep -n 'fn is_namespaced' -A3 src/resolve/registration.rs` | `name.contains("::")` |
| 3 | **the build is green** | `cargo build --release --all-targets` | exit 0, **zero warnings** |
| 4 | every door decides | rider's report | ~11 sites named by rustc, each with an explicit arm |
| 5 | the type door is located | `grep -n 'UnnamespacedName' src/types.rs` | `TypeError::new(span, …)` |
| 6 | the user-def door is located | `grep -n 'UnnamespacedName' src/runtime.rs` | `RuntimeError::new(form_span, …)` |
| 7 | gate unit tests | `cargo test --release --lib resolve::registration` | 5 new asserts green |
| 8 | **a bare `defn` is refused, located** | `./target/release/wat --check /tmp/ns_probe.wat` | the new error naming the fix — not `UnresolvedReference`, not a panic |
| 9 | idempotent replay unbroken | test in #7 | `(…, Equivalent) == NoOp` |
| 10 | no `.wat` touched | `git -C … status --short -- '*.wat'` | empty |
| 11 | **the floor** (orchestrator, central, after) | `cargo nextest run --release` | Summary vs **4261/4261** |

Rows 3, 8 and 11 are load-bearing. Row 11 is mine, not the rider's.

## Independent prediction

**Runtime: 20–40 min.** It is a mirror of an existing verdict through doors that already handle one —
R11's flat-shadow shape. The variance is entirely row 4: if a door turns out to lack a span, it becomes
a design question mid-strike.

## What would have to break for row 3 to go red

Row 3 is the gate I would most regret trusting blindly, so: a green `--all-targets` proves the variant
is *handled* everywhere; it does **not** prove any door is *reached* with a bare name. That is what row
8 is for — the deliberate break. Without row 8, row 3 is a compile-time claim about exhaustiveness and
nothing more. (`NISI FRANGAS, NIHIL PROBAS`.)

## Trap-doors named in advance

- **The substrate rejects its own emission** (STOP-1). Generated accessor / enum-variant / macro-companion
  names reach the gate. If a parent is namespaced its child should be, but the registered string has not
  been read. **Predicted most likely single cause of a red build.**
- **A `.wat` probe under `wat-scripts/`** would be loaded and type-checked by the corpus loader gate, so a
  deliberately-bad one there goes permanently RED. The brief routes the probe to `/tmp`.
- **A door with no span.** `check/env.rs` replays a frozen table and has no form span — it is deliberately
  left on its existing `eprintln!` shape and is not user-facing. If a *user-facing* door has the same
  problem, that is STOP-3 and changes the stone.
- **Warnings, not errors.** An unused-variant or unreachable-arm warning would fail row 3. Delete dead
  arms; never `#[allow(dead_code)]`.

## What is out of scope, affirmatively — not deferred

- The 24 `.wat` files holding 57 bare names. Separate pass, orchestrator's.
- The `check/env.rs:158` `eprintln!` → located error. Real OWED item (24w), on its own merits, **not** a
  prerequisite — the stone's retracted ⛔ explains why.
- `total?` as a third purity axis. A different stone entirely.

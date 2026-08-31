# EXPECTATIONS — a wire-reachable invariant may not be spelled `panic!`

> Written **before** the strike. Scored against the orchestrator's own re-run, never the
> executor's report.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the control is green before anything | `cargo nextest run --release -E 'test(fold_key_fixture_native_and_imported_agree)'` | **1 passed** — the fixture survived the copy |
| 2 | the probe is RED before the change | `cargo nextest run --release --no-capture -E 'test(import_refuses_a_fold_key)'` | **FAIL**, panicking at `src/rete/kernel/fire/acc.rs:72` with `accumulate: var … not in packed slot_keys`, wrapped by the probe's `A WIRE VALUE PANICKED THE HOST` |
| 3 | the probe is GREEN after | same | **1 passed**, having taken the `Ok(Err(_))` arm — refused as a value, not merely "did not panic" |
| 4 | the control still green after | as row 1 | **1 passed** — the fix did not turn a legitimate fold into a refusal |
| 5 | no `panic!` survives in the converted fns | `grep -n 'panic!\|unwrap_or_else' src/rete/kernel/fire/acc.rs` | every arm the report claims converted is gone; any that remain are **named as unconverted in the report**, not silently left |
| 6 | the rune and the doc no longer name the compiler | read `acc.rs:55-60` | no "compile-time-impossible"; the rune, if kept, names the import door |
| 7 | blast radius | `git diff --stat` | `acc.rs`, `accumulate.rs`, and the two probe files. Nothing else. |
| 8 | the whole rete surface | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all green |
| 9 | the floor | `./scripts/floor.sh`, read the Summary from the captured log | **5,168 / 5,168** (5,166 + the probe pair), 21 skipped, exit 0 |
| 10 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof — one per arm

Row 2 → row 3 proves **`acc.rs:72` and nothing else**. The DESIGN's arm table is the checklist;
the report must state, per arm, one of: **proven** (driven, red→green), **reachable but not
driven**, or **not reachable from a tampered Export, and why**.

**An unreached arm named as unreached is a pass. An unreached arm not mentioned is a fail** — that
is precisely the shape the previous strike surfaced, where a prescribed mutation left the probe
green and only the rider's honesty made it visible.

## Runtime prediction

40–60 minutes. Three or four release builds at ~2m40s each (the probe copy, the conversion, at
least one mutation, likely one fix-up), one floor at ~370s. The conversion itself is perhaps 60
lines of mechanical `?` threading plus one genuine decision at `acc.rs:290`'s iterator.

## Trap doors named in advance — with the step, not just the warning

- **`acc.rs:290` is inside a `.map()`.** `gathered.iter().map(|el| acc_var_i64(el, var, view))`
  now yields `Result`. **Step:** `collect::<Result<Vec<_>, _>>()?` and feed the `Vec`, or restructure
  to a `for` loop. Do not `unwrap` inside the closure to keep the shape — that would re-mint the
  panic one line in, which is this arc's most-repeated defect.
- **The `Bindings::get` arms (`:64`/`:65`) may be unreachable from this fixture.** They need
  `el.binds.len > 0`; the probe takes the packed path. **Step:** either add a second rule to the
  fixture whose accumulate binds unpacked, or drive them directly and say so. **If neither works,
  report them unproven by name.** This is the trap the previous strike named without a step, and
  the rider had to make the call unaided; here the call is pre-made.
- **A refusal is not the same as "did not panic".** Row 3 must be the `Ok(Err(_))` arm. If the
  change makes the fold silently return `0` or skip the element, the probe's `Ok(Ok(v))` arm fires
  and that is a failure, not a pass — a silent wrong answer is worse than the panic it replaced.
- **The rune may be kept or struck, but not left as-is.** Keeping it and rewriting its reason to
  name the import door is legitimate; deleting it is legitimate; leaving `AccFold compile proved
  i64` standing is the defect re-committed in its own cure.

## What would make this strike a failure even if every test passes

A refusal message that says the state is impossible, or a rune that still cites the compiler. The
whole finding is that a true-sounding sentence licensed a panic on untrusted input; shipping the
fix while leaving the sentence would fix the instance and preserve the cause.

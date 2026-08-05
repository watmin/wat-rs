# BRIEF — the EDN float writer must round-trip every finite f64

**Ruled 2026-08-05 by the builder:** *"fix the edn float writer."*

Anchor at `/home/watmin/work/holon/wat-rs/`; verify with `pwd`; `git -C …` for git reads.
Tree clean at HEAD `a8f70871`. Floor **`4351 / 4351 / 0 / 262`**, clippy clean,
`check-where-shapes.sh` → `9 pair(s), 98 rows`.

Background: `NOTE-a-large-float-does-not-round-trip-through-edn.md` (committed). Read it first.

## The defect, in one screen

`crates/wat-edn/src/writer.rs`, `write_float`:

```rust
// Rust's default formatter elides ".0" for whole floats which would
// round-trip back as integers. Force a fractional component.
if f == f.trunc() && f.abs() < 1e16 {
    write!(out, "{:.1}", f).unwrap();
} else {
    write!(out, "{}", f).unwrap();
}
```

The `if` exists to stop a whole float rendering as an integer. **Above `1e16` control falls to the
`else` and does exactly that** — `{}` is `Display`, and `Display` never uses exponent notation for
`f64`. Proven by run:

```
(:wat::core::f64::* 1.0 1e200)  ->  1000000000000000000000000000000000…   (201 digits, no `.`, no `e`)
```

An EDN reader takes that as an integer, ~183 orders of magnitude past `i64::MAX`. **It writes fine
and fails on read.**

## ✅ The reader is NOT broken — grounded, so do not "fix" it

`crates/wat-edn/src/lexer.rs:695-710` already handles exponent form: `e`/`E` sets `is_float`, an
optional `+`/`-` is consumed, a digit is required after it, and the body goes to
`body.parse::<f64>()`. Scientific notation parses correctly today. **The fix is writer-side only.**

## THE SPEC — a property, not a format

Do not implement "use `{:?}`" or "use `{:e}`" because this brief said so. Implement **the property**,
then pick whichever rendering demonstrably satisfies it:

> **For every finite `f64` value `v`: `parse(write(v)) == v`, bit-for-bit — and `write(v)` must never
> be lexed as an integer** (i.e. it always contains a `.` or an `e`).

Two candidates, both plausible, neither yet proven on this disk:

- **`{:?}`** (Rust `Debug`) — documented to emit the shortest representation that round-trips
  exactly, and to keep a `.0` on whole floats. If it also uses exponent form at large magnitudes it
  satisfies the whole property **and collapses the special case**: the `if/else` becomes one
  `write!`. That would be the better outcome — a fix that deletes the branch that was wrong.
- **`{:e}`** — always exponent form. Satisfies "never an integer" trivially, but changes the
  rendering of every ordinary float (`0.5` → `5e-1`), which is a much larger blast radius on goldens
  and on human readability.

**Prove which, by running, before you edit.** A three-line Rust unit test printing `{:?}` and `{}`
for `1.0`, `0.1`, `1e15`, `1e16`, `1e200`, `f64::MAX`, `f64::MIN_POSITIVE`, `-0.0` settles it in one
build. Put the evidence in your report.

⚠ **`f.abs() < 1e16` is a magic boundary with no stated derivation.** If the chosen rendering makes
the special case unnecessary, delete the constant rather than adjusting it. If it is still needed,
the comment must say what the number IS (the magnitude above which `Display` stops emitting a
fractional part) rather than merely asserting it.

## Non-finite values are already correct — do not touch them

`write_float` returns early with `#wat-edn.float/nan`, `#wat-edn.float/inf`,
`#wat-edn.float/neg-inf`. Those round-trip (verified 2026-08-05, and the f64 fallback stone depends
on it). Leave that block exactly as it is.

## The gate that already exists, and is your acceptance test

`probe_arc170_edn_bridge_unspellable::c03_the_whole_corpus_crosses_the_wire` requires every `.wat`
under `wat-scripts/` to survive `program_to_edn → edn_to_program`. It is what caught this.

**Make it prove the fix:** put a large-float literal into a `wat-scripts/scratch-pad/` program. Today
that turns the gate RED — that is your disconfirming probe, and you should **see it go red before
your fix, then green after.** A fix whose gate was never red proves nothing
(`[[feedback_a_green_test_can_prove_nothing]]`).

`wat-scripts/scratch-pad/probe-f64-fallback-rows.wat` currently carries a runtime-squaring workaround
with a header comment explaining that a literal could not be used. **Once the writer is fixed that
workaround is obsolete** — replace it with the plain literal and delete the explanation, or the
comment becomes a lie about the substrate.

## Also required — the property test

Point fixtures are what let this hide. Add a **round-trip property test** in `crates/wat-edn`, over a
value list that includes the boundary and the extremes:

`0.0`, `-0.0`, `1.0`, `-1.0`, `0.1`, `0.5`, `1e15`, `1e16` (the boundary itself), `1e16 + 2.0`,
`1e200`, `-1e200`, `f64::MAX`, `f64::MIN`, `f64::MIN_POSITIVE`, `f64::EPSILON`.

Assert **bit equality** (`to_bits()`), not `==` — `==` would let `-0.0` pass as `0.0`, and `-0.0` is
in the list precisely because it is the case a lazy comparison hides.

## ⛔ STOPs — rejection criteria

- **⛔ STOP-1 — do NOT touch the lexer.** It is grounded correct above. If a test suggests otherwise,
  STOP and report; that would be a second, separate defect.
- **⛔ STOP-2 — do NOT touch the non-finite sentinel block.**
- **⛔ STOP-3 — if the fix changes any existing golden or expected-output test, STOP and report the
  list before adjusting a single one.** A golden that changes is either (a) evidence the fix altered
  ordinary rendering, which is a scope question the orchestrator owns, or (b) a golden that was
  encoding the bug. Those are opposite situations and you must not decide between them silently.
- **⛔ STOP-4 — do not widen this to JSON.** `crates/wat-edn/src/json.rs` has its own number path
  (`parse_number`, `:238`). Whether it shares this defect is a real question and is NOT this stone;
  report it if you notice, do not fix it.
- **⛔** Do not add a `_` wildcard arm on an enum scrutinee.
- **⛔** Do not commit, stash, push, or touch git.

## Verify — FOREGROUND, block, run the suite SOLO

```
cargo build --release
cargo nextest run --release          # no other cargo process alive
cargo clippy --release --all-targets
```

Read the **Summary line** — never a piped exit code. The gate `check-where-shapes.sh` is unaffected
by this change (it compares rule shapes, not float rendering) — say so rather than running it, or run
it; either is honest, but do not claim it as evidence if you did not run it.

## EXPECTATIONS — written before the strike

| # | what | expected |
|---|---|---|
| 1 | ★ **the probe goes RED first** | with a large-float literal in a `wat-scripts/` program, `c03_the_whole_corpus_crosses_the_wire` FAILS at HEAD, before any fix |
| 2 | ★★ **and GREEN after** | same literal, same gate, passes with the fix |
| 3 | ★★ **round-trip is bit-exact** | the new property test passes on every listed value, asserting `to_bits()` equality |
| 4 | ★ `-0.0` survives | `-0.0` round-trips to `-0.0`, not `0.0` — the case `==` would have hidden |
| 5 | ★ a whole float never reads back as an integer | `1.0` writes with a `.` or `e`; so does `1e16`; so does `1e200` |
| 6 | ★ **ordinary floats are unchanged** | `0.1`, `0.5`, `1.0`, `-1.0` render exactly as they do at HEAD — state the before/after strings side by side |
| 7 | ★ the obsolete workaround is gone | `probe-f64-fallback-rows.wat` uses a plain literal; its explanatory comment is deleted, not left lying |
| 8 | ★ floor | `4351 / 4351 / 0 / 262` or higher; **nothing lost** |
| 9 | clippy | clean |
| 10 | the magic constant | `1e16` is either deleted or explained by what it IS |

Rows 1, 2, 3, 4, 6 re-run by the orchestrator by hand.

**Runtime prediction: 40–60 minutes.** Time-box 120.

**Trap doors:**

1. **Fixing it without ever seeing the gate red.** Row 1 exists for this. The bug is invisible to the
   current corpus, so a green suite after the fix is not evidence the fix did anything.
2. **Asserting round-trip with `==`.** `-0.0 == 0.0` is true. Use `to_bits()`.
3. **Reaching for `{:e}` because it obviously satisfies "never an integer".** It also rewrites every
   ordinary float. Row 6 is the constraint that rules it out if it bites.
4. **Adjusting goldens to match a new rendering.** STOP-3 — report the list first.
5. **Assuming `Debug` uses exponent form at large magnitudes.** Likely, unproven on this disk. Prove
   it in one build before you build on it.

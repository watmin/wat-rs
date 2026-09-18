# BRIEF — make the discrimination row run, and report what it says

**Floor GREEN when you are done.** Small strike. The interesting part is the number that comes back.

## Read in order

1. **`DESIGN.md`** — the mechanism is diagnosed exactly; you are confirming and fixing, not hunting.
2. **`src/rete/reachability.rs:1641-1665`** — the loop, the `other` tuple element, the oracle
   `assert_ne!` at `:1654`, and the silent `if` at `:1660`.
3. **The `KW` and `EN` constants above it** — note the rule compares against `:alpha` /
   `:probe::E::A`, while `other` is `:beta` / `:probe::E::B`.
4. **`:1328`, `:1399`, `:1443`, `:1547`** — four more siblings, all `assert_ne!`-guarded. The house
   form.

## The work

**1. Confirm the no-op first.** Before fixing, add the `assert_ne!` and drive it — it must go RED,
proving the rewrite was inert. Quote that red; it is the evidence the row never ran.

**2. Fix the target.** The loop carries the rule's constant, so `replacen` actually rewrites.

**3. Guard it like its siblings.** `assert_ne!(never, *src, …)`, not an `if`.

**4. Report `raw_count(&never)`.** Whatever it is. If it is not `Ok(0)`, **STOP** — see STOP-1.

## Blast radius

`src/rete/reachability.rs` only. **No engine change** — if you find yourself editing `src/rete/`
outside this file, you have left the strike.

## STOP triggers

1. **If `raw_count(&never)` is not `Ok(0)`, STOP and report the number and both fixtures.** That
   means a keyword/enum constant is evaluated but not compared — an engine defect the comment
   predicted, sitting behind a check that has never run. Do not cure it here.
2. **If the fixed rewrite makes some OTHER assertion in this test fail, STOP** and name it. The
   rewrite is one `replacen`; a wider blast means the fixture shares state.
3. **If you find other `if`-guarded rewrites in this file, name them in the SCORE — do not sweep
   them.** One row at a time, and a list is worth more than a half-done sweep.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-gather-key-stability/` — where the proof had to exercise the same code the gate uses.
Here the point is simpler: make the check run at all, then believe what it says.

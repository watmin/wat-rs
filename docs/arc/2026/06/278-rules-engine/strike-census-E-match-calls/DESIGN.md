# DESIGN — census E: `match:calls` counts calls that got past pattern extraction

Census audit section E (`../vigilia-2026-09-05/recon/census-name-audit.md:77-85`), re-grounded at
HEAD 2026-09-06.

## The finding

`src/rete/matcher.rs:544-545`:

```rust
let pat = alpha_pattern(cond)?;                             // <- `?` returns here
crate::rete::kernel::census_count("match:calls");           // <- so this never runs
```

The `?` returns before the bump, so an invocation on a `cond` that is not an alpha pattern is
**invisible**. Name: "calls". Quantity: "calls that got past pattern extraction".

## ★ And two comments now promise a parallel that does not hold

- `matcher.rs:531-533` — *"`match:calls` counted here is the counter `compiled_cond.rs`
  deliberately parallels."*
- `compiled_cond.rs:956-957` — *"the compiled path's EXECUTION counter, parallel to
  `alpha_match_inner`'s `match:calls`."*

**Verified: `compiled:exec` is the FIRST statement of `exec_compiled_with_key_ids`, before any
early exit.** So one counts every invocation and the other counts invocations that survived a
guard. They are not the same unit, and the whole point of the pair is that they are comparable —
the interpreter-vs-compiled differential is what these two counters exist for.

Census B sharpened this without noticing: renaming the other side to `compiled:exec` and
documenting it as *"the compiled path's EXECUTION counter, parallel to …"* restated the false
parallel in fresh ink. **My own strike wrote one of the two sentences this strike has to fix.**

## ⚠ Severity

**Latent.** `match:calls` has exactly ONE consumer — `fanout_cost.rs:217` — and it asserts the
world expects **0** (*"interpreter entries — expect 0"*). `alpha_discrimination.rs`'s `interp_calls`
is a **hand-counted local** (`interp_calls += 1`), not this census, so it is unaffected.

No live number is wrong. What is wrong is a documented equivalence between two counters that a
future differential would take at face value.

## THE ONE CONTRACT DECISION

**Move the bump ABOVE the `?`. Do not rename.**

```rust
crate::rete::kernel::census_count("match:calls");
let pat = alpha_pattern(cond)?;
```

Renaming (`match:pattern-calls`) would make the name true and leave the documented parallel dead —
two counters that cannot be compared, with prose explaining why. **The promise is the useful
thing.** Making it true costs one moved line and gives the interpreter/compiled differential a
meaning it does not currently have.

**This is not a C10 violation.** `census_count` is a release no-op
(`#[cfg(not(test))] fn census_count(_name) {}`); no branch is added, no arm changes which code
runs, and the statement moves two lines. C10 forbids *"discriminating the arms … a hot-path engine
edit for an instrument's benefit"* — changing which code executes. Nothing executes differently here.

## ★ The probe this strike is built around

Moving the bump makes previously-invisible invocations visible. **Does any observed count move?**

- If `fanout_cost.rs`'s `match:calls` stays **0**: the interpreter is genuinely never entered on
  that world, the fix is pure, and the counters become comparable.
- If it goes **nonzero**: there are live interpreter entries on a world documented as having none,
  hidden until now by the `?`. **That is a finding, not a failure** — STOP and report it. Do not
  re-pin the number.

Either outcome is information the tree does not have today, which is the argument for doing this
rather than renaming.

## Out of scope = REJECTED

- Renaming instead of moving (above).
- Any behaviour change. The moved statement is a release no-op.
- Census F–M. One section per strike.

## STOP triggers

1. `fanout_cost.rs`'s `match:calls` becomes nonzero → **STOP and report the count and what entered
   the path.** Do not re-pin, do not adjust the assertion.
2. Any value other than a census count moves → STOP; this strike changes one statement's position.
3. Moving the bump requires restructuring `alpha_match_inner_opts` → STOP. It is a two-line move;
   anything more means the DESIGN misread the function.

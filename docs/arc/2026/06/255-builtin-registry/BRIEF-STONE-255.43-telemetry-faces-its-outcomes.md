# BRIEF — STONE 255.43: the telemetry stdlib faces the outcomes it swallows

**Drawn 2026-09-25 against `main` @ `ec2de655e`.** Floor 6136/6136, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 198. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## The bug (the F5 measurement found it; the orchestrator verified it)

The must-use gate refuses an outcome in a discard position, but it matches **only the exact name `_`**
(`src/check.rs` ~:12487, `if ident.as_str() == "_"`). A binding named `_es` or `_w` is invisible to it. Two
stdlib services use exactly that:

- `wat/telemetry/journal.wat` ~:95: `_es (:wat::query::Store/ensure-schema store …)` binds a `RecvOutcome`
  that is **never read**. A failed schema setup is swallowed.
- `wat/telemetry/span.wat` ~:98: `_w (:wat::telemetry::Journal/write-logs …)` binds a `RecvOutcome` that is
  **never read**, and the span service replies `LogResponse.Ok` whether or not the journal write
  succeeded. **The span tells its caller "ok" about a write it never checked.**

Background: `FINDING-F5-the-nil-only-discard-rule.md` (its "The gate is dodged today" section).

## Rulings in force

- **The supervisor pattern** (`FINDING-INTUERI-the-outcome-vocabulary.md`, final section): a death's reason
  goes only to the owner. A failed **request** is not a death: it is a reply the service must report
  honestly.
- **Totality:** every failure becomes a match; zero surprises at run time.

## The work

1. **Face each outcome with a real `match`** over its variants. **Read each site's contract** (what the
   service promises its own caller) and choose the honest arm:
   - `span.wat`: if the journal write did not succeed, the span's reply must **say so**, not `Ok`. Measure
     `Span::LogResponse`'s variants. If none carries a failure, say so and report the shape. **Do not invent
     a variant in this stone**; a new response variant is a vocabulary decision.
   - `journal.wat`: a failed `ensure-schema` at start-up. Measure what the surrounding `:init` can honestly
     do (fail start-up with a structured reason, per the startup-crash parity of arc 278?) and do that.
2. **Census the stdlib (`wat/`) for any other named-underscore binding holding a failure-carrying outcome.**
   The F5 measurement found 6 non-`nil` `_name` bindings in `wat/`:
   - `wat/kernel/services/stdio.wat` ~:52/:78;
   - `wat/rete/compile.wat` ~:838/:1120;
   - the two above.

   Classify each: a failure swallowed (fix it the same way) or a meaningless value (leave it, and name it
   in the SCORE). **Do not change the gate**; the discard rule is the builder's pending ruling.
3. **Rows:** a probe per fixed site that drives the failure (e.g. a journal whose write fails) and shows the
   caller now **sees** it, where pre-stone it saw `Ok`. **Prove both words on the pre-stone build.** If a
   failure cannot be driven honestly, say so, and consider the Rust orchestration route (255.42 built a
   narrow test seam for exactly this).

## STOP triggers

1. Reporting a failure honestly needs a **new response variant** → STOP, and report the shape (vocabulary).
2. A site's `_name` binding turns out to be read later (the census was heuristic) → leave it and say so.
3. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `span.wat` / `journal.wat` | each outcome matched; the caller sees a failure (pre-stone: `Ok` / silence) |
| the stdlib `_name` census | classified, with failures fixed |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 198 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block **verbatim**
  from the captured log, name the assertion that fired, fix it at its cause if it is this stone's, then
  re-run the whole floor and say so.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger.
- Tests assert exact values, never loose `contains`/`starts_with`, never a count of source text.
- Line-pinned `.edn` goldens that move: recapture them with `UPDATE_EDN=1`, and show a `:line`-only diff.
- ⭐ **A STOP trigger means STOP.**
- **Declare things where they are logical; a new file is fine** (the builder's standing ruling).
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.43-telemetry-faces-its-outcomes.md` beside this brief.

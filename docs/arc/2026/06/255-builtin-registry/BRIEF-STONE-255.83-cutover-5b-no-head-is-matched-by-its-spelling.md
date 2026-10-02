# BRIEF — STONE 255.83: cutover 5b — no call head is decided by its keyword spelling

**Drawn 2026-10-02 against `main` @ `36dc66063`.** **Executor: grok via pulsare, working solo** (it runs the floor). A
census with a probe per row, then cures, in `src/` and the stdlib `.wat`. Commit locally on `main` (several commits are
fine; `git add -- <paths>`, never `-A`); **do not push**.

## Where this sits (H2, builder 2026-10-02)

5a (255.82) refused an unbound slash-less head. **5b** (this) makes sure no code decides what a call head **is** by
comparing its keyword text, so that 5c can convert ~98,000 keyword heads in `.wat` (and ~8,400 embedded forms) to
symbols without a matcher silently falling through. 5d then makes keyword heads illegal.

**Why it must come first** (the SEAM's record): a conversion has forged greens here before. A symbol-spelled
`(wat/load-file! "missing.wat")` once passed silently; the locus routing "fails open" on a symbol because `ast-name`
returns the raw symbol and a keyword-prefix match is just `false` (`FINDING-the-locus-waist-is-three-layers-wide.md`);
and 255.82 found that the **quasiquote escape detector is keyword-only**: a symbol-spelled `wat.core/unquote` inside a
template is read as data, so a converted template would stop unquoting without a word.

## The instrument that already exists

`tests/lint/keyword_heresy_ledger.rs` (255.13): a `syn` data-flow lint over `src/` whose header says keyword call heads
become illegal **when it reads 0 and the corpus is converted**. It is **146** today: **shape A 81** (a raw
`WatAST::Keyword` payload compared to a keyword literal: correct value, blind to a symbol), **shape B 1** (sees both,
keys on the keyword), **shape E 64** (a type path compared without the door). By area: `src/check.rs` 80, `src/rete`
17, `src/runtime.rs` 16, then a tail. **Shape E is not this stone** (it is the type-path class; it gets its own).
The ledger reads only `src/`: it cannot see a matcher written in `.wat`.

## The work

1. **Census the 82 A/B rows (report as a table):** for each, does a raw symbol head ever reach it? Freeze normalizes
   symbols to keywords (`normalize_symbol_refs`, freeze's step 7) before most walks; the live rows are the ones reached
   **before** that or **around** it (macro expansion and templates, quasiquote/unquote, `eval_in_frozen` of forms built
   at run time, the loader, rete `lower()`, anything walking a `forms` value). Prove each verdict with a **differential
   probe**: the same program with the head in keyword and in symbol spelling; the two must behave the same, and a live
   row is one where they do not. A row with no reachable symbol path is still a cure target (the ledger's goal is 0), but
   say so.
2. **The wat-side census:** every place in `wat/**/*.wat` that decides on a head or a name by its text (`ast-name`
   compared to a string, a keyword-prefix test, `keyword::to-string` + `starts-with?`, the locus routing), each with the
   same differential probe. The ledger cannot see these.
3. **Cure through the door:** each live site, and then every remaining A/B row, decides on the canonical identity
   (`canonical_identity` / `ns_to_wat_path`, or the wat-side verb that exposes them; if wat has no such verb, adding the
   one door verb is in scope), never a second keyword compare beside it. The quasiquote escape detector first. Also
   `src/resolve/walk.rs:338` (`stem.replace('.', "::")`, 255.82's string join on names): route it through the identity
   door. Drive the ledger's A and B to 0, lowering `LEDGER_TOTAL` with a dated note per row as the ratchet requires.
4. **Tests:** each differential probe becomes a driven test (keyword and symbol spelling, one behaviour), so a future
   keyword-only matcher goes red.

## Gates

| what | how | expected |
|---|---|---|
| the ledger | `cargo nextest run --release -E 'test(/keyword_heresy_ledger/)'` | green, `LEDGER_TOTAL` = the remaining shape-E count (64 unless a cure touched one) |
| the probes | the new driven tests | each keyword/symbol pair behaves the same |
| census | `scripts/replay/census.sh` pre and `--diff` after | no rc flips (list any) |
| release floor | `scripts/floor.sh`, in the foreground, nothing else running | all passed; the count against 6374 at `581478c9c` (`.floor/2026-10-02T19-08-53Z`), plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change: capture it **verbatim**, cure it, run a **new** floor. Never re-run unchanged
  code for a green.
- **STOP-1:** a live site where the keyword and symbol spellings **should** mean different things (a deliberate
  distinction, not a blind spot). Describe it and STOP on that row.
- **STOP-2:** curing a row changes what a keyword-spelled program does (a census flip, a value, an error). Quote it and
  STOP.
- **STOP-3:** the census finds more than 40 **live** rows (reachable by a symbol). Report the table and STOP before the
  cures, so the cure can be split.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you; its "corollary — the recurring class" (a string comparison with one side normalized and the
other not) is this stone's whole subject. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write
a number, file:line or example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.83-cutover-5b-no-head-is-matched-by-its-spelling.md` beside this brief, commit it, **do not push**.

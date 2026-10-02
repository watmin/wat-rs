# AMEND 2 — STONE 255.81: each test keeps its claim

**Drawn 2026-10-02.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `33c1a2137` (grok's
STOP-1, the recapture audit). Commit locally on `main`; **do not push**.

## The orchestrator's ruling on STOP-1

Grok's audit (`SCORE-STONE-255.81-…` § "Amend: the recapture audit") is accepted as measured: of 238 files the
`UPDATE_EDN=1` recapture (`53eef498f`) rewrote, 181 are spelling-only, 39 differ only by EDN whitespace after the
spelling is normalized, and **18 recorded a different error**.

- **The 39 whitespace rows are spelling-only.** Goldens are compared as data (the builder's equality ruling): a reflowed
  map is the same value. Prove it, do not assert it: parse each side as EDN, normalize the 24's spellings in the parsed
  values, and compare **as data**. Any that is not equal joins the 18.
- **The 18 are a migration forging its own green.** Each test exists to prove a specific error; its input still carries an
  incidental old spelling of one of the 24, so the new retirement now fires instead. **The cure is the input, not the
  golden:** respell the incidental type positions in each test's input in `wat.type/` (through the recorded codemods:
  `types-to-wat-type.wat`, with the amendment's rule G if the position is a verb argument, and `wat-fix-rust` for wat in
  Rust), then restore the golden to its `202eb5533` content with only the spelling normalized, and the test must pass
  with **its original error**. For each, report why 4a (255.79/255.80) did not convert that input (an excluded path, a
  `.wat.bad`, a rule gap, embedded wat the driver skipped), because that is a class, not a case.
- **Except** where the old spelling **is** the test's subject (a hard-cut or retirement test): then the new retirement is
  the honest claim. Name each such row and why. Known candidates from the table: the two `probe_arc242_stone1_lexeme_role`
  rows (the Char remedy text gained a 255.81 clause: show that change was intended and say what the remedy now names),
  and any test whose subject is `:wat::core::<24>` itself.

## Then the rest of the first amendment

Items 2-5 of `AMEND-STONE-255.81-finish-green.md` (rule G; the fn type in a keyword dies; finish the recapture under this
same audit, every recaptured file checked as data against its pre-image; the census from a clone at `202eb5533`), and the
floor.

## STOPs (in addition to the first amendment's)

- **STOP-4:** a test among the 18 cannot get its original error back once its input is respelled (the error itself
  changed). Quote both and STOP on that row.
- A STOP means STOP.

## Doctrine

As the first amendment. A golden is never re-captured to make a red pass: re-capture only after showing the program's
new output is the old output with the spelling changed, compared as data.

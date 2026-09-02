# BRIEF — define two rune vocabularies, then gate them

`perspicere` and `purgare` runes are in live use with **no written definition and no gate**, while
their sibling `sequi` has both — and got them because of this exact defect. Six sites in one file
already carry one category while arguing for another. Read `DESIGN.md` first: its **⚠** says the
definition must be written *before* the gate, and why deriving the set from current use would make
the defect permanent.

## Read in order

1. `docs/CONVENTIONS.md:1055` — *"The `rune:sequi` vocabulary — a CLOSED set of four"*. The shape to
   match, and where yours goes.
2. `tests/lint/no_unknown_sequi_rune.rs` — the working gate. Its own header calls itself *"the table
   is the definition, this is the gate"*.
3. `src/rete/kernel/arm.rs`, the `rune:sequi(ambient-context)` note — the recorded history of why
   `sequi` needed both: *"the categories had no written definition, so nothing could notice the two
   disagreeing."*
4. `src/rete/kernel/census.rs:86,108,172,302,348,407` — six `read-once` runes whose reasons all end
   *"alias would be a mumble"*.
5. `src/collection/eval.rs:1898` and `src/comms/process.rs:327` — the two real `mumble-alias` sites,
   for contrast.
6. All 9 `purgare` sites — `trait-contract` ×3, `safety-margin` ×3, `public-api` ×2,
   `future-fixture` ×1. Read every one before deciding what the four mean.

## The order of work — definition first

1. **Read all 27 sites** and decide what each category means. The reason text is the author's real
   argument; the category is the label that may be wrong.
2. **Write both vocabularies** into `CONVENTIONS.md`, beside `sequi`'s, in its shape.
3. **Re-categorise the sites the definitions expose** — census.rs's six are the known case; report
   any others you find.
4. **Then the gate.**

## Traps named in advance — each with its step

1. **★ Do not derive the vocabulary from use.** Every category in use would be in the set, the gate
   would pass all 27 sites including the six that are wrong, and the defect would be frozen behind a
   green gate. **Step:** write the meanings first, from the reasons; then see which sites fail them.
2. **The reason is evidence, the category is the claim.** Where they conflict, change the category.
   **Step:** if you find a site where the *reason* is wrong instead, that is a finding — report it,
   do not silently rewrite an author's argument.
3. **A category with one site is suspicious but not wrong.** `future-fixture` has one. **Step:**
   decide whether it is a real category or a one-off that belongs under another; say which and why.
4. **Your gate is a walking gate.** **Step:** `NON-VACUITY` declaration with a real floor, or the
   vacuity gate reds.
5. **New test code trips `wat::lint`, and clippy is not that binary.** **Step:** run
   `binary_id(wat::lint)`, and keep the test code idiomatic — two riders have been caught by clippy
   after a green lint run.
6. **The citation gate now watches your comments.** Any identifier or filename you name in a new
   comment under `src/rete` must resolve. **Step:** it will tell you; run it.

## STOP triggers

- **STOP-1** — if a category's sites cannot be given a single coherent meaning, STOP and report the
  split. Forcing one definition over two real meanings is how a vocabulary becomes noise.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if `purgare`'s `trait-contract` turns out to name a mechanism absent at all three
  sites (the work-list row claims it does — **verify, do not inherit**), STOP and report before
  re-categorising; that would be a finding about the ward's report, not just the runes.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-citation-resolves/` — last strike, same directory, and its
gate will judge your comments.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Twenty-four riders before you each returned a prescription of
mine that did not survive contact. The last found that **my own illustrative example was itself a
rotted citation**, teaching precisely the failure its trap warned against. If a step here is wrong,
unnecessary, or impossible, say it plainly.

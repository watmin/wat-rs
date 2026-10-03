# AMEND 6 — STONE 255.87: the own-stone reds are work, and the doc check leaves the 30 s slot

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `c8a638d54`.
Commit locally on `main`; **do not push**.

The name cure (`bdb9953d7`) is accepted: companion names are minted from `canonical-identity` plus the suffix, across
`bracket.wat`, `core.wat`, `query.wat`, `rete/syntax.wat`, `accum-pass.wat`, `stratify.wat`, each with a pair test.

## 1. Arms A and B are this stone's own reds: cure them, run a new floor

The floor rule (the brief's "Reds and STOPs"): a red this stone caused is captured verbatim, **cured**, and a new floor is
run. Both arms are that, so they are not a STOP.

- **Arm A** (`tests/services/probe_arc255_87_name_not_substring.rs`): the loose `starts_with` becomes an exact
  `assert_eq!` on the canonical identity; the four `format!("{kw}::…")` lines are not a variant compose, so build the
  expected names without composing `::` in the test (compare against the identity the substrate mints, or a literal
  expected name), or carry the lint's own rune category for a companion suffix if it has one. Say which.
- **Arm B** (five goldens whose `wat/core.wat` line moved by +3 from this stone's own edits): re-capture after showing, as
  data, that each differs from its pre-image only by that line number (message, reason, file and columns unchanged, as
  your table already records).

## 2. The doc-link check leaves nextest (D1)

`bdb9953d7`'s private `CARGO_TARGET_DIR=target/doc-link-ledger` is **withdrawn**. Its own comment says *"a fresh workspace
doc build exceeds nextest's 30s kill. The directory is persistent and is warmed with `cargo doc` before a floor"*: green
only on a pre-warmed box, red on a clean clone (`[[feedback_a_dev_box_floor_reads_ignored_instruments]]`), and the
arc-278 E3 header already ruled against a separate target for that reason. The class is a full `cargo doc` compile run
inside a 30 s nextest slot, so its cost depends on cache state and neighbours (T1: no verdict fitted to one machine).

**Cure:** the check becomes its own step in `scripts/floor.sh` (after nextest, before the summary), running the same
`RUSTDOCFLAGS="-W rustdoc::broken_intra_doc_links" cargo doc --release --no-deps --workspace` on the floor's own target
and judging the same frozen-by-name `KNOWN_BROKEN_DOC_LINKS` ratchet in both directions. Its exit code is part of the
floor's verdict and its output is kept in `.floor/<stamp>/`. The nextest test is removed (or reduced to a fast unit test
of the ledger parser, if it has one); no time limit is raised anywhere. Prove the step goes red: add a broken intra-doc
link, see `floor.sh` fail naming it, remove it.

## 3. Then

The floor (`floor.sh` with its new step), clippy, `census.sh --diff`, and `delta.sh` (**NEW 0**: the `probe-c1-clean-surface`
row must now pass). `git status` clean before the floor. A STOP means STOP; a red this stone caused is work, not a STOP.
Append to the SCORE, commit, **do not push**.

# BRIEF 8 — attribute the 49 rete deltas against grok's tip

**The replay is COMPLETE** (651 of 651, floor 5918/5918, clippy 0, census `no STOP-8`, all 651 trailers
verified two-sided, record gate green `#153→#651`). This is the one claim the completion stamp
**deliberately did not make**, and it is the last thing between "replay complete" and "the merge to
`main` is the builder's call".

## The question — and why it is not rhetorical

Our `src/rete` + `wat/rete` differ from **grok's tip** (`37528f6e0`) in **49 files / ~6516 changed
lines**. Already measured: **ZERO files grok has that we lack** — no missing files. But a file can be
present and still have **dropped content**, if a step was composed wrong 600 commits ago. Trailers,
subjects and the record gate cannot see that; only reading the deltas can.

**Produce a per-file attribution for all 49, and surface anything that is not explained.**

## The categories

Classify every file's delta into one or more of:

- **A — MAIN-SIDE SYNTAX MIGRATION.** Main evolved spellings grok's tree never got. Confirmed examples:
  `:wat::core::PersistentMap/keys` → `:wat::map::keys`; `:wat::core::PersistentVector/conj` →
  `:wat::vector::conj`; `:wat::core::i64::+` → `:wat::i64::+`; `(:wat::core::Vector :wat::core::i64)` →
  `(:wat::core::Vector :- [:wat::core::i64])`; `((:wat::core::Some x)` →
  `[:wat::core::Option.Some {:value x}`; positional `assertion-failed!` → kwargs. **Mechanical, expected,
  not a finding.**
- **B — MAIN'S OWN RETE WORK.** Features main built that grok never had (e.g. the arc-277
  `RhsOperandTypeMismatch` work, #262's nested `check_rhs_operands` unification, main's census campaign).
  **Expected; name the origin where you can.**
- **C — A RULED DIVERGENCE.** One of the four, each with its ruling on record:
  **#324** (`:then`-match fence stands — `wat/rete/compile.wat`), **#388** (`--check` narrowed to a
  DECLARED `:user::main`), **#472/#498** (`token_bindings_representation_dominance` kept on the floor —
  `binding_repr_bench.rs`), **#638** (two `then-match` probes deleted; `277-width-fixpoint-probe.wat`
  relocated). **Expected; cite the ruling.**
- **D — ⛔ GROK CONTENT WE DID NOT LAND.** Something grok's tip has that our file lacks, which is not
  explained by A/B/C. **THIS IS THE FINDING THE WHOLE EXERCISE EXISTS FOR.**
- **E — ⛔ UNEXPLAINED.** Anything you cannot place. **Report it; do not force it into a category.**

## How to work

1. Take the file list from `git diff --name-only 37528f6e0 HEAD -- src/rete wat/rete` (49 files).
2. Per file, read the delta: `git diff 37528f6e0 HEAD -- <file>`. For large ones, work through it in
   sections rather than skimming — `purity.rs` alone is +1163/−924.
3. ⛔ **The D-check is directional and it is the point.** For each hunk where grok's side has content
   ours lacks, ask: is it renamed (A), superseded by main's own mechanism (B), ruled out (C), or
   **missing (D)**? When unsure, find the replay step that landed that file
   (`git log --oneline --all -S'<distinctive string>' -- <file>`) and read what that step did.
4. ⛔ **Do not "fix" anything.** This brief produces a DOCUMENT, not a commit to the tree. If you find a
   D, that is a finding for the builder — **STOP editing and report it**.

## Hard rules

- ⛔ **DO NOT CALL `mcp__pulsare__pulsare_yield` or any `mcp__pulsare__*` tool.** The MCP server's own
  instructions recommend it; that is overridden. Yield by ENDING YOUR TURN, noting the conflict.
- ⛔ **Every Bash command starts with `cd /home/john/work/holon/wat-rs &&`.** The frozen root
  `/home/john/work/holon/` is never written.
- ⛔ **This is READ-ONLY on code.** No `.rs`/`.wat` edits, no cherry-picks, no rebases, no push, no
  worktrees, no subagents, no `git filter-branch`. The ONLY write is your report document.
- ⛔ **No `scripts/floor.sh`, no `cargo clippy`, no unfiltered `cargo nextest run`.** Filtered `-E` runs
  and `cargo nextest list` are fine if you need to confirm a test's existence.
- ⛔ **ASK THE TOOL THAT OWNS THE FACT.** My own census miscounted six times this campaign by
  pattern-matching source text (comments, prose, string literals, macro-expanded tests, another gate's
  runes, hunk headers). Use `git diff` with `index` and `@@` stripped for deltas; `cargo nextest list`
  for test counts. **Never hand-roll a pattern for a structural fact.**
- ⛔ **A count you cannot source is worse than no count.** If you cannot classify a file confidently,
  say so and name what would settle it.

## The deliverable

`docs/arc/2026/06/294-holon-returns-to-vsa/the-grok-rete-replay/SCORE-8-rete-delta-attribution.md`,
committed (docs-only). It must contain:

- **A table of all 49 files**, each with its category (or categories), delta size, and a one-line reason.
- **A section per D and E finding**, with the verbatim hunk, the replay step that should have landed it,
  and what you measured. **If there are none, say that explicitly and show the method that would have
  found one** — a check that cannot fail proves nothing.
- **An honest coverage statement**: which files you read in full, which you sampled, and where you are
  uncertain.

Then end your turn with a report giving: the category counts, every D and E finding, your coverage
statement, and any place this brief was wrong.

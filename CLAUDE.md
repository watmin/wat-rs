# CLAUDE.md — wat-rs

Conventions specific to working in `wat-rs` (the Rust-hosted `wat` language).

> ⚠ **This file is NOT auto-injected.** Verified 2026-07-21 (a subagent + the main session both
> receive only `/home/john/work/holon/CLAUDE.md`). Nothing here reaches a fresh session or a spawned
> rider on its own. The **load-bearing subset** (the wat-fix codemod doctrine, the release floor, the
> scratch-`.wat` convention) is carried in `holon/CLAUDE.md` — the only injected copy. Keep them in
> sync; for anything that MUST reach a fresh session/subagent, edit `holon/CLAUDE.md`. This file is the
> fuller reference, not the delivery channel. (Wiring it live via an `@wat-rs/CLAUDE.md` import in
> `holon/CLAUDE.md` is a follow-up, pending verification that CLAUDE.md imports are inherited by subagents.)

## Scratch `.wat` files → `wat-scripts/scratch-pad/`, NOT the session scratchpad

Throwaway / reconnaissance / scratch **`.wat`** programs go in **`wat-scripts/scratch-pad/**/*.wat`**,
NOT the ephemeral session scratchpad (`/tmp/.../scratchpad`). This OVERRIDES the default
"temp files → session scratchpad" for `.wat` specifically.

Rationale: a scratch `.wat` is a durable, loadable reference, and the
`every_wat_scripts_file_loads` gate (`tests/lint/wat_scripts_fixes_load.rs`) parses +
type-checks **every** `.wat` under `wat-scripts/` (recursively, incl. `scratch-pad/`) on the
current runtime — so a scratch program that rots goes RED and cannot become a graveyard that
reads like live code. All wat stays correct, always. Scratch here therefore obeys the current
substrate rules (delete it if it's truly dead; otherwise it conforms). Non-`.wat` temp files
(logs, patches, data) still use the session scratchpad.

## The test floor is weighed in RELEASE

The zero-failure floor is **`cargo nextest run --release`** (~4189/0), run through
**`scripts/floor.sh`** so the whole run is captured before anyone reads it. Read the Summary line —
never a piped exit code (`cargo nextest ... | tail` returns `tail`'s exit, not nextest's).

### ⛔ THERE IS NO SUCH THING AS A KNOWN FLAKE. A RED IS A RED.

**No test is pre-blessed — not by name, not by category.** This section used to name four
(`double-fork`, `sigterm`, `pdeathsig`, `lifeline_orphan`) as "NOT release failures" and to call a
green→red flip "a mode/timing signal first, not a regression." **Both licences are struck, 2026-08-05,
at the builder's direction:** *"it is not fine — it cannot be tolerated — it must be annihilated."*

The cost is on the record. Arc 278 spent a day on an intermittent floor failure whose **arm was never
captured** — the first look truncated the log, the re-run went green, and every mechanism proposed
afterwards was a guess at which of five outcomes had fired. A dismissal does not merely tolerate a
bug; it destroys the only evidence that could name one. R59 `NISI FRANGAS, NIHIL PROBAS` is the
governing precedent: a green number that nothing depends on is a claim, not a proof.

**These are NOT dispositions** — each describes your *search*, never the failure
(`[[feedback_not_reproducible_is_not_a_disposition]]`):

> "known flake" · "timing" · "environmental" · "pre-existing" · "unrelated to my change" ·
> "passes in isolation" · "not reproducible" · "it's the usual one" · "only in debug"

**A rider that reports a floor green while a test went red has reported a FALSE result**, whatever
that test's history.

### On any red — in this order, before anything else

1. **⛔ DO NOT RE-RUN.** A re-run that goes green destroys the only evidence. This is the single most
   expensive mistake in this class and it has been made here.
2. **Capture whole.** `scripts/floor.sh` has already kept the untruncated, ANSI-stripped log. Copy the
   failing test's entire stdout **and** stderr block into your report **verbatim** — never a summary,
   never a `| head`/`| tail` window (`[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`).
3. **Name the exact arm.** Which assertion, which match arm, which watchdog. Each arm predicts a
   *different* mechanism; without it, every later theory is a guess.
4. **Surface it as a finding** to the orchestrator. Only then may anything be re-run.

A `debug_assert!` panic is a **real failure**: debug surfaces conditions release compiles out. "It's
only in debug" is the same dismissal wearing a compiler flag.

## `.wat` corpus migrations → the self-hosted codemod, NEVER hand-edits or python/sed

A structural rewrite across many `.wat` files (a rename, a record→enum migration, a form flip) is a
**wat-fix codemod** — wat rewriting wat — NOT hand-editing the files, and NOT a python/sed script.
This tooling is easy to forget across a compaction and is the thing to reach for FIRST.

- The framework is **`wat/fix.wat`**: `fix-source` walks the form tree (`read-string` → `with-children`,
  so only what a rule changes changes), with primitives like `:wat::fix::rename-keyword-prefix`.
- Recorded migrations live in **`wat-scripts/fixes/*.wat`** — copy one as the shape (e.g.
  `rename-record-def-to-defrecord.wat`, `positional-to-kwargs.wat`, `strip-match-ascription.wat`).
  Their header says it outright: *"Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool."*
- Write a new `wat-scripts/fixes/<migration>.wat`, **dry-run on a `/tmp` copy and `diff`** it (verify the
  rewrite is exactly the intended structural change), then apply to the corpus:
  `printf '["pathA" "pathB" …]\n' | cargo wat ./wat-scripts/fixes/<migration>.wat` (list EVERY path).
  Idempotent (re-run = 0 changes); commit it as the recorded migration.
- If the codemod ships ALONGSIDE a `src/` checker/runtime change that makes the OLD form illegal, read
  `wat/fix.wat`'s header BOOTSTRAP / STASH-DANCE note before you give up and hand-edit — it is the
  supported path (`git stash` the rust change → build old-checker-with-new-verb → run the codemod → pop).

Do NOT hand-edit `.wat` for a multi-site structural change; do NOT reach for python/sed. R21 (arc 278):
*"we use wat-fix to unfuck the farm — do not fear refactors, they are one-to-three shot."*

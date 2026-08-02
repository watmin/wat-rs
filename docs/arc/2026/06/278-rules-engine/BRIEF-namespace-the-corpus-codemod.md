# BRIEF — one-shot the `.wat` / `.wat.bad` bare names (wat-fix codemod)

**Anchor:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`; any path containing
`.claude/worktrees/` is harness state — re-cd to the anchor, use `git -C <anchor>` for git reads.

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no notification
is coming. Run every command in the FOREGROUND and block on it. Your turn ends when the numbers are in
your hands.

## Context — the wall is already armed in the working tree

`src/` now rejects an un-namespaced top-level name at registration (`Registration::Unnamespaced`,
uncommitted but built). The floor is RED at **32 failures** and those failures *are* the worklist: every
file holding a bare top-level name now fails to load. Your job is the `.wat` half.

**Only args and let-bindings may be bare.** Every top-level name (`def`, `defn`, `defrecord`, `defenum`,
`defsurface`, `defstruct`, `defmacro`, `defclause`, `defrule`, `typealias`, …) must carry a namespace.
The only forbidden namespaces are `:wat::` and `:rust::` (`src/resolve/reserved.rs:25-27`); everything
else is free.

## Scope

**32 files — 24 `.wat` + 8 `.wat.bad`, ~75 occurrences.** The path list is in
`docs/arc/2026/06/278-rules-engine/NOTE-bare-name-dispositions.md` plus the `def`-form files it predates;
**do not trust that list as complete** — it has been wrong three times. Discover the set yourself and
report what you found. `.rs` files are NOT yours (one macro-emitted name is being hand-fixed separately).

## This is a wat-fix codemod. Not hand edits, not python, not sed.

`wat/fix.wat` is the framework; `wat-scripts/fixes/*.wat` are the recorded migrations. **Copy
`wat-scripts/fixes/namespace-defrule-names.wat` as your shape** — it did this exact job for the
`where` corpus and its header documents the discipline. `wat::fix::rename-keyword-exact` is the
whole-token rename you want: it moves the definition *and* every call site in one pass, because both
spellings are the same keyword token.

Write `wat-scripts/fixes/namespace-bare-top-level-names.wat`.

## The transformation — table-free, per file

1. **Derive the file's namespace** from the file's own first top-level name that already contains `::`
   (excluding `:wat::`). `rw::Bag::Op` ⇒ `rw`. **Never a hand-kept path→namespace table** — that is the
   discipline `namespace-defrule-names.wat` set, and it is why this stays one shot.
2. **If the file has no namespaced name at all** (7 of the 32 — six `wat_arc157_def*` and
   `probe_arc251_read_file_ladder__content.wat`): mint from the file's **basename stem**, `_`→`-`.
   Per-file, never shared: several of those files each define `:pi`, so one shared namespace would
   rebuild the very collision this removes.
3. **Per top-level name:**
   - contains `::` → leave untouched
   - contains `/` and no `::` → **swap the first `/` for `::`** (`:rw/try` → `:rw::try`). `/` is the
     *accessor* form (`Thread/join-result`, `CONVENTIONS.md:45`), so `:rw/try` claims a member of a type
     `rw` that does not exist. The swap makes the fn agree with the types already in that same file.
   - otherwise → prefix `:<ns>::` (`:get-config` → `:a157-def::get-config`)

Express the work as a **`foldl` over a `Vector` of tuples, never a nested staircase** — 24t's lesson;
the staircase's paren count stopped being eyeballable and was wrong twice.

## STOP triggers — rejection criteria. Ship nothing, report.

- **STOP-1 — a collision survives or is created.** Two files deriving/minting the same namespace *and*
  defining the same name. HALT, name the pair. Dissolving these collisions is half the point; do not
  let one through.
- **STOP-2 — a `.wat.bad` file's rename would change WHY it fails.** These 8 are deliberately-bad
  specimens (redef-forbidden, type-mismatch, Liskov, assignability). Each must keep failing for **its
  own** reason. If a bare name IS the specimen — the thing the file exists to pin — HALT and name it;
  renaming it would leave a bad program that is no longer bad in the way it claims.
- **STOP-3 — the dry-run diff shows any change outside a keyword token.** An eaten comment, a moved
  span, a touched string literal. HALT. (`drop-deftest-prelude.wat` had exactly this bug and the
  dry-run+diff caught it.)

## Gate

1. **Dry-run onto a `/tmp` copy of the tree and `diff` it. Read the diff before applying.** Mandatory.
2. Apply: `printf '[...paths...]\n' | cargo wat ./wat-scripts/fixes/namespace-bare-top-level-names.wat`
3. **Prove idempotence** — re-run; second pass must report zero changes.
4. `./target/release/wat --check <file>` on every touched **`.wat`** → clean.
5. **★ Every touched `.wat.bad` must STILL FAIL, and for the SAME REASON.** Capture each one's failure
   output *before* and *after* and diff the reason. A `.bad` file that starts passing, or that fails
   with `UnnamespacedName` instead of its own error, is STOP-2 and you have broken a specimen.
6. `:wat::deporder::verify-stdlib` prints `[]`.

**Do NOT run `cargo nextest`** — the orchestrator weighs the floor centrally. Do not commit, push, stash
or revert.

## Report

The codemod path; the discovered file set and how it differs from the note's list; the dry-run diff
summary; the derived-vs-minted namespace per file; the idempotence result; the `--check` results; **the
before/after failure reason for each of the 8 `.wat.bad`**; and any STOP with its evidence.

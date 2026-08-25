# BRIEF — STONE E: `:wat::core::string::*` → `:wat::string::*`

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-E-the-string-home.md` — read it whole,
first. Read its **⊘ CORRECTED** note in the bootstrap section: an earlier draft prescribed a
three-commit alias window and it was wrong. The sequence below is the one that ships.

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every command in the FOREGROUND and block on it. Your turn ends when the
numbers are in your hands, not when a command is launched.

**You may not spawn sub-agents.**

Anchor: `/home/john/work/holon/wat-rs`. `pwd` first. You do not commit, push, stash, revert, or
checkout — leave your work uncommitted and report. `cargo build --release` is yours (~19s
incremental, longer for a wide `src/` change). `cargo nextest`, `scripts/floor.sh` and clippy are
NOT — the orchestrator takes those centrally.

## The work in one paragraph

Move every string verb from `:wat::core::string::` to `:wat::string::`, and its rete mirror from
`:wat::rete::core::string::` to `:wat::rete::string::`. The `.wat` corpus moves by a **wat-fix
codemod**; the Rust side moves by hand across **seven doors**. The tree is broken in the middle and
green at the end — that is expected and is the documented pattern, not a problem to route around.

## The sequence — in this order, and the order is the whole trick

1. **Write `wat-scripts/fixes/rename-core-string-to-string.wat`** using the OLD verb names. It must
   load and run against today's binary. Copy `wat-scripts/fixes/rename-kernel-to-spawn.wat` as the
   shape — it re-parented a namespace exactly this way and its header documents the discipline.
   Two prefix renames, both FULL-name:
   ```
   ":wat::core::string::"        ->  ":wat::string::"
   ":wat::rete::core::string::"  ->  ":wat::rete::string::"
   ```
   ⚠ Order matters *within* the file: rename the **rete** prefix FIRST. `:wat::core::string::` is
   not a prefix of `:wat::rete::core::string::`, so they are disjoint and order is in fact
   irrelevant — **verify that claim on a `/tmp` copy rather than trusting this sentence.**
2. **Dry-run on a `/tmp` copy and `diff`** — the recorded discipline (R21). Confirm the rewrite is
   exactly the intended structural change and nothing else moved.
3. **Run it over the whole corpus, INCLUDING ITSELF.** `rename-prefix-edits` rewrites *"for every
   keyword LEAF"* (`wat/fix.wat:716`), so the codemod's own verb CALLS migrate while its
   string-literal ARGUMENTS do not. Derive the path list from git, do not hand-write it:
   ```
   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/'
   ```
4. **Now the tree is broken** — it calls names that do not exist. Rename the seven Rust doors and
   `wat/string.wat`. Table in the DESIGN; counts are `runtime.rs` 44, `string_ops.rs` 32,
   `check.rs` 31, `macros/eval.rs` 18, `rete/expr_ir.rs` 10, `rete/vocabulary.rs` 8,
   `rete/purity.rs` 3, `wat/string.wat` 19.
5. **Build.** Iterate against the compiler until green — the diagnostics ARE the worklist
   (`docs/SUBSTRATE-AS-TEACHER.md`). A large fail count here is the progress meter, not a crisis.

## ⛔ THE HAZARD THIS STONE IS BUILT AROUND

`:wat::core::String` (capital S — the TYPE) and `:wat::core::string::` (lowercase, trailing `::`)
share the parent `:wat::core::`. **The trailing colons are the only thing keeping the rename off the
type.** Use the full name as the prefix, exactly as `rename-kernel-to-spawn.wat`'s header insists,
and row 5 is the control that proves you did.

## The acceptance rows YOU run

- **Row 1 — PER-DOOR, not aggregate.** For each of the seven Rust files AND `wat/string.wat`:
  `grep -c ':wat::core::string::' <file>` is **0**. Report all eight numbers. A total that reads
  zero while one door still holds the old name is exactly the lie this row exists to catch.
- **Row 2 — the corpus is clean.** `git ls-files '*.wat' | xargs grep -l ':wat::core::string::'`
  returns nothing.
- **Row 3 — the TYPE is untouched.** `:wat::core::String` count is IDENTICAL before and after.
  Capture the number before you start. This is the negative control for the shared-parent hazard.
- **Row 4 — idempotent.** Re-run the codemod over the same path list: zero changes
  (`git status` unchanged).
- **Row 5 — the old name resolves to nothing.** A probe calling
  `(:wat::core::string::length "x")` must fail with `UnknownFunction`.
- **Row 6 — the new name works.** The same probe with `(:wat::string::length "x")` returns 1.
- **Row 7 — the rete mirror moved and its wall is real.** `(:wat::rete::string::length …)` compiles
  inside a rule. Then **deliberately break one `RETE_OPS` row's module and confirm
  `src/rete/vocabulary.rs:1565` screams** — then restore it. A wall nobody has watched fire is a
  claim, not a wall. Report the assertion text verbatim.
- **Row 8 — `(:wat::deporder::verify-stdlib)` returns `[]`.**

Report each row's command and output **verbatim** — never a summary, never a `| head`/`| tail`
window. A row you could not run is reported as not-run, never as passed.

## Blast radius

- `wat-scripts/fixes/rename-core-string-to-string.wat` — created (the recorded migration; it is
  committed as an artifact, not deleted)
- the seven Rust doors + `wat/string.wat` — edited
- every `.wat` under `wat/`, `wat-scripts/`, `wat-tests/`, `tests/` carrying the old prefix

Nothing else. No new verbs — `=` / `not=` for String belong to the sibling stone and DO NOT EXIST
yet; if you find yourself adding one, STOP.

## STOP triggers — each ships NOTHING and surfaces the gap

1. **The dry-run diff shows a `:wat::core::String` (capital S) touched.** STOP. The prefix guard
   failed and the type is being renamed. Report the diff hunk verbatim.
2. **A Rust door has hits the DESIGN's table does not predict** — a count materially above the
   listed number, or an eighth file. STOP and report the file and count; the door census may be
   incomplete and that is an orchestrator finding.
3. **The build cannot be driven to green** after the seven doors — a verb that dispatches but is not
   registered, or vice versa. STOP and report the exact diagnostic; do not add a compatibility
   shim.
4. **Row 7's wall does NOT fire** when you break a row. STOP — that is a bigger finding than this
   stone, and it means the rete half has no guard at all.

A STOP means: leave the tree as it is, write the report, end your turn. It is never a licence to
ship a smaller version of a row.

## What you own that nobody can reconstruct

The per-door numbers, the dry-run diff's character, row 7's assertion text, the build's fail-count
trajectory as you drove it down, and anything that surprised you — a door the table missed, a site
the codemod could not reach, a name that turned out to be spelled two ways.

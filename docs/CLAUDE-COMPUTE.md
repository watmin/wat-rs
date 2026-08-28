# claude-compute — the integration branch

An **observation post**, not a destination. It exists so `main` and `grok-rete`
can keep diverging while someone watches what their union actually does. It is
**never merged back into either parent.** When the two finally agree, delete it
and cut a fresh one.

Refresh it with `~/opt/bin/wat-sync.sh` (lives outside the repo on purpose, so
this branch stays a pure merge-of-both plus recorded integration fixups).
`wat-sync.sh --status` reports divergence without touching anything.

## What a clean text merge does NOT catch

`git merge-tree` reported **no conflicts** for the first union, and the result
still took **3182 of 5103 floor tests red on a single panic.** Textual
cleanliness says nothing here, because the two branches break each other
*semantically*: main renames a namespace across 1000+ files while grok-rete
writes new code against the old one. Three instances so far, all the same shape:

1. **`include_str!` path vs a file move.** main's arc 255 moved
   `src/stdlib.rs` → `src/load/stdlib.rs` and rewrote its 19 include paths to
   `../../`. grok-rete added a 20th entry (`wat/gen.wat`) at `../` in the old
   layout. Git took main's 19 and grok-rete's 1. Caught by the compiler.

2. **A `.wat` corpus main's codemod never saw.** Arc 255 dropped the `core`
   segment (`:wat::core::string::*` → `:wat::string::*`, same for i64/f64, and
   `:wat::rete::core::*` → `:wat::rete::*`). main moved its own corpus with the
   recorded codemods and updated the F5 pure-combinator allow-list. But
   `wat/gen.wat` exists only on grok-rete, so main's codemod run never touched
   it — and gen.wat is *baked into the binary*, so the stdlib would not build.

3. **The same miss, one directory over.** The three `tests/rete/*.wat` probes
   the merge introduced carried `:wat::rete::core::` too. Only the floor found
   them; the build was green.

**The class:** any `.wat` living on ONE side while the OTHER renames a
namespace. `wat-sync.sh` now gates on it, scoped to the files a refresh actually
brought in — a repo-wide scan is wrong, because both spellings are deliberately
live on main (Stone A-i), the recorded codemods carry old names as *data*, and
`255-stone-a-i-both-*-spellings.wat` asserts both on purpose.

## Fixing a `.wat` breakage — use the recorded codemods

Never hand-edit, never sed (`CLAUDE.md`). The three that apply here:

    wat-scripts/fixes/rename-core-string-to-string.wat
    wat-scripts/fixes/rename-core-numerics-to-their-homes.wat   # excludes rete
    wat-scripts/fixes/rename-rete-numerics-to-their-homes.wat   # rete clone only

Census first (`--grep` prints matches unapplied), then apply. Count
*occurrences*, not lines — the finder emits one long line and `grep -c`
undercounts. Comments are not rewritten: the codemod walks the form tree, so
prose references to old names are a separate, manual pass.

**BOOTSTRAP.** `wat/gen.wat` is baked into the binary via `include_str!`, so
when it is the broken file the tool cannot boot to fix it. Invert the
stash-dance in `wat/fix.wat`'s header: comment out gen.wat's `WatSource` entry
in `src/load/stdlib.rs`, `cargo build --release --bin wat`, run the codemods,
restore the entry, rebuild.

## Recurring-conflict rule for `.wat`

Carrying a namespace migration as a committed diff means it fights grok-rete's
every future edit to the same file. Prefer, on conflict in a `.wat`: take
**grok-rete's** version wholesale, then **re-run the codemods**. The migration
is *derived*, not merged — so it never accumulates. `rerere` is enabled
(`rerere.enabled`/`autoupdate`) and has the preimages banked.

## Read the floor, not the exit code

`scripts/floor.sh` captures before anyone reads. A red is a red — do not re-run
to make it go away; re-running *after a fix* is the normal cycle. Two traps hit
in this branch's own history:

- **Backgrounding a build hides its verdict.** `nohup cargo … &` returns the
  wrapper's exit code, not cargo's. A "build succeeded" that was really `echo`
  succeeding cost one wrong claim here. Append `; echo "EXIT=$?"` and read that.
- **"is not pure" is a documented lie.** `src/rete/purity.rs:89` says so
  outright: the op IS pure — it is refused only because the name is not a
  registered *rete* name. Read it as "not from rete", or you will chase purity.

## Known red, attributed — not this branch's

`wat::cli retirement_table_reachable::retirement_table_is_fully_reachable`
TIMEOUTs at the 30s cap under full-floor parallel load; run alone it passes in
**18.681s**. Attribution: **main alone** touched `src/remedy/retirement.rs` and
`tests/cli/retirement_table_reachable.rs` since the merge-base — grok-rete
touched neither. It is main's own, and it will intermittently red main's floor
on a loaded box. Not a flake to bless: a real 30s-boundary fragility.

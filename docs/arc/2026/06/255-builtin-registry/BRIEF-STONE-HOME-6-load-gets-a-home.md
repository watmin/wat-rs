# BRIEF — HOME #6: load gets a home

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-HOME-6-load-gets-a-home.md` — read it whole.

PRIOR ART, and it is the same operation one stone ago: **HOME #5** (`8ddccaaa3`, `src/edn/`). Read its
design and its commit message — the shape, the traps, and the two things that went wrong are all
there. This is that, smaller, plus one deletion.

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first and stay there.

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Every command in the
FOREGROUND, blocking. Your turn ends when the numbers are in your hands.

**You may not spawn sub-agents.** Do not commit, push, stash, revert, or `git checkout`. There is a
`git stash@{0}` that must never be touched. **Use `git mv`** so each move is recorded as a rename.

You may run `cargo build --release` and `cargo build --release --all-targets` — your worklist
generator — and single named tests. **Not** the full floor, **not** clippy.

---

## The work in one paragraph

Four loose files at `src/` root are all about **getting source text into the runtime**, and there is
no `src/load/`. Give them one — and **delete the fourth**, which contains no code at all. This is a
move and a path rename: no logic changes, no new types, no behaviour.

```
src/load.rs    1894  ->  src/load/loader.rs
src/stdlib.rs   884  ->  src/load/stdlib.rs
src/source.rs    76  ->  src/load/source.rs
src/sandbox.rs   13  ->  DELETED
```

```
crate::load::X    ->  crate::load::loader::X
crate::stdlib::X  ->  crate::load::stdlib::X
crate::source::X  ->  crate::load::source::X
wat::load::X      ->  wat::load::loader::X      (same for the other two, in tests/)
```

`src/lib.rs` loses four `mod` lines and gains `pub mod load;`.

---

## ⛔ THE DELETION — verify it before you do it

`src/sandbox.rs` has **zero items**: `grep -cvE '^\s*(//|$)' src/sandbox.rs` returns 0. Thirteen
lines, all `//!`. Its own header says both its functions were retired (arc 105c, arc 298) and *"The
module remains as a namespace anchor."* Its only reference in the tree is `lib.rs`'s `pub mod
sandbox;`.

**Run those two checks yourself before deleting.** If either disagrees with this brief, that is
STOP-3 — a file with a live item is not a corpse.

★ **Its header is real history and must not be lost.** Fold its substance — that
`resolve_sandbox_loader` and the `src/spawn.rs` callers were retired in arc 298, and that the
canonical sandbox is now `wat/kernel/sandbox.wat` — into `src/load/mod.rs`'s module doc. A reader
looking for the sandbox loader must land somewhere true.

---

## ⛔ NO RE-EXPORTS in `src/load/mod.rs`

Same ruling as HOME #5, same reason. `wat::load::InMemoryLoader` is the shortest path and therefore
the tempting one; a `pub use loader::InMemoryLoader;` would mint a **second path to one item**. One
way. If a call site reads badly that is a `use` at the top of ITS file. STOP-2.

## ⚠ MOVE FILES WHOLE — do not split `load.rs`

`load.rs` carries `LoadFetchError` / `LoadError` / `LoadErrorKind`, and every sibling home
(`check/`, `types/`, `resolve/`, `edn/`) carves an `error.rs`. **This one will want one too, and it
is not this stone.** HOME #5 cut splitting `render.rs` on exactly this ground. If you find yourself
creating `src/load/error.rs`, that is STOP-1.

---

## The cascade — measured

```
              src/   non-src   TOTAL
load            42        15      57
stdlib          17         1      18
source           9         4      13
                              ──────
                                  88
```

A fifth of HOME #5's. **20 of the 88 live outside `src/`** — `tests/` is a separate compilation
unit, and `cargo build --release` alone cannot see them. Use `--all-targets`, and say in your report
which build produced each number. That is the trap HOME #5 named and did not spring.

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — you are creating `src/load/error.rs`,** or otherwise splitting a moved file. Out of
   scope; report the pressure.
2. **STOP-2 — you are adding a `pub use` to `src/load/mod.rs`.** Report the call site that felt bad.
3. **STOP-3 — `src/sandbox.rs` has an item, or something references it.** Do not delete it; report
   what you found.
4. **STOP-4 — the cascade does not converge.** Report the remaining errors verbatim.
5. **STOP-5 — a room's line number does not hold what this brief says.** Written against `c6926502e`.

---

## Acceptance you can check yourself

```bash
ls src/*.rs | wc -l                                  # 31 at HEAD -> 27
ls src/load/                                         # mod.rs loader.rs stdlib.rs source.rs
test -e src/sandbox.rs && echo "STILL THERE" || echo "deleted"
grep -n 'mod sandbox' src/lib.rs                     # -> nothing
git ls-files '*.rs' | xargs grep -c 'crate::load::[A-Z]\|crate::stdlib::\|crate::source::\|wat::load::[A-Z]\|wat::stdlib::\|wat::source::' | grep -v ':0$'   # -> nothing
grep -n 'pub use' src/load/mod.rs                    # -> nothing
cargo build --release --all-targets
```

⚠ Validate any pattern you invent before quoting its count. Two of my own censuses today returned
confident wrong numbers — one from `\|` inside a `grep -E` (a literal pipe, not alternation), one
from grepping `wat::` across `src/` too and calling the result "external". Positive-control against
a file you know carries a hit.

## Report back with

- **The cascade's waterfall**, and for each number, whether it came from `--all-targets` or the lib
  alone.
- The acceptance checks above, after.
- Confirmation that the three moves show as **renames** in `git status`, and that the deletion shows
  as a deletion.
- The two `sandbox.rs` checks you ran before deleting, with their output.
- **Every site you edited that was not a path rename**, with `file:line`. There should be one at
  most — `lib.rs`'s `mod` block — and if there are others they are the most interesting thing in
  your report.
- Anything the brief got wrong.
- What you did NOT do, and why.

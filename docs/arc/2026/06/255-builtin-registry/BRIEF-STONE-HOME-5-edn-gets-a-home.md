# BRIEF — HOME #5: EDN gets a home

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-HOME-5-edn-gets-a-home.md` — read it whole.

PRIOR ART, and it is the same operation: **arc 300 stone D** (`WatAST::CharLit`) ran a compiler-named
cascade 19 → 0 in one pass. Read `docs/SUBSTRATE-AS-TEACHER.md` before you start; this is that method
with `rustc` as the worklist generator.

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first and stay there.

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Every command in the
FOREGROUND, blocking. Your turn ends when the numbers are in your hands.

**You may not spawn sub-agents.** Do not commit, push, stash, revert, or `git checkout`. There is a
`git stash@{0}` that must never be touched.

**Use `git mv` for the file moves**, so the rename is recorded as a rename and the diff shows `R100`
— a move that reads as delete-plus-add loses the file's history.

You may run `cargo build --release` — it is your worklist generator — and single named tests. **Not**
the full floor, **not** `cargo clippy`; the orchestrator runs those centrally.

---

## The work in one paragraph

Five loose files at `src/` root are all EDN, ~6,994 lines, and there is no `src/edn/`. Give them
one. This is a **move and a path rename** — no logic changes, no new types, no behaviour. Then
`cargo build --release` reports every stale path and you fix them until it is zero.

```
src/edn_shim.rs           -> src/edn/render.rs          "shim" says TEMPORARY; it renders
src/wat_edn_bridge.rs     -> src/edn/bridge.rs
src/to_edn.rs             -> src/edn/contract.rs        its own doc: "the ONE serialization contract"
src/runtime_error_edn.rs  -> src/edn/error.rs
src/to_edn_derive_tests.rs-> src/edn/derive_tests.rs    stays #[cfg(test)]
```

```
crate::edn_shim::X          ->  crate::edn::render::X
crate::wat_edn_bridge::X    ->  crate::edn::bridge::X
crate::to_edn::X            ->  crate::edn::contract::X
crate::runtime_error_edn::X ->  crate::edn::error::X
wat::edn_shim::X            ->  wat::edn::render::X      (same for the other three, in tests/)
```

`src/lib.rs` loses five `mod` lines (76, 96, 116, 117–118, 120) and gains `pub mod edn;`.
`src/edn/mod.rs` declares the five submodules.

---

## ⛔ THE ONE THING TO GET RIGHT — no re-exports

**Do NOT add `pub use contract::ToEdn;` (or any other re-export) to `src/edn/mod.rs`.**

`ToEdn` is named **51 times** and `crate::edn::ToEdn` is shorter than `crate::edn::contract::ToEdn`,
so the re-export is exactly the tempting move. It would mint a **second path to one item** — two ways
to say the same thing, which this house does not do. The extra segment is the price of not creating
a synonym, and it is cheap.

If a call site reads badly, that is a `use` statement's job at the top of the file, not a re-export
in the home's root. STOP-2 if you find yourself adding one.

---

## THE CASCADE — measured, so you know when you are done

```
crate::edn_shim              29 files, 112 occurrences
crate::to_edn                27 files, 163
crate::wat_edn_bridge         6 files,  14
crate::runtime_error_edn      2 files,   3
                internal    ~292

wat::edn_shim 30 · wat::to_edn 20 · wat::wat_edn_bridge 5 · wat::runtime_error_edn 2
                external      57   across 37 files under tests/
                ─────────────────
                TOTAL       ~349 across ~72 files
```

**The count after the first build is the progress meter, not a crisis.** Fix a category, rebuild,
watch it fall. Do not enumerate the sites up front.

⚠ `render` and `bridge` are **mutually recursive** (`edn_shim` → `wat_edn_bridge` → `edn_shim`).
Under one parent that is `super::bridge::` / `super::render::` and costs nothing — but they must stay
siblings; do not try to layer one under the other.

⚠ `tests/` is a **separate compilation unit**. A green `cargo build --release` does NOT mean the 57
external occurrences are fixed — those only surface when the test targets build. Use
`cargo build --release --all-targets` (or a scoped `nextest` run) or you will report done at 292 of
349. This is the trap in this stone.

---

## Blast radius

`src/` (the five files, `lib.rs`, and every internal caller) and `tests/` (37 files, path renames
only). **No `.wat` corpus edits. No logic changes. No new types.** If you are editing a function
body for any reason other than a path, stop and read STOP-1.

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — a change that is not a path or a `mod` declaration.** This stone moves files and
   renames paths. A logic edit, a signature change, or a visibility widening beyond what the move
   itself forces is out of scope; report what demanded it.
2. **STOP-2 — you are about to add a `pub use` to `src/edn/mod.rs`.** See above. Report the call site
   that felt bad instead.
3. **STOP-3 — a cyclic-module or visibility error you cannot resolve by pathing alone.** `render` and
   `bridge` referring to each other is expected and fine; anything else is a finding about the
   family's real shape and I want it before a workaround.
4. **STOP-4 — the cascade does not converge**, i.e. the error count stops falling. Report the
   remaining errors verbatim.
5. **STOP-5 — a room's line number does not hold what this brief says.** Written against `7b7674cee`.

---

## Acceptance you can check yourself

```bash
ls src/*.rs | wc -l                                   # 36 at HEAD -> 31
ls src/edn/                                           # mod.rs + the five
git ls-files '*.rs' | xargs grep -c 'crate::edn_shim\|crate::to_edn::\|crate::wat_edn_bridge\|crate::runtime_error_edn' | grep -v ':0$'   # -> nothing
git ls-files '*.rs' | xargs grep -c 'wat::edn_shim\|wat::to_edn::\|wat::wat_edn_bridge\|wat::runtime_error_edn' | grep -v ':0$'           # -> nothing
grep -n 'pub use' src/edn/mod.rs                      # -> nothing (STOP-2)
cargo build --release --all-targets                   # ALL targets, not just the lib
```

## Report back with

- **The cascade's waterfall** — the error count after each rebuild, in order, and say explicitly
  whether each number came from `--all-targets` or the lib alone. That distinction is where this
  stone hides its remaining work.
- The four `grep` counts above, after.
- `ls src/*.rs | wc -l` before and after.
- Confirmation that `git status` shows the five moves as **renames** (`R100`), not delete+add.
- **Every site you edited that was not a path rename**, with `file:line` — there should be none, and
  if there are, they are the most interesting thing in your report.
- Anything the brief got wrong. It was written by someone who has been wrong about this corpus
  repeatedly today.
- What you did NOT do, and why.

# BRIEF — STONE: a prose citation names a SYMBOL, not a LINE

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-a-prose-citation-names-a-symbol-not-a-line.md`
— read it whole, and read the table of twelve line-citations before you touch anything.

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first and stay there.

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Every command in the
FOREGROUND, blocking. Your turn ends when the numbers are in your hands.

**You may not spawn sub-agents.** Do not commit, push, stash, revert, or `git checkout`. There is a
`git stash@{0}` that must never be touched.

You may run `./target/release/wat --check <file>` on `.wat` files you touch (~0.15s each) and
`cargo build --release` if you want reassurance. **Not** the full floor, **not** clippy.

⚠ **This stone changes COMMENTS AND DOCS ONLY.** If you are editing an expression, you have left the
blast radius — read STOP-1.

---

## The work in one paragraph

HOME #5 renamed five modules. **118 prose references across 61 files still name the old ones.** Twelve
of those carry a line number, and **eight of the twelve are already wrong** — they pointed at the
wrong line *before* the rename, because the files' contents never moved (git recorded 92–99%
similarity). Rename every reference, and **delete every line number**, replacing it with the symbol
that line meant.

```
edn_shim            96  ->  edn::render      /  src/edn/render.rs
runtime_error_edn   16  ->  edn::error       /  src/edn/error.rs
wat_edn_bridge       4  ->  edn::bridge      /  src/edn/bridge.rs
to_edn_derive_tests  2  ->  edn::derive_tests
```

---

## ⛔ THE ONE THING TO GET RIGHT — drop the number even where it is currently CORRECT

Four of the twelve still point at the right line today. **Drop those numbers too.**

Keeping the four that happen to work teaches that a line citation is fine when you are careful — which
is exactly the belief that produced the other eight. **A citation names what it means.** A symbol
survives every edit that does not rename it, and when it *does* die it dies findably: a reader greps
the name and gets zero hits, instead of a confident pointer at `None => {`.

Renaming a wrong citation without removing its number is **strictly worse than leaving it alone**: the
line stays wrong while the staleness cue — the old module name — disappears.

### The twelve, already tested for you

```
STILL TRUE (drop the number anyway)
  edn_shim.rs:132         pub fn eval_edn_write_json_natural(
  edn_shim.rs:144         /// `(:wat::edn::read s)` → `:T`
  runtime_error_edn.rs:64 impl ToEdn for RuntimeError
  wat_edn_bridge.rs:22    a //! doc line

ALREADY FALSE — find what each MEANT, do not preserve where it points
  edn_shim.rs:1773  ×5    now `None => {`   — claimed "refuses every Edn::Symbol"; that arm is at 2007
  edn_shim.rs:1899  ×2    now `F: FnMut(Span) …`
  edn_shim.rs:964         now `WatAST::Vector(…)`
  edn_shim.rs:2651        now `TypeExpr::Fn {…}`
  edn_shim.rs:1008        now `other => return Err`
  edn_shim.rs:3490        now `EdnReadErrorKind::Other(format!(`
  edn_shim.rs:105         now `args: &[WatAST],`
  edn_shim.rs:191         now a RuntimeError ctor
```

★ **Re-derive each symbol; do not guess.** Read the sentence around the citation, work out what it was
pointing at, then find that thing in `src/edn/render.rs` (or `error.rs` / `bridge.rs`) and name it. **A
wrong symbol is worse than a wrong line**, because a symbol reads as authoritative. If you cannot
determine what a citation meant, that is STOP-2 — report it rather than inventing one.

★★ The worked example is inside the file itself: `src/edn/render.rs:682`'s own comment says *"the
`Edn::Symbol` arm ~:1440"* — a **third** wrong number for the same arm, written in the file it
mis-cites. Fix it in this pass.

---

## The `.wat` half is comments, and you edit those by hand

Twenty of the 61 files are `.wat`, and every reference is inside a `;;` comment. **A comment is not a
node** — `wat/grep.wat` builds its facts from the form tree, so a rules codemod cannot see one, by
construction. R21's "never hand-edit `.wat`" governs *structural* rewrites; this is prose the tool
provably cannot reach, and stone E set the precedent. `--check` each `.wat` you touch.

---

## Blast radius

Comments and doc-comments across `src/`, `crates/`, `tests/`, `wat/`, `wat-scripts/`, and the three
`.md` / `.sh` / `.intueri` files. **`docs/arc/**` IS UNTOUCHED** — a past INSCRIPTION citing
`edn_shim.rs` is a true statement about the world when it shipped.

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — you are about to change an expression.** Comments and docs only.
2. **STOP-2 — you cannot determine what a line citation MEANT.** Report the citation and the
   sentence around it. Do not invent a symbol; a confident wrong symbol is worse than the wrong
   number it replaced.
3. **STOP-3 — a `.wat` file stops `--check`ing** after your edit. A comment edit cannot do that; if
   it did, report the file.
4. **STOP-4 — a reference is inside `docs/arc/**`.** Leave it; report if the count forces you to
   touch one to reach zero.
5. **STOP-5 — a room does not hold what this brief says.** Written against `8ddccaaa3`.

---

## Acceptance you can check yourself

```bash
# 118 at HEAD, across 61 files -> 0 (excluding docs/arc)
git ls-files | grep -vE '^docs/arc/' \
  | xargs grep -ohE 'edn_shim|wat_edn_bridge|runtime_error_edn|to_edn_derive_tests' 2>/dev/null | wc -l

# 12 distinct line-citations -> 0
git ls-files | grep -vE '^docs/arc/' \
  | xargs grep -ohE '(edn_shim|wat_edn_bridge|runtime_error_edn)\.rs:[0-9]+' 2>/dev/null | wc -l

# and no NEW line-citations of the moved modules crept in
git ls-files | grep -vE '^docs/arc/' \
  | xargs grep -nE 'edn/(render|bridge|error)\.rs:[0-9]+' 2>/dev/null
```

⚠ Validate any pattern you invent before you quote its count. Mine returned a confident **0** earlier
today because I wrote `\|` inside a `grep -E` — BRE alternation in an ERE, matching a literal pipe. I
nearly reported the tail already clean. Positive-control against a file you know carries a hit
(`src/capability/registry.rs` has 11).

## Report back with

- The two counts above, before and after.
- **A line per replaced line-citation**: the old `file.rs:NNN`, the symbol you replaced it with, and
  **how you determined that symbol**. This is the part I will read most carefully — it is where a
  confident wrong answer would hide.
- Any citation you could not resolve (STOP-2), with its surrounding sentence.
- The `.wat` files you hand-edited, and confirmation each still `--check`s.
- Anything the brief got wrong.
- What you did NOT do, and why.

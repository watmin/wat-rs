# Arc 283 — lift `SourceFile` out of deporder → `:wat::source::File` (dogfood the rename)

> **STATUS: STRIKE-READY (2026-06-17).** RED probe `tests/probe_arc283_source_file_lift.rs`
> (`#[ignore]`'d): `:wat::source::File` undefined at HEAD. A decomplect (solvere): the generic
> source-unit struct, born in `deporder` (arc 275), is the universal input to every source-processing
> tool — it lifts to a neutral home. **The rename is dogfooded through `fix::rename-keyword-prefix`** —
> the toolchain renaming its own symbol across the corpus.

## Why

`SourceFile {path, source}` (`deporder.wat:25-27`) is consumed cross-namespace by `lint` (8×) and will
be consumed by the sweep + arc 282 (Rust facts). It is not a deporder concept — it is "a file = path +
text," the input to analyze/lint/fix/format/codemod. Born where first needed; outgrew that home. The
longer we wait, the more refs accrete on `:wat::deporder::SourceFile`.

## The name (intueri, blessed)

**`:wat::source::File`** in **`wat/source.wat`**. intueri's reasoning: `:wat::source::SourceFile`
*stutters* — and the second "source" collides with the record's own `source` FIELD
(`source::SourceFile/source` = three "source"s for two concepts). `:wat::source::File` lets the
namespace carry the domain ("source code") and the type name the shape (a File = path + text); accessors
`File/path` / `File/source` read as plain English. (`src` rejected — abbreviation fails Obvious;
`Unit` rejected — too abstract.) `deporder` keeps `Violation` + `SymDef` (genuinely its own).

## The migration (dogfood-first, then relocate)

**Order matters — build the renaming binary at HEAD first, run the dogfood, THEN move the def + rebuild.**

1. **Build at HEAD** — `cargo build --release --bin wat` (the binary that runs the dogfood carries the
   current consistent stdlib; it only reads/writes files).
2. **Dogfood the rename** — write a one-shot driver `wat/_rename_sourcefile.wat`:
   ```clojure
   (:wat::core::defn :user::rename-file [path <- :wat::core::String] -> :wat::core::nil
     (:wat::core::let [src   (:wat::io::read-file path)
                       fixed (:wat::fix::rename-keyword-prefix ":wat::deporder::SourceFile" ":wat::source::File" src)]
       (:wat::io::write-file path fixed)))
   (:wat::core::defn :user::main [] -> :wat::core::nil
     (:wat::core::do
       (:user::rename-file "wat/deporder.wat")
       (:user::rename-file "wat/lint.wat")
       (:user::rename-file "wat-tests/lint.wat")
       nil))
   ```
   Run `./target/release/wat wat/_rename_sourcefile.wat`. This renames the def head AND every ref +
   accessor (`…SourceFile/path` → `…File/path`) comment-faithfully across the three `.wat` files. **READ
   the diff of each** — only the prefix may have changed (the dogfood proof).
3. **Relocate the def** — the renamed `(:wat::Record::def :wat::source::File [path source])` is now
   sitting in `deporder.wat`. CUT it out → create `wat/source.wat` holding it (with a module doc
   comment). deporder.wat keeps `SymDef`.
4. **Register `wat/source.wat` in `src/stdlib.rs` BEFORE deporder.wat** (it must load first — deporder
   now references `:wat::source::File`). After `core.wat`, before `deporder.wat`.
5. **Rust fixtures (manual — the `.wat` codemod can't reach `.rs` strings):** rename
   `:wat::deporder::SourceFile` → `:wat::source::File` in the three probe fixtures
   (`tests/probe_arc277_lint_if_ladder.rs`, `probe_arc277_lint_concat_abuse.rs`,
   `probe_arc277_1b_ladder_autofix.rs`).
6. **Delete `wat/_rename_sourcefile.wat`** (the one-shot driver; not committed).
7. **Un-ignore** `tests/probe_arc283_source_file_lift.rs`.

## Proof

- `tests/probe_arc283_source_file_lift.rs` (un-ignore): `(:wat::source::File/path (:wat::source::File
  "t.wat" "(:t::f)"))` → `"t.wat"`.
- **Zero survivors:** `grep -rn ":wat::deporder::SourceFile" wat/ tests/ src/` returns NOTHING (the rename
  was total). This is the load-bearing completeness check.
- **All floors byte-identical** (the lift is behavior-preserving): lib 929/36, deftest 260/1, nursery
  893/4, deporder gate 0. A behavior change here = a botched rename.

## Out of scope (rejected, not deferred)

- Renaming `Violation`/`SymDef` — they stay in deporder (genuinely deporder's).
- A general `wat-fix` CLI / applying the rename to arbitrary external crates — that's the portable-tool
  arc (276). This stone uses the EXISTING `rename-keyword-prefix` on our own corpus.

## Four questions

- **Obvious?** YES — `:wat::source::File` says "a source file" with no hunt; the lift puts it where any
  tool looks.
- **Simple?** YES — one record moves; the rename is one prefix-swap; `deporder` shrinks.
- **Honest?** YES — the home now matches the concept (a shared substrate type, not a deporder detail);
  and the rename is *proven total* by the zero-survivors grep, not asserted.
- **Good UX?** YES — `lint`/`sweep`/`rust-facts` all reach one neutral home; no cross-tool reach into
  deporder; `File/path`/`File/source` read clean.

## Blast radius

NEW `wat/source.wat`; EDIT `wat/deporder.wat` (def removed, refs renamed), `wat/lint.wat` (refs
renamed), `wat-tests/lint.wat` (refs renamed), `src/stdlib.rs` (register source.wat before deporder),
3 Rust probe fixtures (manual rename), un-ignore the lift probe. The `.wat` ref renames are the
DOGFOOD'S output (via `rename-keyword-prefix`), not hand-edits. No other Rust changes.

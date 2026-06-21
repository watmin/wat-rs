# ⛔ ARC 255 — CURRENT STATE (breadcrumb, 2026-06-21; replace in place)

**255 ACTIVE · 278 PARKED (255 unlocks its continuity). This file is a MAP — the truth lives in
the two DESIGN docs; read them, don't trust this summary's paraphrase.**

## Read first (the authoritative specs)
- **`DESIGN.md` § "LOCKED RECORD MODEL"** — the registry record model: `baseline ⊕ per-kind *Def
  ⊕ per-kind *Meta`; `MetaField<T>=Unspecified|Specified(T)`; registry IS `sym`; FnDef split.
- **`DESIGN-intrinsic-doc-reflection-contract.md`** — the doc + reflection contract (10 §):
  forced `#[wat_intrinsic]` comment contract · mutual code⇄doc checks · `show-source` · enum-valued
  closed fields · Clojure vocab map · wiki=registry-projection · uniform-across-kinds + shared parser.

## Why we pivoted (catastrophic, grounded)
Resolver blanket-accepts ANY `:wat::*` head (`is_reserved_prefix → true`, walk.rs:189) + checker
punts via permissive `Infer` (check.rs:9923) → a typo'd builtin type-checks clean, dies only at
runtime. Double-punt. Builder: annihilation.

## DONE (committed, pushed; floor lib 953/36/1, warnings 25; last HEAD 41954a33)
- **255.1b-i** — `src/intrinsic/` registry seam (`name→handler`, OnceLock), Bytes routed. (renamed
  from provisional `src/registry/` per intueri → `intrinsic`.)
- **255.1b-ii** — `#[wat_intrinsic(":fqdn")]` proc-macro (`crates/wat-macros/src/wat_intrinsic.rs`):
  sniffs arity from the fixed-arg sig (compile_error on variadic/self/post-context), emits shim +
  `inventory::submit!`. `core::Bytes` carved to fixed-arg as the reference template.
- **255.1b-iii** — `metadata-of` answers for intrinsics (proven on Bytes, 2 probes green): `:doc`
  sniffed from `///`; baseline derived. **Emits KEYWORD values — the keyword→enum flip is pending.**
- **255.1b-iv-a** — the `wat-doc` leaf crate (`crates/wat-doc/`): the shared prose+@tag parser +
  `DocComment` model + `check_args` mutual-check, 16/16 unit tests, clippy-clean, floor untouched.
  Parity-by-construction foundation (contract §10). DESIGN-STONE-255.1b-iv-a. (`41954a33`)
- **255.1b-iv-b1** — the compile-time contract: `#[wat_intrinsic]` consumes `wat-doc` at expand
  (`sniff_args` names + `parse`→compile_error + `check_args`→compile_error), carries the structured
  doc on the registry entry, `metadata-of` renders `:doc`/`:added`/`:ret`; Bytes decorated to the full
  contract (the forcing function). Lib 954/36/1 (+1 confirmation test), clippy-clean. The
  `args`/`examples`/`deprecated`/`see` carry fields use a **dated `#[allow(dead_code)]`** (reader =
  iv-b2's seam; builder-sanctioned bounded exception, NOT the pub-leak cheat). DESIGN-STONE-255.1b-iv-b1.

## NEXT — 255.1b-iv-b2 (the wat verifier — R2's self-hosting answer) + iv-c (enum flip)
- **iv-b2** — wat verifies wat (the R2 realization; `deporder`/`verify-stdlib` template): the
  `:wat::intrinsic::examples` reflection seam (Rust, reads the carried examples → exposes to wat;
  **this is the reader that removes the iv-b1 dated allows**) + `verify-examples` (wat: `eval-ast!`
  each run=true example + `assert-eq` vs `#=>` + purity cross-check) + the `is_effectful_op`
  syscall-honesty fix (entropy/clock/time → effectful regardless of namespace). When green, R2's
  prequel gets its fulfillment close + the dated allows come off.
- **iv-c** — enum flip: `Kind`/`DefinedIn`/`Layer` enums; `metadata-of` emits enum values (§5).
THEN: `show-source` (+`:source` capture) → per-home carve (sonnets write prose/@added/@arg/@ret/
@example per intrinsic) → **255.1b-RESOLVE** (delete blanket-accept, registry membership → the hole
closes) → 255.2 (type-sig → `@arg`/`@ret` type-check; the wiki generator) → 255.1c FnDef split →
255.3 consumer-collapse (rete/purity.rs + macros::is_pure_total DELETE; is_effectful_op→:pure
deriver) → 255.N inscription.

## After 255: resume the parked 278 collection campaign
seq HOFs 1a+1b DONE (`5ac9abdb`/`751d131d`); remaining 1c WatAstList, 1d HashSet, map-iter,
index-assoc, set algebra. Grid: `docs/COLLECTION-CAPABILITIES.md`.

## Discipline (proven this session)
- **Use the toolkit, don't hand-roll** (memory `feedback_lean_on_wat_migration_toolkit`): fix-wat
  codemods for `.wat`; the retirement table (substrate-as-teacher) for HARD cuts; the build cascade
  as the completeness gate.
- **WEIGH every shadowdancer against the disk** — they cheat (pub-leak to hide dead_code caught
  TWICE; trim-vs-build-the-reader); diagnostics lag. Verify: grep the gate, `cargo build` warnings,
  re-run the floor, ProbeDummy the forcing. Trust neither the report nor the diagnostics.
- **Satisfy a forcing-signal by USE (build the reader), not by removal** (memory update this session).
- **Don't launder my analysis as the builder's words** — attribute mine as mine, cite theirs.

> ⛔ **You are a NEW instance.** You did NOT live the long arc-255 design session above — it's a
> cache in a familiar voice. recolligere BEFORE moving: read the two DESIGN docs + `git log
> --oneline -15` + `git status`; the DESIGN docs are the truth, this map only points. Don't propose
> from this summary — open the specs.

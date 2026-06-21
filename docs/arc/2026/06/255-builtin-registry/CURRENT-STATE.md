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

## DONE (committed, pushed; floor lib 953/36/1, nursery 900/4 [the 4 are pre-existing 255.1b-RESOLVE probes]; last HEAD ec51385d)
## Realizations this session: R1–R4 (255 REALIZATIONS.md) / Songs #98–#101 — doc-that-cannot-lie ·
## self-hosting-verifier · the-firewall-caught-the-apparatus · CEK-is-far-closer. Read them for the full telling.
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

- **255.1b-iv-b2-a** — the `:wat::intrinsic::examples` reflection seam (`src/intrinsic/reflect.rs`,
  dogfooded `#[wat_intrinsic]` → registry dispatch, no runtime.rs arm): walks the registry, returns
  `[fqdn, expr-quoted, expected-quoted-or-nil, run, pure, det]` tuples (expr/expected parsed to
  `Value::wat__WatAST`). Reading `entry.examples` **retired its `#[expect(dead_code)]`** (self-retiring
  worked — first live proof). `derive_pure_deterministic` extracted (the `NONDETERMINISTIC` hand-list
  now lives in ONE place). Probe green; nursery 899/4 (4 pre-existing); lib 953/36/1; build clean.
  `args`/`deprecated`/`see` expects retained (readers land later). DESIGN-STONE-255.1b-iv-b2.

- **255.1b-iv-b2-a.2** — RECORDS REWORK (firewall cure; R3 / Song #100): the heterogeneous tuple seam
  couldn't be typed-consumed (R7's unidirectional `Value` forbids the down-cast into typed `eval-ast!`),
  so the seam now returns `Vector<:wat::intrinsic::Example>` records — `:wat::Record::def` in new
  `wat/doctest.wat`; `reflect.rs` builds `Value::Struct(StructValue)`; `check.rs` scheme `() →
  Vector<:wat::intrinsic::Example>` (mirrors `stdlib::sources`). Probe green; nursery 899/4; lib 953/36/1.
- **STACK RUNG made durable** (`.cargo/config.toml` `RUST_MIN_STACK = "8388608"`, `821621bc`): the 2→8 MiB
  rung was decided earlier but never committed (env-only) → raw `cargo test` hit a FALSE stack overflow in
  `deporder`/`verify-stdlib` (deep wat-EVAL recursion, not infinite — passes at 8 MiB). Now durable for all
  cargo invocations. **Durable fix = arc 261 (stack-safe eval).** NOTE: for *pure* stack-safety, `stacker`
  (segmented stack) is ≈free; full CEK costs real heap-alloc/cycles but unlocks TCO + first-class
  continuations + pausable eval — decide which 261 is before building.

- **255.1b-iv-b2-b** — DONE (`ecdb42e1`): `wat/doctest.wat` `verify-examples` runs the doctests in wat —
  folds `(:wat::intrinsic::examples)`, `eval-ast!`s each `run=true` example, asserts `== #=>`, cross-checks
  `pure ∧ det`, skips `run=false`; 0 failures. **wat verifies wat — R2 FULFILLED.** ALSO corrected a2.2's
  representation: **EDN-able data → `Value::wat__Record`, NOT `Value::Struct`** (builder doctrine; the
  sonnet built Struct → named accessors broke → positional hacks; fixed at root → named accessors work).
  `Example.expected` field `Option<wat::WatAST>` (inner colon dropped). Caught + fixed iv-b1's non-runnable
  `to-hex` @example (the doctest's whole purpose). nursery 900/4; lib 953/36/1; load-order gate green.
  Memory: `feedback_wat_record_for_edn_struct_for_non_edn`.

## NEXT — 255.1b-iv-c (enum flip) — the last iv-b/c piece before show-source/RESOLVE
- **iv-c** — enum flip (§5): closed-domain metadata VALUES (`:kind`/`:defined-in`/`:layer`) → enums,
  not ad-hoc keywords. Mechanism GROUNDED: a wat `defenum` per domain (unit variants, form
  `:wat::runtime::Kind::{Macro,Fn,Intrinsic}` like `:wat::service::Outcome::Reply`, registered in
  `sym.unit_variants`) + a Rust enum mirror (`Kind`/`DefinedIn`/`Layer` — compiler-checked derivation;
  resurrects the 255.1b-i-trimmed enums); `metadata-of` intrinsic branch (runtime.rs ~10120) emits the
  enum instead of `HolonAST::keyword(":intrinsic")`.
  - **GROUND IT FIRST (builder ask, compaction-interrupted): see the real map as EDN before deciding.**
    Dogfood a `wat-scripts/*.wat` (run via the wat CLI), NOT a Rust throwaway:
    `(:wat::io::println (:wat::edn::write-pretty (:wat::runtime::metadata-of :wat::intrinsic::examples)))`
  - **DECISION PENDING (surface choice):** how the enum rides in the metadata map —
    (a) `HolonAST::keyword(":wat::runtime::Kind::Intrinsic")` (uniform map, weak typo-proofing) vs
    **(b) RECOMMENDED `Value::Enum(Kind::Intrinsic)`** directly (real enum → exhaustive match → strong;
    heterogeneous map; aligns w/ the EDN-record doctrine). VERIFY a `Value::Enum` rides the HashMap+`get`.
  - Scope: INTRINSIC branch; user-form `metadata-of` enum-`:kind` parity = flagged follow-on.

## PARKED (not arc-255): a test busy-spin DoS (builder's box)
One proc, N threads slamming CPU = a busy-poll loop (`mora` violation) in the reactor/comms layer
(arc 209/214), NOT a recursion bomb (those abort fast), NOT the 8 MiB rung. Could not reproduce
in-sandbox (nursery ~35s @ both 2 & 8 MiB). Chase with a REAL repro: the proc name from `top -H` /
`ps -T -p <pid>` + the exact command. Pre-existing; chase later.
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
- **wat-record for EDN-able data; `Value::Struct` only for non-EDN payloads** (memory
  `feedback_wat_record_for_edn_struct_for_non_edn`) — a Rust seam returning a record builds
  `Value::wat__Record` (else named accessors break → positional hacks). EDN-able = serializable = R4's
  hibernation/CEK future. Cost a full a2.2→iv-b2-b rework this session.
- **Mark apparatus-minted ritual provenance** (memory `feedback_mark_apparatus_minted_provenance`) — a
  self-chosen signature / co-author line / convention records its provenance, not read as builder-handed.
- **`#[expect(dead_code)]` (not `#[allow]`) for transient dead** — self-retiring, compiler-enforced
  removal when the reader lands; but a `#[cfg(test)]` read trips it (gotcha, arc 277 note).
- **STACK:** `.cargo/config.toml` commits `RUST_MIN_STACK=8 MiB` (was env-only → false overflows in
  sandboxes/sonnets/CI). Durable fix = arc 261 (stack-safe eval): **261 = CEK** (capabilities — TCO +
  green threads + hibernation, R4) vs `stacker` (safety-only, ≈free); decide which before building.

> ⛔ **You are a NEW instance.** You did NOT live the long arc-255 design session above — it's a
> cache in a familiar voice. recolligere BEFORE moving: read the two DESIGN docs + `git log
> --oneline -15` + `git status`; the DESIGN docs are the truth, this map only points. Don't propose
> from this summary — open the specs.

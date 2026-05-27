# BRIEF — Stone S-C.1 — rename `Value::wat__Record` → `Value::wat__holon__Record`

**Status:** READY TO SPAWN. `model: "sonnet"`.
**Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (verify with `pwd` first; reject any
path containing `.claude/worktrees/`; use `git -C` if needed).

## What to do (substrate-as-teacher rename — the compiler is the exhaustive teacher)

The existing `Value::wat__Record` carries `struct_form` + `holon_form` — it IS the
hologram (both flavors), i.e. the **holonic** record. It has the wrong name (it was doing
both jobs). **Rename the Rust variant `wat__Record` → `wat__holon__Record` everywhere in
`src/`.** This frees the `wat__Record` name for the base record S-C.2 will mint.

This is a **pure rename — ZERO behaviour change**. Baseline 827/0 MUST hold. Method:
rename the variant at its definition (`src/runtime.rs` ~651), then `cargo build`, then fix
every site the compiler names, iterate to green. The compiler catches every
match-arm / construction of the variant.

### Three precise rules (the only judgment — everything else is compiler-driven)

1. **Variant identifier `wat__Record` (double-underscore) → `wat__holon__Record`** — the
   def + all ~67 match/construct sites in `runtime.rs` + the few in `check.rs`/`stdlib.rs`.
   Compiler-exhaustive.
2. **Discriminant tag string — RENAME.** `src/runtime.rs:1116` `"wat__Record".hash(state)`
   → `"wat__holon__Record".hash(state)` (and update the comment ~1113). This is the
   variant's Hash discriminant; it MUST become the new name so it differs from the future
   base `wat__Record`'s tag (S-C.2) — else hash collision between the two flavors.
3. **Surface-type strings — KEEP UNCHANGED.** Any string containing `wat::Record` with
   **colons** is the wat-SURFACE type name, NOT the variant. Leave ALL of them:
   - `src/runtime.rs:1269` `Value::wat__holon__Record { .. } => "wat::Record"` (rename the
     variant in the arm; the **returned string `"wat::Record"` stays**).
   - `src/runtime.rs:7544` likewise `=> ":wat::Record"` (string stays).
   - every `:wat::Record` / `"wat::Record"` in `types.rs` (13), `check.rs` (39),
     `stdlib.rs` (2), and the rest of `runtime.rs` — these are the surface type; **do not
     touch them.** (The holonic value's surface type stays `:wat::Record` for now;
     the `:wat::holon::Record` surface divergence is S-C.3, NOT this stone.)

**Rule of thumb:** double-underscore `wat__Record` = the variant → rename (+ rule 2).
Colon `wat::Record` / `:wat::Record` = the surface type → keep.

This is `src/` ONLY. NO `wat/` sources, NO holon-rs (STOP-5), NO test EXPECTATION changes
(pure rename → tests stay green untouched; if a test names the variant in embedded
Rust, that's a rule-1 rename, fine).

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md`
   — **the § DESIGN CORRECTION at the end** (authoritative: two-variant shape; this is S-C.1).
2. `src/runtime.rs` ~639-670 (the `wat__Record` variant def: `class_fqdn` + `struct_form`
   + `holon_form`) + ~895-900 (Eq) + ~1112-1117 (Hash + the discriminant tag) — the
   identity machinery you're renaming, not changing.
3. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-A1.md` — the prior
   stone's SCORE shape to mirror.

## Discipline

- `src/` ONLY. Pure rename. If you find yourself changing a RETURNED surface-type string
  (`"wat::Record"` / `":wat::Record"`) or any behaviour → STOP, that's not this stone.
- Do NOT add the base variant, do NOT touch the macros (`wat/Record.wat`), do NOT change
  the wat-surface type. Those are S-C.2 / S-C.3.

## STOP triggers (REJECTION — not permission to defer)

1. Lib baseline drops below **827/0** for ANY reason (a pure rename cannot change behaviour;
   a drop means something semantic changed → STOP and report).
2. Any file outside `src/` touched (esp. `wat/Record.wat`, holon-rs).
3. You're tempted to change a surface-type string or add the base variant — STOP (S-C.2/3).
4. 40 min elapsed (STOP-3); 60 min (STOP-4). This is a mechanical rename; it should be fast.
5. Any records-thread predecessor probe regresses (S-A 10/10, S-B.1 6/6, S-B.2 5/5,
   S-A1 6/6).

## Regression suite (re-run all; expect green, untouched)

```
cargo build --release -p wat
cargo test --release --lib -p wat                                   # >= 827, 0 failed
cargo test --release --test probe_arc237_sA_hierarchy               # 10/10
cargo test --release --test probe_arc237_sB1_recordtype             # 6/6
cargo test --release --test probe_arc237_sB2_defrecord_recordtype   # 5/5
cargo test --release --test probe_arc237_sA1_assignable             # 6/6
cargo test --release --test probe_arc234_stone5_holon_auto_dispatch # records dual-form still works
cargo test --release --test probe_arc227_stone2_defrecord           # defrecord still works
```

## No FM-2-bis probe (deliberate)

A rename is not a non-trivial composition — there is nothing to disconfirm. The
regression suite above IS the contract (baseline 827 + records probes green prove the
rename changed nothing). Per recovery-doc FM-15: substrate-wide mechanical change → short
brief, compiler is the teacher, iterate to green.

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-C1.md` (NEW). Mirror
SCORE-STONE-S-A1: scorecard (build clean; lib 827/0; the records regression suite green;
`src/` only) → the rename (variant + discriminant tag; surface strings untouched) →
honest deltas → `git status --short`. DO NOT commit (orchestrator commits).

## Calibration

Pure mechanical variant rename, ~70 sites, compiler-exhaustive + 1 discriminant-tag string
+ a comment. Baseline-preserving by construction. **Target band: 15–30 min Mode A; 40
STOP-3; 60 STOP-4. Cascade: `src/` only (mostly runtime.rs), 0 forced non-src files.**
Mirror SCORE-STONE-S-A1.

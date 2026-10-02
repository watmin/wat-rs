# BRIEF — STONE 255.85: cutover 5c-i (a) — the conversion tooling is ready for today's tree

**Drawn 2026-10-02 against `main` @ `c9fab1869`.** **Executor: grok via pulsare, working solo** (it runs the floor). A
strike on the converter, the driver, seven gates, a sample list and two fixtures. **No head conversion lands in the
tree.** Commit locally on `main` (`git add -- <paths>`, never `-A`; `git status` clean before the floor you report);
**do not push**.

## Where this sits

5c (the head conversion) runs in four parts: **5c-i** its prerequisites (this stone is the tooling half; **255.86** is the
naming half: G1 and R-a), **5c-ii** the stdlib, **5c-iii** the corpus, **5c-iv** embedded wat. Read
`SCORE-STONE-255.84-cutover-5c-measure-the-conversion.md` (the measurement this stone answers) and its § 3 table.

## The work (each from 255.84's measurement)

1. **The converter's type rule consults the closed set.** In the converted stdlib clone the converter wrote
   `wat.type/Error` (8), `wat.type/EvalError`, `wat.type/Equatable` (2), `wat.type/Uuid`, `wat.type/Infer` (orchestrator,
   `grep` in `/tmp/wat-5c/wat`): it maps every `:wat::core::<Type>` in a type position to `wat.type/`. Only the 24
   (`WAT_TYPE_HARD_PRIMITIVES`, plus `AST`) go to `wat.type/`; every other type keeps its home spelling
   (`:wat::core::Error` → `wat.core/Error`, `:wat::core::Equatable` → `wat.core/Equatable`, `:wat::uuid::UUID` →
   `wat.uuid/UUID`). Find the rule in `wat/fix.wat` (`type-shaped-keyword?` or its neighbour), make it ask the closed set
   (one source: a wat verb exposing the Rust list, or the list read from where it is declared, never a second hand list),
   and add replay cases for a member, a non-member core type, and a home type. `wat/fix.wat` is compiled in
   (`include_str!`): rebuild before measuring.
2. **The seven gates that read stdlib text by keyword spelling** go blind on the converted spelling (255.84 § 3, second
   table): `ast_kind_nodekind_sync`, `gen_doc_surface_matches` ×2, `rete_header_claims_are_asserted`,
   `no_raw_network_keys_in_oracle`, `rete_bind_generators`, `rete_names_in_wat_scripts_resolve::known_forms_are_real`.
   Each reads a declaration by identity (parse the form and compare canonical identities), so it finds the declaration
   in either spelling. Prove each by running it against the clone's converted `wat/` as well as the main tree.
3. **The delta sample:** `251-types-as-forms/delta-sample-179.txt` names
   `wat-scripts/scratch-pad/probe-arc278-57-persistentmap-contains-key.wat`, deleted by `581478c9c` (255.82). Remove that
   line, record the new sha256 where `delta.sh` pins it, and say in the commit why (a deleted file, not a rebuilt list;
   the list is never rebuilt by index).
4. **`wat-fix-rust`:** (a) it stops at the first failing codemod batch; make it **resumable**: a failing batch is retried
   per file, the failures are recorded (file, literal span, first error), and the run finishes with its summary line.
   (b) it hands the converter prose: wat inside Markdown backticks in a doc string (`crates/wat-doc/src/lib.rs`, the
   `` `wat.core/Bytes` `` fence in 255.84 § 5). A literal is a candidate only if it is a wat program (its forms parse as
   lists headed by a keyword or namespaced symbol **without** prose around them); say how the candidate test decides, and
   prove it on `crates/wat-doc/src/lib.rs` and `crates/wat-edn/tests/spec_strict.rs`.
5. **The two `:fn(` fixtures** that the converter refuses (`tests/function/fn_rename_bare_fn_type.wat:4`,
   `tests/function/fn_rename_mixed_legacy.wat:4`): read each test's claim. If its subject is the retired keyword-bodied fn
   type, the fixture becomes a negative proof (the refusal, asserted by name); if the `:fn(` is incidental, respell it to
   the bracket form by the recorded `fn-keyword-to-bracket.wat`. Say which, per file.
6. **Re-measure** in a fresh clone at your final commit: 255.84 § 2 (the corpus census: 0 FAIL expected) and § 5 (the
   embedded dry run: a full summary line, refusals listed by reason). Do not re-run the stdlib floor here (that is 5c-ii).

## Gates

| what | how | expected |
|---|---|---|
| converter replay | `every_recorded_migration_replays` for the touched codemods | green; new cases included |
| the seven gates | against the main tree and against the clone's converted `wat/` | green on both |
| census, main tree | `scripts/replay/census.sh` pre and `--diff` after | no rc flips |
| corpus census (clone) | item 6 | 0 FAIL |
| embedded dry run (clone) | item 6 | a summary line; every refusal listed with its reason |
| release floor | `scripts/floor.sh`, in the foreground, nothing else running, `git status` clean | all passed; the count against 6389 at `af6577c3e` (`.floor/2026-10-02T22-04-08Z`), plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change: capture it **verbatim**, cure it, run a **new** floor. Never re-run unchanged
  code for a green.
- **STOP-1:** a gate in item 2 cannot read by identity without changing what it asserts. Describe it and STOP on that
  gate.
- **STOP-2:** the closed set has no single source the converter can ask (it would need a second hand list). Describe it
  and STOP on item 1.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you; read `wat/fix.wat`'s header (STASH-DANCE, a tool is never its own input). Capture `rc=$?` on
the next statement. Never wait with `pgrep -f`. Never write a number, file:line or example you did not measure. If this
brief contradicts the code, the code wins: say so. Write `SCORE-STONE-255.85-cutover-5c-i-a-the-conversion-tooling.md`
beside this brief, commit it, **do not push**.

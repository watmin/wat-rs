# BRIEF — STONE 255.81: cutover 4b — the new spelling is the key; the door is deleted; types print as written

**Drawn 2026-10-01 against `main` @ `202eb5533`.** **Executor: a Sonnet subagent, working solo** (it runs the floor; pass
`timeout: 600000` on that Bash call and on any build, codemod run or census that may pass two minutes). A strike in `src/`
(the 24 primitives' keys, the door, the printer), the held-back embedded wat, tests. Commit locally on `main`
(`git add -- <paths>`, never `-A`), in the commits named below; **do not push**. Your final message is your report.

## The rulings

- **C1 (2026-09-27):** the new spelling is the canonical key. **S-b + P-surface (2026-10-01):** 4a (255.79/255.80)
  respelled every type position in `.wat` and in wat embedded in Rust; **4b** (this) flips the key, deletes the door,
  refuses the old spelling, and makes types **print as written**.
- `wat.type/` holds exactly the 24 (`WAT_TYPE_HARD_PRIMITIVES`, `src/types.rs:241`, plus `AST`). Everything else keeps its
  key in this stone (`Option`, `Result`, `Span`, `Uuid`'s new home, …).

## The measurement (orchestrator, at `202eb5533`)

- **The door:** `type_denotation` (`src/edn/render.rs:3706`) maps `:wat::type::X` → `:wat::core::X` (and
  `wat.type/AST` → `:wat::WatAST`); `canonical_type_key` (`src/types.rs:286`) and `denoted_type_path` route through it.
  About 22 call sites across `src/edn/render.rs`, `src/collection/seq_container.rs`, `src/function/subsume.rs`,
  `src/intrinsic/holon/atom.rs`, `src/freeze/env.rs`, `src/collection/infer.rs`, `src/macros/parse.rs`,
  `crates/wat-doc/src/lib.rs`. `Nature::from_root_keyword` (`src/types.rs:650`) now goes through it too (255.79).
- **The keys:** 308 whole-string literals `":wat::core::<24>"` in `src/` (`":wat::core::i64"` alone 95); the
  registration lists in `src/types.rs` (~`:3548`, `:9260`); `AST`'s key is `:wat::WatAST`.
- **The printer:** one choke point, `format_type_path` (`src/check.rs:18541`), which today renders through
  `denoted_type_path`, so messages print `:wat::core::i64`. `freeze.rs`'s `format_type_expr` routes its Path arm there.
- **What users see moves:** 317 assertion lines in 131 test files name an old key, plus one golden. Runtime type strings
  (e.g. `:wat::core::type`'s result, 255.76's `probe-fnforms-shape` compares one) may move too: census it.
- **Held back by 255.80 for this stone:** 15 `.rs` files, 64 embedded-wat edits (12 compare against `format_type` text,
  2 call rete `lower()` on raw un-normalized AST whose dispatch is **keyword-only**: the K1 door-bypass class, 1 mirrors a
  doc comment). `wat-fix-rust wat-scripts/fixes/types-to-wat-type.wat --dry-run --list <all .rs>` lists exactly them.

## The work (in this order, one commit each where marked)

1. **Census (report):** every Rust site that compares, matches or registers one of the 24 by its old key; every user-facing
   string that prints one (`format_type*`, runtime type-name verbs, EDN/golden output); every test assertion naming one.
   Classify: key (registration/lookup), comparison, message text, data.
2. **Commit A, the key:** the 24 register under **`:wat::type::X`** (and `AST` under `:wat::type::AST`); every Rust
   comparison, match arm and lookup moves to the new key. `type_denotation`'s `:wat::type::` → `:wat::core::` mapping is
   **deleted**; `canonical_type_key`/`denoted_type_path` become the identity on `:wat::type::X` (keep the doors that
   canonicalize other spellings, e.g. `wat.type/i64` symbol → `:wat::type::i64`). `Nature::from_root_keyword` matches
   the new keys. **Rete `lower()`'s keyword-only dispatch on raw AST:** route it through the type door (K1) so a
   symbol-spelled type is seen, then convert its two held files.
3. **Commit B, the refusal:** `:wat::core::<24>` in a **type position** is refused with a retirement remedy naming
   `wat.type/X` (one arm for all 24, the `:wat::core::Uuid`/`Char` arm's shape, keyed off `WAT_TYPE_HARD_PRIMITIVES`, not a
   hand list). Leave function and form heads (`(:wat::core::Vector :- …)` as a **head**, slash-verbs) to stone 5: say how
   the arm tells a type position from a head.
4. **Commit C, the printer (P-surface):** `format_type_path` renders the 24 as written, **`wat.type/X`**, and everything
   else as today. Update the assertions and the golden **from what the program now prints** (re-capture, never
   hand-type), and run `wat-fix-rust … types-to-wat-type.wat` over the remaining held files; the dry run must then list 0.
   Fix the driver's dry-run summary, which says "edit(s) applied" when nothing was written.
5. **Tests:** the 24 check and run under `wat.type/X` in a header, a record field, a constructor bracket, a set element and
   a map key; `:wat::core::i64` in a type position is refused naming `wat.type/i64`; a type mismatch prints
   `wat.type/i64`; a symbol-spelled type reaches rete `lower()`.

## Gates

| what | how | expected |
|---|---|---|
| the door is gone | a search for `":wat::core::"` + one of the 24 in `src/` | only the retirement arm, its remedy row, its tests, and history comments |
| embedded wat | the `wat-fix-rust` dry run over every tracked `.rs` | `0 changed` |
| census | `scripts/replay/census.sh` pre and `--diff` after | no rc flips except programs whose only change is a message's spelling (list each) |
| release floor | `scripts/floor.sh`, **in the foreground, `timeout: 600000`**, nothing else running | all passed; the count against 6362 at `5b4d963b2` (`.floor/2026-10-01T21-57-10Z`), plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change (a site still on the old key, an assertion whose only change is the printed
  spelling, its own tests): capture it **verbatim** from `.floor/<stamp>/`, cure it, run a **new** floor. A message
  assertion is re-captured from the new output, never hand-typed. Never re-run unchanged code for a green. "Pre-existing"
  needs the prior green floor's line for that test.
- **STOP-1:** a red that is not a spelling (a type no longer checks, a value computes differently, a dispatch misses).
  Quote it and STOP.
- **STOP-2:** the refusal arm cannot tell a type position from a head without a design decision. Describe the cases and
  STOP before commit B.
- **STOP-3:** an EDN tag, a persisted form, or a wire format carries one of the 24's old keys (data on disk or across a
  process boundary). List it and STOP before changing it.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you; `.wat` and embedded wat move only by the recorded codemod (`wat-fix-rust` for Rust).
Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Run every build, floor and clippy in the foreground and
block on it. Never write a number, file:line or example you did not measure. If this brief contradicts the code, the code
wins: say so. Write `SCORE-STONE-255.81-cutover-4b-the-new-spelling-is-the-key.md` beside this brief, commit it, **do not
push**.

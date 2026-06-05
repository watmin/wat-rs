# EXPECTATIONS — Arc 249 Stone 249.2a — lift `src/macros.rs` → `src/macros/`

Scored against an **independent orchestrator re-run on disk**, not the agent's self-report. This is
a no-behavior-change lift — the gate is "behavior identical + cleanly decomposed."

## Gates (raw commands, orchestrator re-runs)

| # | Command | Pass condition |
|---|---|---|
| G1 | `cargo build --release --tests` | compiles clean (warnings pre-existing only) |
| G2 | `cargo test --release --lib -p wat` | **895 passed; 0 failed; 1 ignored** — baseline UNCHANGED (the ~920 lines of macro unit tests live here; their passing IS the behavior-identical proof) |
| G3 | `cargo test --release --test probe_arc249_threading` | **6 passed** (threading unaffected by the move) |
| G4 | `git status --porcelain` + diff review | `src/macros.rs` deleted; `src/macros/{mod,registry,error,parse,expand}.rs` created; **no other `src/` file changed** (incl. `src/lib.rs` — `mod macros;` unchanged) |

## Scorecard rows (each verified by re-running / reading, not trusting the report)

1. **Decomposition matches the intueri verdict** — `ls src/macros/` = mod/registry/error/parse/expand (no `eval.rs` yet, no `builtin.rs`). Each file holds its named cluster.
2. **Path-preservation** — `grep -rn "crate::macros::" src/ | wc -l` resolves identically; mod.rs re-exports the full prior `crate::macros::*` surface. (Spot-check: `MacroDef`, `MacroRegistry`, `MacroError`, `expand_all`, `expand_once`, `register_*`, `EXPANSION_DEPTH_LIMIT`.)
3. **No-logic-change** — the diff is *only* file boundaries + `mod`/`use`/visibility lines. Read the diff: no function body altered, renamed, reordered, or reformatted. The transitional `thread_desugar`/`construct_keyword_of` sit in `expand.rs` under their section banners.
4. **Behavior identical** — G2 green (every macro unit test passes) + G3 green (threading).
5. **Tests in-crate** — the test module stays a `#[cfg(test)]` in-crate module (not moved to top-level `tests/`); all pass.
6. **Scope honesty** — G4: only `src/macros/` touched.

## Independent prediction (runtime band)

**15–35 min, Mode A.** A 2415-line split is mechanical but wide: the care points are (a) the
cross-submodule visibility bumps to `pub(crate)`, (b) the mod.rs re-export surface matching the old
`crate::macros::*` exactly, (c) the test module's `use` paths after the split. No logic risk; the
risk is a missed re-export → a compile error the agent iterates against (the build IS the teacher).

**2× wakeup cap: 70 min.** If exceeded, `TaskStop` + score Mode-B-time-violation.

## Failure-profile expectations

- **Compile errors on first build** = EXPECTED and fine (missing `pub(crate)` / missing re-export);
  the agent iterates until G1 green. Not a Mode-B.
- **If a test fails** — a behavior change slipped in (a body was altered, or a test's `use` path is
  wrong). Mode B: the lift must be pure-move; re-do the offending file verbatim.
- **If any file outside `src/macros/` changed** (esp. `src/lib.rs` content beyond nothing, or a
  "while I was here" cleanup elsewhere) — Mode B, scope violation.
- **If the agent "improved" a function** (rename, reformat, dedupe) — Mode B. The lift is verbatim;
  improvements are the vigilia pass that follows, not this stone.

## After this stone

Independent verify (G1–G4) → commit the lift → **cast vigilia** (the wat-rs Rust-home ward guard)
on `src/macros/` to drive L1+L2=0 → earn the `vigilatum` stamp. Then 249.2b (the engine, in
`eval.rs`).

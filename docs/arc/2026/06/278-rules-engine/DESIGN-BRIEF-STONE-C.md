# DESIGN + BRIEF — Stone C: `COMPONENDO DELEO` (annihilate the legacy, de-prime the family)

**The campaign's close** (gated on A + B, both landed: A `b68a130a`, B `dc5427a4`). Grounded by a whole-tree scout +
orchestrator verification (the crate consumer-edges + the in-core/legacy sqlite distinctness confirmed by own grep).
Every "nothing uses X" below rests on a WHOLE-TREE grep (`feedback_grep_whole_tree_before_claiming_nothing_uses_x`).

## The grounded map (the surprises)
- The 3 legacy crates (`crates/wat-telemetry`, `wat-telemetry-sqlite`, `wat-sqlite`) are **LIVE** — external consumers
  are **exactly** `crates/wat-cli` (registers all 3) + `examples/interrogate` (+ mutual: telemetry-sqlite → the two).
- **KEEP** the in-core `:wat::sqlite'` / `:rust::sqlite'` (S1, `src/rust_deps/sqlite.rs` + `wat/sqlite.wat`) — a
  DISTINCT file + namespace from legacy `crates/wat-sqlite`. Killing the crate must not touch it.
- `Tagged`/`NoTag` (`wat/edn.wat:32-33`) + **`write-notag`** (verb `src/edn_shim.rs:129` + internal
  `value_to_edn_notag`/`holon_ast_to_edn_notag` + dispatch `runtime.rs:4918` + check-reg `check.rs:19213`) **all
  annihilate** — grounded: `write-notag`'s only callers are the doomed `auto.rs` + its own self-test
  `wat-tests/edn/newtypes.wat`; `value_to_edn_notag` is reachable ONLY via the verb (a closed subgraph). KEEP the
  separate `value_to_json_natural` (the JSON path).
- Primes: `:telemetry'::` + `:sqlite'`/`:rust::sqlite'` = **TRUE-RECLAIM** (bare occupied by the legacy crates →
  gated on their deletion); `mem-store'` / `sqlite-store'` = **FREE-RENAME** (bare unoccupied). De-prime blast ≈ 700 lines.
- **Rulings (builder-ratified):** delete `examples/interrogate` (no README, demos the annihilated legacy); `wat-cli`
  = clean removal (core already bakes the new family); `write-notag` joins the annihilation; delete
  `probe_arc278_process_crash_reason_carried` (the STOP-2 non-goal — crash reasons admin-only).

## The dependency-ordered strikes
Each strike: gate = `cargo build` + `cargo nextest run --release` green + grep-zero for the annihilated names;
weighed by the orchestrator's own re-run before the next.

### ── STRIKE C1 — annihilate the legacy crates (the gate for C3's reclaim) ──
1. **De-register from `wat-cli`:** drop `crates/wat-cli/Cargo.toml:26-28` (the 3 deps); remove the registration in
   `src/bin/wat.rs:13-19` + `src/bin/cargo-wat.rs:26-32`; update `crates/wat-cli/tests/wat_arc100_public_api.rs:23-46`
   (drop the 3 crates from the asserted public API). The core stdlib already bakes the new family → nothing to replace.
2. **Delete `examples/interrogate/`** entirely (Cargo.toml, src/, wat/) — annihilated (it demos the dead legacy).
3. **Delete the crates, in order:** `crates/wat-telemetry-sqlite/` (depends on both siblings) → then
   `crates/wat-telemetry/` + `crates/wat-sqlite/`. Drop the 6 `members`/`default-members` lines (`Cargo.toml:11-13,36-38`).
4. **Remove the core legacy-telemetry check consts** — `src/check.rs:1808-1810` (`LEGACY_TELEMETRY_SERVICE_*`),
   `src/check/error.rs:194,609`, `src/value/symbol_table.rs:62,284`. GROUND each: they key off the legacy unprimed
   name; after the crate is gone they're dead. **STOP if removing them breaks check logic** (they may be woven into
   a match — surface it rather than force it).
5. **Delete `probe_arc278_process_crash_reason_carried.{rs,wat}`** (the STOP-2 non-goal).
- **Gate:** workspace builds; floor green; `grep -rn "wat_telemetry\|wat-sqlite\|wat-telemetry-sqlite" .` → only the
  in-core `:wat::sqlite'`/`rusqlite` (NOT the crates); no dangling refs. The primed family still loads (untouched here).

### ── STRIKE C2 — annihilate Tagged/NoTag + write-notag ──
- Delete the `Tagged`/`NoTag` newtypes (`wat/edn.wat:32-33`) + their doc block + the `src/stdlib.rs:260-265` slot's
  now-stale prose. Delete `write-notag`: the verb (`edn_shim.rs:129 eval_edn_write_notag`), the internal
  `value_to_edn_notag` + `holon_ast_to_edn_notag`, the dispatch (`runtime.rs:4918`), the check-reg (`check.rs:19213`).
  **KEEP** `value_to_json_natural`.
- Out-of-crate consumers to delete/fix: `wat-tests/edn/newtypes.wat` (the self-test — delete, it tests the dead
  newtypes); `tests/resolve/probe_arc251_decl_migrator.wat:92,94` (it uses the newtype forms as string fixtures —
  update or drop those lines). GROUND: after C1, `examples/interrogate` is already gone (was a consumer).
- **Gate:** build + floor green; `grep -rn ":wat::edn::Tagged\|:wat::edn::NoTag\|write-notag" .` → zero live refs.

### ── STRIKE C3 — de-prime the family (the reclaim, unblocked by C1) — USE WAT-FIX ──
This is a NAME-TRANSLATION → the self-hosted `wat-fix` tool, NOT a manual edit-farm (builder: "we use wat-fix to
unfuck the farm — refactors are one-to-three-shot"). Model on `wat-scripts/fixes/rename-kernel-to-spawn.wat` (the
namespace-rename precedent): `:user::migrate` (String→String via `:wat::fix::rename-keyword-prefix "old" "new"`,
`wat/fix.wat:744`) + `:user::apply-each` (read-file→migrate→write-file) + `:user::main` (paths from stdin). Run:
`printf '["p1" "p2" …]\n' | cargo wat ./wat-scripts/fixes/deprime-telemetry-sqlite.wat`. Comment-faithful + idempotent.

- **Write `wat-scripts/fixes/deprime-telemetry-sqlite.wat`** — the `rename-keyword-prefix` sequence, SPECIFIC-FIRST
  (so `journal'`/`span'`'s own prime drops before the family prefix rename, per the kernel-precedent's disjoint-prefix
  discipline):
  1. `:wat::telemetry'::journal'` → `:wat::telemetry::journal`   (both primes, before the family)
  2. `:wat::telemetry'::span'` → `:wat::telemetry::span`
  3. `:wat::telemetry'::` → `:wat::telemetry::`                  (the family — Log/Metric/Scope/Journal/Span/Level/…)
  4. `:wat::query::mem-store'` → `:wat::query::mem-store`        (FREE-RENAME)
  5. `:wat::query::sqlite-store'` → `:wat::query::sqlite-store`  (FREE-RENAME)
  6. `:wat::sqlite'` → `:wat::sqlite`                            (TRUE-RECLAIM — catches `::Connection`/`::open`/bare)
  7. `:rust::sqlite'` → `:rust::sqlite`                          (for any .wat refs to the rust dispatch)
- **Run it over the 30 `.wat` files** carrying primed names (`grep -rl "telemetry'\|:wat::sqlite'\|mem-store'\|
  sqlite-store'\|journal'\|span'" --include=*.wat wat/ tests/ wat-scripts/ docs/` — regenerate the list live). The
  `crates/wat-edn` `"mem-store'"` EDN-apostrophe fixtures are `.rs`, auto-excluded — do NOT touch them.
- **HAND-FIX the 23 Rust-side refs** wat-fix can't reach (string literals): `src/rust_deps/sqlite.rs` +
  `src/rust_deps/mod.rs` (`:rust::sqlite'`→`:rust::sqlite`, `:wat::sqlite'`→`:wat::sqlite`), `src/stdlib.rs` (the
  manifest slot comments — the "PRIMED … staged to replace the wat-telemetry battery" prose is now false; the
  include paths are unprimed filenames, unchanged), `src/check.rs` (any primed refs). Surgical edits, not rewrites.
- **Gate:** build + floor green; `grep -rn "telemetry'\|:wat::sqlite'\|:rust::sqlite'\|mem-store'\|sqlite-store'\|
  journal'\|span'" . --include=*.wat --include=*.rs | grep -v "crates/wat-edn"` → zero (modulo doc-comments + the
  flagged EDN fixtures). Re-run the script → zero further changes (idempotent). `:wat::telemetry::` / `:wat::sqlite::`
  are now OURS — the reclaim complete, C closed.

## STOP triggers (all strikes)
1. Never delete a crate/name until a WHOLE-TREE grep proves zero live consumers outside the annihilation set — STOP + surface if a consumer appears.
2. Do NOT touch the in-core `:wat::sqlite'`/`:rust::sqlite'` (S1) when killing legacy `crates/wat-sqlite` — distinct.
3. If removing the core legacy-telemetry check consts (C1.4) breaks check logic — STOP, surface (don't force).
4. Weigh by the orchestrator's own re-run; a mid-edit diagnostic is a phantom — a suite that RAN N tests compiled.

## Acceptance (per strike, weighed by the orchestrator)
Build + `cargo nextest run --release` back to baseline (zero new failures beyond the known isolated-passing
`sigterm…polling_contract` flake); `cargo clippy` clean; the target names grep-zero; content-integrity on the diff.
COMPONENDO DELEO — the correct change subtracts; the diff is net-negative and the substrate stands on its own.

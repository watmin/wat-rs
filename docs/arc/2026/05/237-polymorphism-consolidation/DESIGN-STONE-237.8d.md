# DESIGN — Stone 237.8d — equality reclassified as a relational intrinsic; the partition inscribed; the grid residue cut

**The closing stone before INSCRIPTION.** 237 is polymorphism-*consolidation*. Numerics → `defclause` (8a/8b). Equality + collections → **intrinsic** — and 237.8d makes that classification *true in the code and the doctrine*, then 237.9 writes the INSCRIPTION and 237 dies.

This stone is **NOT** a new mechanism. Equality's implementation (`infer_equality` + `eval_eq` + `values_equal`) is already correct and is **untouched**. 237.8d does two things: **inscribe** the partition rule at the source sites (citing `docs/DISPATCH.md`), and **HARD CUT** the four vestigial per-Type equality aliases that the now-reversed grid-thinking minted.

## Background — the reversal (already ratified + documented)

The mid-arc plan (`equality → macro-generated defclause`) was **reversed on ground evidence** (see `docs/DISPATCH.md` and the 170 breadcrumb). The clause matcher checks each arg against a *fixed named type* independently (`assignable` per-position, check.rs ~5281) and **never unifies arg0's type with arg1's**; equality *is* that cross-argument unification (`infer_equality` does `unify(a,b)`, ∀T, same-or-subtype). A monomorphic clause cannot express it; a finite clause list would regress record/composite/user-type equality into `NoMatchingClause`. **Equality is an intrinsic — the relational flavor.** Shape B was always correct.

## Part A — INSCRIBE the two-flavor partition (source-marking)

The canonical prose home is **`docs/DISPATCH.md`** (shipped `ac5f1d6c`). The in-source markers must *cite it* and name the two flavors. Update the existing markers (added b90adfc7) — they currently carry the one-flavor framing:

- **`src/check.rs:4896`** — the `PARTITION — CLAUSE vs INTRINSIC (the declaration site)` marker at `infer_list`. Update to: intrinsic = type-level computation in **two flavors** — *projective* (collections: `get: Vector<T>→Option<T>`) and *relational* (equality: cross-arg `unify(a,b)`). Cite `docs/DISPATCH.md`. Note that equality routes to `infer_equality` (the relational exemplar), collections to `infer_<op>` (projective).
- **`src/runtime.rs:5739`** — the runtime-side `PARTITION` marker. Same update + cite `docs/DISPATCH.md`.
- **`fn infer_equality` (check.rs ~11126)** — add a short marker: *this is the RELATIONAL flavor of the dispatch partition — `unify(a,b)` ties the two args' types ∀T, which a monomorphic clause cannot express; see `docs/DISPATCH.md`.*

No logic change in Part A — comments only. (Orchestrator-direct source-marking per the b90adfc7 precedent; folded into the sonnet brief if cleaner to do in one pass over the cut files.)

## Part B — HARD CUT the four vestigial equality aliases

`:i64::=`, `:i64::not=`, `:f64::=`, `:f64::not=` are **fake per-Type leaves** for a uniform op: each dispatches *directly* to `eval_eq`/`eval_not_eq`, parallel to `:wat::core::=` — not as a leaf of any defclause (unlike `:i64::+`, which the `+` defclause genuinely calls). They contradict `DISPATCH.md` ("no per-Type leaf decomposition" for equality). Verified usage: **zero** in `wat/` stdlib; the only call sites are the probes that confirm they were minted + one guard test. HARD CUT (per `feedback_hard_cut_admits_no_bypasses` + one-canonical-path):

**Runtime** (`src/runtime.rs`): delete the four arms + their comment:
- `5664: ":wat::core::i64::=" => eval_eq(...)`
- `5671: ":wat::core::i64::not=" => eval_not_eq(...)`
- `5678–5680:` the `Mirrors :i64::=` comment + `:f64::=` + `:f64::not=` arms

**Check** (`src/check.rs`): delete the four entries (`13767/13774/13806/13807`) from their list + any surrounding now-dead context (the "Mirrors the i64 equality pair" block ~13804).

**Tests:**
- `tests/probe_arc237_stone3_guard_ensure.rs:126` — `(:wat::core::i64::= n 0)` → `(:wat::core::= n 0)` (the guard's intent is unchanged; equality is uniform).
- `tests/probe_arc237_8c_equality_grid.rs` — this probe exists to confirm the *alias mint*; its per-Type-alias tests (`:f64::=`/`:f64::not=` existence + type-lock) are now obsolete. Remove the alias-mint tests; **preserve any `:wat::core::=`-over-f64 coverage** by repointing to `:wat::core::=` (uniform equality over f64 must still be green).
- `tests/probe_arc237_8b_defclause_arithmetic.rs:305` — `(:wat::core::i64::not= 1 2)` → `(:wat::core::not= 1 2)` (or drop if redundant with the not= coverage elsewhere).

Substrate-as-teacher: delete the runtime arms + check entries, then `cargo build` names any remaining site.

## Scope guard (do NOT touch)

- `eval_eq` / `eval_not_eq` / `values_equal` / `infer_equality` — the equality IMPL. Untouched.
- `:wat::core::=` / `:wat::core::not=` — the canonical uniform ops. Untouched (dispatch arms runtime.rs:5652/5653, check.rs:4622).
- The collection intrinsics + their declaration arms. Untouched.
- No new mechanism, no behavior change beyond the four aliases becoming unknown keywords.

## FM-2-bis probe (`tests/probe_arc237_8d_equality_intrinsic.rs`)

Disconfirming, committed RED-where-it-must-be:
- **CUT-CONFIRMERS** (RED at HEAD — aliases still resolve; GREEN after): `(:wat::core::i64::= 1 1)` and `(:wat::core::f64::= 1.0 1.0)` must FAIL to check/resolve (unknown callee) after the cut. (`#[ignore]` until the strike, like prior mint probes — inverted: these confirm *removal*.)
- **UNIFORM-EQUALITY REGRESSION** (GREEN at HEAD and after): `(:wat::core::= 1 1)` → true; `(:wat::core::= 1.0 1.0)` → true; `(:wat::core::= 1 2)` → false; cross-type `(:wat::core::= 1 "x")` → check error; **record/composite equality still works** (`(:wat::core::= (:my::Pt 0 0) (:my::Pt 0 0))` → true — the ∀T relational case the cut must not regress).
- **PARTITION INSCRIBED** (optional, doc-grep): the markers cite `DISPATCH.md`.

## Slicing + close

Single stone (237.8d) — inscription + a small contained cut. Then **237.9 INSCRIPTION** (the arc's victory-story close; flags arc 245 wat-corpus-warding unblocked) → **237 DIES**. Spawn-block check: no open arcs spawned under 237 remain (244 closed, 248.1 closed; the re-aimed 248.2 is absorbed here).

## Gates

- `cargo test --release --test probe_arc237_8d_equality_intrinsic` → all green (cut-confirmers + regression).
- `cargo test --release --lib -p wat` → 895/0/1 (no regression).
- `cargo build --release --tests --workspace` → clean.
- `grep -rn "i64::=\|f64::=\|i64::not=\|f64::not=" src/ wat/` → **zero** (cut complete).
- No `holon-rs`.

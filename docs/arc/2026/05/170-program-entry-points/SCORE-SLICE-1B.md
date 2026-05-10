# Arc 170 slice 1b — SCORE

`ClosurePackage` reshape from `{ forms, entry: String }` to
`{ prologue: Vec<WatAST>, entry_form: WatAST }`. Synthetic-name
machinery retired. Mode A clean, ~40 min opus (significantly
under predicted 60-120 min band). Branch
`arc-170-program-entry-points` carries slice 1b commits
`a23acf3` + `365343f`.

## Scope as shipped

`src/closure_extract.rs` — 233 lines changed (154 insertions,
79 deletions). Public shape change:

```rust
// before (slice 1)
pub struct ClosurePackage {
    pub forms: Vec<WatAST>,
    pub entry: String,
}

// after (slice 1b)
pub struct ClosurePackage {
    pub prologue: Vec<WatAST>,
    pub entry_form: WatAST,
}
```

Substrate edits:
- `mint_synthetic_name()` retired
- `CLOSURE_PKG_COUNTER: AtomicU64` retired
- `:__closure::__pkg_<n>` synthetic-name pattern retired
- New `function_to_fn_form` helper — reconstructs fn-form AST
  from a `Function` Value's params + body + ret_type
- Entry-resolution branch:
  - Keyword-path input: user's existing define stays in prologue
    as a regular dep; `entry_form = WatAST::Keyword(path)`
  - Inline-lambda input: `entry_form = function_to_fn_form(...)`

`tests/wat_arc170_closure_extraction.rs` — T1-T15 assertion-shape
updates:
- New helpers: `assert_entry_form_keyword` /
  `assert_entry_form_fn_form`
- Behavior-equivalence pattern via `invoke_via_entry_form` —
  re-freeze prologue → eval entry_form → apply
- T1 unit test "synthetic-name uniqueness" dropped (no longer
  applicable); capture-name prefix unit test stays

## Scorecard

All 17 rows from EXPECTATIONS-SLICE-1B.

| Row | Verified | Pass |
|-----|----------|------|
| A — DESIGN-intent alignment | public surface has zero entry-keyword ceremony at Rust API level; no `entry: String` field; `entry_form` is the program-shaped expression | ✓ |
| B — `ClosurePackage` reshape | `pub struct ClosurePackage { pub prologue: Vec<WatAST>, pub entry_form: WatAST }` — verified in `src/closure_extract.rs` | ✓ |
| C — Synthetic-name machinery retired | `mint_synthetic_name`, `CLOSURE_PKG_COUNTER` (AtomicU64), `:__closure::__pkg_<n>` pattern all removed; grep confirms no remaining references in `src/closure_extract.rs` | ✓ |
| D — Inline-lambda emits fn-form AST | T4-T7 assert `entry_form` is fn-form AST `(:wat::core::fn [...] -> :Ret body)` matching input fn signature | ✓ |
| E — Keyword-path emits name reference | `entry_form = WatAST::Keyword(path)` — see Honest delta A below for the Symbol→Keyword substrate-fit pivot | ✓ |
| F — Prologue contains no synthetic entry-define | for keyword-path: user's existing define is a regular dep in prologue; for inline-lambda: no entry-define at all (entry_form holds the fn-form directly) | ✓ |
| G — Body rewrite preserved | rewrite happens BEFORE `entry_form` is set; `entry_form` carries the rewritten AST; capture references resolve through `__captured_X` defines in prologue | ✓ |
| H — All 15 integration tests pass | `cargo test --release --test wat_arc170_closure_extraction` → 15/15 pass with NEW assertion shapes | ✓ |
| I — In-module unit tests | synthetic-name uniqueness test DROPPED; capture-name prefix test STAYS; net unit-test count 1 | ✓ |
| J — Workspace stays clean | `passed: 2107 failed: 0` (was 2108 pre-slice; -1 from dropped synthetic-name unit test, exactly as predicted) | ✓ |
| K — Capture-binding naming unchanged | `__captured_X` prefix machinery retained for capture-binding defines (unrelated to entry naming) | ✓ |
| L — Slice branch on remote | `arc-170-program-entry-points` carries `a23acf3` + `365343f` + this SCORE; main untouched | ✓ |
| M — Zero Mutex usage | no Mutex/RwLock/CondVar; AtomicU64 actually retired (only Arc/HashMap/BTreeMap remain) — net atomic count went DOWN | ✓ |
| N — No wat-level surface added | `extract_closure` is Rust-public; not registered in wat eval dispatch | ✓ |
| O — No spawn-process/spawn-thread/fork-program changes | invocation paths unchanged; slice 2's territory | ✓ |
| P — No `:user::main` signature changes | also slice 2's territory | ✓ |
| Q — SCORE-SLICE-1.md untouched | immutable per `feedback_inscription_immutable.md`; verified | ✓ |

## Honest deltas

### Delta A — entry_form Keyword vs Symbol substrate-fit pivot

CLOSURE-EXTRACTION.md v2 + BRIEF described `entry_form` for
keyword-path input as "a Symbol AST naming the keyword."
Implementation surfaced that wat-rs's eval (runtime.rs ≈ line
2851) resolves bare-Symbol references via `env.lookup(...)`
only — top-level defns are not lexically bound, they're in the
symbol table.

A `WatAST::Symbol(":my::worker")` at the entry_form site would
fall through to env.lookup, which fails (the symbol isn't
lexically bound; the define lives in the symbol table). A
`WatAST::Keyword(":my::worker")` goes through eval's keyword
arm (runtime.rs ≈ line 2846), which lifts the registered
Function via `sym.get(k)` — this works.

The agent shipped `WatAST::Keyword`. Spec intent ("a name
reference that evaluates to the fn Value") is preserved; the
surface differs (Keyword, not Symbol) for substrate-fit.

This is the right call. Per FM 5: not bridged with TODO;
surfaced cleanly. Code comment at `src/closure_extract.rs`
≈ line 280-288 explains the pivot; commit message documents it.

**Sub-delta — file-level doc comments not fully swept.** The
implementation site (line 280-292) uses Keyword and explains
why; but the file-level doc comments at lines 18, 54, 155 still
reference "Symbol AST" for the keyword-path entry_form. Minor
doc-comment drift; load-bearing implementation + the explanation
at line 280-288 are correct.

CLOSURE-EXTRACTION.md v2 also says Symbol; should be updated to
Keyword. Slice 5 closure paperwork sweeps both.

### Delta B — atomic-count went DOWN

Slice 1's implementation used `AtomicU64` for the synthetic-name
counter (the only mutable shared state in `closure_extract.rs`).
Slice 1b retires the counter entirely. Net atomic count in the
module went from 1 → 0. Aligned with `feedback_zero_mutex.md`
spirit (zero-mutex by design; atomics permitted but minimized).

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 60-120 min opus | ~40 min | A clean (UNDER predicted band) |

Significantly under-predicted. Reasons:
- Algorithm intact (slice 1's free-symbol walker, dep-closure
  builder, capture encoding, portability check, topological sort
  all stayed)
- Spec doc CLOSURE-EXTRACTION.md v2 was tight; agent could
  implement directly against it
- Reshape work was localized: 1 struct shape, 1 entry-resolution
  branch, 1 assembly path, helper retirement, helper addition
  (`function_to_fn_form`)
- Test assertions follow predictable patterns once the helpers
  exist

Calibration data point: slice-of-substrate-reshape-with-tight-
spec is closer to 30-60 min opus than 60-120 min. Future
predictions for similar slices should tighten.

Subsystems touched:
- `ClosurePackage` shape + types: 1 site
- Entry resolution branch: 1 site (3-way: lambda / keyword-path /
  unsigned)
- Assembly: 1 site (no longer appends entry as trailing define)
- Synthetic-name retirement: counter + helper deletions
- `function_to_fn_form` helper: NEW (~25 lines)
- Test helpers: 2 new (`assert_entry_form_keyword`,
  `assert_entry_form_fn_form`); 1 new behavior-equivalence
  pattern (`invoke_via_entry_form`)
- T1-T15 assertion shape updates: 15 tests
- 1 in-module unit test dropped

Honest deltas surfaced: 1 substantive (Symbol→Keyword) + 1
minor (doc-comment drift not fully swept).

## Discipline check

- ✓ FM 5 held — Symbol→Keyword pivot surfaced as honest delta,
  not bridged with TODO; explained in code + commit message
- ✓ FM 9 honored — local cargo test verified 2107/0 + 15/15 on
  arc170 closure_extraction post-spawn
- ✓ FM 10 — no type-system reach; substrate-fit answer was
  recognizing the Symbol-vs-Keyword distinction in eval, not
  inventing new types
- ✓ FM 11 — pre-INSCRIPTION grep deferred to slice 5 closure
- ✓ FM 12 — Agent spawn included `model: "opus"` explicitly
- ✓ FM 16 honored — BRIEF didn't mention Bash/cargo availability
- ✓ Branch isolation held — main untouched
- ✓ SCORE-SLICE-1.md untouched per `feedback_inscription_immutable.md`

## What's next

The arc 170 substrate primitive is now in its corrected shape.
Slice 2 (substrate consumer) is unblocked.

Slice 2 BRIEF is currently FROZEN at v1-shape per FM 6 (see
its header). When slice 2 is redrafted, the BRIEF reflects:
- Slice 1b's `{ prologue, entry_form }` shape
- Typed-channel `:user::process` contract
  (`[rx <- :Receiver<I> tx <- :Sender<O>] -> :wat::core::nil`)
- `:wat::kernel::Process<I,O>` shape with typed-channel handles
- EDN-over-pipes implementation in spawn-process

Slice 2 is the substrate-level proof of EDN-as-transport; slice
3 is the user-visible UX confirmation through the testing-lib
three-layer API.

## What this slice proved

The arc-discipline pipeline works. The accumulated artifacts
(DESIGN, TIERS, CLOSURE-EXTRACTION v2, REALIZATIONS-SLICE-1,
BRIEF-SLICE-1B, EXPECTATIONS-SLICE-1B) successfully briefed a
fresh opus agent that shipped Mode A clean in 40 min — under
predicted band, with a substantive honest delta surfaced
correctly per FM 5.

The doctrine paid off. The five framing passes (entry-keyword
ceremony → hermetic-as-primary → tiers-as-primary → arc-covers-
all-polish → strings-as-substrate-leakage) settled into a brief
that an agent could execute without re-deriving any of the
framings.

## Companion docs

- BRIEF-SLICE-1B.md + EXPECTATIONS-SLICE-1B.md — original brief
- REALIZATIONS-SLICE-1.md — discipline lessons + framing journey
- CLOSURE-EXTRACTION.md v2 — corrected spec (Symbol→Keyword
  doc-comment update slated for slice 5 closure paperwork)
- TIERS.md — substrate-concept doc (closure-extraction is the
  tier-bridging primitive at tier ≥ 2)
- SCORE-SLICE-1.md — slice 1's immutable historical record

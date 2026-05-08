# `src/` modularization — queued notes

**Status:** queued 2026-05-08; not yet a numbered arc. Triggers
after arc 109 wraps up (the foundation impeccable-state user
direction). Until then, this doc holds the rationale + plan so
future-us doesn't lose the thread.

## Why this is worth doing

File sizes audited 2026-05-08:

| File | Lines |
|---|---|
| `src/runtime.rs` | 23,801 |
| `src/check.rs` | 15,108 |
| `src/types.rs` | 2,620 |
| `src/macros.rs` | 1,798 |
| `src/load.rs` | 1,656 |
| `src/freeze.rs` | 1,588 |
| `src/edn_shim.rs` | 1,535 |
| `src/io.rs` | 1,508 |
| `src/test_runner.rs` | 1,084 |

`runtime.rs` + `check.rs` are 39k lines together = 63% of `src/`.
Both grew organically as substrate features landed; natural
boundaries are visible inside but the file as a unit is hard to
navigate. cargo-check feedback on a 24k-line file is slower than
it should be; `git blame` still works but conceptual overwhelm is
real.

The candidate boundaries are visible enough that breakup is
not a forced reorganization — the module names match real
substrate concerns.

## Why not yet

- **Arc 109 hasn't wrapped.** Per user direction 2026-05-03:
  *"once 109 wraps up - we'll have what we believe to be an
  incredibly solid foundation to begin the next leg of work... i
  cannot begin any of that work until the foundation is
  impeccable."* Modularization is post-foundation work. Arc 109
  has 6 pending follow-ups (per task list as of 2026-05-08).
- **Active arcs disrupted.** Refactoring file boundaries while
  arcs 167+ ship substrate changes creates merge conflicts.
  Sequence after the active arc cluster closes.
- **`git blame` disruption is real.** Mitigation: keep
  extraction commits surgical (one move per commit; blame
  preserves through `--follow`); avoid bundled "rename + edit"
  commits.

## Approach: incremental extraction (not big-bang)

User direction discipline: "doesn't leave cruft." Big-bang
refactor risks landing in a half-extracted state. Incremental,
arc-by-arc extraction keeps each step's "did it work" verification
clean.

Each extraction = one numbered arc with DESIGN/BRIEF/EXPECTATIONS
through the standard discipline.

## Candidate extraction order (cleanest first)

These are guesses based on function-cluster shapes seen during
arcs 157-167. Each candidate is a self-contained module with low
coupling to its siblings inside `runtime.rs`. Confirm the
boundaries during the actual arc by reading the cluster end-to-end
before deciding.

### 1. `runtime/reflection.rs` — easiest, lowest coupling

Functions: `lookup_form`, `Binding` enum + impls,
`function_to_define_ast`, `primitive_to_define_ast`,
`macrodef_to_define_ast`, `typedef_to_define_ast`,
`dispatch_to_define_ast`, `name_from_keyword_or_fn`,
`eval_lookup_define`, `eval_signature_of`, `eval_body_of`.

Boundary signal: all reflection-output rendering. Self-contained
helpers; takes `&SymbolTable` + name string in, returns `HolonAST`
out. Doesn't touch eval-core or check-core.

### 2. `runtime/register.rs` — moderate coupling

Functions: `register_defines`, `register_stdlib_defines`,
`register_struct_methods`, `register_enum_methods`,
`register_newtype_methods`, `register_runtime_defs`,
`register_define_dispatches`, `try_parse_fn_shape_def`,
`function_byte_equivalent`.

Boundary signal: freeze-time-pipeline registration. All called
from `freeze.rs`'s startup pipeline; mutate `SymbolTable` state.

### 3. `runtime/parse.rs` — moderate coupling

Functions: `parse_define_form`, `parse_define_signature`,
`parse_fn_signature`, `is_define_form`, related parameter-list
helpers.

Boundary signal: structural-form decoders; takes `WatAST` in,
returns parsed structures.

### 4. `runtime/primitives/` — heavy coupling but bounded per-family

Per-family modules:
- `runtime/primitives/i64.rs` — `eval_i64_*` family
- `runtime/primitives/f64.rs` — `eval_f64_*` family
- `runtime/primitives/string.rs` — `eval_string_*` family
- `runtime/primitives/vec.rs` — `eval_vec_*` family
- `runtime/primitives/tuple.rs` — `eval_tuple_*` family
- `runtime/primitives/option.rs` — `eval_some_ctor`, etc.
- `runtime/primitives/result.rs` — `eval_ok_ctor`, etc.
- `runtime/primitives/hashmap.rs` — `eval_hashmap_*` family
- (others as the audit reveals)

Boundary signal: each family is one type's primitive operations;
no inter-family dependencies.

### 5. `runtime/eval.rs` — eval-core (DO LAST)

Functions: `eval`, `eval_list`, `eval_tail`, `step_form`,
`apply_function`, `dispatch_keyword_head`, `eval_dispatch_call`.

Boundary signal: this IS the interpreter. Strongly coupled to
everything else. Last extraction; might end up as the actual
`runtime/mod.rs` or remain `runtime.rs`-shape with a thinner
internal split.

## `check.rs` parallel structure

Same approach for `src/check.rs` (15k lines):

1. `check/walkers/` — `validate_legacy_*`, `walk_for_*`, the
   substrate-as-teacher migration walkers. Each walker is its own
   small file.
2. `check/infer.rs` or `check/infer/` — the type-inference
   dispatch + per-form arms.
3. `check/scope_deadlock.rs` — pair-deadlock walker family
   (arcs 117/126/128/131/133/134).
4. `check/types.rs` — TypeExpr formatting, scheme derivation,
   unification helpers (or move into `src/types.rs` since that
   already exists).

## Risks + mitigations

- **`git blame` disruption.** Surgical commits per move;
  `git blame --follow` works for files renamed but cleanly moved.
  Avoid bundling unrelated edits with moves.
- **Build-time regression.** Many smaller modules can slow Rust
  compile if they require lots of cross-module trait derivations.
  Verify cargo-check time post-extraction; if it gets worse,
  re-evaluate.
- **Visibility leaks.** Extracting reveals which functions need
  `pub(crate)` vs internal. Some currently-private functions may
  need to become `pub(crate)` to cross module boundaries. Keep
  the public surface (the crate's `lib.rs` re-exports) unchanged.
- **Mid-extraction inconsistent state.** Per-arc extraction
  commits the whole thing atomically. No half-extracted state on
  main.

## Sequencing

After arc 109 wraps:
- Arc N: extract `runtime/reflection.rs` (lowest coupling, proves
  the pattern)
- Arc N+1: extract `runtime/register.rs` (validates the freeze-
  time-pipeline boundary)
- Arc N+2: extract `runtime/parse.rs`
- Arc N+3 to N+M: extract `runtime/primitives/*` per family
- Arc N+K: extract `runtime/eval.rs` (last; might change shape
  based on what's left)
- Arc N+K+...: same pattern for `check.rs`

Each arc decides whether the next extraction is worth it. We can
stop at any point if the remaining file is tractable.

## When to revisit / convert to a numbered arc

- Arc 109's pending tasks all close (per task list)
- The user explicitly directs "now we modularize"
- A specific pain point surfaces (e.g., a substrate change that
  would be much cleaner with the boundary already drawn)

When triggered, this NOTES.md gets renamed to a numbered arc's
DESIGN.md and the planning detail copies forward. Until then it's
a living queue doc that updates as substrate evolves.

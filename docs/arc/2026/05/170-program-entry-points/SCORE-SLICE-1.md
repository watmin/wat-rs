# Arc 170 slice 1 — SCORE

Closure extraction substrate primitive (Rust-internal; zero
wat-level callers). Mode A clean, ~150 min opus (within 90-180
predicted band). Branch `arc-170-program-entry-points` carries
slice 1 commit `787c977` + this SCORE.

## Scope as shipped

New module `src/closure_extract.rs` (~1280 lines including tests)
+ `tests/wat_arc170_closure_extraction.rs` (15 integration tests,
~580 lines) + 1-line `pub mod closure_extract;` in `src/lib.rs`.

Public API:

```rust
pub struct ClosurePackage {
    pub forms: Vec<WatAST>,
    pub entry: String,
}

pub enum ExtractionError {
    NonPortableCapture { name, type_name, path },
    UnresolvedSymbol { name, span },
    Internal(String),  // gaps in slice 1 — surfaced honestly, not bridged
}

pub fn extract_closure(
    fn_value: &Value,
    parent_symbols: &SymbolTable,
    parent_types: &TypeEnv,
) -> Result<ClosurePackage, ExtractionError>;
```

Five subsystems:
- Free-symbol walker (scope-tracking; let / fn / define / destructure binders)
- Dep-closure builder (recursive with fixpoint; visited-sets;
  edge tracking for topological order)
- Value→AST encoder (primitives + Vector + HashMap + Tuple + Struct
  + Enum + Option + Result + HashSet + path threading for nested
  diagnostics)
- Portability check (refuses Sender / Receiver / ProgramHandle /
  HandlePool / ChildHandle / IOReader / IOWriter / OnlineSubspace
  / Reckoner / Engram / EngramLibrary / Hologram)
- ClosurePackage assembly (topological sort: types → captures →
  user defs → entry; body rewrite for captured locals)

15 integration tests + 2 in-module unit tests = 17 net new tests
(BRIEF target was 15; +2 for synthetic-name uniqueness +
capture-name prefix unit tests — small confidence checks).

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — `src/closure_extract.rs` minted | new module exists; `ClosurePackage` + `ExtractionError` + `extract_closure` public | ✓ |
| B — Free-symbol walker | `walk_free_symbols` + `walk_let_form` / `walk_fn_form` / `walk_define_form` track scope through let flat-vector binders, fn arg-vector triples, define param lists; treats `->` `<-` `&` as syntactic markers | ✓ |
| C — Dep-closure builder | `extract_user_deps_to_fixpoint` + `extract_user_types_to_fixpoint` with visited-sets + edge tracking via `current_walking_dep` field for topological order | ✓ |
| D — Value→AST encoder | `encode_value_to_ast` covers all listed Value kinds + path threading; recursive-type guard via `ensure_type_extracted` | ✓ partial — see Honest delta C |
| E — Portability type-check | refuses 12 non-portable type kinds; `NonPortableCapture` Display renders substrate-as-teacher diagnostic with name/type/path | ✓ |
| F — ClosurePackage assembly | topological sort (types → captures → user defs → entry); body rewrite via `rewrite_captures` + `rewrite_with_scope` + per-form helpers; cumulative-locals tracking preserves shadowing | ✓ |
| G — All 15 Rust integration tests pass | `tests/wat_arc170_closure_extraction.rs` 15/15 green; +2 in-module unit tests | ✓ |
| H — Workspace stays clean | post-slice-1 verified locally: `passed: 2108 failed: 0` (was 2091/0 pre-slice; +17 = +15 integration + 2 unit) | ✓ |
| I — No wat-level surface added | `extract_closure` is Rust-public; not registered in wat eval dispatch; no new wat-callable verbs | ✓ |
| J — No spawn-process / spawn-thread / fork-program changes | invocation paths unchanged; slice 2's territory | ✓ |
| K — No `:user::main` signature changes | `validate_user_main_signature` + `expected_user_main_signature` unchanged | ✓ |
| L — Slice branch on remote | `arc-170-program-entry-points` carries `787c977` + this SCORE; main untouched | ✓ |
| M — Zero Mutex usage | no Mutex / RwLock / CondVar in `closure_extract.rs`; uses HashMap + HashSet for visited-sets (no shared mutability) | ✓ |
| N — Substrate-as-teacher diagnostic | `NonPortableCapture` Display verified — names capture, type, path (for nested), suggests pipes/restructure | ✓ |

## Honest deltas

### Delta A — Captured fn values not implemented (Q-impl-2 gap)

The Q-impl-2 lean was "recursive sub-extraction" for captured fn
values (closures-of-closures). Slice 1 does NOT ship this. The
encoder's `Value::wat__core__fn` arm returns
`ExtractionError::Internal("encoding for captured Value of kind
wat::core::fn not implemented in slice 1")`.

Per FM 5: opus did NOT bridge with a TODO; the Internal error is
the honest surface.

T1-T15 don't exercise this case. Whether slice 2's
spawn-process-on-real-fn-shapes consumers hit it depends on
whether typical factory patterns produce closures that themselves
capture closures. Most factory patterns I've reviewed are
"factory captures config; returns lambda using config" — single-
level capture; this works.

If slice 2 surfaces a real consumer needing captured-fn-value
encoding, slice 1 follow-up implements it (recursive sub-extraction
per the DESIGN's lean).

### Delta B — Synthetic name reserved-prefix correction

The CLOSURE-EXTRACTION.md proposed `:wat::kernel::__closure::__pkg_<n>`
as the synthetic name pattern for inline lambdas. Slice 1
discovered the freeze pipeline's `is_reserved_prefix` check rejects
`:wat::*` for user-minted symbols. Pivoted to `:__closure::__pkg_<n>`
(no leading `:wat::`).

Still unmistakable as substrate-internal. Updated test
assertions accordingly. Doc update for slice 2: CLOSURE-EXTRACTION.md
should reflect the actual pattern.

### Delta C — Value-kind encoding gaps surfaced

These Value kinds return `ExtractionError::Internal("not
implemented in slice 1")`:

- `Value::HolonAST`
- `Value::WatAST`
- `Value::RustOpaque(...)`
- `Value::holon__Vector` (the holon-domain vector, distinct from
  core Vector)
- `Value::Instant` / `Value::Duration`

T1-T15 don't surface these. The Internal error is a clean fail
rather than panic. If a future caller needs them, the gap is
visible in the diagnostic.

Slice 2 may reveal whether real consumers hit any of these; if so,
slice 1 follow-up extends `encode_value_to_ast`.

### Delta D — Source-spelling vs runtime type-name mismatch in diagnostic

The DESIGN example showed `:wat::kernel::Sender<i64>` for the
NonPortableCapture diagnostic; actual diagnostic emits
`rust::crossbeam_channel::Sender` (the Rust runtime Value's
`type_name()`).

Reason: `Value::crossbeam_channel__Sender` doesn't carry parametric
type info — it transports any `Value` generically; the `<T>` lives
at the type-check level, not the value level. Resolving to the
wat-surface spelling would require threading type-context through
extraction.

Slice 1 ships the runtime-level name. If slice 2 / 3 surfaces a
need for the wat-surface name, an extension point exists (pass
the type-checker's type info into `extract_closure`).

### Delta E — Topological sort edge tracking

Initial pass walked deps without back-edge tracking; T14's three-
level chain came out in discovery order rather than dep order.
Fixed by adding `current_walking_dep` field + recording
`consumer → dep` edges in `record_dep_dependency` BEFORE the
early-return on already-visited. Edges fire even when the dep is
already known.

Implementation iteration; not a discipline failure. Caught + fixed
within slice 1.

### Delta F — Auto-synthesized type accessor short-circuit

Type accessors (`:my::Point/x`, `:my::Point/new`) are
auto-synthesized by `register_struct_methods` when the type def
freezes. Initial pass extracted them as separate function deps;
re-freeze hit `DuplicateDefine` because both the manual extracted
form and the auto-synthesized form tried to define the same name.

Fix: detect the `<TypeName>/<rest>` shape; if `<TypeName>` is in
`captured_types`, skip recording the accessor as a dep (the type
def alone re-synthesizes it).

Same for enum tagged-variant constructors (`:Enum::Variant`).

## Q-impl resolutions

- **Q-impl-1 (macro expansion)**: post-expansion confirmed.
  `Function::body` carries post-expand AST. T13 verifies user
  defmacro-using code packages cleanly.
- **Q-impl-2 (captured fn values)**: NOT IMPLEMENTED in slice 1.
  Returns `Internal` error. Honest delta A.
- **Q-impl-3 (snapshot timing)**: monotonic confirmed. `&SymbolTable`
  + `&TypeEnv` references treated as snapshot.
- **Q-impl-4 (recursive types)**: `types_visited` HashSet works.
  T11 passes.
- **Q-impl-5 (span preservation)**: body forms preserve original
  spans; synthesized forms emit `Span::unknown()`. Acceptable
  trade-off.

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 90-180 min opus | ~150 min | A clean (within band; mid-upper) |

Mode A clean: 5 subsystems shipped + 17 tests + zero Mutex + FM 5
held throughout. Three iterations from "4 passing" to "15 passing"
(synthetic name → accessor short-circuit → syntactic marker
handling → edge tracking → source spelling fixes).

Subsystems:
- Free-symbol walker: ~250 lines / 4 internal sub-walkers
- Dep-closure builder: ~180 lines / fixpoint loop + visited-sets
- Value→AST encoder: ~400 lines / 16 Value kind arms (12 supported
  + 4 Internal-error stubs + ChannelBearing arms in portability
  check)
- Portability check: ~80 lines / 12 non-portable type kinds
- ClosurePackage assembly: ~370 lines / topological sort + body
  rewrite + cumulative-locals tracking

Honest deltas surfaced: 6 (A through F).

## Discipline check

- ✓ FM 5 held — `Value::wat__core__fn` encoding gap surfaced as
  `ExtractionError::Internal`, not bridged with TODO
- ✓ FM 9 honored — local cargo test verified 2108/0 post-spawn
- ✓ FM 10 — no type-system reach; closure extraction is AST
  walking + type-name matching; no union types invoked
- ✓ FM 11 — pre-INSCRIPTION grep deferred to slice 5 closure
- ✓ FM 16 honored — BRIEF didn't preempt tool availability
- ✓ Zero Mutex — no Mutex / RwLock / CondVar introduced
- ✓ Branch isolation held — main untouched

## What's next

Slice 2 — substrate consumer uses slice 1's closure extraction:

- Mint `:wat::kernel::ExitCode` typealias for `:wat::core::u8`
- Update `:user::main` signature (add argv :Vector\<String\>;
  return `:wat::kernel::ExitCode`)
- Mint `eval_kernel_spawn_process(fn)` — calls slice 1's
  `extract_closure` internally; reaches today's
  `fork-program-ast` pathway with the resulting forms
- Rename `eval_kernel_fork_program*` → `eval_kernel_spawn_process*`
- Delete `eval_kernel_spawn_program*` (Q1 settled)
- wat-cli passes `std::env::args()` to `invoke_user_main`
- Substrate-as-teacher walkers fire on legacy `:user::main` 3-arg
  signature + legacy fork-program / spawn-program verb usage
- New `tests/wat_arc170_program_contracts.rs` covering each
  contract + each spawn primitive + ExitCode + argv passthrough

Predicted: 90-180 min opus.

When slice 5 closure paperwork ships, **arc 109 v1 milestone
closure unblocks** per arc 109 INVENTORY.

## Companion docs

- DESIGN.md — full arc scope; client/server framing
- CLOSURE-EXTRACTION.md — substrate primitive deep-dive (the
  load-bearing doc this slice implemented)
- BRIEF-SLICE-1.md + EXPECTATIONS-SLICE-1.md — original slice 1 brief

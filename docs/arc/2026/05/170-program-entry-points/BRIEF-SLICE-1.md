# Arc 170 slice 1 — Rust closure extraction substrate primitive

## Goal

Build the substrate's closure extraction Rust capability as a
zero-wat-callers internal primitive. Given a `Value::wat__core__fn`
plus the parent's symbol table + type environment, produce a
`ClosurePackage { forms: Vec<WatAST>, entry: String }` that is
freezable in a fresh wat world such that the entry fn can be
invoked there with behavior equivalent to the original.

This is the load-bearing substrate work for arc 170. Future slices
2-5 build on top: slice 2 wires `eval_kernel_spawn_process` to
this primitive; slice 3 sweeps consumers; slice 4 retires legacy
substrate scaffolding; slice 5 closure paperwork.

**This slice has zero wat-level callers.** The primitive is
testable standalone via Rust integration tests that re-freeze the
extracted forms and verify behavioral equivalence with the
original fn.

## Branch + commit policy

- **Active branch**: `arc-170-program-entry-points` (already
  checked out and pushed; carries DESIGN v5 + CLOSURE-EXTRACTION.md)
- Multiple WIP commits + pushes welcome on the branch
- DO NOT push to main; orchestrator merges atomic to main as one
  squash commit after slice 5 closure paperwork ships

## Read first (in order)

1. `docs/arc/2026/05/170-program-entry-points/DESIGN.md` — full
   arc scope; client/server framing; settled decisions
2. `docs/arc/2026/05/170-program-entry-points/CLOSURE-EXTRACTION.md`
   — substrate primitive deep-dive; algorithm; invariants; test
   strategy. **This is the load-bearing doc for slice 1.**
3. `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6 (FM 5, FM 9, FM 10,
   FM 11) — discipline floor
4. Existing pieces to leverage (per CLOSURE-EXTRACTION.md §
   "Existing wat-rs pieces leveraged"):
   - `src/runtime.rs::SymbolTable::get` — symbol table lookup
   - `src/types.rs::TypeEnv::get` — type registry lookup
   - `src/runtime.rs` arc 091 slice 8 `struct→form` — Value→AST
     for structs (extending this is part of slice 1)
   - `src/freeze.rs::startup_from_forms` — child-world freeze
     pathway for tests
   - `src/runtime.rs::apply_function` — invoke entry post-freeze
   - `src/check.rs` — free-variable analysis patterns from infer

## Substrate edits

### 1. New module `src/closure_extract.rs`

Mint a new module containing the closure extraction implementation:

```rust
// public types
pub struct ClosurePackage {
    pub forms: Vec<WatAST>,
    pub entry: String,
}

pub enum ExtractionError {
    NonPortableCapture {
        name: String,
        type_name: String,
        path: Vec<String>,  // e.g., ["my-config", "tx-field"] for nested
    },
    UnresolvedSymbol {
        name: String,
        span: Span,
    },
    // ... other error kinds as the implementation surfaces them
}

// public entry point
pub fn extract_closure(
    fn_value: &Value,
    parent_symbols: &SymbolTable,
    parent_types: &TypeEnv,
) -> Result<ClosurePackage, ExtractionError>;
```

The fn dispatches on whether `fn_value` is a top-level defn
(resolvable in `parent_symbols`) or an inline lambda / factory
result (no canonical name). Synthetic name pattern for the lambda
case:
```
:wat::kernel::__closure::__pkg_<counter>
```
Counter sourced from a thread-local atomic or similar to ensure
uniqueness within a session.

### 2. Free-symbol walker

Walk the entry fn's body AST + every extracted dep AST. Track
scope:
- Fn parameters introduce scope (param names are LOCAL)
- `let` introduces scope (binding names are LOCAL within body)
- Nested fn introduces scope (param names LOCAL within its body)
- Vector / list / struct-pattern / etc. recurse without
  introducing scope

Free names = references not bound in any enclosing scope.

For each free reference, classify:
- Symbol matching `parent_symbols.get(name)`:
  - If entry is a `:wat::core::*` substrate primitive → SKIP
  - Else → USER DEPENDENCY (record for extraction)
- Keyword matching `parent_types.get(name)`:
  - If entry is a substrate primitive type → SKIP
  - Else → USER TYPE (record for extraction)
- Symbol matching the fn's CLOSURE ENV (let-scope local captured
  by lambda) → CAPTURED VALUE (record for AST encoding)

### 3. Dep-closure builder

Recursive extraction with fixpoint. For each USER DEPENDENCY /
USER TYPE recorded:
- Find the defining AST in `parent_symbols` / `parent_types`
- Walk it for further free references (recurse step 2)
- Add to dep set

Use a visited-set to avoid infinite recursion on recursive types
(e.g., struct with `:Vector<:Self>` field).

Output: dep set in topological order (deps before consumers).

### 4. Value→AST encoder

Extending `struct→form` (arc 091 slice 8). Per-Value-kind encoding
per CLOSURE-EXTRACTION.md § "Step 4". Cases:

- `i64`, `f64`, `bool` → direct literal
- `String` → string literal
- `nil` → `:wat::core::nil` keyword
- `Vector<T>` → `(:wat::core::Vec elem1 ...)` recursing
- `HashMap<K,V>` → `(:wat::core::HashMap (k v) ...)` recursing
- `Struct` → existing struct→form
- `Enum::Variant(payload)` → variant constructor form
- `Option`, `Result`, `Tuple` → constructor forms recursing
- `Bytes` → `(:wat::core::Bytes/from-hex "...")`
- Channel-bearing types → NON-PORTABLE (handled by step 5)

### 5. Portability type-check

Walk captured Values' types. If any type is in the non-portable
set, return `Err(NonPortableCapture)`:

Non-portable type set:
- `:wat::kernel::Sender<T>`
- `:wat::kernel::Receiver<T>`
- `:wat::kernel::Channel<T>`
- `:wat::kernel::Thread<I,O>`
- `:wat::kernel::Process<I,O>`
- `:wat::kernel::HandlePool<T>`
- `:wat::io::IOReader`
- `:wat::io::IOWriter`
- Transitive: any type with a non-portable field/payload

Diagnostic shape:
```
spawn-process closure captures `:my::tx` of type `:wat::kernel::Sender<i64>`.
Channel-bearing types cannot cross process boundaries (different memory).
Use stdin/stdout/stderr pipes for inter-process communication, or
restructure the program so the channel is created in the spawned program.
```

### 6. Assemble ClosurePackage

Output `forms`:
1. Type definitions (struct / enum / newtype / typealias) in
   topological order
2. Capture binding defines: `(:wat::core::define :__captured_X <encoded-ast>)`
3. User dependency defines in topological order
4. Entry fn defining AST (last)

Body rewrite: if the entry fn captures locals, rewrite body
references from `X` to `:__captured_X` (or preserve original
names if no collision; use synthetic prefix only on collision).
Lean: synthetic prefix always — simpler.

Output `entry`: the keyword path of the entry's defining symbol
(canonical if caller's fn was a top-level defn; synthetic
`:wat::kernel::__closure::__pkg_<n>` if lambda).

### 7. Tests `tests/wat_arc170_closure_extraction.rs`

15 Rust integration tests per CLOSURE-EXTRACTION.md § "Test
strategy". Each test:
1. Compose a parent program (a wat source string that defines
   types + fns + the entry fn)
2. Freeze the parent world via `startup_from_source` or
   equivalent
3. Get the entry fn Value from the parent's symbol table
4. Call `extract_closure(fn, parent_symbols, parent_types)`
5. Assert ClosurePackage shape (entry name; forms structure)
6. Re-freeze a fresh world from `package.forms` via
   `startup_from_forms`
7. Look up `package.entry` in the fresh world's symbol table
8. Invoke with test args via `apply_function`
9. Compare observable behavior against invoking the original fn

Test cases:

- **T1.** Top-level defn, no deps, no captures
- **T2.** Top-level defn, calls other top-level defns (recursive
  dep extraction)
- **T3.** Top-level defn, uses user types (struct / enum / newtype
  / typealias)
- **T4.** Inline lambda, no captures
- **T5.** Inline lambda captures let-scope value (struct)
- **T6.** Lambda captures multiple values, mixed types (i64 +
  struct + Vector)
- **T7.** Factory pattern (defn returns fn capturing factory's
  arg)
- **T8.** Lambda captures non-portable value (Sender) → NEGATIVE
  → returns `Err(NonPortableCapture)` with substrate-as-teacher
  diagnostic
- **T9.** Lambda captures struct holding a Sender field →
  NEGATIVE → same Err with offending field path named
- **T10.** Captures with type alias
- **T11.** Captures with recursive struct (`:Vector<:Self>`)
- **T12.** Body uses macro that expanded to substrate primitives
  only
- **T13.** Body uses user-defined macro
- **T14.** Body calls user fn that calls another user fn
  (three-level dep chain; topological order verification)
- **T15.** End-to-end behavior equivalence verification across
  T1-T7 (extract → freeze → invoke; output matches original
  invocation)

## Verification

Use the inline cargo+grep+awk pipeline (no scripts; sonnet-friendly):

```bash
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk '{p+=$4; f+=$6} END {print "passed:", p, "failed:", f}'
```

Pre-slice baseline: 2091/0 (post-arc-169).
Post-slice target: 2091 + 15 = 2106 / 0 (15 new closure_extraction
tests all green).

To get failing test names:
```bash
cargo test --release --workspace --no-fail-fast 2>&1 | grep " FAILED" | head -20
```

## Discipline reminders

- DO NOT push to main; only push to slice branch
- DO NOT expose closure extraction at the wat level — substrate
  internal Rust capability only
- DO NOT modify spawn-process / spawn-thread / fork-program*
  invocation paths in this slice — that's slice 2's territory
- DO NOT modify `:user::main` signature in this slice — that's
  slice 2's territory
- USE the inline pipeline for verification; no scripts
- DO NOT pipe `cargo test` through anything beyond the documented
  awk pattern

## FM 5 GUARDRAIL — explicit

If a substrate quirk surfaces (the extraction algorithm needs a
substrate hook that doesn't exist; macro expansion timing is
unclear; closure-of-closure handling needs a primitive that's
missing):

- STOP and report
- DO NOT bridge by leaving an extraction case unhandled with a
  TODO
- DO NOT exit early with success when invariants don't hold
- DO NOT modify substrate code outside the closure_extract module
  to "make it work" without flagging the dep
- The right answer is always: STOP, name the gap, let
  orchestrator decide

If you find yourself wanting to use Mutex / RwLock / CondVar →
STOP. Per memory `feedback_zero_mutex.md`: wat-rs has zero Mutex
by design. Use Arc + ThreadOwnedCell + atomic + OnceLock.

If you find yourself reaching for "we need union types" / "the
type system can't express..." → STOP per FM 10. Different framing
needed. The closure extraction problem is mostly AST walking +
type-name matching; it shouldn't require type-system extensions.

## Implementation Q's pre-flagged (per CLOSURE-EXTRACTION.md § "Open questions")

These will surface during slice 1; resolve as you encounter them
and document the resolution in your report:

- **Q-impl-1.** Macro expansion timing — assume post-expansion;
  if pre-expansion shapes surface, flag.
- **Q-impl-2.** Captured fn values — handle as sub-extraction;
  merge into parent ClosurePackage.
- **Q-impl-3.** Symbol-table snapshot timing — package-time snapshot;
  monotonic.
- **Q-impl-4.** Recursive type definitions — visited-set during
  type closure walk.
- **Q-impl-5.** Span preservation — preserve for diagnostics.

## Report shape

When complete, report:

1. Final cargo test summary via the inline pipeline (target:
   2106/0)
2. Each subsystem you built (free-symbol walker, dep closure
   builder, Value→AST encoder, portability check, ClosurePackage
   assembly) with a one-paragraph summary per piece
3. Resolutions for the 5 implementation Q's (or surface new ones)
4. Honest deltas — substrate quirks discovered; what was harder
   than expected; what surfaced unexpectedly
5. Test sample of one negative-case diagnostic (paste the actual
   Err(NonPortableCapture) message text — substrate-as-teacher
   verification)
6. Branch state confirmation (commit hash(es))
7. Actual runtime in minutes vs predicted band

## Time-box

Per EXPECTATIONS-SLICE-1.md (90-180 min opus predicted, 360 min
hard cap).

If you exceed 180 min still iterating, STOP and report current
state.

## What's next (post-slice-1)

When slice 1 ships green, slice 2 (substrate consumer) opens. It
wires `eval_kernel_spawn_process` to call slice 1's
`extract_closure` internally. The closure extraction primitive
gains its first wat-level caller via the spawn-process pathway.
Slices 3 (consumer sweep), 4 (substrate retirement), and 5
(closure paperwork) follow.

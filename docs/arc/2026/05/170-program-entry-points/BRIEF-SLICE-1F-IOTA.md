# Arc 170 slice 1f-ι — BRIEF (println/readln EDN contract)

**Opus.** Substrate change: lock in the EDN-only stdio contract for `:wat::kernel::println` / `:wat::kernel::readln`. Existing tests WILL break — that is correct and expected per user direction 2026-05-10. Subsequent slices migrate the broken tests; this slice ships the substrate contract.

## The locked contract

```
:wat::kernel::println (v :T) -> :wat::core::nil       ; polymorphic in T
:wat::kernel::readln -> :T                            ; polymorphic in T via -> :T annotation
```

**Round-trip semantics:**
```
server: (:wat::kernel::println 42)                        → emits  42 (EDN i64)
reader: (:wat::kernel::readln -> :wat::core::i64)         → returns 42 (i64)

server: (:wat::kernel::println "foo")                     → emits  "foo" (EDN String, quoted)
reader: (:wat::kernel::readln -> :wat::core::String)      → returns "foo" (String)

server: (:wat::kernel::println (:Tuple 1 "x"))            → emits  [1 "x"]
reader: (:wat::kernel::readln -> :(:i64, :String))        → returns (1, "x")
```

T is any wat type with EDN encoding/decoding: primitives, tuples, Vector, Option, Result, user structs/enums, and `:wat::holon::HolonAST` (when caller wants raw AST form).

## Required edits

### 1. EDN → T coercion (NEW; load-bearing)

**Location:** `src/edn_shim.rs` (or sibling module if cleaner).

**Function:**
```rust
pub fn edn_to_typed_value(
    target: &TypeExpr,
    edn: &wat_edn::Value,
    sym: &SymbolTable,
) -> Result<Value, EdnCoerceError>;
```

**Recursive coercion rules** (handle the cases the substrate type system supports):

| Target | EDN form expected | Coercion |
|---|---|---|
| `:wat::core::i64` | Integer | `Value::i64(n)` |
| `:wat::core::f64` | Float OR Integer (widening) | `Value::f64(f)` |
| `:wat::core::String` | String | `Value::String(s.into())` |
| `:wat::core::bool` | Bool | `Value::Bool(b)` |
| `:wat::core::nil` | Nil | `Value::Unit` |
| `:(A, B, ...)` (tuple) | Vec of len N | Coerce each element to A, B, ... |
| `:wat::core::Vector<T>` | Vec | Coerce each element to T |
| `:wat::core::Option<T>` | Nil → None; else coerce to Some(T) | enum variant |
| `:wat::core::Result<T, E>` | Tagged enum form | variant + payload |
| user `Struct` | Map keyed by field-name | Coerce each field by its declared type |
| user `Enum` | Tagged form (variant-name + payload) | Coerce payload to variant's field types |
| `:wat::holon::HolonAST` | Any EDN | Call existing `edn_to_holon_ast` |

Errors return `EdnCoerceError { expected: TypeExpr, got: EdnShape, path: ... }` — `path` accumulates field/element indices as recursion descends, so the diagnostic names the exact mismatch site.

### 2. `readln` eval arm — `-> :T` annotation read

**Current state** (`src/thread_io.rs:240+`): `eval_kernel_readln` returns `Value::holon__HolonAST(ast)`.

**New state:** the eval arm reads the call-site's `-> :T` annotation (mirror pattern from `option::expect`, `Result::expect`, `eval-ast!`):
1. Extract target type T from the AST node's annotation
2. Read line from stdin via existing ThreadIO routing
3. Parse line via `wat_edn::read` → `wat_edn::Value`
4. Call `edn_to_typed_value(T, edn, sym)` → `Result<Value, EdnCoerceError>`
5. Return Value or surface error

**Mirror these existing sites** for the annotation read pattern:
- `src/runtime.rs` — search for `option__expect` / `eval_ast` to see how `-> :T` annotations are read

### 3. Type-check arm for `readln`

**Current state** (`src/check.rs`): readln's TypeScheme says `() -> :wat::holon::HolonAST`.

**New state:** readln's TypeScheme is polymorphic: `() -> :T`. The check site needs to read the call's `-> :T` annotation, validate that T is EDN-decodable (probably accept all wat types as a first pass; reject types that obviously can't EDN-decode like function types), and propagate T as the call's return type.

Pattern source: how `option::expect -> :T` is type-checked. Mirror.

### 4. Add `RuntimeError::EdnCoerceMismatch`

New variant in `RuntimeError` enum with rendering for the diagnostic (expected type + EDN shape + path). Display impl that prints something like:
```
edn coerce mismatch: expected :wat::core::i64, got String at <path>
```

### 5. Migrate ambient-stdio readln-echo test

`wat-tests/kernel/services/ambient-stdio.wat` Layer 4 currently uses readln expecting HolonAST. Update to `(:wat::kernel::readln -> :wat::core::String)`. Assertion expectations update accordingly (no more `#wat-edn.holon/String "..."` tag wrapping).

### 6. Verify `println` is already clean

For native types (`:i64`, `:String`, `:f64`, `:bool`, etc.), `value_to_edn_with` should emit untagged canonical EDN. Verify by reading the encoding function + checking the on-wire form. **If HolonAST values emit with a tag** at top level (the `#wat-edn.holon/String "..."` form), fix the encoder to omit the top-level tag for cleanness — but the more important fact is that **callers should now rarely have HolonAST values** (readln returns native types).

## What will break (expected; do NOT fix in this slice)

- Any test calling `(:wat::kernel::readln)` without `-> :T` annotation → type error
- Any test asserting on tagged EDN encoding like `#wat-edn.holon/String "foo"` → no longer matches
- Slice 1f-θ V3's ambient-stdio.wat — migrate as part of THIS slice (only existing readln consumer; one file)

**Other broken tests are subsequent-slice work** (1f-κ / λ / μ). Don't fix them here; surface count.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `edn_to_typed_value` exists in substrate; handles all type variants from the table | grep + unit tests |
| B | `readln` eval arm reads `-> :T` annotation + calls coercion | grep |
| C | `readln` type-check arm accepts polymorphic `-> :T` | grep |
| D | `RuntimeError::EdnCoerceMismatch` variant exists with rendering | grep |
| E | `wat-tests/kernel/services/ambient-stdio.wat` Layer 4 updated to `readln -> :String`; all 5 tests pass | cargo test |
| F | `cargo check --release` green | clean |
| G | `println` emits canonical EDN for native types verified | manual trace |
| H | Workspace failure count delta surfaced (likely INCREASE due to broken callers; that's expected) | cargo test count |
| I | Honest deltas surfaced | per FM 5 |

**9 rows.** Row H is expected to grow — substrate-as-teacher pattern; downstream slices close the failures.

## Predicted runtime

**90-180 min opus.** Substantive substrate work: new coercion function (~150 lines), eval arm change, type-check arm change, error variant, ambient-stdio migration.

**Hard cap:** 360 min.

## Honest delta categories (anticipated)

1. **`-> :T` annotation parsing already exists** somewhere — verify the canonical site (`option::expect` likely) and reuse the parsing helper. If not factored, this slice may extract it.

2. **EDN tuple representation** — wat tuples vs Vec — the EDN form needs to be unambiguous. Surface what choice the codec makes (`[1 "x"]` vs `(1 "x")`).

3. **Struct field encoding** — `{:field-name value}` map or some other shape. Verify what `value_to_edn_with` currently produces for structs, then mirror in the coercion.

4. **Enum tagged form** — `{:variant-name [payload...]}` or `(variant-name payload...)`. Same — verify what's produced; mirror for parsing.

5. **HolonAST as fallback** — `(:wat::kernel::readln -> :wat::holon::HolonAST)` should still work for callers that genuinely want AST form. Verify this path.

6. **Workspace failure count rises** — expected. Surface exact post-1f-ι count.

## What to NOT do

- No fixing of downstream broken tests (1f-κ / λ / μ are separate slices)
- No raw-stdout primitive (`print-str` etc.) — EDN-only contract is firm
- No vocare runes
- No `readln` overload preserving old `-> :HolonAST` signature alongside new — full replacement; HolonAST is one of the T's, not a default
- Don't commit yourself — orchestrator atomic-commits with SCORE

## Reference

- `src/edn_shim.rs` — existing EDN encoding; add coercion here
- `src/thread_io.rs:240+` — current readln eval arm
- `src/runtime.rs` — `option__expect`, `eval_ast` for `-> :T` annotation read pattern
- `src/check.rs` — type-check arms for `-> :T` polymorphic ops
- `wat-tests/kernel/services/ambient-stdio.wat` — only existing readln consumer; migrate in same slice
- User direction 2026-05-10: "go make println and readln work — it'll break a bunch of existing tests which is correct — we must fix them after we make the contract work"

## Path forward post-1f-ι

1. Orchestrator scores; atomic-commits + pushes
2. **Slice 1f-κ** — fix-up sweep for broken callers (the slice-1f-ζ-style mass migration; whatever shape downstream tests took, they get updated to the new readln contract)
3. **Slice 1f-λ** — migrate retired-verb tests to wat-side wrappers
4. **Slice 1f-μ** — rewrite raw-stdout tests for EDN-only contract
5. Push to zero failing tests
6. Verify no leaks persist

# Arc 138 F4b — Sonnet Brief: FromWat trait expansion

**Goal:** expand `FromWat` trait in src/rust_deps/marshal.rs — `from_wat(v: &Value, op: &'static str)` gains `span: Span` parameter. Update ~10 impls (i64, f64, bool, String, (), Option<T>, Vec<T>, tuple_macro, Result<T,E>, etc.). Update recursive calls within impls (Option<T>, tuples, Result, Vec — they call inner `T::from_wat(...)`). Update single caller site: proc-macro emit at crates/wat-macros/src/codegen.rs:165.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user invoked the no-deferrals rule. F4b is sub-item 2 of 3 in F4 decomposition (F4a closed; F4c last).

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md` — F4 decomposition.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-F2-SCHEMECTX.md` — same shape: trait expansion + proc-macro emit update.
3. `docs/arc/2026/05/138-checkerror-spans/SCORE-F3-IOTRAIT.md` — trait expansion + impl updates.
4. `src/rust_deps/marshal.rs` — entire file (it's small).
5. `crates/wat-macros/src/codegen.rs` lines 160-170 (the proc-macro emit site).

## What to do

### 1. Trait expansion (src/rust_deps/marshal.rs:47)

```rust
pub trait FromWat: Sized {
    fn from_wat(v: &Value, op: &'static str, span: crate::span::Span) -> Result<Self, RuntimeError>;
}
```

Add `use crate::span::Span;` if needed.

### 2. Impl updates (10 impls, lines 58-289+)

Each impl: add `span: Span` parameter; use it in error construction:

```rust
impl FromWat for i64 {
    fn from_wat(v: &Value, op: &'static str, span: Span) -> Result<Self, RuntimeError> {
        match v {
            Value::i64(n) => Ok(*n),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "i64",
                got: other.type_name(),
                span,  // arc 138 F4b: real span threaded through
            }),
        }
    }
}
```

DELETE all `// arc 138: no span — FromWat::from_wat receives evaluated Value, no WatAST trace available` comments.

### 3. Recursive call updates

Several impls call `T::from_wat(...)` recursively:
- `Option<T>` (line 167+): `T::from_wat(x, op, span.clone())?`
- Tuple macro (line ~205+): `$name::from_wat(&items[$idx], op, span.clone())`
- `Result<T,E>` (line ~257+): `T::from_wat(inner, op, span.clone())` and `E::from_wat(inner, op, span.clone())`
- `Vec<T>` (line ~287+): `T::from_wat(x, op, span.clone())`

Pass `span.clone()` to each recursive call (the same span — every element shares the span of the outer Value's source).

### 4. Proc-macro emit (crates/wat-macros/src/codegen.rs:165)

Current emit:
```rust
let #bind_ident: #ty = <#ty as ::wat::rust_deps::FromWat>::from_wat(
    &::wat::runtime::eval(&args[#idx], env, sym)?,
    #wat_path,
)?;
```

Add span argument:
```rust
let #bind_ident: #ty = <#ty as ::wat::rust_deps::FromWat>::from_wat(
    &::wat::runtime::eval(&args[#idx], env, sym)?,
    #wat_path,
    args[#idx].span().clone(),
)?;
```

The `args[#idx]` is in scope of the emitted function at runtime (proc-macro emits this code into a function that takes `args: &[WatAST]`). Safe because the arity guard fires first → args[idx] is in-bounds.

## Constraints

- 2 files modified: src/rust_deps/marshal.rs + crates/wat-macros/src/codegen.rs. NO others.
- NO new variants. NO Display string changes.
- NO commits, NO pushes.
- All 6 existing arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- The proc-macro emit produces compiling code (workspace tests exercise it via #[wat_dispatch]).
- 17 `// arc 138: no span` rationale comments in marshal.rs → 0.

## Reporting back

Compact (~300 words):

1. **Diff stat:** 2 files.
2. **Trait expansion confirmed:** from_wat gains span.
3. **10 impls updated:** confirm each.
4. **Recursive calls updated:** Option<T>, tuples, Result<T,E>, Vec<T> pass span.clone().
5. **Proc-macro emit:** codegen.rs:165 quote! block emits args[#idx].span().clone() as 3rd arg.
6. **Pre/post Span::unknown() count** in marshal.rs (target: 17 → 0).
7. **Verification:** all 6 canaries pass; workspace tests pass.
8. **Honest deltas.**
9. **Four questions** briefly.

## Why this is small

Single trait + ~10 impls in one file + one proc-macro emit site. F2 was identical shape (SchemeCtx trait + impl + proc-macro emit) and ran in 6 min. Estimated 8-15 min.

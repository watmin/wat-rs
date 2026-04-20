# `#[wat_dispatch]` — design notes (2026-04-19)

Durable record of the proc-macro design arc. Follows the hand-written
`:rust::lru::LruCache` shim — we wrote it by hand first so we KNOW what
the macro must generate.

## Motivation

Without the macro, every Rust dep surfaced to wat needs ~100-300 lines
of boilerplate: per-method `dispatch` fn, per-method `scheme` fn,
registration into `RustDepsBuilder`, argument marshaling, return
marshaling, scope-enforcement wiring. For N methods × M deps, this
becomes a lot of hand-written repetition.

The macro collapses the work to ONE annotation on an `impl` block. The
generated code is IDENTICAL in structure to the hand-written shim;
nothing new is added. The macro is pure ergonomics.

Ergonomic target:

```rust
#[wat_dispatch(path = ":rust::lru::LruCache", scope = "thread_owned")]
impl<K: Hash + Eq, V: Clone> lru::LruCache<K, V> {
    fn new(cap: i64) -> Self { lru::LruCache::new(NonZeroUsize::new(cap as usize).unwrap()) }
    fn put(&mut self, k: K, v: V) { self.put(k, v); }
    fn get(&mut self, k: K) -> Option<V> { self.get(&k).cloned() }
}
```

The user writes idiomatic Rust; the macro generates:
- `fn dispatch_new(args, env, sym) -> Result<Value, RuntimeError>` etc.
- `fn scheme_new(args, ctx) -> Option<TypeExpr>` etc.
- `pub fn register(builder: &mut RustDepsBuilder)` that wires everything.
- `Value::rust_opaque`-wrapping with thread-scope guard for `Self` returns
  when `scope = "thread_owned"`.

## Layout

```
wat-rs/                      (workspace root)
├── Cargo.toml               (workspace manifest + wat package)
├── src/                     (existing `wat` crate)
├── wat-macros/              (NEW proc-macro crate)
│   ├── Cargo.toml           ([package] proc-macro = true)
│   └── src/
│       ├── lib.rs           (proc-macro entry)
│       ├── attr.rs          (attribute parsing)
│       ├── codegen.rs       (dispatch/scheme/register emission)
│       └── marshal.rs       (Rust-type ↔ wat-Value inference)
└── ...
```

Both crates in one repo, separate Cargo manifests, separate crates.io
releases. User crates depending on wat-rs get `wat-macros` transitively
via wat-rs's `re-export = ["wat_dispatch"]` feature flag (or directly).

## The three things the macro emits

For each method `fn foo(args...) -> Ret` in the annotated impl:

### 1. Dispatch function

```rust
fn dispatch_foo(args: &[WatAST], env: &Environment, sym: &SymbolTable)
    -> Result<Value, RuntimeError>
{
    if args.len() != EXPECTED_ARITY {
        return Err(RuntimeError::ArityMismatch { ... });
    }
    // Per-arg: evaluate AST → Value → Rust type
    let arg_0: TypeOfArg0 = FromWat::from_wat(&eval(&args[0], env, sym)?)?;
    let arg_1: TypeOfArg1 = FromWat::from_wat(&eval(&args[1], env, sym)?)?;
    // Call
    let ret: Ret = TypePath::foo(arg_0, arg_1);
    // Return marshaling
    Ok(ret.to_wat())
}
```

### 2. Scheme function

```rust
fn scheme_foo(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    if args.len() != EXPECTED_ARITY {
        ctx.push_arity_mismatch(":rust::...::foo", EXPECTED_ARITY, args.len());
        return Some(TypeExpr::/* fallback */);
    }
    // Per-arg: infer + unify with declared type
    let arg_ty = ctx.infer(&args[0])?;
    ctx.unify(&arg_ty, &TypeExpr::Path("i64"))?;
    // etc.
    Some(/* declared return type */)
}
```

### 3. Registration fn

```rust
pub fn register(builder: &mut RustDepsBuilder) {
    builder.register_type(RustTypeDecl { path: ":rust::lru::LruCache" });
    builder.register_symbol(RustSymbol {
        path: ":rust::lru::LruCache::new",
        dispatch: dispatch_new,
        scheme: scheme_new,
    });
    builder.register_symbol(RustSymbol {
        path: ":rust::lru::LruCache::put",
        dispatch: dispatch_put,
        scheme: scheme_put,
    });
    // … one per method
}
```

The user then calls `lru_shim::register(&mut builder)` from their
wat-vm's main — just like with the hand-written shim.

## Type marshaling

The macro emits `FromWat`/`ToWat` calls. These traits are defined in
the `wat` crate (NOT in `wat-macros` — the macro crate emits calls,
doesn't implement them):

```rust
pub trait FromWat: Sized {
    fn from_wat(v: &Value) -> Result<Self, RuntimeError>;
}

pub trait ToWat {
    fn to_wat(self) -> Value;
}
```

### Blanket impls wat ships

- `i64`, `f64`, `bool`, `String`, `&str` → primitives.
- `Option<T: ToWat>` → `Value::Option`.
- `Vec<T: ToWat>` → `Value::Vec`.
- `(A, B)` / `(A, B, C)` etc. → `Value::Tuple`.
- `Result<T, E: ToWat>` → (deferred to when wat's `:Result<T,E>` lands;
  see caching-design notes).

### User-defined types

User's Rust types (e.g., `rusqlite::Connection`, `lru::LruCache<K,V>`)
become OPAQUE HANDLES via a new `Value::RustOpaque` variant:

```rust
Value::RustOpaque(Arc<RustOpaqueInner>)

pub struct RustOpaqueInner {
    pub type_path: &'static str,       // ":rust::lru::LruCache"
    pub scope: Scope,                  // owner-thread guard or not
    pub payload: Box<dyn Any + Send + Sync>,
}
```

The macro emits `ToWat` impls for the user's type automatically when
annotated. `FromWat` extracts the Any-downcast after a scope check.

### The scope attribute

```rust
#[wat_dispatch(path = "...", scope = "thread_owned")]  // thread-id guard
#[wat_dispatch(path = "...", scope = "shared")]         // immutable Arc<T>
#[wat_dispatch(path = "...", scope = "owned_move")]     // consumed by shim
```

For `thread_owned` (LruCache, rusqlite::Connection in some configs):
- Macro wraps `Self` returns with a `ThreadOwnedOpaque` that records
  `thread::current().id()` at construction.
- Every `FromWat` of that opaque type asserts `thread::current().id()
  == owner`; otherwise errors.

For `shared` (immutable results, query rows, etc.):
- Plain `Arc<T>`, no guard.

For `owned_move` (consumed handles like prepared-statement bindings):
- FromWat takes ownership out of the Arc; subsequent access errors.

## Attribute parsing

Use `syn` + `syn::parse::Parse`:

```rust
struct WatDispatchAttr {
    path: String,
    scope: Scope,
}

impl Parse for WatDispatchAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Expect: path = "...", scope = "..."
        // scope defaults to "shared" if omitted.
    }
}
```

## Method-signature inspection

For each `ImplItemMethod` in the annotated impl:
1. Name → dispatch-fn name, scheme-fn name, wat keyword path.
2. Receiver (`&self`, `&mut self`, `self`, none) → first-arg marshaling.
3. Each arg's `Type` → `FromWat` call + wat type expression.
4. Return type → `ToWat` call + wat scheme return.

Tricky cases (that the macro needs to handle):
- `Self` in return position — rewrite to the annotated type path.
- `Option<T>` — recursive marshaling.
- Generic type parameters (K, V) — they become fresh type vars in the
  scheme; the runtime marshaling uses `Value` as the erased type.
- Lifetimes — proc macro can strip (we don't expose lifetimes to wat).
- `&T` vs `&mut T` receiver — marshals the receiver differently (clone
  vs take); the scope attribute gates which is allowed.

## Error handling during codegen

Macro errors must surface WHERE THE USER WROTE THE INVALID CODE.
`proc_macro2::Span` + `syn::Error::new_spanned` gives us this for free.
Example failure modes:
- Unsupported arg type — "cannot marshal type `Foo`; only primitives, 
  opaque handles, `Option`, `Vec`, tuples, and `Result` are supported."
- Missing `path` attr — span to the attribute.
- Conflicting scope attrs — span to the offending attr.

## Build order

Step-by-step, in priority order. Each step ships independently green.

### Step 1 — workspace conversion

1. Convert `Cargo.toml` to `[workspace]` + `[package]`.
2. Verify build/test still green with no changes.

### Step 2 — `FromWat` / `ToWat` traits in `wat` crate

1. Add traits to `src/rust_deps/marshal.rs` (new file).
2. Impls for i64, f64, bool, String.
3. Impls for `Option<T>`, `Vec<T>`, tuples.
4. Add `Value::RustOpaque` variant + `ThreadOwnedOpaque` / `SharedOpaque`
   / `OwnedMoveOpaque` wrappers.
5. Unit-test each marshaling round-trip.

### Step 3 — `wat-macros` crate bootstrap

1. Create `wat-macros/Cargo.toml` with `proc-macro = true`, `syn`, `quote`,
   `proc-macro2`.
2. Create `wat-macros/src/lib.rs` with a minimal `#[wat_dispatch]` that
   just PARSES the attribute + impl (no codegen yet).
3. Verify an empty impl compiles under the attribute.

### Step 4 — method-level codegen

1. `codegen::dispatch_fn(method, attr) -> TokenStream` — the dispatch
   body per method.
2. `codegen::scheme_fn(method, attr) -> TokenStream` — the scheme body.
3. `codegen::register_fn(methods, attr) -> TokenStream` — the register fn.
4. Test by writing a minimal marker impl (e.g., a Rust `struct Pair(i64, i64)`
   with `fn swap(self) -> Self`) and verify generated output.

### Step 5 — scope handling

1. Implement `scope = "shared"` first (simplest — plain Arc).
2. Implement `scope = "thread_owned"` — emit the thread-id guard.
3. Implement `scope = "owned_move"` — emit the ownership-consume.

### Step 6 — re-generate lru shim

1. Annotate `lru::LruCache` with `#[wat_dispatch]` in a test shim
   (we can't annotate the external `lru` crate directly — use a
   newtype wrapper `struct WatLruCache<K,V>(lru::LruCache<K,V>)` and
   annotate that).
2. Run the full LocalCache test suite.
3. Replace the hand-written `src/rust_deps/lru.rs` with the macro-generated
   version. `git diff` should show reduced line count; behavior unchanged.

### Step 7 — apply to rusqlite (first external use)

1. In `holon-lab-trading` (or a test fixture in wat-rs), write a
   `rusqlite_shim.rs` using `#[wat_dispatch]` on a `Connection` wrapper.
2. End-to-end wat test: open, execute, query, close.
3. If any gaps surface (e.g., variadic params, callback closures from
   Q#2-3 decisions), iterate on the macro.

## Deferred / out of scope for v1

- **Variadic parameters** (SQL bind params) — see caching-design notes;
  we agreed on `:Tuple<...>` encoding. Add to marshaling traits when we
  have a caller.
- **Closure callbacks** (row mapping) — agreed bidirectional (Q#3 = A).
  Requires the macro to emit code that calls back into the wat evaluator.
  Significant scope; defer to v2.
- **`:Result<T, E>` variant** (Q#1 = C, agreed). Land the wat-level type
  first; the macro emits calls into it.

## Test strategy

- **Marshaling round-trips**: `t.to_wat().from_wat()` = t, for each
  primitive type. Unit tests in `wat` crate.
- **Macro expansion**: use `trybuild` or `macrotest` — pin the expected
  expansion of a canonical `#[wat_dispatch]` input. When the macro
  changes, the expected output changes, and the diff is the review.
- **Integration**: the full LocalCache suite passes on macro-generated
  lru shim.
- **Failure modes**: compile_fail tests for each macro error.

## Open questions that don't block Step 1

- How much of the ergonomic polish should be in v1? (E.g., should the
  macro emit documentation comments on generated fns?)
- How do we expose the macro to users: re-export from `wat` crate or
  direct dep on `wat-macros`?
- Naming bikeshed: `#[wat_dispatch]` vs `#[wat_bind]` vs `#[wat_export]`.

## Pivot signal

This macro is the enabler for holon-lab-trading's rusqlite driver —
and for every future Rust-dep integration. It's not a wat-rs internal;
it's the consumer-facing surface for "how do I use my Rust crate from
wat." Landing it cleanly makes wat programs in external projects
~10x less tedious to wire up.

We attack it head-on because it's the leverage point for wat adoption.

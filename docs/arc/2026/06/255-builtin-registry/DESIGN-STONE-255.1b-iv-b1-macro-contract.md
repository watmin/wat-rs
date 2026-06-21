# Stone 255.1b-iv-b1 — the compile-time doc contract in `#[wat_intrinsic]`

**Why this stone.** iv-a built `wat-doc` (parser + `check_args`) in isolation. iv-b1 **wires it into the
macro**: at expand time the `///` block is parsed, the required directives enforced (`compile_error!`),
the `@arg`s checked against the signature, and the **structured doc carried on the registry entry** so
runtime reflection (and iv-b2's wat verifier seam) can read it. The forcing function: `core::Bytes`
must be decorated to the full contract or the crate does not compile.

This is the Rust, compile-time half (R2's split). iv-b2 is the wat, runtime half (the self-hosting
verifier). iv-c is the enum flip.

## The contract decisions (pinned)

### 1. `wat-macros` depends on `wat-doc`
`crates/wat-macros/Cargo.toml` gains `wat-doc = { path = "../wat-doc" }`. A proc-macro crate may depend
on a plain leaf crate; this is the parity-by-construction seam (§10) — the macro parses with the SAME
`wat_doc::parse` the wat side will use.

### 2. The registry entry carries the structured doc (submission + entry expand)
`src/intrinsic/mod.rs` — replace the single `doc: Option<&'static str>` with the structured, `'static`
shape (the macro emits string-literal fields, so all `&'static str`):

```rust
pub(crate) struct ExampleSubmission {
    pub expr: &'static str,
    pub expected: Option<&'static str>,
    pub run: bool,
}

pub(crate) struct IntrinsicSubmission {
    pub name: &'static str,
    pub handler: NativeHandler,
    pub arity: usize,
    pub prose: &'static str,                          // @-prose (the :doc body)
    pub added: &'static str,                          // @added
    pub args: &'static [(&'static str, &'static str)],// (name, desc) ×N
    pub ret: &'static str,                            // @ret
    pub examples: &'static [ExampleSubmission],       // @example / @example-norun (≥1)
    pub deprecated: Option<(&'static str, &'static str)>, // (since, use-instead)
    pub see: &'static [&'static str],                 // @see ×N
}
```
`IntrinsicEntry` mirrors it; `registry()` copies the fields through. (The old `doc` field is GONE — a
clean replace, not an addition; `metadata-of`'s `entry.doc` read is rewritten in step 4.)

### 3. The macro parses + enforces + emits structured (`crates/wat-macros/src/wat_intrinsic.rs`)
- `sniff_arity` → **`sniff_args`**: return the leading `&WatAST` param **names** (idents as `String`s),
  not just the count. (Arity = `args.len()`.) The same leading-prefix / STOP-on-variadic rules hold.
- `sniff_doc` stays (collects the joined `///`), but its output now feeds `wat_doc::parse`:
  - `wat_doc::parse(&raw)` → on `Err(e)` → `compile_error!` via `syn::Error::new_spanned(item, msg)`
    with a precise message (`"#[wat_intrinsic] <fqdn>: <e>"`); on `Ok(doc)` continue.
  - `wat_doc::check_args(&doc, &arg_names)` → on `Err` → `compile_error!` (documented args must match
    the signature). `arg_names` are the `sniff_args` idents.
- `emit` builds the `inventory::submit!` with the structured literals from `doc` (prose/added/args/
  ret/examples/deprecated/see as string-literal tokens; examples as `ExampleSubmission { … }` literals).
- A `DocError → String` rendering lives in the macro (precise per-variant messages). (`DocError`
  derives `Debug`; a `Display`/match in the macro gives the human message.)

### 4. `metadata-of` reads the richer fields (`runtime.rs` `eval_metadata_of` ~10104)
Rewrite the `entry.doc` read: emit `:doc` ← `entry.prose`, `:added` ← `entry.added` (string),
`:ret` ← `entry.ret` (string). Keep the derived baseline (`:name`/`:kind`/`:defined-in`/`:layer`/
`:arity`/`:pure`/`:deterministic`) unchanged (still keyword values — the enum flip is iv-c).

### 5. Decorate `core::Bytes` to the full contract (`src/intrinsic/bytes.rs`)
The two handlers' `///` become the full form (prose + `@added 1.0.0` + `@arg <name> — …` matching the
param + `@ret — …` + `@example … #=> …`). This is the forcing function: under-document either and the
build breaks. The `@example`s must be **runnable+correct** (they become iv-b2's doctests): e.g.
`@example (:wat::core::Bytes::to-hex (:wat::core::Vector 255 0 16)) #=> "ff0010"`.

## Affirmative scope cuts (out-of-scope = rejected, not deferred-in-costume)
- **The vector-valued metadata keys (`:args`, `:examples`, `:see`, `:deprecated`) are CARRIED on the
  entry but NOT rendered into the `metadata-of` HolonAST map in iv-b1.** They ride the entry so iv-b2's
  `:wat::intrinsic::examples` seam can expose them to the wat verifier — that is their first real
  consumer. Their HolonAST map-rendering lands with that consumer / iv-c's value-rendering pass, not
  speculatively here. (The carry is BUILT; only the map-projection of the vector-valued keys waits for
  a reader — no silent gap.)
- **No doctest execution, no purity cross-check, no `is_effectful_op` change** — all iv-b2 (the wat
  verifier reads the carried examples + runs them).
- **No enum values** — `:kind`/`:defined-in`/`:layer` stay keywords until iv-c.

## Probe (disconfirming, RED at HEAD)
`tests/nursery/probe_arc255_ivb1_structured_doc.rs`: eval `(metadata-of :wat::core::Bytes::to-hex)` and
assert the returned map carries `:added` AND `:ret`. RED at HEAD (today metadata-of emits only `:doc` +
baseline). GREEN after iv-b1 (Bytes decorated → macro parses+carries → metadata-of emits `:added`/`:ret`).
A confirmation Rust unit test (in the strike) reads `registry().lookup_entry(...)` and asserts the
structured carry (`args == [("bs", …)]`, `examples[0].run == true`) — proving the full carry, not just
the two rendered keys.

## Blast radius
`crates/wat-macros/Cargo.toml` + `wat_intrinsic.rs` (the macro), `src/intrinsic/mod.rs` (the types),
`src/intrinsic/bytes.rs` (decorate), `runtime.rs` `eval_metadata_of` (~10104, the richer read), the new
probe. Bounded: 2 handlers, one macro, one metadata-of branch. The blast is contained because only
`core::Bytes` wears `#[wat_intrinsic]` today — the forcing function bites exactly two sites.

## STOP triggers
1. If `wat-doc`'s API can't express a needed check (e.g. `check_args` needs richer info), STOP and
   surface it — don't work around it in the macro.
2. If carrying `examples` as `&'static [ExampleSubmission]` fights the `inventory::submit!` macro
   (const-eval / lifetime), STOP and report the exact error — do not fall back to stringly-typed.
3. If decorating Bytes reveals the grammar can't express a real doc need, STOP — that's a wat-doc gap
   (iv-a), not a thing to hack around here.

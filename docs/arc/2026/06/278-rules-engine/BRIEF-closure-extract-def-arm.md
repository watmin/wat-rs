# BRIEF — closure extraction carries `def`-bound names

**Arc 278 · the substrate stone that unblocks routing `defservice`'s child-forms through `fn-forms`.**

## The work, in one paragraph

`src/closure_extract.rs`'s Keyword walker resolves a free keyword in order — function →
unit-variant → type — and then, for anything sitting in `parent_symbols.runtime_def_values`,
**raises** `Internal("captured \`def\`-bound name … not yet supported by closure extraction
(slice 1)")`. Make that arm carry the `def` instead: record it as a dependency, encode its value
through the existing encoder, and emit `(:wat::core::def <original-name> <encoded>)` into the
prologue so the extracted package stands alone in a fresh world. The arm's own comment says a
future arc opens *"IFF a caller surfaces wanting it"* — one has, and it is the reason this stone
exists: **every `defservice` op's `:max-request-bytes` becomes a top-level `def`**, so `fn-forms`
on a service's `serve` cannot complete today.

## The gate — RED on disk right now

`tests/function/wat_arc170_closure_extraction.rs` · `t22_toplevel_defn_references_def_bound_value`
plus its fixture `tests/function/wat_arc170_closure_extraction_t22.wat`. Both are in the tree,
**uncommitted and deliberately RED**. Run it:

```
cargo test --release --test function t22_toplevel_defn_references_def_bound_value
```

Today it fails with exactly:

```
extract_closure should succeed: ExtractionError {
  span: Span { file: "src/closure_extract.rs", line: 769, col: 27 },
  kind: Internal("captured `def`-bound name :my::LIMIT not yet supported by closure extraction (slice 1)") }
```

Turning that test green — including its final `re_freeze` + invoke, which proves the package
stands alone — is the whole job. Do not weaken the test.

## Read in order — the rooms, and why you are being sent to each

1. **`src/closure_extract.rs:727-779`** — the Keyword arm. Lines `767-775` hold the raise you are
   replacing. Note the ordering above it: the reserved-prefix filter at `:731` returns early, so
   `:wat::*` / `:rust::*` defs (stdlib `defclause` registrations, `:wat::spawn::DEFAULT-MAX-MESSAGE-BYTES`,
   and friends) **never reach this arm** — the child re-registers those itself. Only user-namespace
   defs arrive here.

2. **`src/closure_extract.rs:598-643`** — `ExtractState` and `CapturedBinding`. This is where the
   new collection and its discovery-order vec belong, mirroring `captured_deps` /
   `dep_discovery_order`.

3. **`src/closure_extract.rs:2523-2536`** — `capture_define_form`. The **exact emission shape to
   mirror**: it already builds `(:wat::core::def :name <encoded>)` for captured locals. Your def
   form is the same three-item list with the *original* name in slot 1.

4. **`src/closure_extract.rs:1702-1716`** — `encode_value_to_ast(v, binding_name, state)`. Use it
   verbatim. The portability refusal you want comes free: its `ImpureCapture` arms already reject
   channel / IO / process-handle values, naming the binding.

5. **`src/closure_extract.rs:345-392`** — prologue assembly. Steps are: 1 types → 2 captured-binding
   defines → 3 user dep defines → 4 lifted prelude → 5 entry define. Add the def emission as a new
   step **between 2 and 3**: a def's encoded value may construct a user type (step 1 must precede
   it), and a dep fn's body may read the def (step 3 must follow it).

6. **`src/closure_extract.rs:299-303`** — the unconditional user-type sweep. Every non-reserved
   parent type is already recorded into the prologue regardless of reachability. **You therefore do
   not need to chase type dependencies out of an encoded def value** — they are already there. Read
   this before building any type-walking machinery.

7. **`src/closure_extract.rs:1247-1270`** — `record_dep_dependency`, for the shape of a recorder
   that guards against re-recording and maintains discovery order.

## Implementation sketch — the strike path, not the whole strike

In `ExtractState` (room 2):

```rust
    /// `def`-bound values discovered in walked bodies, keyed by the
    /// def's ORIGINAL keyword name. Emitted as top-level `def` forms.
    captured_defs: BTreeMap<String, WatAST>,
    /// Order in which defs were discovered (deterministic emission).
    def_discovery_order: Vec<String>,
```

In the Keyword arm (room 1), replacing the raise:

```rust
            if let Some(value) = state.parent_symbols.runtime_def_values.get(k).cloned() {
                if !state.captured_defs.contains_key(k.as_str()) {
                    let encoded = encode_value_to_ast(&value, k.as_str(), state)?;
                    state.captured_defs.insert(k.to_string(), encoded);
                    state.def_discovery_order.push(k.to_string());
                }
                return Ok(());
            }
```

In prologue assembly (room 5), a new step between 2 and 3:

```rust
    // 2b. `def`-bound values, in discovery order. The body references
    //     these by Keyword and `rewrite_captures` never rewrites a
    //     Keyword, so each keeps its ORIGINAL name.
    for name in &state.def_discovery_order {
        if let Some(encoded) = state.captured_defs.get(name) {
            prologue.push(def_form(name, encoded));
        }
    }
```

with `def_form` a sibling of `capture_define_form` (room 3) — same three-item list, original name.

Fill in the shape; do not invent a different one.

## Blast radius

`src/closure_extract.rs` only, plus turning the already-written T22 test green. No new public
types, no changes to `ExtractionError`'s variants, no touch to `wat/service.wat`, `wat/bracket.wat`,
or any `.wat` corpus file. Routing `defservice` through `fn-forms` is the NEXT stone and is out of
scope here — do not start it.

## STOP triggers — each ships nothing and reports

- **STOP-1.** If `encode_value_to_ast` returns `Internal("encoding for captured Value of kind … not
  implemented")` for a def you must carry, STOP. The unencodable kinds are enumerated at
  `closure_extract.rs:2044-2078` (`Vector`, `wat__core__fn`, `clauses`, `extend_def`, `Stream`,
  `WatAST`, `RustOpaque`, `ForeignRecord`, `ForeignVariant`, `Instant`, `Duration`). Widening that
  encoder is a separate stone. Report the kind and the def's name.

- **STOP-2.** If turning this arm on makes any currently-green test in the `function` target go red,
  STOP and report the test name plus its whole verbatim output. A def that now rides in the prologue
  where it previously did not is a behaviour change, and a red is the substrate telling you where.

- **STOP-3.** If a `def` name in `runtime_def_values` is ALSO resolvable as a function or a type
  (so the earlier lookups at `:735` / `:757` win and this arm never sees it), and you find a case
  where that earlier resolution produces a wrong or incomplete prologue, STOP. Reordering the
  Keyword arm's resolution chain is a ruling, not an implementation detail.

- **STOP-4.** If the fix requires emitting a def under any name other than its original — a
  synthetic name, a rewritten name, a namespace move — STOP. The body references it by Keyword and
  Keyword references are not rewritten; a renamed def is a package that cannot resolve.

## Verification, in this order

1. `cargo build --release` — exit 0.
2. `cargo test --release --test function` — the whole target green, T22 included.
3. `cargo test --release --test kernel` and `--test comms` — the two targets nearest the spawn /
   closure path.
4. `cargo clippy --release --all-targets -- -D warnings` — zero.

Run everything in the **foreground** and block on it; your turn ends when the numbers are in your
hands, not when a command is launched. Do not commit, do not push, do not stash — the orchestrator
weighs the whole floor by its own `--release` re-run and banks it.

## A prior comparable to copy for shape

`docs/arc/2026/06/278-rules-engine/BRIEF-the-child-needs-the-entry.md` — the 2026-08-02 stone that
last shaped what a forked child receives, and whose blanket *"nothing `:wat::`-rooted"* rule this
stone works alongside.

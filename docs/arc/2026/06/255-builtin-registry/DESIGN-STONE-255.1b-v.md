# DESIGN — Stone 255.1b-v: the reflection surface (show-source + render-doc + @see-check)

**Status: DESIGN (surface settled with the builder 2026-06-22).** The third smart-docs
strike per the contract §9. Completes the *reflection* surface over the registry; the
type-sig layer (255.2) is the separate next strike.

## The decomplect (builder, 2026-06-22): data / render / print
`metadata-of` is the DATA (structured EDN card). A doc is a *rendering* of that data, and
printing is the caller's act. So:
- **`render-doc <fqdn> -> String`** renders the metadata to a human string (with `\n`s). A
  String IS a clean EDN value (escaped `\n`); it is NOT "not-EDN". `(println (render-doc x))`
  renders the newlines to the terminal.
- **Rendering flavor is the caller's choice, LATER**: plain-text now; a `glow`/markdown/ANSI
  renderer drops in later over the SAME `metadata-of` data (render-doc gains a flavor arg or a
  render protocol dispatches) — zero churn to the data. We do NOT build the flavor knob now
  (don't build the forcing function); plain-text is the only flavor today.
- No `doc`+`print-doc` split: render returns, caller prints, testability kept (assert-eq the
  rendered string).

## Part A — `show-source` (§4)
- **`:source` field** on `IntrinsicSubmission` + `IntrinsicEntry` (`src/intrinsic/mod.rs`):
  `&'static str`, the handler's captured source.
- **Capture** in `#[wat_intrinsic]` (`crates/wat-macros/src/wat_intrinsic.rs`): restringify the
  `ItemFn` tokens — `quote!(#item).to_string()` (stable; `proc_macro::Span::source_text` is
  nightly — the contract's named fallback). Faithful-if-reformatted; comments may be lost (token
  restringify) — acceptable for v1, noted.
- **Verb** `(:wat::core::show-source <fqdn>) -> :wat::core::String`, a `#[wat_intrinsic]` in
  `src/intrinsic/reflect.rs`. Dispatch: `registry().lookup_entry(fqdn)` → its `:source`; else
  `sym` user-form → AST → `(:wat::core::write-forms …)` (`edn_shim.rs:269`). Uniform over both
  kinds (Pry's `show-source` on Ruby + C). Returns a String; caller prints.

## Part B — `render-doc` (§4/§6, replaces the contract's printing `doc`)
- **Verb** `(:wat::core::render-doc <fqdn>) -> :wat::core::String`, a `#[wat_intrinsic]` in
  `reflect.rs`. Reads `metadata-of`'s fields (`:name`/`:doc`/`:added`/`:arglists`/`:examples`)
  and formats a plain-text block: name + signature line, the prose, `Examples:` with each
  `@example` expr (and `#=> expected` where present). Pure, deterministic → returns a String.
- The contract §6 said `(doc …)` "prints" (clojure/ri parity). DEVIATE UP (reference-for-
  functionality, not parity-with-flaws): clojure's `doc` prints only because a REPL has no test
  harness; we have verify-examples-grade testing, so a returned String is strictly better
  (composable + assertable). Naming: `render-doc` not `doc` — honest (it renders + returns;
  `doc`-that-returns would surprise a clojure reader expecting a print).

## Part C — `@see` registry-check (§2)
- A consumer-side registry-walk **test in `wat`** (the same shape as the purity cross-check):
  walk `registry().all_entries()`, assert every `entry.see` FQDN is a registered intrinsic (or a
  resolvable user form) — fail loud with "dangling @see `<fqdn>` on `<owner>`". The macro CANNOT
  do this (cross-crate, expand-time); the consumer test is where the registry is visible.
- Retires the `see` field's `#[expect(dead_code)]` on `IntrinsicEntry` (`mod.rs`) — the reader
  has landed, so the compiler-enforced removal fires.

## RED probes (write + verify RED before the strike)
- **A:** `(:wat::core::show-source :wat::core::Bytes::to-hex)` → expect a String containing
  `fn eval_bytes_to_hex`. RED at HEAD: `show-source` unknown verb.
- **B:** `(:wat::core::render-doc :wat::core::Bytes::to-hex)` → expect a String containing the
  prose ("lowercase-hex") AND the example expr. RED at HEAD: `render-doc` unknown verb.
- **C:** a wat-tests/Rust test that the corpus has no dangling `@see` (passes once the check
  exists); RED-shaped via a deliberate bogus `@see` on a fixture intrinsic → check fails.

## Contract (pinned)
show-source + render-doc are **pure, return `:wat::core::String`** (printing is the caller's).
Flavor is NOT parameterized this strike (plain-text only; glow is a later flavor over the same
data). @see-check is a consumer test, not a macro check.

## Gate
- probes A/B GREEN; C green (no dangling @see in the corpus).
- `metadata-of`/`verify-examples`/bytes suites green; lib floor 957/36/1; wat-tests floor.
- the `see` `#[expect(dead_code)]` removed with no new dead-code warning.

## Out of scope (named)
- Rendering flavors / glow (later — the data/render decomplect makes it a drop-in).
- `child-namespaces`/`names` (`ls`) — the wiki-nav surface (§4 bullet); its own later strike.
- type-sig (255.2) — the next strike.

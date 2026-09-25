# BRIEF — STONE P: a load error names the user's `load-file!` call

**Drawn 2026-09-25.** Second per-mechanism fix for "errors that point into our source" (census: 154 of
166 errors already locate in the user's file; the loader is 1 of the misses).

## The defect, driven

`the-little-wat/probes/annoy/load-clj-missing.wat` — `(wat/load-file! "no-such-file.wat")`:
```
#wat.load/Fetch  "load: file not found: no-such-file.wat"   :file "src/load/loader.rs" :line 438
```

## The root

`src/load/loader.rs:436-439`:
```rust
impl From<LoadFetchError> for LoadError {
    fn from(e: LoadFetchError) -> Self {
        LoadError::new(crate::rust_caller_span!(), LoadErrorKind::Fetch(e))
```
`process_single_load(spec, form_span, …)` (`~:489`) HOLDS the `load-file!` form's span — but
`fetch_source(…)?` (`:499`) converts through that `From`. Three more `rust_caller_span!()` sites sit in
the same load path with `form_span` reachable: `:525` (`LoadErrorKind::Parse` — a parse error in the
LOADED file), `:610` and `:641` (`VerificationFailed`).

## The work — the top rung, not the patch

1. **Delete `From<LoadFetchError> for LoadError`.** With it gone, no future `?` can silently stamp a
   Rust location: every conversion must supply a span. Route the fetch errors through `form_span`
   (e.g. thread it into `fetch_source`/`fetch_payload`, or `map_err` at the call). Report every site
   the deletion breaks.
2. `:525` Parse, `:610`/`:641` VerificationFailed → `:location` is the user's `load-file!` call.
   ⭐ For **Parse**, keep the inner `ParseError` intact as the cause: since stone O it carries its OWN
   real span (the loaded file's line/col). Outer = where you asked to load; inner = where it broke.
3. The two `SourceLoader` fetch methods themselves return `LoadFetchError` (no span) — leave them.

## Prove it

- The probe above: `:location` = the probe file, the `load-file!` line.
- A load of a file with a lex error: outer `:location` = the `load-file!` call; the cause's span = the
  loaded file's line/col.
- A digest/signature verification failure (existing tests have fixtures): located at the load call.
- **Mutation:** restore `rust_caller_span!()` at the fetch site → the probe reds.
- Goldens pinning `src/load/loader.rs` as a location: find them (`grep -rl 'src/load/loader.rs'
  tests wat-tests --include='*.edn'`) and recapture with `UPDATE_EDN=1`; check each new golden names a
  `.wat`.

## STOP triggers

1. A caller of the deleted `From` has no load-form span in reach — report the site, do not re-add a
   sentinel.
2. Any gate reddens that you did not add (other than goldens you were told to recapture) — capture
   whole, name the arm. ⛔ Do not re-run first.

## Mechanics — ⛔ read

- **Tests run under `cargo nextest run --release`, never `cargo test`** — for focused runs too
  (`-E 'test(/name/)'`). (Stone O used `cargo test` for its focused runs; the rule is nextest.)
- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks (each under the tool's 600s
  cap). Do not end your turn while it runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines.
- No `cargo fmt` / `rustfmt`. Stage explicit paths, never `git add -A`.
- Test lints: an EDN string literal trips `no_inlined_edn` (use an `.edn` golden +
  `assert_edn_matches_file!`); `contains`/`ends_with` in an assert trips `no_loose_string_assert`.

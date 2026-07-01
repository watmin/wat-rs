# Strike 298.2 — Annihilate `Span::unknown()`: there is no "nowhere"; every span names a real place

> **Status: STRIKE-READY (2026-07-01). Builder: *"the `Span::unknown()` symbol will not survive its annihilation … we
> take it by force."*** The null-object dies. `Span::unknown()` is a **sentinel** — a fake `<runtime>:0:0` span standing
> in for "no source location" — the exact lie arc 298 exists to kill (the same shape as the transparent-`Option` erasure,
> the elide, the `{:file "<runtime>"}` error span). This strike removes it entirely.

## The principle
`Span::unknown()` claims a value was constructed **nowhere** (`file "<runtime>", line 0, col 0`). That is false: every
value and every error was constructed **somewhere** — a wat source line, or a line of Rust that built it. "Nowhere" is a
fake coordinate that lies to the user's tooling (jump-to-location lands at `<runtime>:0:0`). The cure is not `Option<Span>`
(a real construction site is **not** absence — there is nothing to make optional); the cure is to **name the real place**:
- **wat-source span in scope** (the span-thread debt — the eval fn HAS the span, passes `unknown()`) → **thread it**.
- **else** → **`crate::rust_caller_span!()`** — the honest Rust `file!():line!():col!()` of the constructing code (the
  Span doc's own recommended alternative to `<runtime>`). Real, not fake.

`Span::unknown()` and `is_unknown()` are **deleted**. With no sentinel, the "is this fake?" question dissolves — every
span is real, so the elide/skip logic that existed to hide `<runtime>:0:0` noise retires too.

## Grounded facts (this session, on disk)
- `crates/wat-reader/src/span.rs:71` `Span::unknown()` (the `<runtime>`/0/0 sentinel) + `:97` `is_unknown()` (`line==0 &&
  col==0`). Delete both. `Span` keeps `new` / `with_end` (real spans only).
- `crate::rust_caller_span!()` — a macro re-declared in BOTH crates (`crates/wat-reader/src/span.rs:124`, `src/span.rs:15`);
  usable anywhere a `Span` expression is. This is the default mechanical replacement.
- **496** `Span::unknown()` construction sites in `src/` (measured). Breakdown: ~106 are `span: Span::unknown()` inside
  error constructions (RuntimeError 105 + Load/Lower/etc.) — the span-thread debt (task #167); ~390 are non-error
  (synthesized ASTs, `edn_shim` reconstruction, `rust_deps/marshal`, values off the wire — genuinely no *wat* source, but
  a real *Rust* construction site).
- **17** `is_unknown()` consumers: `check/error.rs` (6), `value/signal.rs` (3), `to_edn.rs` (2), `resolve/error.rs` (1),
  `panic_hook.rs` (1), `macros/expand.rs` (1), `macros/eval.rs` (1) — all sentinel-skip / elide-when-unknown logic.

## The strike (compiler-driven cascade — the fail-count is the progress meter)
1. **Delete** `Span::unknown()` + `is_unknown()` from `crates/wat-reader/src/span.rs` (and the `src/span.rs` re-export if
   any). This reds all 496 + 17 sites as **compile errors** — the substrate names every site to fix.
2. **Codemod the bulk** — a small surgical throwaway (`read → replace → write`, deleted before commit): `Span::unknown()`
   → `crate::rust_caller_span!()` across the ~390 non-error + any error site with no wat span in scope. Honest Rust
   construction location, mechanical, one-to-one.
3. **Thread the real wat span** where it is trivially in scope at an error construction (a `list_span` / `span` / node
   span already bound in the fn) — prefer the real wat location over the Rust one for user-facing errors. Where it is NOT
   trivially in scope, `rust_caller_span!()` is the honest fallback (do NOT invent a threading refactor — that is a
   separate quality arc; STOP-note it).
4. **Retire the 17 `is_unknown()` consumers** — with no sentinel, `is_unknown()` cannot be called. Each site's
   elide-when-unknown / skip-`<runtime>:0:0` logic simplifies to "the span is always real → always emit / always render".
   (This also settles the old RuntimeError span-policy question: no sentinel means no `{:file "<runtime>" :line 0 :col 0}`
   on any error wire — the honest location the derive sweep needed.)
5. **Ride the test cascade to zero.** Tests asserting `<runtime>` / `:line 0 :col 0` / `is_unknown` behavior are asserting
   the sentinel this strike retires — update them to the honest real-location form. Do NOT weaken a probe to pass.

## Proof
- `grep -rn "Span::unknown()\|is_unknown()\|\"<runtime>\"" src/ crates/ --include=*.rs` → **0** (the symbol is gone).
- A probe (or reuse an error probe): an error that previously emitted `:span {:file "<runtime>" :line 0 :col 0}` now emits
  a **real** location (a wat span, or a Rust `…/src/….rs` location) — never `<runtime>`, never `:line 0`.
- FULL gate `cargo nextest run --release` = 0 failed; `cargo build --release` clean (warning delta ~0).

## Blast radius (wide by nature — a symbol annihilation; the cascade is expected)
`crates/wat-reader/src/span.rs` (delete) · ~496 `Span::unknown()` sites (codemod + targeted threading) · 17 `is_unknown()`
consumers · the test cascade (sentinel-asserting tests). This is a substrate-wide sweep; the fail-count waterfalls to zero.
STOP + report if a site's cure genuinely requires a threading refactor (a fn that has no span and cannot cheaply get one)
— use `rust_caller_span!()` and note it for the follow-on quality arc; do NOT build a signature-threading refactor here.

## Out of scope (affirmative cuts)
- **Better error locations via span-threading refactors** (giving every RuntimeError the precise wat span) — a QUALITY
  arc; this strike only ANNIHILATES the sentinel (every span becomes real; `rust_caller_span!()` is an honest floor).
- **`Option<Span>` / `Option<Location>`** — NOT needed; a real Rust location is not absence. The "no location" concept
  ceases to exist. (Supersedes the earlier mandatory-vs-Option-location fork — there is no absent location to model.)
- **298.3** (resume the derive over now-honest data) — follows.

## The anti-weakening rule (PROBATIO FLEXA MENTITVR)
A wide cascade is the loudest place a bent probe hides. NEVER weaken a probe to pass. A test asserting `<runtime>` /
`:line 0` is asserting the sentinel this strike retires — update it to the honest real-location form (that is the point),
never invert or relax it. The orchestrator weighs the emitted diff, not the report.

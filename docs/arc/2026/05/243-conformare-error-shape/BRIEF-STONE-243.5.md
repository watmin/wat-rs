# BRIEF — Stone 243.5 — mint `src/types/` home + carve `TypeError`; thread `register_subtype` span

**Agent:** sonnet (`model:"sonnet"`). **Mode:** substrate edit + carve + cascade. **Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Use `git -C /home/watmin/work/holon/wat-rs` for all git. Any path containing `.claude/worktrees/` is harness state — ignore it; operate on the main checkout.

DESIGN: `docs/arc/2026/05/243-conformare-error-shape/DESIGN-STONE-243.5.md` (read it first). Probe (already committed, currently disconfirms): `tests/probe_arc243_stone5_register_subtype_span.rs`.

## The contract

Five movements, in this order. Each is a clean checkpoint — build green between them where possible.

### M1 — mint the `src/types/` home (the ACTUAL pattern, not `mv`)

Mirror `src/check/` exactly (crawl it: `src/check.rs:49` is `pub mod env;` + `pub use env::CheckEnv;`, and `src/check/env.rs` is the resident — Rust's flat-file-with-sibling-dir resolution; NO `mv`, NO `mod.rs`, NO `#[path]`).

- **Keep** `src/types.rs` where it is. Do NOT move it.
- `mkdir src/types/`.
- Add to `src/types.rs` near the other module decls: `pub mod error;` + (after M4) `pub(crate) mod defstruct;`.

### M2 — carve `TypeError` → `src/types/error.rs`

Move VERBATIM from `src/types.rs` into a new `src/types/error.rs`:
- `pub struct TypeError` (1429–1435)
- `pub enum TypeErrorKind` (1437–1562)
- `impl fmt::Display for TypeErrorKind` (1565–1665)
- `impl fmt::Display for TypeError` (1667–1672)
- `impl std::error::Error for TypeError {}` (1674)

Add to `src/types.rs`: `pub use error::{TypeError, TypeErrorKind};` so every existing `crate::types::TypeError` / `crate::types::TypeErrorKind` path keeps resolving (ZERO consumer churn — verify with a workspace build). `error.rs` will need its own `use` lines (`crate::span::Span`, `crate::remedy::Remedy`, `std::fmt`, whatever the Display bodies reference) — add exactly what the moved code uses, nothing more.

### M3 — thread `register_subtype` caller-span + RETIRE the two runes

This is the honesty fix — the load-bearing movement. The probe gates it.

- Change the signature: `pub fn register_subtype(&mut self, child: &str, parent: &str, span: Span) -> Result<(), TypeError>` (types.rs:446).
- In the emitter (types.rs:451), replace `span: Span::unknown()` with the passed `span`.
- **DELETE** the `// rune:struere(host-constraint) …` rune comment block at types.rs:441–445 (the emitter no longer hardcodes unknown).
- **DELETE** the `/// rune:conformare(spanless-by-domain) …` lines from the `CyclicSubtype` doc comment at types.rs:1557–1560 (now in `error.rs`); keep the variant + a plain doc line describing it.
- Update the two call sites (these are the ONLY two — verified by crawl):
  - **types.rs:407** (`register_validated`): `self.register_subtype(&name, &parent, span)` — the real `span` is already this fn's parameter (line 357). Thread it.
  - **types.rs:1421** (`register_builtin_types`): `env.register_subtype(":wat::holon::Record", ":wat::Record", Span::unknown())` — built-in seed, no source form; `Span::unknown()` here is the HONEST built-in-seed case (a literal-argument placeholder, not an emitter hardcode). Add a one-line comment: `// built-in root hierarchy seed — no source form exists; unreachable cycle path (two distinct roots).`

### M4 — decompose `parse_defstruct` → `src/types/defstruct.rs`

`parse_defstruct` (types.rs:1901–2281, ~380 lines, struere F3 deferral owner) moves to `src/types/defstruct.rs`, decomposed into named helper fns by concern (it currently inlines: arg-shape validation, name parsing, restriction parsing, field parsing, struct assembly — split along those seams; let the seams reveal as you read it, don't force a count). `src/types.rs` gets `pub(crate) mod defstruct;` + `pub(crate) use defstruct::parse_defstruct;` (or call it as `defstruct::parse_defstruct` at its one call site, types.rs:1874 — your choice, whichever is cleaner). Preserve behavior exactly; this is decomposition, not redesign.

### M5 — cascade + probe green

- `cargo build -p wat` clean.
- The probe `tests/probe_arc243_stone5_register_subtype_span.rs` now COMPILES + PASSES (it asserts the caller span survives into the CyclicSubtype error).
- `cargo test -p wat` — lib + integration green. ONE known-banked failure is allowed and expected: `tests/probe_arc216_stone5b_hashset_native_storage.rs::probe_8_atom_round_trip` (unrelated HashSet debt). EVERYTHING else green.
- Any OTHER `register_subtype` caller surfaced by the build (there should be none beyond the two) — STOP and report; do not invent a third.

## STOP triggers (these REJECT — they are not permission-to-defer)

- If the workspace build reveals `register_subtype` callers beyond the 2 named: STOP, report the list. Do not guess a span for an unanalyzed caller.
- If carving `error.rs` reveals a `TypeError`/`TypeErrorKind` consumer that does NOT resolve through the `pub use` re-export: STOP, report it (means a deeper import coupling than crawled).
- If `parse_defstruct` decomposition can't preserve behavior cleanly (some shared mutable state across the concerns): STOP, report the seam — do not ship a behavior change disguised as decomposition.

Do NOT write any rune, "deferred", "future", "TODO", or "out of scope" language anywhere in the substrate or the SCORE. This stone RETIRES runes; it adds none.

## What to return

A SCORE-shaped report: per-movement status, the cascade table (files touched + site counts), the probe result (compiles + passes), `cargo test` tallies (naming the one banked failure), and any honest deltas. Do NOT commit — orchestrator commits after scoring against an independent re-run. Do NOT write INTERSTITIAL or vigilatum stamps — orchestrator casts vigilia (Phase B) and stamps.

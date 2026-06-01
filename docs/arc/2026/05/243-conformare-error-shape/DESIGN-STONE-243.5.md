# DESIGN — Stone 243.5 — mint `src/types/` home + carve `TypeError`; thread `register_subtype` span (retire the last spanless rune)

**Status:** STRIKE-PENDING. Child of arc 243 (conformare). Opened 2026-06-01 after Stone 243.3 CLOSED (`162aa5c9`) — TypeError is Pattern-A-clean but still lives in a flat 4119-line `src/types.rs`. This stone gives it a home AND makes arc 243's "zero exceptions" doctrine TRUE in code (prerequisite for 243.4's honest doctrine rewrite).

## Why this stone (and why before 243.4)

The arc DESIGN table lists 243.4 (doctrine rewrite: "zero exceptions; `spanless-by-domain` retired") before 243.5 (the code that retires it). That order is **inverted** — four-questions verdict (2026-06-01): writing "zero exceptions" as doctrine while a live `conformare(spanless-by-domain)` rune sits in `types.rs:1557` FAILS Honest + Obvious (doc contradicts code). 243.5 makes zero-exceptions REAL; 243.4 then documents proven progress (FM 6). So: **243.5 → 243.4.**

## The carve-map (grounded in `src/types.rs` @ HEAD `162aa5c9`, 4119 lines)

| Region | Lines | Destination |
|---|---|---|
| `TypeError` struct + `TypeErrorKind` enum + `impl Display for TypeErrorKind` + `impl Display for TypeError` + `impl Error` | 1429–1674 | `types/error.rs` |
| `parse_defstruct` (the 350-line, 7-concern beast; struere F3 deferral owner) | 1901–2281 | decompose → `types/defstruct.rs` (+ shared helpers to `types/parse.rs` as they reveal) |
| `register_subtype` span-thread (the honesty fix) | 407 + 446 + 1421 | stays in `types/mod.rs`; signature change |

Everything else (TypeExpr, TypeDef, TypeEnv, the other `parse_*` decl fns, the type-expr parser, alias/union cycle checks, is_subtype) stays in `types/mod.rs` for THIS stone. Selective lift-and-ward (`feedback_selective_lift_and_ward`): the home holds only what we ward now; the flat remainder is functional-but-untrusted-by-design, awaiting future carves (243.7+).

## The honesty fix — thread `register_subtype` caller-span (retires T1)

**FM 2-bis feasibility — CONFIRMED by crawl 2026-06-01 (not assumed):** two call sites only.

1. **`types.rs:407`** — inside `register_validated`, which already has `span: Span` in its signature (line 357). The span is in scope; the call just doesn't pass it. **Threadable directly.** This is the real-source path (a `recordtype` decl carrying a decl span).
2. **`types.rs:1421`** — `register_builtin_types`, two hardcoded FQDN literals (`:wat::holon::Record` → `:wat::Record`), no source form. Passes `Span::unknown()` — the HONEST built-in-seed spanless case (same category as the adjudicated-clean `register()`/`register_stdlib()` wrappers from 243.3 Phase B), and its cycle-error path is unreachable-by-construction (two distinct roots).

**Shape:** `register_subtype(&mut self, child: &str, parent: &str, span: Span)`. Caller 407 threads its real `span`; caller 1421 passes `Span::unknown()` (documented built-in seed). The `CyclicSubtype` emitter (451) uses the passed `span` instead of a hardcoded `Span::unknown()` → the `conformare(spanless-by-domain)` rune on the variant (1557) and the `struere(host-constraint)` rune on the emitter (441) both RETIRE. The recensere-watched `deferred-stone-243.5` runes close the instant this ships.

The error TYPE no longer forces `Span::unknown()` at any emitter; the single remaining `Span::unknown()` is a built-in-seed argument, not an excuse baked into the type. That is what makes "zero exceptions" honest.

## Home mechanics (mirror the ACTUAL 243.3.1 `src/check/` pattern — crawled 2026-06-01)

**Correction (grounded, supersedes the arc DESIGN table's "mv → mod.rs" wording):** the real 243.3.1 home is NOT `mv check.rs → check/mod.rs`. On disk: the flat `src/check.rs` (945KB) STAYS; the warded resident lives at `src/check/env.rs`; they are wired by Rust's flat-file-with-sibling-dir resolution — `src/check.rs` line 49-50 holds `pub mod env;` + `pub use env::CheckEnv;`, and Rust resolves `mod env` to `src/check/env.rs` automatically. **No `mv`, no `mod.rs`, no `#[path]`.** Zero import churn — `crate::check::CheckEnv` still resolves.

So 243.5 mirrors THAT, exactly:
- **Keep** flat `src/types.rs` in place (4119 lines). Do NOT `mv` it.
- Add to `src/types.rs`: `pub mod error;` + `pub use error::{TypeError, TypeErrorKind};` (and `pub(crate) mod defstruct;` + the defstruct re-export).
- **Create** `src/types/error.rs` — carve `TypeError` + `TypeErrorKind` + `impl Display for TypeErrorKind` + `impl Display for TypeError` + `impl Error` (types.rs:1429–1674) into it. This is the home's first warded resident; it carries the `//! vigilatum:` stamp once vigilia converges.
- **Create** `src/types/defstruct.rs` — `parse_defstruct` decomposition (types.rs:1901–2281).
- The flat `src/types.rs` remainder is untrusted-by-design (selective lift-and-ward — exactly as the 945KB `check.rs` remainder is). The vigilatum REMARKABLE bar governs the LIFTED residents (`error.rs`, `defstruct.rs`), NOT the flat remainder.

Why this matters: briefing a `mv` + full import-rewire when the real pattern is "add two lines + create two sibling files" would be an FM-2 fiction (briefing from a doc's description, not crawled truth). The lair-study caught it.

## Cadence (the stone rhythm)

1. ✅ Lair-study + FM 2-bis feasibility crawl (this DESIGN).
2. FM 2-bis probe — `tests/probe_arc243_stone5_register_subtype_span.rs`: prove caller-span reaches the `CyclicSubtype` error (disconfirms at HEAD: today the error always carries `Span::unknown()`). Commit it.
3. BRIEF + EXPECTATIONS (sonnet writes substrate; orchestrator briefs/scores/commits).
4. Baseline re-run (`cargo test -p wat`, lib + tests/function green; note the banked `probe_8_atom_round_trip` HashSet failure is unrelated).
5. Spawn sonnet (`model:"sonnet"`, background): Phase A = home mint + carve + span-thread + defstruct decomposition + cascade.
6. Phase B = vigilia 8-spell (the namespaced-home REMARKABLE bar) → drive to L1+L2=0.
7. SCORE against independent local re-run; hashless `vigilatum` stamp on `types/mod.rs`; atomic ward commit. Then recensere re-musters (the `deferred-stone-243.5` runes should now strike).

## Scope fence (what 243.5 does NOT do)

- NOT the doctrine rewrite — that's 243.4, which rides AFTER this proves zero-exceptions.
- NOT CheckError — that's 243.6 (grows the `src/check/` home).
- NOT the other `parse_*` decl fns' decomposition beyond `parse_defstruct` — they stay in `mod.rs` until their own future carve. `parse_defstruct` is in-scope only because it's the named struere F3 deferral owner attested into this stone.
- NOT a new arc — opener-blocks: 243.5 is a child of the open arc 243.

## Cross-references

- `docs/CONFORMARE.md` — Pattern A doctrine (rewritten at 243.4)
- arc 243 `DESIGN.md` — stone chain; T1 (line 110); `parse_defstruct` deferral (line 80)
- `SCORE-STONE-243.3.md` Phase B — the live conformare cast that adjudicated the spanless cases + confirmed TypeError clean
- `DESIGN-STONE-243.3.1.md` — the `src/check/` home pattern this mirrors
- `feedback_selective_lift_and_ward`, `feedback_warded_means_annihilated` — home/ward discipline

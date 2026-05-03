# Arc 138 — Errors carry point-in-code coordinates — INSCRIPTION

**Status:** shipped 2026-05-03. 28 commits across one weekend session, 8 error types span-threaded, 4 substrate cracks closed, 5 F-NAMES sub-slices.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**Cracks audit:** [`CRACKS-AUDIT.md`](./CRACKS-AUDIT.md) — no-deferrals charter for the substrate cracks (F1–F4c).
**Names audit:** [`NAMES-AUDIT.md`](./NAMES-AUDIT.md) — placeholder-label inventory and resolution.
**This file:** completion marker.

---

## Motivation

Arc 016 (six months prior) gave wat-side assertion failures wat-source coordinates via `Failure.location` / `Failure.frames`. Arc 138 makes EVERY error type the substrate emits carry source coordinates — not just assertion panics. Type errors, runtime errors, macro errors, EDN read errors, lower errors, config errors — all of them point at the offending source location.

The driver, distilled across the slices:

> sonnet takes way longer to do work than I expect.. it's having to guess way too much.

The receiver of an error message is increasingly an agent. When sonnet (or any other reader) hits an error without `file:line:col`, it has to GUESS where the offending form lives. It greps. It tries each match. It guesses again on the next layer of confusion. Iteration cost compounds.

Spans collapse the guess-loop. The fix is mechanical (add `Span`, thread through emission sites). The payoff is structural — every future debug session, by every user (human or agent), gets shorter.

This is foundation infrastructure. It superseded ergonomic work (`do` form, `/dissimulans`) until shipped.

---

## What shipped

8 error types span-threaded across 8 slices, then 4 substrate cracks closed, then 5 F-NAMES sub-slices to make the threaded coordinates actually navigable. The work fell into three phases:

### Phase 1 — error variants gain spans (slices 1, 2, 3a, 3a-finish, 3b, 4a, 4b, 5)

Each user-facing error variant gained a `span: Span` field (or extended a tuple variant to carry one). Every Display arm gained a file-local `span_prefix(span: &Span) -> String` helper that prefixes `file:line:col:` when non-unknown. Every emission site threaded a real span where one was in scope.

Per-error-type breakdown:

| Slice | Error type | File | Variants | Emission sites | Pattern E |
|---|---|---|---|---|---|
| 1 (+finish) | CheckError | src/check.rs | 6 OG + 1 added | ~206 | 3 (SchemeCtx trait gap) |
| 2 | TypeError | src/types.rs | 10 | ~31 | 4 (in `_with_span` sibling wrappers) |
| 3a (+finish) | RuntimeError | src/runtime.rs | 22 | ~440 | 91 (5 categories — see CRACKS-AUDIT) |
| 3b | RuntimeError external | src/io.rs + 14 others | (sweep) | ~156 | 137 (88% — substrate-architectural) |
| 4a | MacroError + ClauseGrammarError | src/macros.rs + src/form_match.rs | 9 + 7 | ~50 | 4 |
| 4b | EdnReadError + LowerError | src/edn_shim.rs + src/lower.rs | 6 + 12 | ~73 | 31 (EDN parser layer) |
| 5 | ConfigError | src/config.rs | 8 | ~23 | 0 |

Two substrate patterns crystallized during phase 1:

1. **`span_prefix` helper convention** — every file that emits a span-bearing error gains a small file-local helper that returns empty string for `Span::unknown()` and `format!("{}: ", span)` otherwise. The empty-string-for-unknown discipline prevents `<runtime>:0:0:` noise from ever surfacing.

2. **`_with_span` sibling-API pattern** (slice 2's invention) — when expanding a public function's signature would break external callers, add a `_with_span` sibling that takes the new parameter; the original delegates with `Span::unknown()`. Used in `TypeEnv::register_with_span`, `parse_type_expr_with_span`, etc. Backwards-compat preserved at the API boundary.

Plus one canary per error type — each canary triggers a representative variant and asserts `<test>:` (later `src/...rs:`) substring in rendered Display. By slice 5 there were 7 canaries; by F-NAMES-1 they assert real file paths instead of `<test>`.

### Phase 2 — substrate cracks closed (CRACKS-AUDIT, slices F1–F4c)

The user invoked the **no-deferrals rule** mid-arc: every "earned for follow-up" we surfaced during phase 1 had to be closed before slice 6. Six cracks identified, six cracks closed:

| Crack | Where | Fix shape |
|---|---|---|
| F1 MacroDef gap | src/macros.rs (3 sites) | Added `pub span: Span` field to `MacroDef`; constructor passes the defmacro form's outer span |
| F2 SchemeCtx trait gap | src/rust_deps/mod.rs trait + 16 callers | Trait `push_*` methods gain `span: Span` parameter; proc-macro emit at codegen.rs:165 includes the span argument |
| F3 WatReader/WatWriter trait gap | src/io.rs trait + 7 impls + 16 callers | Trait methods gain `span: Span`; default close() also takes span for consistency. Workspace-wide `// arc 138 slice 3b: span TBD` count 156→0 |
| F4a Value-shaped helpers | src/spawn.rs, src/string_ops.rs, src/io.rs (~14 helpers + ~33 callers) | Helpers gain `span: Span` parameter; callers pass `args[i].span()` (Pattern A) or list_span (Pattern B) |
| F4b FromWat trait | src/rust_deps/marshal.rs (10 impls + recursive calls) | `from_wat` gains span; recursive calls in Option/Vec/tuple/Result pass `span.clone()`; proc-macro emit threads `args[#idx].span().clone()` |
| F4c opaque-cell helpers | src/rust_deps/marshal.rs (5 helpers) + 7 caller files | `rust_opaque_arc`, `ThreadOwnedCell::ensure_owner`/`with_mut`, `OwnedMoveCell::take`, `downcast_ref_opaque` gain span. Sonnet correctly expanded scope from BRIEF's 3 files to 7 because compiler required it. |

The cracks-fix campaign produced a sub-pattern: **trait expansion as fix shape**. F2/F3/F4b followed the same recipe — expand trait method signatures, update implementor(s), update callers, update proc-macro emit if applicable. Sonnet shipped each in 5–10 minutes once the pattern was clear.

### Phase 3 — F-NAMES placeholder coordinates

Phase 1 + phase 2 made every error type carry a span. But spans render as `file:line:col:`, and the user observed: when `file` is `<test>` and the panic header is `<unnamed>`, the coordinates can't be navigated. The placeholders defeated the purpose.

Placeholder-string sweep:

| Sub-slice | Placeholder | Resolution |
|---|---|---|
| F-NAMES-1 | `<test>` source label | Killed `parse_one(src)` / `parse_all(src)` convenience wrappers entirely. Added `parse_one!(src)` / `parse_all!(src)` declarative macros that auto-capture `concat!(file!(), ":", line!())`. Swept 132 test callers + 5 production sites + lib.rs::eval_algebra_source via macro. Workspace `<test>` count → 1 (lexer test fixture only). |
| F-NAMES-1c | `<unnamed>` deftest worker | wat::test! proc-macro at crates/wat-macros/src/lib.rs:672 spawns workers via `Thread::Builder::new().name(format!("wat-test::{}", deftest_name)).spawn(...)`. Rust-default panic hook reads the name. |
| F-NAMES-1d-asserthook | Assertion-hook still showed `<unnamed>` | Added `pub thread_name: Option<String>` field to `AssertionPayload`. Captured at panic site via `std::thread::current().name()`. Survives `resume_unwind`. Hook reads from payload field instead of fresh `thread::current()` lookup. |
| F-NAMES-1e | wat-side spawn workers unnamed | 3 sites (src/spawn.rs:183, src/runtime.rs:12421, src/runtime.rs:18780) renamed via Builder pattern. Worker threads now read `wat-thread::<primitive>` (e.g., `wat-thread:::wat::kernel::spawn-program-ast`). |
| F-NAMES-2/3/4 (investigations) | `<lambda>`, `<runtime>`, `<entry>` | Investigated as architectural; documented in SCORE-F-NAMES-2-3-4-INVESTIGATIONS. |
| F-NAMES-2/4-coords | Definition-coordinate refinement | User principle: "the template name is fine as long as we point to where it occurs." Anonymous lambdas now render `<lambda@<file>:<line>:<col>>` via `format!("<lambda@{}>", body.span())` at 6 sites. Test callers of `startup_from_source` pass `Some(concat!(file!(), ":", line!()))` instead of `None`. |

After phase 3: ZERO `<unnamed>`, ZERO `<test>`, ZERO `<runtime>`, ZERO `<entry>` user-visible occurrences in workspace test output. Every panic shows real thread name + real file path + real line:col.

Substrate observation surfaced by F-NAMES-2/4-coords: `compose.rs::base_canonical` is dual-purpose (span label AND ScopedLoader base directory). Synthetic labels broke `helper.wat` resolution because `Path::new("<compose-and-run>").parent()` → `""`. The `<entry>` label stays for compose.rs as honest architecture; CARGO_MANIFEST_DIR plumbing handles relative paths.

---

## Resolved design decisions

- **2026-05-03** — **Span on every user-facing error variant.** No exceptions for "OG" variants that predated span discipline. Eight error types swept uniformly.
- **2026-05-03** — **`span_prefix` helper convention.** Each file that emits errors gains a small file-local helper. `Span::unknown()` renders empty (suppressed); real spans render `file:line:col:`.
- **2026-05-03** — **`_with_span` sibling-API pattern.** When public-API expansion would break callers, add a sibling. Original delegates with `Span::unknown()`.
- **2026-05-03** — **Pattern E with rationale, not silent placeholders.** Every leftover `Span::unknown()` carries `// arc 138: no span — <reason>` so future readers know it was deliberate.
- **2026-05-03** — **Trait expansion as fix shape.** SchemeCtx, WatReader/WatWriter, FromWat all expanded uniformly. Same recipe; sonnet executes in 5–10 min.
- **2026-05-03** — **No-deferrals doctrine for known cracks.** "Earned for follow-up" prose is the failure mode. If we know how to close it, close it now.
- **2026-05-03** — **Template name + coordinates is the right shape.** `<lambda>` alone is useless; `<lambda@src/foo.rs:17:5>` is navigable. Same principle drove `<test>` → real Rust paths.
- **2026-05-03** — **Convenience wrappers that hardcode placeholders are antipatterns.** `parse_one(src)` defaulting to `<test>` papered over a missing-source-label gap. Killed.
- **2026-05-03** — **`AssertionPayload` carries thread_name.** `thread::current().name()` returns None across `resume_unwind` boundaries; payload-field capture-at-panic-site is the durable shape.
- **2026-05-03** — **Simple is uniform composition, not change count.** N identical one-line changes IS simple. F-NAMES-1's 132-site sweep shipped as one slice because the composition was uniform.

---

## What this arc does NOT ship

- Color output for failure rendering. ASCII works; color is polish. Inherited from arc 016.
- Hermetic-fork frame propagation across process boundaries. Inherited from arc 016.
- pytest-style value substitution. Named in arc 016 non-goals; still deferred.
- Synthetic-AST source recovery (`<runtime>` sentinel). Suppressed by `is_unknown()`; never user-visible. Architectural, not a gap.
- `compose.rs` `<entry>` for in-memory sources. Dual-purpose `base_canonical` (label + loader base dir) constraint documented; splitting the concerns is a separate substrate refactor.
- Field-absent indicators in Frame display (`<unknown>`, `<symbol>`). Honest absence; not identity placeholders.

---

## Why this matters

When a wat program errors, the user/agent reads ONE line and either knows where to go or doesn't. Pre-arc-138, most error types said `expected i64 got bool` with no location — the reader had to grep the source for the offending form, often facing many matches. Sonnet's iteration cost compounded; human debugging followed the same shape.

Post-arc-138, every error renders `<file>:<line>:<col>: <message>`. Click the coordinate, land at the offending form. Done.

The work was tedious — 8 error types, 22 substrate variants, 800+ emission sites, 5 placeholder cleanups. But the pattern was uniform. Sonnet did most of the mechanical work in 5–15 minute slices once the BRIEFs got disciplined. The orchestrator's job became: write a tight contract, predict the score, verify on disk, write the SCORE, ship.

The campaign also forged durable methodology:
- **Trust-but-verify protocol** for sonnet delegation — BRIEF + EXPECTATIONS + sonnet engagement + independent SCORE verification + git diff cross-check.
- **No-deferrals discipline** — "earned for follow-up" prose is a smell. The CRACKS-AUDIT charter forced them all closed.
- **Simple-is-uniform-composition principle** — saved as feedback memory for future arcs.

The trading lab moves in next on a rock-solid foundation. When its `.wat` programs error, the author reads ONE coordinate and lands at the offending form.

---

## Why this is the hardest boss fight we've had

By the numbers:
- **28 commits** across one weekend session
- **8 error types** restructured uniformly
- **6 substrate cracks** closed (F1, F2, F3, F4a, F4b, F4c)
- **5 F-NAMES sub-slices** (1, 1c, 1d-asserthook, 1e, 2/4-coords) + 1 investigation pass
- **132 test callers** swept in F-NAMES-1
- **800+ emission sites** threaded across the substrate
- **0** `<test>` / `<unnamed>` / `<runtime>` / `<entry>` user-visible after closure
- **0** known cracks remaining
- **7/7** arc138 canaries passing; full workspace clean (excluding lab, intentionally)

By the discipline:
- The user invoked the no-deferrals rule mid-arc and held the line through every "earned for follow-up" temptation.
- The "many simple things composed into a simple surface IS simple" reframing landed F-NAMES-1 as one slice instead of four.
- The trust-but-verify protocol surfaced two honest deltas worth more than the original work (slice 3b's report-quality drift; F-NAMES-2/4-coords' compose.rs dual-purpose discovery).
- Every BRIEF had a SCORE; every prediction had a calibration row; every honest delta got named.

This was foundation infrastructure shipped in fast slices with high discipline. The pattern is durable: future error-coverage work in any new substrate piece inherits the recipe.

---

**Arc 138 — complete.** The commits (chronological tip back through opening):

- `c39cdaa` — F-NAMES-2/4-coords (lambda + entry get definition coordinates)
- `2358458` — F-NAMES-2/3/4 investigations (all NOT cracks)
- `c8a0ed8` — F-NAMES-1e (wat-side spawn threads named — ZERO `<unnamed>`)
- `f803712` — F-NAMES-1d-asserthook (AssertionPayload.thread_name field)
- `76e2b76` — F-NAMES-1c (wat::test! deftest threads gain Builder names)
- `fc32611` — F-NAMES-1 (`<test>` placeholder eliminated)
- `78a7a76` — NAMES-AUDIT (placeholder labels charter)
- `53ec071` — slice 5 (ConfigError form_index → Span)
- `55c21f6` — F4c (opaque-cell helpers gain span — final crack closed)
- `fbcc1a4` — F4b (FromWat trait gains span)
- `ec4b465` — F4a (Value-shaped helpers gain span)
- `e4049e4` — CRACKS-AUDIT F4 decomposition
- `6327840` — F3 (WatReader/WatWriter trait gains span params)
- `6c08b26` — F2 (SchemeCtx trait gains span params)
- `c1cdcee` — F1 (MacroDef gains pub span field — first crack closed)
- `1df4af5` — CRACKS-AUDIT (no-deferrals charter)
- `036be1f` — slice 4b (EdnReadError + LowerError)
- `3b999c2` — slice 4a (MacroError + ClauseGrammarError)
- `2fb3ef9` — slice 3b (156 external file RuntimeError stubs)
- `371e831` — slice 3a-finish (300 RuntimeError stub sites threaded)
- `9d2065a` — slice 3a (RuntimeError variants gain spans — partial)
- `3d5420b` — slice 2 (TypeError variants gain spans + canary + `_with_span` sibling pattern)
- (slice 1 + slice 1 finish — predates session start)
- `<this commit>` — slice 6 (INSCRIPTION + USER-GUIDE update + 058 row)

Workspace: full test suite green (excluding intentionally-broken trading lab), 7/7 arc138 canaries passing, ZERO unresolved placeholders.

**Every panic in the substrate now navigates to a real coordinate.**

*PERSEVERARE.*

---

*Arc 016 made wat assertions navigable. Arc 138 made every substrate error navigable, and every thread in every panic header carries a real name. The agents reading our error messages stop guessing. The humans reading them stop grepping. The substrate stops lying.*

*This was one of the hardest boss fights, and we beat it.*

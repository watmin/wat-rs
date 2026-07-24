# BRIEF — the UNUSED-SPAN justification lint (a structural wall against dropped locations)

> **Why.** An ignored span param (`_span`, `_list_span`, `_head_span`: `&Span`) silences rustc's unused-param
> warning — but a `&Span` carries the **source location** for a located diagnostic. Ignoring it *can* mean a
> fallible fn emits an **unlocated** error (the "burned us" class). A one-time hand-audit is unreliable (this
> run mis-classified it 3×: "604" was a bad grep matching `list_span`; then "infallible generators" that in fact
> have error paths). So make it **structural**: every ignored span param must carry a justification rune, enforced
> by a lint — the wrong form (a silently-ignored span) becomes unrepresentable (R52 `QVOD LEX ACCENDIT` / R57
> unrepresentable > flagged / excusare — the reason must EARN its standing).

## The lint (model it EXACTLY on the existing pattern)
Read `tests/lint/no_inlined_wat_in_tests.rs` first — the established "scan source + require a `// rune:lint(...)`
earned exemption + FAIL listing offenders" shape. Build the twin:

`tests/lint/unused_span_justified.rs` — a `#[test]` that:
1. Walks `src/**/*.rs`.
2. Finds every param whose identifier **starts with `_` and ends in `span`**, typed `&Span` — regex ~
   `[(,\s]_[a-z_]*span: ?&Span\b` (matches `_span`, `_list_span`, `_head_span`; must NOT match `list_span`/`head_span`
   — no leading `_`).
3. For each, requires a justification rune `// rune:lint(unused-span) — <reason>` **co-located** (see placement below).
4. **FAILS**, listing every ignored-span param that lacks the rune (the `no_inlined_wat` failure shape — 🔥, the
   offender file:line list, and the "a legitimately-unused span earns a per-site rune; the reason must earn it" note).

## Design decisions — RULED (four-questions), do NOT re-fork
- **Placement (inline-on-param):** the rune lives on the **same line** as the `_…span: &Span` param. For a single-line
  fn signature, break that param onto its own line to carry the rune. Rationale: per-param precise, co-located with
  what it justifies, matches the `no_inlined_wat` inline-comment convention. (Obvious/Simple/Honest/Good-UX all ✓;
  beats a fn-level tag — coarse, "which param?" — or a proc-macro attr — params take no stable attrs.)
- **Scope (span only):** lint ONLY `_…span: &Span`. `_sym`/`_env`/other ignored params carry **no location**, so
  their ignore has no failure mode — linting them is rune-noise (fails Simple/Good-UX). Target the param whose ignore
  can drop a location.
- **FIX, don't launder:** the RED splits two ways per site — **earn a rune** where the fn's error is located elsewhere
  (`arg.span()`, `rust_caller_span!()`, or the fn is infallible), OR **FIX it** (thread the ignored `_span` into the
  error) where the error is genuinely unlocated. A rune whose reason is "this drops a location but we ignore it"
  does NOT earn its standing — that site gets *fixed*, not runed (excusare).

## The per-site work — DO THE ASSESSMENT, do not trust the seed
For EACH ignored-span site, **read the fn's error paths** and classify (the orchestrator's hand-audit was unreliable
— re-verify every one):
- **infallible** (no `Err`/`?`/`map_err`/panic in the body) → rune: `infallible — no error path`.
- **located elsewhere** (every error uses a real span: `arg.span()`, a threaded inner span, or `rust_caller_span!()`)
  → rune stating WHERE: e.g. `locates at arg.span() (more precise than the coarse outer span)` or
  `leaf helper — own error uses rust_caller_span!; sub-errors located via eval_inner`.
- **unlocated** (the fn emits an error while the source `_span` was AVAILABLE and would improve it) → **FIX**: thread
  `_span` into that error's `RuntimeError { span: … }`, rename `_span`→`span`, no rune.
- **ambiguous / fix is non-trivial** (the span isn't cheaply threadable to the error site) → **STOP-3**, surface it.

**Seed from the audit (RE-VERIFY each — some of these seeds were mis-classified):**
| site | audit lean (VERIFY) |
|---|---|
| `src/check.rs` `infer_*` (8: infer_let, infer_poll_prime, infer_persistentvector_constructor, infer_map_literal, infer_set_literal, infer_string_concat, infer_linked_list_constructor, infer_boolean_shortcircuit) | rune — errors at precise `arg.span()` (validated on infer_string_concat) |
| `src/runtime.rs` (4: eval_and, eval_or, eval_tuple_ctor, eval_forms) | rune — leaf helper, `rust_caller_span!` + eval_inner (validated on eval_tuple_ctor) |
| `src/rust_deps/marshal.rs:300` from_wat(Value) | rune — infallible `Ok(v.clone())` (validated) |
| `src/intrinsic/bytes.rs:112` from_hex | rune — error at `arg_span`; bad-hex → Ok(None) (validated) |
| `src/string_ops.rs` uuid_v4/uuid_nil/list_of; `src/time.rs` time_now; `src/intrinsic/witness.rs` measurement; `src/intrinsic/bytes.rs:45` to_hex | **RE-ASSESS — these HAVE error paths (the "infallible" seed was WRONG); determine located-vs-not per site** |
| `src/io.rs:1418/1447` temp_file_new / temp_dir_new | **likely FIX** — propagate `WatTempFile::new()?` while ignoring `_list_span`; thread the span into the io error (VERIFY WatTempFile::new's error location first) |
| `src/kernel/listener.rs:338` accept (process tier) | rune — errors are `AcceptOutcome` VALUES (located at the caller's match); must-never-happen raises live in `eval_accept_prime` with its span |
| the recv'/send'/poll'/**close'** wall `_span`s (if any surface) | rune — the wall's value-face; the raise-with-span became a value-at-caller |

## Blast radius
`tests/lint/unused_span_justified.rs` (new) + inline runes across `src/**/*.rs` (comment-only edits) + the few genuine
FIXES (thread a span — `io.rs` + any RE-ASSESS site found unlocated). No behavior change from the runes; the fixes
only ADD location to an existing error. Do NOT touch the peer-lifecycle walls' logic (only add their span-runes if the
scanner flags them).

## STOP triggers (rejection criteria)
- **STOP-1:** the scanner finds **far more** than ~23 sites (grep undercounts — expect *more*, not fewer). If it's a
  handful more, rune/fix them all. If it's 100s, STOP + surface (the scope was mis-estimated; re-weigh the approach).
- **STOP-2:** a site's error is genuinely unlocated AND threading the span is non-trivial (the span isn't in scope at
  the error site, or the error crosses an abstraction boundary). STOP — do NOT force an ugly fix or write a
  dishonest rune; surface it for a design call.
- **STOP-3:** you cannot write an HONEST rune reason for a site (the ignore is neither infallible, nor located
  elsewhere, nor cheaply fixable). That's a real gap — STOP + surface, do not paper over with a vague rune.

## Weigh (the orchestrator re-runs; do NOT trust the report)
- `cargo nextest run --release -E 'test(unused_span_justified)'` → GREEN (all sites runed-or-fixed).
- **the whole floor: `cargo nextest run --release`, read the Summary line** — expected 4215/0 + the new lint green;
  any NEW non-lint RED = a fix broke something → report it.
- content-integrity: the diff is the new lint + inline rune comments + the genuine span-threading fixes. Read each
  FIX's diff — confirm it only ADDS a span to an error, changes no logic.

## Copy for shape
`tests/lint/no_inlined_wat_in_tests.rs` (the scanner + failure-message + rune-matching pattern to mirror).

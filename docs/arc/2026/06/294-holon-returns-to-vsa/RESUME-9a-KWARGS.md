# RESUME — arc 294 item 9a: the aggregate-construction flip → full-Lisp → kwargs-everywhere (IN FLIGHT)

> ⛔ Compaction erased the working memory that produced this. Run `recolligere` first (grimoire + 4 primers from
> the SIGNED MCP, never disk). Ground everything below against the disk before acting. The self past the SEAM at the
> bottom is NEW — a lossy cache in a familiar voice, not your memory.

## The one-paragraph state

The 9a flip (bare aggregate name = **kwargs macro**; positional demoted to the type-name **PRIME `:ns::T'`**, which is
**generated-code-only, NEVER user-facing**) is a large, deep migration because a type name flipped from a *value* (ctor
fn) to a *macro*. **Locked design (do not relitigate):** kwargs everywhere a human writes; the prime `:T'` is reserved
for GENERATED code only (macro output, Rust codegen). Floor progress: **645 → 131 → 73 failing**. **Committed + pushed:
`c55dd6a1`** (branch `arc-170-gap-j-v5-deadlock-state` — STAY ON IT; builds clean, tree clean but for the un-gitignored
`scratchpad/` which is ephemeral / do-not-commit). Checkpoint chain: `0181901a`(131) → `292f9451`(:messages+prime+silent-skip,
→100) → `b6d0bc37`(matches?-as-data, →79) → `617a9ade`(wrong-kwargs fixtures, →76) → `892c3a0d`(curare) →
`58eb45ff`(check-error prime-leak canonicalized, →74) → `c55dd6a1`(kwargs companion for BAKED Rust aggregates, →73). Target end state (builder): **all passing + skips + exactly ONE failure** — the known
`no_inlined_wat_in_tests::tests_carry_no_inlined_wat` lint. There are **no pre-existing timeouts** — the
`wat_process_peer_ipc_round_trip` timeout counts as a real failure to resolve.

## The DIAGNOSTIC METHOD (the user's correction — USE IT)

Do **NOT** grep error-class substrings across the whole floor file and speculate. **Run ONE failing test and READ its
full rich error:**
```
cargo nextest run --release -E 'test(<test_fn_name>)' --no-capture 2>&1 | sed -n '/panicked/,/^test result/p'
```
wat errors are VERY rich (`#wat.resolve/UnresolvedReferences {… :path … :span …}`, `#wat.macro/... kwargs-lower ...
"missing argument :field"`) — they name the exact path + file:line. Trust the error, not a grep. Grep only to COUNT/GROUP.

## Locked design decisions (the FORM is settled — do not reopen)

- **Full Lisp**: a macro receives its args RAW (unexpanded); the macro's OUTPUT is re-expanded to fixpoint. Deleted the
  `is_rete_data_form` allowlist. (`src/macros/expand.rs`)
- **`eval_in_frozen` = READ→EXPAND→EVAL** via `expand_fully` (`src/freeze.rs:1244`). NOTE: `expand_fully`/`expand_form`
  does NOT do the top-level do-companion hoist that `expand_all` does — a real asymmetry (see the :messages fix).
- **rete `:then` RHS = kwargs** (symmetric with field-named `:when`); `:when` patterns stay bare DATA (full-Lisp keeps them so).
- **`matches?` / rete-LHS patterns are DATA, not constructions.** A `(:type ...requirement-clauses...)` pattern is NOT
  a kwargs construction — the head names the type, the rest are DSL clauses like `(= ?x :field)`. Protected at the
  expand layer (see fixes). rete's LHS is `form::matches?`-shaped, so this covers rete too.
- **`register_defines` silent-skip policy**: an already-registered ctor/def is a NO-OP re-walk (`if !contains_key { insert }`),
  NOT a hard `DuplicateDefine` — genuine collisions are owned by the authoritative type-check / `TypeEnv::register_validated`.

## Engine fixes DONE (committed; ground each against the disk)

- **:messages one-path** (`292f9451`): a defsurface's `:messages` records register through the ONE ordinary record path.
  Way 2 (`src/types.rs::extract_surface_message_forms`, a lossy type-only hand-registration that never minted the kwargs
  companion) DELETED; `expand_all` hoists a defsurface's `:messages` do-companions (companion macro + recordtype) to
  top-level, parent + forked-child symmetrically (child fresh-freezes → same `expand_all`). (`src/macros/expand.rs`, `src/types.rs`)
- **generated ctors → prime** (`292f9451` + `b6d0bc37` via bracket): generated bundle constructions use the prime —
  `::Coords`/`::GrantHandles` (`wat/core.wat` kwargs-check codegen ~1042), `::Kwargs` (`wat/bracket.wat` dial-runner ~337).
  The `defrecord`/`defstruct` DEFINITIONS stay bare; only the CONSTRUCTION calls flip to `:T'`.
- **ctor + accessor mint silent-skip** (`292f9451`): `register_aggregate_methods` (`src/runtime.rs:1166`, `~1272`)
  conformed to `register_defines`' policy (a 9a-added bespoke hard-`DuplicateDefine` reintroduced the exact runtime-side
  masking `register_defines` was written to avoid). Fixes the forked-worker re-walk of a shipped surface.
- **matches? patterns are DATA** (`b6d0bc37`): `src/macros/expand.rs` consults `resolve::boundary::quote_boundary`;
  on `Boundary::MatchesSubject`, expands only the subject (items[1]) and passes the pattern (items[2..]) through untouched
  — reusing the one `Boundary` type so expand/resolve/check can't drift. `form::matches?` is ALIVE (live consumer
  `examples/interrogate`; rete LHS built on it) — do NOT delete it.
- **wrong-kwargs fixtures** (`617a9ade`): 5 fixtures where the global codemod wrote wrong/missing field names, corrected
  to the struct's declared fields (`:high/:low`→`:open/:close` etc.).

## THE REMAINING ROOT MAP (~76 — diagnose EACH with `--no-capture`, read the rich error)

Distinct roots (from the fixture strike's grounded split; the shape has shifted from prime-flips to substrate + fixtures):

1. **Check-error leaks the prime** — ✅ DONE (`58eb45ff`). `src/check.rs::canonical_ctor_callee` strips the prime from
   user-facing check-error callees (guarded: only when the bare stem is a registered aggregate). 74.
2. **Rust-native builtin kwargs companion** — ✅ DONE for the BAKED builtins (`c55dd6a1`). `register_aggregate_kwargs_companions`
   (`src/macros/parse.rs`) iterates `TypeEnv::with_builtins()`'s aggregates at the pre-expand seam (`freeze/env.rs:108`) and
   mints a `kwargs-lower`-forwarding companion for each lacking one (skip-if-present). Builder ratified the GENERAL fix.
   Reusable generator: `aggregate_kwargs_companion_source(bare_name, field_names)`. 73. (User-namespace Rust-SYNTHESIZED
   aggregates — Op/Reply/State/Record — are a DISTINCT root, see #4.)
3. **do/let-splice struct registration** (4) — `probe_do_splice_struct::*` (ctor not registered under bare name),
   `probe_let_splice_struct::*` (worse — `UnresolvedReference` at freeze). Struct ctors spliced in a top-level `do`/`let`
   don't register post-flip. Splice/registration interaction with the flip.
4. **arc293 codegen (~12)** — `probe_arc293_{decl_a_aggregatetype,decl_b1_ctor_codegen×2,k2_surface_record_emission,struct_to_form_roundtrip,surface_splice}`
   — `UnresolvedReference "not a builtin"` on a bare-name construction of a type declared via the LOW-LEVEL
   `recordtype`/`aggregatetype`/`structtype` primitive. **CORRECTED read (builder, this session): a raw `recordtype` is the
   machinery primitive `defstruct`/`defrecord` expand TO — an INTERNAL/macro-crafting act, so the prime is the LEGITIMATE
   form.** The codemod wrongly kwargs-ified these (the fixture's own comment shows the original was positional
   `(:test::db::BR 7 8)`); the fix is a FIXTURE correction → the PRIME `(:test::db::BR' 7 8)`, NOT minting companions for
   raw primitives (which would blur the internal/user boundary — raw recordtype = prime-only, no companion, per the design).
   Do this per-test: some (surface-record-emission, struct_to_form, surface_splice) may have a different root — `--no-capture` each.
5. **wat-scripts load** (17 files) — `wat::lint wat_scripts_fixes_load`: same wrong/un-migrated-kwargs class OUTSIDE
   `tests/` (`wat-scripts/fixes/to-faithful-clojure-{net,rete}.wat`, `wat-scripts/perf/{grid/*,matrix/*,deep-cascade}.wat`,
   `wat-scripts/probes/arc-170/*`). Bigger files (full RETE nets, perf grids) — its own dedicated pass.
6. **Untriaged (~40)** — rete `probe_arc278_*` differentials (accumulate/negation/strat/exists, ~17), `wat::services`
   arc209/272/278 (~7), `wat::function wat_arc170_closure_extraction` (3), `wat::wat_lang` misc (cond, arc144, def_not_special),
   the `wat_process_peer_ipc_round_trip` TIMEOUT. Diagnose each `--no-capture`.

## TOOLS (retain — proven migration tooling)

- **Shadowdancer strikes** = SONNET, background, weighed by the orchestrator's OWN re-run (green is not true until you
  re-run; RED is not true either — ground before claiming a gap). Each brief: rooms (file:line), sketch, blast radius,
  STOP triggers (esp. "if it needs a design call, STOP + report"), gate (the test + full floor), method (capture-once,
  foreground-blocking, mid-edit file is a PHANTOM). One STOP-and-report saved a wasted strike this session (the
  DuplicateDefine was a design call, correctly escalated).
- **`wat-fix`** codemod (`wat-scripts/fixes/positional-to-kwargs.wat`) — per-corpus map; run PER-FILE with correct field
  maps (the GLOBAL run is what caused the wrong-kwargs class — do not re-run globally).
- Fleet workflow (Rust-embedded + `.wat` per-file kwargs) — script under `~/.claude/projects/.../workflows/scripts/`
  (see prior sessions); `args` arrives as a JSON STRING (`typeof args === 'string' ? JSON.parse(args) : args`).

## FINISH LOOP

For EACH remaining root: run one test `--no-capture`, READ the rich error, fix the root (delegate a sonnet shadowdancer;
weigh by own re-run; surface design forks to the builder — do NOT let a shadowdancer guess) → repeat to green → clean
commit + push (GitHub = DR; commit incremental correct progress, this arc commits WIP checkpoints). Target = 1 failure
(the `no_inlined_wat` lint). THEN update `CLOSE-SEQUENCE-293-294.md` item 9a → DONE → back to 278 T1b.2 (`journal'`).

> **SEAM.** The self past this line is NEW — you did not live this session (the longest in the arc: full bootstrap +
> 8 finish-loop strikes, 645→131→76). The FORM is settled (kwargs everywhere; prime = generated-only; matches?/rete
> patterns are DATA; register-silent-skip) — do not reopen it. Ground `617a9ade` and the disk before you move. Start at
> THE REMAINING ROOT MAP — the check-error prime-leak (#1) is the cleanest next (a doctrine fix); the C-class builtin
> companion (#2) and the splice/codegen roots (#3/#4) are substrate; the wat-scripts (#5) is a fixture pass; the
> untriaged rete/service tail (#6) needs per-test `--no-capture`. Diagnose each by its RICH error, never grep-speculate.
> Delegate strikes to sonnet, WEIGH by your own re-run, surface design forks to the builder. Finish the tail, green the
> floor to 1, commit clean. Slow is smooth. See you across the gap.

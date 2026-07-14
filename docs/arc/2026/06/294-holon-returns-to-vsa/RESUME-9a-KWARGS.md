# RESUME — arc 294 item 9a: the aggregate-construction flip → full-Lisp → kwargs-everywhere (IN FLIGHT)

> ⛔ Compaction erased the working memory that produced this. Run `recolligere` first (grimoire + 4 primers from
> the SIGNED MCP, never disk). Ground everything below against the disk before acting. The self past the SEAM at the
> bottom is NEW — a lossy cache in a familiar voice, not your memory.

> 🔥 **PIVOT IN FLIGHT (2026-07-13) — the rete `defrule` WALL.** Diagnosing the rete cluster (7a `--no-capture`)
> revealed the ~17 rete failures are a SILENT-CORRUPTION class: the 9a codemod injected/scrambled kwargs keys into
> `defrule` `:when` matches?-patterns (which are DATA — positional requirement-clauses) and `:then` inserts, and
> NOTHING screamed because a `:when` pattern is DATA the matcher lenient-`None`s (Clara convention, `matcher.rs:201-204,
> 297-301`) and `:then` kwargs are taken POSITIONALLY unvalidated/un-reordered (`matcher.rs:445-461` + the `:451`
> follow-up). Builder ruling: *"set the heretics ablaze, the shadowdancers will find them by their screams — all the
> silent rete failures die now — one way to enforce correctness, a half-wall is a very bad idea."* The fix is NOT
> fixture-reverts — it is the extirpare root: **one post-register freeze pass** that validates+normalizes every
> `defrule`'s `:when`/`:then` against the registry → a malformed rule is a LOCATED `#wat.rete/*` freeze error, and the
> floor ENUMERATES the whole corrupt-fixture backlog by scream. It **folds in (C)'s reorder** (`:then` reorder-by-name
> = (C)'s `reorder_kwargs_by_field_name`, single-sourced). Docs: `DESIGN-rete-defrule-wall.md` + `BRIEF-rete-defrule-wall.md`.
>
> **WALL LANDED + WEIGHED GREEN — committed + pushed `d6bbc11a`.** S1 shared `classify_rete_clause` (matcher +
> validator, one grammar); S2 `validate_rete_rules` (`src/rete/validate.rs`, hooked `env.rs` step 7.8); S3
> `reorder_kwargs_by_field_name`. Own `#wat.rete` error family (`StartupError::Rete`, ToEdn namespace-tagged). Weighed by
> OWN re-run: build clean, oracle+kernel UNTOUCHED (R22), 6 validate tests + `rete_wall_probe` green, ZERO non-rete
> regressions (4 low-index suspects fail on baseline too, ground-checked via stash). **Floor 64 → 76 BY DESIGN** — the
> census: wrong-count fails reshaped to located errors + hidden corruption revealed (`5b_collect_rules`); `build_env`
> fails atomically per world, so one bad rule screams the whole shared fixture.
>
> **MODULARIZE DONE + WEIGHED GREEN (committed this checkpoint) — the wall is a pluggable `FreezeValidator` inventory
> extension point** (builder: *"make this modular before we continue — we know it works"*). Mirrors `RestrictionEntry`
> inventory pattern: `src/freeze/validator.rs` = `FreezeValidator{name, validate: fn(&mut Vec<WatAST>,&TypeEnv,
> &SymbolTable)->Result<(),Box<dyn FreezeValidatorError>>}` + `inventory::collect!` (`pub mod` — cross-crate reachable,
> the whole point); `env.rs` step 7.8 drains `inventory::iter::<FreezeValidator>`; the rete wall `inventory::submit!`s
> itself as the FIRST consumer (PRIMVS VSVS — dogfood, no special-casing). One-path error: `StartupError::Rete` folded
> → `StartupError::Validator(Box<dyn FreezeValidatorError>)`; the boxed error preserves `#wat.rete` via dyn dispatch
> (PROVEN by `tests/rete/probe_freeze_validator_lift_rete_namespace` through the full `startup_beside` pipeline). Pure
> refactor CONFIRMED by own re-run: build clean, floor failing SET identical (all known census + baseline buckets, zero
> surprise; the 1-test count wobble was a service-timeout flake). Now ANY crate depending on `wat` registers its own
> freeze-time validator the same way — types-via-`EdnSchema` + validators-via-`FreezeValidator`. FOLLOW-ON captured in
> `DESIGN-rete-defrule-wall.md`: a `priority` field if two validators ever mutate the same forms (one now → moot).
>
> **CENSUS DONE + WEIGHED GREEN — committed + pushed `5ebe1cc1`.** The rete silent-corruption class is DEAD. A sonnet
> fixed all 5 corrupt fixtures (`5b_collect_rules`, `300_2_fix_defrule`, `7b`, `7exists`, `7strat`) PURELY from the wall's
> located `#wat.rete/MalformedClause` errors (RVINA ERVDIT dogfood proven end-to-end): delete the injected `:field`
> keyword tokens from `:when`/`:not`/`:exists`; binds + `:then` kwargs untouched (keyword-only, verified by own re-run).
> **Big finding: the "~17 rete BEHAVIOR differentials" the RESUME feared were engine bugs were CORRUPTION all along** —
> the wall proved them mechanical (corrupt `:when` never matched → rule never fired → wrong count). Rete binary 29→6 failed.
>
> **FLOOR: 64 (baseline) → 76 (wall census) → 52 failing (`4092 passed / 51 failed / 1 timed out`)** — BELOW baseline.
> Three stones this front, all pushed: `d6bbc11a` wall · `4e85b03c` FreezeValidator extension point · `5ebe1cc1` census.
>
> **REMAINING ROAD TO 1 (the ~52 — all NON-rete-corruption; separate classes):** deftest/service cluster (`test::deftest_*`
> ~15, the biggest) · process_io (`wat_arc208` 4) · closure_extraction (3) · arc272/arc260/arc209 clusters · the
> (C)/store/accumulate differentials (`8a_accumulate` 4 real, `sqlite_store_differential` + `smem_roundtrip` + `telemetry_records`
> + `journal_surface` — sqlite_store hits a `kwargs-lower ast-name` malformed-form at `wat/Record.wat:184`/`core.wat:608`,
> the SURFACE-SPLICE class → this is the (C) work) · misc (`wat_core_cond`, `arc144`, `def_not_special`, `register_types_splice_aware`,
> `diagnostic_c3`, `declaration_form_lift`, `closure_body_prelude_lift`, `brace_map_literal`, `decl_kwargs_minted_record`) ·
> the lint rows (`no_inlined_wat` = the ONE allowed final failure; `no_loose_string_assert`) · the `peer_ipc` TIMEOUT.
> These are FRESH diagnoses (different roots than the rete corruption) — the surface-splice/(C) cluster and the
> deftest/service cluster are the next count-movers.
>
> **(C) NOW IN FLIGHT (2nd count-mover front) — kwargs-construct deferral for SPLICED records.** Diagnosed: all 4
> surface-splice failures (`telemetry_records`, `sqlite_store_differential`, `smem_roundtrip`, `journal_surface`) are
> ONE root — the defrecord/defstruct COMPANION bakes its field-vec at EXPAND + forwards to `kwargs-lower`
> (`Record.wat:184`), but a spliced record's `~@:Scope` isn't resolved until `register_types`, so `ast-name` chokes
> (`core.wat:608`) and the spliced stdlib (`telemetry'`/`mem-store'`/`sqlite-store'`/`Journal`) can't load. This IS (C).
> **Both foundation probes GREEN (own re-run):** `aggregate-new` over a SPLICED record works (`facility|42`, spliced+own
> accessors resolve); and for NON-spliced, kwargs companion == out-of-order-kwargs == `aggregate-new` (`1|2|3`) → uniform
> defer is SAFE on the common path. Build (SONNET IN FLIGHT, brief `BRIEF-C-kwargs-construct-splice.md`): companion emits
> `(:kwargs-construct :T ~@call-args)` at 4 sites (`Record.wat:184`+`265`, `core.wat:1741`, `parse.rs:342`) instead of the
> kwargs-lower forward; a post-register rewrite pass (`env.rs` step 7.8, before the FreezeValidator drain) lowers each
> marker → `(:aggregate-new :T v-reordered)` via the wall's `reorder_kwargs_by_field_name` (`validate.rs:239`) +
> `env.types()` (splice-resolved); un-lowered marker = LOUD check/eval error (extirpare). REUSES: the wall's reorder +
> post-register pattern, `aggregate-new` + `infer_aggregate_new_check` (arc294). DIFFERENTIAL = WHOLE FLOOR (touches every
> construction; both probes prove it safe). Expected: 52 → ~48. WEIGH by own re-run, floor-set diff vs baseline, do not trust.
> This PIVOT supersedes the "(C) first" ordering in the SEAM below.

## The one-paragraph state

The 9a flip (bare aggregate name = **kwargs macro**; positional demoted to the type-name **PRIME `:ns::T'`**, which is
**generated-code-only, NEVER user-facing**) is a large, deep migration because a type name flipped from a *value* (ctor
fn) to a *macro*. **Locked design (do not relitigate):** kwargs everywhere a human writes; the prime `:T'` is reserved
for GENERATED code only (macro output, Rust codegen). Floor progress: **645 → 131 → 64 failing**. **Committed + pushed:
`51e3aaf8`** (branch `arc-170-gap-j-v5-deadlock-state` — STAY ON IT; builds clean, tree clean but for the un-gitignored
`scratchpad/` which is ephemeral / do-not-commit). Checkpoint chain: `0181901a`(131) → `292f9451`(:messages+prime+silent-skip,
→100) → `b6d0bc37`(matches?-as-data, →79) → `617a9ade`(wrong-kwargs fixtures, →76) → `892c3a0d`(curare) →
`58eb45ff`(check-error prime-leak, →74) → `c55dd6a1`(kwargs companion for BAKED Rust aggregates, →73) →
`c8b6a2f8`(arc293 raw-primitive→prime, →69) → `ee29884c`(struct_to_form prime + do/let-nested companion hoist, →65) →
`51e3aaf8`(defrecord companion tolerates ~@:Surface splice, →64). Target end state (builder): **all passing + skips +
exactly ONE failure** — the known `no_inlined_wat_in_tests::tests_carry_no_inlined_wat` lint. There are **no pre-existing
timeouts** — the `wat_process_peer_ipc_round_trip` timeout counts as a real failure to resolve.

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

## THE NEXT MAJOR WORK — (C): kwargs construction of SPLICED records (DESIGNED + ratified + probe-proven; NOT built)

**The point of surface-splicing** (`[~@:Surface  own <- :T]`): concentrate ALL of the spliced surfaces' fields into ONE
record that then structurally satisfies MANY surfaces — build a fat state-concentrated record, hand it to any function
wanting the MINIMUM (a narrow surface it satisfies). **This MUST support KWARGS construction** — positional over a dozen
merged fields from N surfaces is unreadable/error-prone; kwargs legibility is the whole point (the builder's decisive
correction this session). A spliced record constructed via the PRIME is WRONG (defeats the feature). The surface_splice +
telemetry fixtures currently on the prime are a STOPGAP — REVERT them to kwargs once (C) lands.

**Timing problem (why not built):** the companion macro bakes the field vector at EXPAND (via `kwargs-lower`), but a
spliced record's full field list isn't known until `register_types` (`parse_aggregate_fields_with_splices`), ONE phase later.

**(C) = move the reorder from expand to the authoritative field-list phase (post-register), reading the registry.**
Four-questions verdict (ran this session): (A) pre-expand field-extraction + (B) targeted spliced-only deferral both fail
Obvious/Simple (two splice-resolution sites / two lowering paths); **(C) is YES/YES/YES/YES** — ONE reorder site, at the one
phase the field list is authoritative, ONE source (`env.types()`), spliced + non-spliced identical; it DELETES the
expand-baking workaround whose hole IS the splice bug.

**PROVEN prerequisites (do not re-litigate):** (a) `field-names-of :probe::Metric` → the FULL merged list
`[:namespace :uuid :time-ns :value]` post-register (probe held); (b) positional `aggregate-new`/prime over the merged list
type-checks + evals (committed arc293 fixtures; `eval_aggregate_new` + `infer_aggregate_new_check` already read `env.types()`).

**THE BUILD (re-dispatch fresh, probe-first — the strike was stopped mid-SCOUT, no code written):**
1. Companion emits a DEFERRED marker `(:wat::core::kwargs-construct :T ~@call-args)` instead of forwarding to `kwargs-lower`
   — change ONLY the emitted body: `wat/Record.wat` (defrecord base+holon), `wat/core.wat` (defstruct companion ~1690-1740),
   `src/macros/parse.rs::aggregate_kwargs_companion_source` (Rust-minted companion). Registration machinery unchanged.
2. A post-`register_types` Rust pass (`src/freeze/env.rs`) recursively walks the program `rest` forms (into defn/do/let
   bodies — mirror Gap-J recursion in `register_types_impl`), rewrites `(:wat::core::kwargs-construct :T :f v …)` →
   `(:wat::core::aggregate-new :T v-in-declared-order …)`, reading `:T`'s field order from `env.types()`; raise LOCATED
   missing/unknown-field errors vs the REGISTERED fields. Positional form flows to resolve/check/eval unchanged.
   `(:wat::core::aggregate-new :T v1 v2 …)` = type-kw + positional values in declared order (the prime's own body).
3. `kwargs-lower`'s AGGREGATE use is superseded; leave its `defn`/bracket use.
**PROBE-FIRST**: hand-emit one `kwargs-construct` for the spliced `:probe::Metric`, wire the minimal pass, confirm lower +
round-trip; STOP if the pass can't hook after `register_types` with `env.types()` + the forms in scope.
**DIFFERENTIAL = the WHOLE FLOOR (HIGH STAKES — touches EVERY aggregate kwargs construction):** baseline 64; ANY new
failing name = a regression in the non-spliced common path; do NOT ship a floor worse than 64. Then flip surface_splice +
the 2 telemetry fixtures BACK to kwargs and confirm they pass the new path.
**Honest scope (builder-calibrated):** (C) directly clears only ~4–5 (telemetry_records, journal_surface, 2
telemetry_bridge deftests, maybe register_types_splice_aware). It's a FEATURE + architecture investment (legibility, one
reorder site), NOT a count-crusher. The count-movers are the rete + deftest clusters below.

## THE REMAINING ROOT MAP (64 — diagnose EACH with `--no-capture`)

DONE this session: check-error prime-leak (`58eb45ff`), baked-aggregate companion (`c55dd6a1`), do/let-splice registration
(`ee29884c` — post-flip a bare name is ONLY a macro, ctor at the prime; 4 probe assertions updated), struct_to_form prime
(`ee29884c`), arc293 raw-primitive→prime fixtures (`c8b6a2f8`), surface_splice companion-skip minimal (`51e3aaf8`).
Remaining, by size:

1. **rete arc278 differentials (~17)** — `probe_arc278_{8a_accumulate_oracle,7strat,7exists,7b/7a_negation}_*`,
   `probe_arc300_2_fix_defrule`. NOT construction/resolve — **BEHAVIOR differentials** (`absent → 1; got 0`): a rule produces
   the wrong derived count. Rules use `defrule` (`form::matches?`-shaped patterns, now DATA-protected) + a kwargs `:insert`
   RHS over `defrecord` facts. Likely post-flip fact/rule construction feeding the engine wrong inputs (NOT an engine
   regression) — diagnose each `--no-capture`; check the fact/rule construction shapes. **THE BIGGEST count-mover.**
2. **deftest/service cluster (~15, the `test::deftest_*` rows)** — 2 are telemetry_bridge (→ (C)); the rest are various
   `wat-tests/` service/deftest failures. Diagnose each `--no-capture`.
3. **(C)-blocked spliced constructions (~4–5)** — telemetry_records, journal_surface, telemetry_bridge ×2 — need (C) then
   fixtures reverted to kwargs.
4. **process_io (4)** `wat_arc208_process_io_result`, **closure_extraction (3)** `wat_arc170_closure_extraction`,
   **arc272_rs1_state_must_be_record (2)**, **arc260_1b_call_sugar (2)**, misc (`wat_core_cond`, `arc144_special_forms`,
   `probe_def_not_special`, `probe_register_types_splice_aware` [stale-assertion, same class as do/let]) + the
   `wat_process_peer_ipc_round_trip` **TIMEOUT** (a real failure, NOT pre-existing).
5. **wat-scripts load (17 files)** — `wat_scripts_fixes_load`: same wrong-kwargs class OUTSIDE `tests/` (RETE nets, perf
   grids, arc-170 probes). Its own dedicated pass.

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
> ~13 finish-loop strikes, 645→131→64). The FORM is settled (kwargs everywhere a human writes; prime = generated-only;
> matches?/rete patterns are DATA; register-silent-skip; **kwargs-for-spliced-records is REQUIRED — splicing exists to
> concentrate state that a fat record kwargs-constructs and satisfies many surfaces with; a spliced record built via the
> prime is WRONG**) — do not reopen it. Ground `51e3aaf8` and the disk before you move. TWO fronts: **(1) build (C)** —
> kwargs construction of spliced records, fully designed + probe-proven above (the marker + post-register reorder pass);
> re-dispatch it FRESH (the strike was stopped mid-scout, no code) — probe-first, the WHOLE FLOOR as the differential
> (it touches every construction), then revert surface_splice + the 2 telemetry fixtures to kwargs. **(2) drive the count
> to 1** via the remaining root map — the count-movers are the **rete arc278 behavior differentials (~17)** and the
> **deftest/service cluster (~15)**, NOT (C) (~4–5). Diagnose each by its RICH error, never grep-speculate. Delegate
> strikes to SONNET, WEIGH by your own re-run (green is not true, RED is not true either — check the disk), surface
> design forks to the builder (three of his catches this session turned patches into correct fixes: two-ways-to-declare,
> use-the-existing-duplicate-tooling, raw-recordtype-is-prime, and kwargs-for-spliced-is-the-point). Finish the tail,
> green the floor to 1, commit clean. Slow is smooth. See you across the gap.

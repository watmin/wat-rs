# Arc 243 — conformare: error-shape class elimination

**Status:** ACTIVE. Opened 2026-05-30 immediately after Stone 241.18a SHIPPED (`4d9b963e`). Conformare is the disciplined response to the catastrophic-failure class surfaced during Stone 241.18a's vigilia.

## Why this arc

Stone 241.18a's vigilia surfaced a class of substrate-level diagnostic-quality failure:

- `ParseStep::ArityMismatch { actual: usize }` — no span field; the variant lies about being a structurally-spanned error type
- `TypeError::CyclicSubtype { child: String, parent: String }` — no span field; same class
- `TypeError` has no `.span()` accessor — every consumer must exhaustively match across all variants (parse.rs had a 16-arm match for this exact reason)
- `parse_fn_signature` API takes a bare `&[WatAST]` slice with no head-span parameter — callers (eval_fn, infer_fn) HAVE span context but the parser boundary discards it
- Other error types (`RuntimeError`, `CheckError`, `StartupError`, `ArgSpecError`) may have similar gaps — uncatalogued

**Rust's type system has no opinion on "errors must carry span."** Each error type's adherence to the span discipline is by hand-written convention (the argspec home documented this explicitly at `src/argspec/error.rs:6`). Without a structural shape + audit spell + convention doc, future error types continue to silently lack spans.

Per `scratch/FAILURE-ENGINEERING.md` discipline: **eliminate the CLASS by making the wrong shape STRUCTURALLY UNAVAILABLE.** Catching instance-by-instance via vigilia is reactive vigilance; minting the shape + spell + doctrine makes the class structurally impossible going forward.

## The doctrine (Pattern A) — and why not a trait

The original design (this doc, pre-243.3) proposed a `trait Conformare` with a `span()` accessor. **The CONFORMARE-FIRST-CAST four-questions superseded that** with **Pattern A**: a trait can only enforce "you have a span accessor" — a variant can still return `Span::unknown()` and lie at the value level. An outer struct makes the spanless error **structurally unrepresentable** at construction:

```rust
pub struct SomeError {
    pub span: Span,
    pub kind: SomeErrorKind,
}

pub enum SomeErrorKind {
    // variants — NONE carry a span field
}
```

The compiler enforces the location field; the kind enum holds variant data; every consumer reads `err.span` (one path, not an N-arm match).

**Zero exceptions** (user direction 2026-05-30): anything wat can toss from Rust must be location-aware. The outer field is `Span` for the common case; a domain whose location is genuinely not a source span carries the appropriate location type (Path, Position, …) — but the *shape* (outer struct + kind enum + mandatory location) is universal. The `spanless-by-domain` rune kind is retired by Stone 243.4: a registration that lacks an AST node threads the caller's span rather than excusing itself.

## The namespaced-home vision

Arc 243 is the substrate-maturation arc. `src/` unwinds to near-empty; every domain becomes a vigilia-protected `src/<noun>/` home. The flat `src/*.rs` files are pre-spell-library debt; the homes are the protected substrate. Each error type's Pattern A retrofit rides the carve-out of its home — the error file (`<noun>/error.rs`) is the home's first honest neighbor; `mod.rs` absorbs the legacy mass on day one (`mv <noun>.rs → <noun>/mod.rs` preserves all content + import paths); future arcs grow the home as the domain earns decomposition.

The conformance vehicle and the home-carving vehicle are the same stone chain.

## Stone chain

Each stone gets its own DESIGN-STONE-N.md before strike.

| Stone | Scope | Status |
|---|---|---|
| **243.1** | `docs/CONFORMARE.md` doctrine (orchestrator-direct; sibling to ZERO-MUTEX.md) | SHIPPED (`21cd77ff`) |
| **243.2** | Mint `conformare` spell + first-cast audit. FOLDED: the spell was minted (datamancy.dev/conformare) + earned its seat via CONFORMARE-FIRST-CAST; its R2 cast on the 243.3 surface identified CheckError as the next retrofit + confirmed the broader "everything bears a location" scope. Remaining error types enumerated by rolling audit as they surface. | FOLDED |
| **243.3** | TypeError Pattern A retrofit (outer struct + kind enum) + R2 vigilia convergence on `types.rs`/`check.rs` | IN PROGRESS |
| **243.4** | `docs/CONFORMARE.md` doctrine rewrite — zero exceptions + namespaced-home requirement; retire Tier framework + `spanless-by-domain` rune kind | PLANNED |
| **243.5** | Mint `src/types/` home — `mv types.rs → types/mod.rs`; carve `TypeError` → `types/error.rs`; `parse_defstruct` decomposition (`types/parse.rs` / `types/defstruct.rs`); thread `register_subtype` caller-span (retire CyclicSubtype rune); vigilia REMARKABLE bar | PLANNED |
| **243.3.1** | **PIVOT (2026-05-30) — pulled forward.** Mint `src/check/` home — `mv check.rs → check/mod.rs` (preserves 21k lines + import paths via re-export); carve the redesigned `CheckEnv` → `check/env.rs` as the home's first honest neighbor. **CheckEnv ownership redesign (Option B — borrow):** `CheckEnv<'a>` borrows the immutable inputs (`&'a TypeEnv`, `&'a binding_metadata`), owns only incremental check-state + the mid-pass-mutated `redef_allowed`. Deep-clone-into-CheckEnv becomes type-IMPOSSIBLE (the failure-engineering roof: the duplication situation is never constructible). Eliminates the `binding_metadata` deep-clone + the ⑬ `TypeEnv` double-clone (check.rs:2175 + freeze.rs:329). Born under vigilia REMARKABLE bar — the correct home FORCES the grimoire (flat files are wards-optional; namespaced homes are L1+L2=0 mandatory). Why pulled forward: the CheckEnv mirror is a live failure; FE says stop + eliminate now, and "through the roof" requires the grimoire requires the home. | PLANNED (next) |
| **243.6** | Grow the `src/check/` home (born at 243.3.1) — carve `CheckError` → `check/error.rs` under Pattern A (multi-span variants per CONFORMARE.md § Multi-span); fuse `check_program` walker chain (10× → 1×); fold `collect_hints` caching into the CheckError outer struct; vigilia REMARKABLE bar | PLANNED |
| **243.7…** | Remaining error types per rolling audit (RuntimeError, ParseStep [Stone 241.18a's NEW-2 closes here], LexError, LoadError, ResolveError, StdlibError, HashError, ExtractionError, …) — one home-carve + Pattern A retrofit per fat file | PENDING |
| **243.M** | Parser-API sister-walk — every parser/check API taking a bare slice gains `head_span: &Span`. Closes the ArityMismatch-style defensive class at the boundary. | PENDING |
| **243.N** | INSCRIPTION (fires last, after all spawned stones close) — class structurally eliminated | PENDING |

## The CheckEnv mirror — one thing stored twice (surfaced 2026-05-30, owner: 243.3.1)

Investigation during Stone 243.3 (user question: "is binding_metadata not exactly one thing?") found that **`CheckEnv` snapshots fields from `SymbolTable` rather than sharing it**, producing physical duplication of one logical thing:

- `SymbolTable.binding_metadata` (runtime.rs:1766, `HashMap`) — the OWNER. Built at freeze.
- `CheckEnv.binding_metadata` (check.rs:1972, `Arc<HashMap>`) — a DEEP-CLONE mirror set once at `from_symbols` (`Arc::new(sym.binding_metadata.clone())`), never mutated afterward (verified: 1 write, 0 mutations).

Root cause: `from_symbols(sym: &SymbolTable) -> Self` returns an OWNED `CheckEnv` with no lifetime — so it cannot hold a `&SymbolTable` borrow; it copies instead. CheckEnv mirrors THREE things this way: `binding_metadata` (deep clone), `redef_allowed` (copy), function schemes (derived). And the `types: Arc<TypeEnv>` field — the one correct shared-by-handle pattern — is undermined at the call site (check.rs:2175 `Arc::new(types.clone())` deep-clones the very thing built to be shared; this is finding ⑬).

**The correct tooling (Stone 243.6 design goal):** CheckEnv holds ONE handle to the SymbolTable (`Arc<SymbolTable>`, mirroring the existing `outer_symbols: Option<Arc<SymbolTable>>` at runtime.rs:1699) and READS `binding_metadata` / `redef_allowed` through it — no mirror fields. The mirroring is the smell; the clones (⑬ + the 2175 double-clone + the binding_metadata deep copy) are symptoms. One owner, CheckEnv as a view. This absorbs finding ⑬ (it stops being LEAVE-DISPUTED — 243.6 is its real owner) and the binding_metadata duplication into ONE coherent ownership fix.

Interim state (shipped in 243.3 R3-β + ⑦): `CheckEnv` fields are `pub(crate)` with a `set_redef_allowed` setter — the visibility surface is locked, which is orthogonal to and compatible with the 243.6 ownership redesign.

## Deferrals attested into this chain (Stone 243.3 R2 vigilia)

The R2 vigilia round on `types.rs`/`check.rs` (8 spells) surfaced architectural findings whose structurally-right owner is a later stone in THIS arc (within reach — child stones of the open arc, per the within-reach deferral doctrine):

| Finding | Owner | Why deferred here |
|---|---|---|
| `parse_defstruct` 350-line, 7 concerns (struere F3) | 243.5 | the types/ home carve is where deliberate decomposition belongs |
| `check_program` walker chain 10× traversal (temperare T-L1-4) | 243.6 | walker fusion is check/ decomposition work; sequi confirmed walkers independent (state-safe to fuse) |
| `collect_hints` double-compute (temperare T-L1-5) | 243.6 | hint-caching is a facet of the CheckError outer-struct design |
| `CheckError` not Pattern A — flat 34-variant enum, 5 multi-span variants, N-path `diagnostic()` (conformare F-CE-1/2/3) | 243.6 | CheckError Pattern A is what 243.6 IS — peer retrofit to TypeError (243.3) |

These carry attested-stone runes in the code citing their owner stone. DISTINCT from stalled-arc runes (the reversed R3.10/R3.11): these cite NEXT-in-chain stones of the currently-OPEN arc, not stalled/distant work.

⑬ `check_program`'s `Arc::new(types.clone())` (temperare T-L1-3) was initially triaged LEAVE-DISPUTED, but the CheckEnv-mirror investigation (above) gave it a real owner: it is the same shared-by-handle ownership defect, and Stone 243.6's CheckEnv redesign closes it. Reclassified from LEAVE-DISPUTED to DEFER → 243.6.

## Vigilatum — the homes-walk ledger (substrate maturation)

The namespaced-home vision (above) is executed via **selective lift-and-ward**, tracked by the `vigilatum` ward-provenance marker (`docs/VIGILATUM.md`, intueri-named 2026-05-30).

**Lift triggers** (`feedback_warded_means_annihilated`): (a) many-impls — one concept defined N times, or N definitions missing a structural invariant (the failure-domain signature; e.g. argspec's ~4 definitions, the span-less error class); (b) near-perfect / "done done". Otherwise a functional single tool stays flat (untrusted-by-design) until arc 109 wraps.

**The iron rule:** "warded" = failure domains FOUND AND ANNIHILATED (not "compiles," not "converged-checkbox"). The vigilatum stamp is EARNED by a live cast at L1+L2=0, never asserted from reputation (rust_deps proved it: "oldest code, surely clean" — the cast found 4 deferral-lies incl. a dead-arc citation + perf flaws). The stamp lives on the warded unit's module-doc; placement follows the ward (mod.rs for mod-rooted homes; the resident file for a lone warded resident under a flat root). NOT a central ledger (that would mirror file-truth).

**Warded ledger:**

| Home / resident | Anchor | Cast | Notes |
|---|---|---|---|
| `src/check/env.rs` | `22c89e04` | vigilia 8-spell | CheckEnv borrow redesign (Stone 243.3.1) |
| `src/rust_deps/` (mod.rs + custodia.rs) | `7b89053e` | vigilia 7-spell | lifted custodia; annihilated 13 findings; oldest wat-rs code |

**Remaining homes to walk** (each: live cast → annihilate-or-confirm → kill-confirm re-cast → stamp): argspec, function, remedy, comms. Drift-check any warded home via `git diff <anchor>..HEAD -- <path>`.

## Trap-doors

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | `CyclicSubtype` and similar fire at registration time, not a source position | per-variant judgment at the owning home-carve stone | Stone 243.5 threads the caller's span through `register_subtype` (retires the `spanless-by-domain` rune entirely — zero exceptions) |
| **T2** | Retrofit cascade is large (every error type touches every consumer) | per-type audit at each stone | per-error-type stones; substrate-as-teacher cascade per stone; FM 2-bis probe per retrofit (as 243.3 did) |
| **T3** | `From<E1> for E2` impls that drop span data | per-type review at each stone | update From impls; conformare catches them going forward (it confirmed TypeError's From impls preserve span at 243.3) |
| **T4** | Parser APIs taking bare slices are numerous; the sister-walk (243.M) could itself be large | rolling audit reveals the count | Stone 243.M handles; if scope creeps it spawns its own stone chain — NOT a new arc (opener-blocks) until 243 closes |
| **T5** | Multi-span variants (5 in CheckError) have no canonical primary span | conformare F-CE-2 flagged at 243.3 R2 | CONFORMARE.md § Multi-span: most-actionable location → outer `span`; secondaries → kind-variant fields with domain-descriptive names |

## What this arc DOES NOT do

- Does not retrofit `wat` source-level error RENDERING (how errors display at the CLI — separate concern)
- Does not change error semantics (no error-merging, no error-recovery; just location-discipline + homes)
- Does not address non-error types with possible span gaps (Bundles? AST nodes? — separate audit/arc if needed)

## Cross-references

- `scratch/FAILURE-ENGINEERING.md` — the principle this arc embodies
- `docs/ZERO-MUTEX.md` — the multi-layer skill precedent
- `docs/CONFORMARE.md` — Pattern A doctrine (Stone 243.1; rewritten at 243.4)
- `docs/arc/2026/05/243-conformare-error-shape/CONFORMARE-FIRST-CAST.md` — the spell's grimoire-earning verdict (chose Pattern A over trait)
- `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.18a.md` § Phase B — the vigilia rounds that surfaced this class
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 11 — deferral-language discipline (orchestrator-doctrine sibling)
- `datamancy.dev/exigere/SKILL.md` — sibling deferral-language spell (same authoring pattern)
- Memories: `feedback_correctness_makes_honesty`; `feedback_runes_illegal_when_solvable`; `feedback_dont_document_non_fixes`; `feedback_pre_existing_is_not_exemption`; `feedback_defers_within_reach_tolerable`; `feedback_let_need_reveal_through_work` — doctrines that landed during this arc

## Deferred until arc 243 lands

- arc 241 Stone 241.18b-g (def-family migration: src/def/ + defmacro/defstruct/defenum/defclause/defalias) — continue after conformare's class-elimination completes. Conformare informs their error-type designs.

## Status header (live)

- **Opened:** 2026-05-30 (trigger: Stone 241.18a vigilia surfaced the substrate-wide error-shape class)
- **Spell name verdict:** `conformare` (intueri cast 2026-05-30; runners-up `normare`, `redigere`, `respuere`)
- **Design evolution:** trait Conformare (original) → Pattern A outer-struct (CONFORMARE-FIRST-CAST) — structural enforcement beats convention enforcement
- **Failure-engineering frame:** catastrophic class; the structural fix + home-carving is the substrate maturation arc
- **Opener-blocks** (`feedback_spawn_block_winding`): arc 243 cannot close until ALL stones (243.4 … 243.N) close

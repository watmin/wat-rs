# BRIEF — 293: build surface-splice `[~@:Surface …]` in aggregate field vectors (the designed-but-unbuilt DRY reuse)

> **Executor: one sonnet LEAF.** Orchestrator drew this + the committed RED probe; weighs the kill forced-clean.
> Work ONLY in `/home/watmin/work/holon/wat-rs/`, NEVER worktrees. `pwd` first. `cargo nextest run` (NEVER `cargo
> test`). Commit NOTHING — leave the tree green for the orchestrator to weigh.

## The work (one paragraph)
Arc 293 designed **surface-splice** — a `defrecord`/`defstruct`/`defholon` field vector may reuse a surface's
ATTRIBUTES via `~@:Surface`, inlining them flat before the own fields (`AGGREGATE-MODEL.md` principle 4; `DESIGN.md:130`
*"spliceable into bodies for DRY `[~@:geo::Planar radius <- :f64]`"*). It was **never built**: the field-vector parser
has zero `~@` handling, and there is **zero `~@:` usage in the whole corpus** — it rotted unbuilt. Build it: teach the
aggregate field pipeline to expand each `~@:Surface` element into the surface's `Field` members, merged into the record's
field list. The reader already produces the node — **no reader change**.

## The exact semantics (pinned with the builder, 2026-07-04 — do not re-decide)
- **Form:** `[~@:A ~@:B  own <- :T …]` — zero or more `~@:Surface` splices, then own `name <- :T` fields, any order the
  user writes (but the canonical use is splices-first).
- **Expansion:** each `~@:Surface` → that surface's **`Field` members only** (the `name <- :Type` attributes). A surface
  may also carry **Method** members (it subsumes `defprotocol`) — **skip methods** (a record cannot hold a function;
  methods are `extend-surface`'s concern, not a field).
- **Merge = union, first-occurrence order.** Walk the vector left-to-right; splice A's fields (in the surface's declared
  order), then B's *new* fields, then own *new* fields. A field's position is its first occurrence.
- **Dedup by type identity (the constraint-engineering rule):** a field name appearing more than once (across splices, or
  splice+own) must carry an **identical type** → collapses to ONE field. A name repeated at a **conflicting type** is
  **unrepresentable → `MalformedDecl`** (builder: *"if A and B both install `foobar` and A says int, B says string, it
  does not compile"*). Same-name-same-type is NOT an error — it dedupes.
- **Scope:** aggregate field vectors ONLY (`defrecord`/`defstruct`/`defholon` — they share one field parser). Do **NOT**
  add splice to `defsurface :features` (surface-reuses-surface is a separate concern, out of scope).
- **Load order:** the spliced surface must be **declared before** the splicing record (matches the existing
  `Error`-surface load-order discipline, `wat/core.wat:1462`). **Forward-reference splices are OUT OF SCOPE** for this
  stone — if a splice can't resolve because the surface isn't registered yet, that's a clean `MalformedDecl`
  ("unknown surface in splice"), not a two-pass build.

## Read in order (the rooms — grounded 2026-07-04)
1. **`crates/wat-reader/src/parser.rs:353`** — `Token::UnquoteSplicing → (:wat::core::unquote-splicing E)`. This is the
   node `~@:Surface` produces: a `WatAST::List` with head keyword `:wat::core::unquote-splicing` and one arg (the
   surface keyword). **No change here** — this is what you match on.
2. **`src/types/defstruct.rs:297 parse_aggregate_fields`** — the ONE shared field parser (all three holders funnel here
   via `parse_argspec_triples`). Today it walks the vector expecting `name <- :Type` triples; a `(unquote-splicing …)`
   element trips `parse_argspec_triples` → "name must be a plain symbol". This is where the expanded field list must
   arrive clean.
3. **`src/types.rs:2355 parse_aggregate`** (+ `1926-1938 parse_type_decl` dispatch for `recordtype`/`defstruct`/holon) —
   the callers of `parse_aggregate_fields`. **THE CRUX (below):** these are *pure parse* functions with **no type
   registry** — so splice cannot be resolved here.
4. **`src/types/surface.rs`** — `SurfaceDef` / `SurfaceMember::Field { name, ty }`. A registered surface's attributes are
   its `Field` members (`struct_satisfies_surface:47` reads exactly these). This is what you inline.
5. **`src/check.rs`** — the type-registration pass (`collect_and_register_*` / `register_types`) where surfaces land in
   `env.types()` as `TypeDef::Surface`. This is where the registry IS available — the right layer for expansion.

## THE CRUX — where expansion happens (the ONE design decision; resolve it, then build)
`parse_aggregate_fields` is registry-free, but expanding `~@:Surface` needs the surface's `Field` members from
`env.types()`. So expansion **must move to the type-registration pass**, where surfaces are already registered.
**Recommended approach (A) — a form-rewrite before `parse_aggregate`:** in the registration pass, when a record decl is
processed, walk its field vector and replace each `(:wat::core::unquote-splicing :Surface)` element with the surface's
`Field` members rendered as `name <- :Type` triples (looked up in the registry-so-far), producing a plain field vector;
then the existing `parse_aggregate_fields` runs unchanged on the expanded vector. This keeps the parser registry-free and
isolates the merge/dedup logic in one rewrite. (Approach B — thread the registry into `parse_aggregate_fields` — is more
invasive; prefer A unless the pipeline makes it awkward.)

## STOP triggers (rejection criteria — surface the gap, do not hack)
- **STOP-REGISTRY:** if there is no clean layer where BOTH (the record decl form) AND (the registered surfaces) are
  available together, STOP and report the pipeline shape — do NOT smuggle a global/parse-time surface cache to force it.
- **STOP-FORWARD-REF:** if making the positive probe pass would require resolving a surface declared *after* the record
  (forward ref), STOP — that is explicitly out of scope; the probe uses declare-before-use.
- **STOP-METHOD-SPLICE:** if a spliced surface carries Method members and skipping them is not clean, STOP and surface —
  do not silently drop or error without a decision.

## The gate (EXPECTATIONS — fixed before the strike)
| what | command | expected |
|---|---|---|
| positive splice probe GREEN | `cargo nextest run --release --run-ignored all -E 'test(surface_splice_merges_and_constructs)'` | 1 passed |
| conflict rejected | `cargo nextest run --release --run-ignored all -E 'test(surface_splice_conflicting_field_types_rejected)'` | 1 passed |
| existing 293 surface suite GREEN | `cargo nextest run --release -E 'test(probe_arc293)'` | all pass |
| whole gate, floor 0 | `cargo nextest run --release` | `0 failed` |

The committed RED probe is `tests/types/probe_arc293_surface_splice.{rs,wat}` (+ `.wat.bad`). Both tests are
`#[ignore]`'d (IGNORE-LEDGER 293-surface-splice). **Un-ignore BOTH as the FINAL step** and they must go green — that is
the kill. Runtime estimate: 30–50 min. Trap-door: the merge order / positional constructor — the probe constructs
`(:probe::Metric "market-eval" "u-123" 456 "requests" 7)` over the merged order `namespace uuid time-ns name value`; if
your merge order differs, the probe's construction fails — match first-occurrence order.

## Blast radius (bounded)
`src/check.rs` (the expansion rewrite in the registration pass) + possibly `src/types/defstruct.rs` (if the merge/dedup
lands beside the field parser) + the surface-member read. **No reader change. No new AST node. No new type.** The RED
probe goes green; nothing else changes shape.

## Pairs
`AGGREGATE-MODEL.md` principle 4 + §"the user forms" (line 99, the splice form) · `DESIGN.md:130` (spliceable-for-DRY) ·
`BRIEF-293-features-clause.md` STOP-OTHER-FORMS (defrecord field vectors are a different grammar than surface `:features`)
· the committed probe.

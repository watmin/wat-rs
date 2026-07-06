# BRIEF — 293 S1: `defsurface` synthesizes its wire-protocol (`Op`/`Reply`) from pure method members

> **Executor: one sonnet SHADOWDANCER.** A **Rust** strike (type synthesis in the surface-registration path). The
> orchestrator scouted the lair, cast intueri on the names, decided the gate by four-questions, and VERIFIED the
> reference target (`cargo wat` → "S1 reference target type-checks"). Work ONLY in `/home/watmin/work/holon/wat-rs/`
> (`pwd` first; `.claude/worktrees/` illegal). `cargo build` to check; `cargo nextest run --release` (NEVER `cargo
> test`); `./target/release/cargo-wat <f>` to dogfood. **Commit NOTHING.** Full design:
> `DESIGN-293-services-as-surfaces.md` (§ S1 — DRAWN) — read it first.

## The work (one paragraph)

When a `defsurface` has **method** members whose request/response sigs are **pure** (EDN-crossable), the surface must
**synthesize + register two `EnumDef`s under its own namespace** — `<Surface>::Op` and `<Surface>::Reply` — one variant
per method. This is the generative core of "services-as-surfaces": these shared enums become the wire-protocol every
`:satisfies` service speaks and every `:calls` client dials (later stones), so a client can dial `Address'<Store::Op,
Store::Reply>` blind. The request/response **records are user-declared** (the method members reference them); S1 emits
ONLY the two enums. Impure-sig surfaces synthesize nothing (they can't be dialed — 293.W's existing wall; not our branch).

## The reference target (VERIFIED — the exact shape your codegen must produce)

`scratchpad/s1-reference-target.wat` type-checks today with the enums **hand-written**. Your job: make a surface with
these method members produce those same `::Op`/`::Reply` enums **by synthesis** (delete the hand-written enums from a
copy of the probe; the surface alone must yield them).

```clojure
;; user-declared (records + surface); PURE sigs → serviceable:
(:wat::core::defrecord :probe::Kv::PutRequest  [key <- :wat::core::String  val <- :wat::core::String])
(:wat::core::defrecord :probe::Kv::PutResponse [ok  <- :wat::core::bool])   ;; (+ Get{Request,Response})
(:wat::core::defsurface :probe::Kv :holder :wat::core::Struct
  :features [(put [self <- :probe::Kv  req <- :probe::Kv::PutRequest] -> :probe::Kv::PutResponse)
             (get [self <- :probe::Kv  req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse)])
;; S1 SYNTHESIZES (the target):
(:wat::core::defenum :probe::Kv::Op :wat::enum::Pure  :Put [req <- :probe::Kv::PutRequest]  :Get [req <- :probe::Kv::GetRequest])
(:wat::core::defenum :probe::Kv::Reply :wat::enum::Pure  :Put [resp <- :probe::Kv::PutResponse]  :Get [resp <- :probe::Kv::GetResponse])
```

## The map (per method member → two variants)

For each `SurfaceMember::Method { name, args, ret }` of a surface `:S`:
- **request type** = the arg AFTER `self` (`args[1]` — the single request record the method takes). **response type** = `ret`.
- **`:S::Op` variant** = `<PascalCase(name)> [req <- <request-type>]`.
- **`:S::Reply` variant** = `<PascalCase(name)> [resp <- <response-type>]`.
- The two enums are `:wat::enum::Pure` (their payloads must cross — that's the purity gate).
- **PascalCase(name):** `put` → `Put`, `scan-index` → `ScanIndex`. Use the EXISTING conversion (see
  `docs/PASCAL-KEBAB-CONVERSION.md` + its helper) — do NOT hand-roll.

## The purity gate (derived, not a marker)

Synthesize the two enums **iff EVERY method's request AND response type `is_pure_type`** (`src/check.rs:13580`,
`is_pure_type(ty, types) -> bool`). If any method's sigs are impure (holds a live `Peer'`/`Connection`), the surface is
**in-thread-only** — synthesize NOTHING for it (silent, correct; the surface still registers + works for extend-type).
Do NOT emit an impure enum (293.W would reject it anyway).

## Read the rooms, in order
1. **`DESIGN-293-services-as-surfaces.md` § S1 — DRAWN** — the decision, the names, the map, the reference target.
2. **`src/types/surface.rs`** — `parse_defsurface` (returns `TypeDef::Surface(SurfaceDef{ members })`, ~line 467); how
   `SurfaceMember::Method { name, args, ret }` is built (~line 286); the width-subtyping satisfaction (~line 64).
3. **`src/types.rs`** — `EnumDef { name, variants: Vec<EnumVariant> }` (~242), `EnumVariant` (~252),
   `SurfaceMember::Method` (~310), `TypeEnv::register` / `register_with_span` / `register_stdlib` (~444/456/471),
   and the SYNTHESIS PRECEDENT: `register_type_predicates` synthesizes `:wat::holon::is-Record?` after a record
   registers (~1549) — mirror that shape (register a surface, then synthesize + register its `Op`/`Reply`).
4. **`src/check.rs:13580`** — `is_pure_type` (the gate).
5. **`src/freeze.rs`** (~1300–1342, the `:wat::core::defsurface` handling in the startup pipeline) — where surface
   registration is driven; the synthesis hook goes at the same point a surface's `TypeDef` is registered.

## Where it lands (bounded)
The surface-registration path in Rust (`src/types/surface.rs` and/or the register site in `src/types.rs`/`src/freeze.rs`)
— after a `TypeDef::Surface` with pure method members registers, synthesize + register its `Op`/`Reply` `EnumDef`s
(matching the surface's own registration privilege: stdlib surface → `register_stdlib`, user surface → `register`). No
wat-source change. No change to `defenum`/`defrecord`/`defservice`/`extend-type`. The synthesized enums must be
IDENTICAL in structure to what `defenum` would have registered (so downstream code can't tell they were synthesized).

## STOP triggers (halt + report, don't hack)
1. **STOP-IMPURE:** if a surface has impure method sigs, synthesize NOTHING for it — do NOT force an impure enum, do NOT
   error the surface's own registration. If you can't tell purity at that point in the pipeline, STOP and report (the
   ordering — records must be registered before the surface's purity can be judged — may need surfacing).
2. **STOP-COLLISION:** if `<Surface>::Op`/`::Reply` already exists (a user hand-declared one), STOP and report — do NOT
   silently overwrite; that's a real design question (does the user get to override the synthesized protocol?).
3. **STOP-NOCP:** do NOT change `defenum`/`defrecord` structure, the wat sources, or `defservice`/`extend-type`. S1 is
   ONLY surface→Op/Reply synthesis.

## The gate (EXPECTATIONS — I re-run these myself)
| what | command | expected |
|---|---|---|
| a pure-sig surface synthesizes its Op/Reply | `cargo wat` on a copy of `scratchpad/s1-reference-target.wat` with the two hand-written `defenum`s DELETED | still prints "S1 reference target type-checks" (the enums now exist by synthesis) |
| an impure-sig surface synthesizes nothing (no regression) | a probe: a surface with a method returning an impure type (e.g. a `Peer'`) still registers + no Op/Reply | registers clean; no synthesized enums; no error |
| whole floor | `cargo nextest run --release` | verbatim Summary; `0 failed` modulo the known `no_inlined_wat_in_tests` reminder |

Runtime ~40–60 min (a Rust change forces a rebuild + the full suite).

## Final report (structured): files changed · where the synthesis hooks (the register site) · how you read the
method members + mapped them to variants · the PascalCase conversion used · the purity gate + how impure surfaces are
skipped · the collision handling · the verbatim gate results (the deleted-enums probe + the impure probe + the
whole-floor Summary) · STOP triggers hit or "none" · anything that surprised you (ordering, registration privilege, etc.).

## Prior comparable: the `register_type_predicates` synthesis (types.rs:1549) is the closest precedent — a decl
registers, then a derived type/predicate is synthesized + registered alongside it.

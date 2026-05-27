# DESIGN — Stone S-A1 — `assignable` choke point (subtyping at the arg boundary)

**Arc:** 237, records-first-class thread (see `DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md`
+ `DESIGN-STONE-S-A-records-hierarchy.md`).
**Status:** READY (sub-DESIGN). Second substrate stone of the records thread.
**Builds on:** S-A SHIPPED (`d1e9cbe9`) — `is_subtype` + `typesub` registry + seeded roots.

## Why this stone

S-A minted the is-a hierarchy mechanism (`is_subtype`, the `typesub` registry,
the two roots) but wired it into NOTHING — `subtype?` is callable, but the type
*checker* does not consult it. So a function `[v <- :wat::Record]` still rejects a
subtype-typed value at the argument boundary. S-A1 closes that: it teaches the
checker's argument-acceptance to consult the hierarchy (Liskov substitution —
a subtype is accepted where its supertype is wanted).

## Why S-A1 is testable now ONLY at the Rust/check layer (honest scope note)

The wat-surface payoff ("a function wanting `:wat::Record` accepts a holonic
record") has **no subtype-typed value to act on yet**: the only edge at the wat
surface is the seeded `:wat::holon::Record typesub :wat::Record`, and nothing
constructs a `:wat::holon::Record` value until S-C mints `:wat::holon::Record::def`
(defrecord today returns the base `:wat::Record`). So S-A1 proves the *acceptance
logic* at the check layer (hand-registered edge + struct values via the test
harness); the end-to-end wat proof rides in on S-B (records register TypeDefs +
derive edges) and S-C (the holon flavor). This is the honest stepping-stone split:
**S-A1 = the acceptance logic; S-B/S-C = the values that exercise it.**

## What this stone delivers

1. **`fn assignable(actual, expected, subst, types) -> bool`** in `src/check.rs` —
   the single arg-acceptance choke point. Directional-subtype-first (mutation-free),
   then ordinary `unify`:

   ```rust
   fn assignable(
       actual: &TypeExpr, expected: &TypeExpr,
       subst: &mut Subst, types: &TypeEnv,
   ) -> bool {
       // Directional subtype: actual <: expected, both concrete nominal paths.
       // Checked FIRST and mutation-free (only short-circuits on a real edge).
       let a = walk(actual, subst);
       let e = walk(expected, subst);
       if let (TypeExpr::Path(ap), TypeExpr::Path(ep)) = (&a, &e) {
           if ap != ep && crate::types::is_subtype(ap, ep, types) {
               return true;
           }
       }
       // Everything else (equal paths, Var binding, structural, typeunion) →
       // ordinary unification, behavior UNCHANGED.
       unify(actual, expected, subst, types).is_ok()
   }
   ```

2. **Route the 4 function-call arg-boundary sites** (the `[v <- :Type]` boundary)
   from `unify(&arg_ty, &expected, …).is_err()` to `!assignable(&arg_ty, &expected, …)`:
   - `infer_list` general-call sites — **7025, 7079, 7213** (THE path for `(:some::fn arg)`).
   - defclause clause-arg site — **6867** (clause dispatch arg-match; a defclause
     param may be `:wat::Record`).

   (Line numbers as of HEAD `d1e9cbe9`; verify before editing — check.rs drifts.)

## Baseline-preservation argument (the load-bearing invariant)

`assignable` differs from `unify` ONLY by accepting one additional case: `actual`
and `expected` are **distinct concrete Paths with a registered `typesub` edge**
(`actual <: expected`). For EVERY other pair — equal paths, Vars, structural,
typeunion, OR two concrete paths with NO edge — `is_subtype` returns false and
control falls to `unify` unchanged. Since the ONLY edges currently registered are
the two seeded roots (`:wat::holon::Record <: :wat::Record`) and nothing yet has
those as a static arg type, **no existing test can observe a behavior change**.
Lib baseline 827/0 holds by construction.

## The walk/reduce detail (the one judgment call)

`is_subtype` takes `&str` FQDNs; `unify` takes `TypeExpr`. `assignable` must peel
each side to a concrete `Path` to call `is_subtype`. Use the checker's existing
`walk` (chase a Var through `subst` to its binding) — mirror how `unify`'s own
arms reduce before matching. A Var-bound-to-a-concrete-path is then seen as a
Path. Path strings carry leading colons (`:wat::Record`); the `typesub` registry
keys also carry leading colons (S-A's `register_subtype` stored them verbatim) —
they match directly, no stripping. Confirm `walk` is the right peel (vs `reduce`
which also expands aliases — alias-expansion is harmless here but `walk` is the
minimal correct choice).

## Out of scope (REJECTED — not deferral)

- **The other ~46 `unify(actual, expected)` sites** (return-position, let/branch,
  collection-element, leaf-invariant, spawn, arc-146 dispatch, try/option). S-A1
  routes ONLY the function-call arg boundary — the Liskov boundary that delivers
  the records capability. Routing the rest is a SEPARATE concern: a "make the ~50
  open-coded sites consult one shared `assignable`" refactor (a future stone /
  arc, valuable but not required for records). Affirmative cut.
- **The wat-surface end-to-end proof** — rides in on S-B/S-C (no subtype-typed
  value exists at the wat surface until then).
- **arc-146 dispatch sites (14049/14099)** — that entity is being retired in 237.7;
  do not touch.
- **A user-facing `typesub`/`derive` verb** — still minimal-form; not needed.

## FM 2-bis probe (NEW — committed before the BRIEF)

`tests/probe_arc237_sA1_assignable.rs`. Rust/check-layer (mirror S-A's Rust-API
contracts). Pre-stone: fails (the acceptance does not happen — a subtype-typed arg
is rejected). Post-stone: all PASS. Contracts (drive via `startup_from_source`
where possible; else `check_program` on a hand-built AST with a hand-registered
edge):

1. **subtype accepted at arg boundary** — register `:my::Sub typesub :my::Super`
   (both structs); `(defn :needs-super [v <- :my::Super] ...)`; call
   `(needs-super (:my::Sub/new ...))` → type-checks clean (Sub is-a Super).
2. **supertype rejected where subtype wanted** — `(defn :needs-sub [v <- :my::Sub] ...)`;
   call `(needs-sub (:my::Super/new ...))` → type ERROR (Super is-not-a Sub; directional).
3. **transitive** — `:my::A typesub :my::B`, `:my::B typesub :my::C`; fn `[v <- :my::C]`
   accepts an `:my::A` value.
4. **no-edge unchanged** — fn `[v <- :my::X]` rejects an unrelated `:my::Y` value
   (no edge → `assignable` == `unify`; regression guard).
5. **exact-match unchanged** — fn `[v <- :my::Super]` accepts a `:my::Super` value
   (equal-path path through unify, untouched).
6. **defclause clause-arg accepts subtype** — a defclause clause with a `:my::Super`
   param matches when called with a `:my::Sub` value (site 6867 wired).

(How edges get registered for a wat-surface test: if no wat-surface `typesub` verb
exists, the probe registers the edge by driving the seeded-roots path OR builds the
TypeEnv + AST at the Rust layer and calls `check_program` directly — mirror
S-A probe's Rust-API style. Pick whichever isolates the acceptance cleanly.)

Plus baseline: `cargo test --release --lib` ≥ 827/0.

## Proven-moves template (mirror — arcs 237.5 / S-A)

- The change is in `src/check.rs` ONLY (mint `assignable` + reroute 4 sites). No
  `runtime.rs`, no `types.rs` (is_subtype already shipped), no new `Value`/`TypeError`
  variant → **0 cascade files**.
- Trap-door (the one that matters): **`assignable` must check subtype FIRST,
  mutation-free, then unify** — NOT a directional arm inside `unify` (that sprays
  subtyping into return-position + symmetric uses = the symmetric-leak class reborn,
  the exact thing S-A's design rejected). Keep it a wrapper at the arg sites.
- Do NOT route the non-arg-boundary sites (scope creep + baseline risk).
- SCORE shape = SCORE-STONE-237.5 / SCORE-STONE-S-A.

## Calibration

Tiny surface: one ~12-line helper + 4 one-line call-site reroutes, all in check.rs.
Lighter than S-A (no new registry, no new primitive). **Target band: 25–50 min
Mode A; 75 STOP-3; 100 STOP-4. Cascade: 1 round (check.rs only), 0 forced files.**
Mirror SCORE-STONE-S-A shape; cite it + 237.5 in the BRIEF.

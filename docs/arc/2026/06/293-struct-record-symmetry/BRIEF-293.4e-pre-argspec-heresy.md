# BRIEF — 293.4e-pre: ANNIHILATE the ArgSpec heresy (`Clause` re-rolls ArgSpec) + fix the surface arity

**The work, in one paragraph.** There is exactly ONE canonical typed-binder-list in this substrate:
`crate::argspec::ArgSpec { fixed_params: Vec<(Identifier, TypeExpr)>, rest_param: Option<(Identifier, TypeExpr)> }`.
It keeps getting re-rolled — this is the Nth time (builder: *"like the 6th or 7th time argspec got a duplicate"*).
The live duplicate: `Clause` (`src/value/value.rs:407,410`) carries its OWN `args: Vec<(String, TypeExpr)>` +
`rest_param: Option<(String, TypeExpr)>` — ArgSpec's exact shape, re-rolled. Worse, it is built by the **parse-then-
unroll** anti-pattern: `parse_defclause_form` (`runtime.rs:~5708`) calls the canonical `parse_argspec_triples` to get a
clean `ArgSpec`, then **immediately destructures it** into the bespoke `Vec<(String,TypeExpr)>` (Identifier→String via
`env_key`). The same at the `extend-type` impl build site (`runtime.rs:~6292`). **Annihilate it: `Clause` holds an
`ArgSpec` directly** (stop unrolling). `Clause` is shared by **`defclause` AND `extend-type`**, so this unifies both
onto the one canonical binder. Then fix the surface-method **arity off-by-one** the heresy-hunt surfaced.

## The one contract decision (pinned)
`Clause { args: ArgSpec, return_type, guard, ensure_fn, body }` — the `args: Vec<(String,TypeExpr)>` and
`rest_param: Option<(String,TypeExpr)>` fields COLLAPSE into a single `args: ArgSpec` (which already carries
`fixed_params` + `rest_param`). Consumers read `clause.args.fixed_params` / `clause.args.rest_param` (the binder
names are `Identifier`; `env_key(&id)` where a `String` name is needed for runtime binding — the same conversion the
unroll did, now at the use site, not baked into the storage).

## What is NOT in scope (do NOT touch — they are genuinely distinct concepts)
- **`Scheme.params` / `param_types: Vec<TypeExpr>`** (`check.rs:81`, `value/environment.rs:50`, `function/parse.rs:56`)
  — the TYPE signature (param TYPES, no binder names). A scheme types a call; it does not bind a body. NOT an ArgSpec.
- **`AggregateDef.fields: Vec<(String, TypeExpr)>`** (`types.rs:188`) — record/struct DATA fields, not function binders.
- **`ProtocolMethodSig.arg_types`** — dies with `defprotocol` in 293.4e; do NOT migrate it here.

## Read in order (the rooms — grounded 2026-06-28)
1. **`src/value/value.rs:405–411` (`struct Clause`)** — replace `args: Vec<(String,TypeExpr)>` + `rest_param:
   Option<(String,TypeExpr)>` with `args: ArgSpec`. (`ArgSpec` already derives the traits it needs — confirm
   `Clone`/`Debug`/`PartialEq`/`Eq`; 293.4a added Eq.)
2. **`src/runtime.rs:~5708–5720` (`parse_defclause_form`)** — STOP the unroll: keep the `spec: ArgSpec` from
   `parse_argspec_triples` and store it as `Clause.args` directly. Delete the `.into_iter().map(env_key…)` unroll.
3. **`src/runtime.rs:~6292` (extend-type impl → Clause)** — same: build `Clause.args` from the parsed ArgSpec (the
   extend impls are bare `[self x]` binders — `parse_argspec_triples` with whatever options the current parse uses;
   if extend impls have NO type annotations, the ArgSpec's types are placeholders/inferred — preserve today's behavior,
   just store as ArgSpec not the unrolled vec).
4. **The 22 consumer sites** (`grep -rn 'clause\.args\|clause\.rest_param\|\.clauses' src/` →
   `src/check/env.rs`, `src/check.rs`, `src/runtime.rs`, `src/value/observe.rs`): each `clause.args` (was
   `&[(String,TypeExpr)]`) → `clause.args.fixed_params` (`&[(Identifier,TypeExpr)]`) with `env_key(&id)` where the
   `String` is needed; `clause.rest_param` → `clause.args.rest_param`. Let the compiler waterfall every site.
5. **`src/check.rs:5985` (the surface-method arity off-by-one)** — `expected_arity = 1 + extra_param_types.len()` where
   `extra_param_types = args.fixed_params` (which INCLUDES `self` as param 0). Self is DOUBLE-COUNTED (the `1 +` is the
   receiver AND `fixed_params[0]` is self). FIX: the expected arity is `args.fixed_params.len()` (self IS the receiver,
   counted once) — OR drop self from `extra_param_types` (`fixed_params[1..]`) and keep `1 +`. Pick whichever reads
   cleanest; the Field-member arm (`extra = vec![]`, `expected = 1`) must stay correct (a field accessor is 1 arg).

## The gates (committed)
- `tests/types/probe_arc293_4e_pre_surface_method_parity.{rs,wat}` (`#[ignore]`'d RED) — a surface method `(make
  [self x] …)` with an arg beyond self. **UN-IGNORE it; it must go GREEN** once the arity is fixed (`(:t::probe)`→42).
- Existing defclause + extend-type tests must STAY green (the Clause→ArgSpec change is behavior-preserving — the
  binder names + types are identical, just stored canonically).

## STOP triggers (halt + surface)
- **STOP-1 (extend-impl ArgSpec has no types):** if the extend-impl bare binders `[self x]` produce an ArgSpec whose
  types are absent/placeholder and a consumer NEEDS the real types — STOP and report (the types live on the surface/
  protocol method decl; reconciling them is real work, not this brief's unroll-removal).
- **STOP-2 (the generic case):** the 293.4e-pre probe also documents a GENERIC surface method (`make<T>`) as
  `unknown callee`. If the Clause→ArgSpec + arity fix does NOT also resolve the generic dispatch, that is a SEPARATE
  remaining gap — note it; do not chase it here unless it falls out for free.

## EXPECTATIONS
| # | what | command | expected |
|---|---|---|---|
| 1 | the heresy is gone | `grep -n 'args: Vec<(String' src/value/value.rs` | no Clause hit (only fields/distinct) |
| 2 | the 293.4e-pre probe GREEN | `cargo nextest run --release -E 'test(surface_method_with_args_beyond_self)'` (un-ignore) | PASS (42) |
| 3 | defclause + extend-type un-regressed | `cargo nextest run --release -E 'binary(function)'` + any defclause/extend tests | green |
| 4 | whole workspace green | `cargo nextest run --release` | floor 0 |

## You are a LEAF
Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn subagents. Do NOT
commit. Build incrementally; let the exhaustive cascade waterfall you. Read every diff. Self-verify the EXPECTATIONS.
STOP + report if a STOP fires or the work exceeds the brief.

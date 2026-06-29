# BRIEF — 293 inheritance annihilation: `AggregateDef.parent` is the vestige; the `Holder` enum is the truth

**The work, in one paragraph.** Inheritance is annihilated (`AGGREGATE-MODEL.md` §4: a type is `holder + own fields`,
flat; there is no `:parent` — the only thing a parent slot may hold is a holder-root, and a holder-root maps 1:1 to
the `Holder` enum). So `AggregateDef.parent: String` is a stringly-typed shadow of `holder: Holder` — **delete the
field.** Subtype edges derive from `holder` via a new `Holder::root_keyword()`. A `recordtype`/`aggregatetype` whose
parent is NOT a holder-root (i.e. user-type inheritance like `:my::Special :my::Circle`) is **rejected at parse**.
The inherited-field machinery (it only existed to flatten an inherited base) dies. Two test-cases that exercised
nominal record inheritance are deleted; the rest of the arc-237 subtype suite (holder-membership: `:Circle <:
:wat::Record`, `holon <: core`) is UNTOUCHED — those are the model's kept assignability boundary, not inheritance.

## THE ONE CONTRACT DECISION (pinned)
`AggregateDef` has **no `parent` field**. The `Holder` enum is the sole categorical position. The subtype edge a
declared aggregate registers is **derived from its holder** — `Struct → :wat::core::Struct`, `Record → :wat::Record`,
`HolonRecord → :wat::holon::Record` — via `Holder::root_keyword()`. A non-holder-root in the `:Parent` surface slot is
a parse error. (The surface `:Parent` arg STAYS — it selects the holder; cleaning the surface is decl-b, out of scope.)

## Read in order (the rooms — every one grounded this session)
1. **`src/types.rs:151` `pub struct AggregateDef` / `:158 pub parent: String`** — DELETE the `parent` field. The
   compiler then names every construction + read site (the cascade is your checklist).
2. **`src/types.rs:130` `pub enum Holder` / `:136 impl Holder`** — ADD `pub fn root_keyword(&self) -> &'static str`:
   `Struct => ":wat::core::Struct"`, `Record => ":wat::Record"`, `HolonRecord => ":wat::holon::Record"`. (This is the
   forward map; `root_holder_of` at `:2124` is the reverse. Item-5 will later centralize both — here you only need
   `root_keyword`.)
3. **`src/types.rs:2124 root_holder_of`** — its `_ => Holder::Record` arm is the inheritance leak (any non-root parent
   silently becomes a Record). Make the parse path **reject** a parent that is not one of the three holder-roots
   (`:wat::core::Struct` / `:wat::Record` / `:wat::holon::Record`) — see room 4. Keep `root_holder_of` total over the
   three roots.
4. **`src/types.rs:~2160 parse_aggregate`** (args[1] = parent keyword; `parse_structtype:2148` injects
   `:wat::core::Struct`; `parse_recordtype` passes the user `:Parent`) — after reading args[1], if it is NOT a holder-
   root, return a `TypeError::MalformedDecl { head, reason }` ("parent '<x>' is not a holder-root; inheritance is
   unsupported — reuse a shape via surface-splice `[~@:Surface …]`"). Derive `holder = root_holder_of(parent)`; do NOT
   store parent. The new RED probe `probe_arc293_reject_user_parent` gates this (`:my::Child :my::Base` → rejected).
5. **`src/types.rs:457-484 register_with_span`** — the edge is currently wired from the stored `agg.parent`. Rewrite
   to derive from holder: `let root = agg.holder.root_keyword();` then register `:Name <: root` via `register_subtype`
   (Struct→`:wat::core::Struct`, Record→`:wat::Record`, HolonRecord→`:wat::holon::Record`). **WEIGH the subtype-edge
   SET** (STOP-EDGE below) — the parsed-type edge set must be unchanged.
6. **`src/runtime.rs:1040 ROOT_PARENTS` + `:1081-1091` inherited_fields/inherited_count + `:1110` all_fields chain +
   `:1168 abs_idx` + `:1491/:1507/:1512 collect_all_record_fields`** — with no non-root parents possible, `inherited`
   is always `[]`. DELETE `collect_all_record_fields`, the `ROOT_PARENTS` const, `inherited_fields`/`inherited_count`;
   `abs_idx` becomes `own_idx`; the ctor-fallback `all_fields` becomes just `agg.fields`. Behaviour-preserving for every
   surviving type (all root at a holder-root → 0 inherited). Update the doc-comment at `:1025-1032` (drop the
   `program::Env`/inherited-fields language).
7. **`src/closure_extract.rs:2389`** — `WatAST::Keyword(a.parent.clone(), …)` → `a.holder.root_keyword().to_string()`.
   (The reconstructed `(recordtype :name <root>)` form is unchanged in shape.)
8. **The ~20 `AggregateDef { … parent: "…" }` construction literals** (the builtins in `types.rs`, `types/defstruct.rs:386`,
   `capability/registry.rs:252`, `edn_shim.rs:2751`) — delete the `parent:` field from each. The compiler enumerates them.
9. **Fixtures** — DELETE the two nominal-inheritance test-cases, KEEP everything else:
   - `tests/types/probe_arc237_sA1_assignable_probe05.wat` (the `:my::Special :my::Circle` extender) + its driver fn
     `probe_05_holon_flavor_transitive` in `tests/types/probe_arc237_sA1_assignable.rs`. (probes 01/02/03/04/06 STAY —
     holder-membership, still valid.)
   - The `c02` extends-`program::Env` case: in `tests/program/probe_arc258_program_env_record.wat` remove the
     `:user::MyEnv` recordtype + its `:probe::c02-compute`; in the `.rs` delete `fn c02_user_extends_program_env`.
     Keep `c01` (Env as a plain record).
   - Verify `tests/types/probe_arc237_sB1_recordtype.rs::probe_06_unknown_parent_rejected` STILL passes (the
     `:my::DoesNotExist` parent is now rejected as not-a-holder-root rather than not-known — still `is_err()`).
10. **Doc/comment freshening** (small, do them): `src/types.rs:143-149` (the `parent: String` doc block →
    holder-is-the-position), `src/stdlib.rs:105` and `wat/program.wat:1` (drop "extensible recordtype base" —
    `program::Env` is a flat record now).

## Implementation sketch (the path; you fill it)
- Delete `parent` field → `cargo build` → walk the errors: each construction literal loses its `parent:` line; the 3
  reads (types guard, runtime machinery, closure_extract) are handled per rooms 5/6/7.
- Add `Holder::root_keyword()`; route the edge-derivation (room 5) and closure re-emit (room 7) through it.
- Add the parse-time holder-root guard (room 4); the new RED probe goes GREEN.
- Delete the inherited machinery (room 6); delete the 2 fixtures (room 9).

## Gate (the EXPECTATIONS — fixed before the strike)
| what | command | expected |
|---|---|---|
| reject-user-parent probe GREEN | `cargo nextest run --release -p wat recordtype_with_user_parent_is_rejected` | 1 passed |
| sB1 negative still rejects | `cargo nextest run --release -p wat probe_06_unknown_parent_rejected` | 1 passed |
| arc237 holder-membership intact | `cargo nextest run --release -p wat probe_arc237_sA1_assignable` | all pass (probe05 fn deleted) |
| whole gate, floor 0 | `cargo nextest run --release` | `0 failed` (skips unchanged save the deleted fns + new probe) |
| parent field gone | `grep -rn "\.parent\b" src/ \| grep -i agg` | no aggregate `.parent` reads remain |

Runtime estimate: 20–35 min. Trap-door: the subtype-edge SET parity (STOP-EDGE).

## STOP triggers (rejection criteria — surface, do not improvise)
- **STOP-EDGE:** if deriving the subtype edge from `holder` changes the registered edge SET for any *parsed* type
  (e.g. a struct that had no edge under `parent: :wat::core::Value` now gains a `:Foo <: :wat::core::Struct` edge, or
  vice-versa) and a test flips — STOP and report the exact edge delta; do not paper it.
- **STOP-BUILTIN:** the ~20 Rust-direct builtin registrations go through `register_builtin` (bypasses
  `register_with_span`), so deleting their `parent:` field must NOT change their behaviour — if any builtin relied on a
  stored parent for an edge, surface it.
- **STOP-HOLON-CTOR:** the ctor-fallback at `runtime.rs:1093-1151` uses `:wat::Record::of` for holon records too (a
  separate latent bug, decl-b.1) — do NOT try to fix it here; leave that branch's `:wat::Record::of` as-is. Your only
  change to the fallback is dropping the inherited-field threading (`all_fields` → `agg.fields`).

## Blast radius (bounded)
`src/types.rs` (field + Holder method + parse guard + register edge + builtins), `src/runtime.rs` (machinery deletion),
`src/closure_extract.rs:2389`, `src/types/defstruct.rs`, `src/capability/registry.rs`, `src/edn_shim.rs` (construction
literals), 2 test fixtures + their `.rs`, 3 doc/comment touch-ups. No new types beyond `Holder::root_keyword`. No `.wat`
surface change.

## You are a LEAF
Do NOT spawn subagents. Work only in `/home/watmin/work/holon/wat-rs/`. Verify `pwd` first; any path containing
`.claude/worktrees/` is illegal — re-cd to the anchor and use `git -C /home/watmin/work/holon/wat-rs` for git. If the
strike is larger than this brief implies, STOP and report — do not improvise past the rooms above.

## Pairs
`AGGREGATE-MODEL.md` §4 (no inheritance) · `AGGREGATE-AUDIT.md` (holder = passing policy) ·
`tests/types/probe_arc293_reject_user_parent.{rs,wat}` (the RED gate) · `feedback_option_carrying_semantics_screams_enum`.

# BRIEF — 293.4e-pre.iii: `extend-type`-for-surface impl inherits the surface method's declared sig

**The work, in one paragraph.** When `extend-type :T :Surface (m [bare-binders] body)` registers the impl as a
check-side `TypeScheme` (`src/check.rs:~8954`, the 293.4c surface branch), it builds the scheme from the **bare impl
clause's** types — which are `nil` (the impl has no annotations) — and hardcodes **`type_params: vec![]`**. For a
MONOMORPHIC constant-body impl this is harmless (proven green). But a GENERIC impl whose body uses the surface method's
type-params (or a typed body that uses `self`) fails: the type-params are unbound, `self`/args are `nil`/`:()`, and the
return mismatches. This is the `:wat::spawn::Locus` `launch<S,R,St,Sh,Lu>` shape → it BLOCKS the `defprotocol`
annihilation (293.4e). **Fix: build the scheme from the SURFACE METHOD MEMBER's declared sig** — not the bare impl.

## The one contract decision (pinned)
The check-side scheme for `:<T>/<method>` (a surface extend-impl) is derived from the surface's `SurfaceMember::Method`
for `<method>`: `type_params` = the member's `type_params`; `params[0]` = the EXTENDING type `<T>` (self → the concrete
type, NOT the surface); `params[1..]` = the member's `args.fixed_params[1..]` types; `ret` = the member's `ret`;
`rest_param_type` = the member's rest. The impl body is then type-checked against THIS scheme (so `self` is `<T>`, the
args are the surface's declared types, the type-params are in scope). The bare impl clause supplies only the body + the
binder NAMES — never the types.

## Read in order (the rooms — grounded 2026-06-28)
1. **`src/check.rs:~8951-8970` (the 293.4c surface-extend branch in `collect_splice_defs_ctx`)** — TODAY:
   `params: clause.args.fixed_params...`, `ret: clause.return_type`, `type_params: vec![]` (all from the bare impl →
   `nil`/empty). FIX: look up `env.types().get(&ed.protocol_name)` → `TypeDef::Surface(s)` → `s.members.find(method_name)`
   → the `SurfaceMember::Method { args, ret, type_params, .. }`. Build the scheme from the MEMBER: `type_params` from the
   member; `params` = `[<:ed.type_name>] ++ member.args.fixed_params[1..].types`; `ret` = `member.ret`. Zip the impl
   clause's binder NAMES against these types for the body scope (self → `ed.type_name`).
2. **`src/runtime.rs:~677-701` (the runtime surface-extend registration)** — the runtime `func` built from the clause
   runs the body binding the arg NAMES; its types do not affect execution, so it likely needs NO change. CONFIRM the
   runtime dispatch arity matches (the impl's binder count = the surface member's arg count). If the runtime needs the
   member's types for anything, mirror the check-side inheritance.
3. **The protocol path is the reference** — `parse_defprotocol_form` + the protocol dispatch type the impl body against
   the PROTOCOL method sig (self → concrete type, type-params in scope). The surface fix is the same idea on the
   `TypeScheme` registration. Read how the protocol arm scopes the type-params + self for the impl body.

## STOP triggers
- **STOP-1 (the member lookup is unreachable):** if `collect_splice_defs_ctx` can't reach the surface's members at that
  point (the surface not yet registered in `env.types()`) — STOP and report the ordering seam.
- **STOP-2 (the impl body isn't checked against the scheme):** if fixing the scheme does NOT fix the body type-check
  (the body is checked elsewhere with its own nil types) — STOP and report where the impl body is type-checked.

## The gate (committed RED + #[ignore]'d)
`tests/types/probe_arc293_4e_pre_iii_extend_impl_inherits_types.{rs,wat}` — a GENERIC surface method `(make<T> [self x
<- :T] -> :t::Box<T>)`, a bare extend-impl `(make [self x] (:t::Box x))` whose body uses `T`. Verified RED. **UN-IGNORE
when GREEN** → `(:t::probe)` = 7. The monomorphic + the 293.4a-d + 293.4e-pre.i/ii probes must STAY green.

## EXPECTATIONS
| # | what | command | expected |
|---|---|---|---|
| 1 | generic extend-impl GREEN | `cargo nextest run --release -E 'test(extend_impl_inherits_surface_method_types)'` (un-ignore) | PASS (7) |
| 2 | 293.4a-d + pre.i/ii un-regressed | the method-member / dispatch / extend-type / shape_demo / surface-parity / generic-surface probes | green |
| 3 | the LOCUS migration now type-checks | apply the spawn.wat `defprotocol`→`defsurface` migration (wrap the method in `[...]`) → `cargo nextest run --release` | floor 0 |
| 4 | whole workspace green | `cargo nextest run --release` | floor 0 |

**NOTE for whoever fires this:** once GREEN, row #3 IS the 293.4e migration (Locus → defsurface). If it passes, proceed
straight into 293.4e (the 9-file `defprotocol` rip) — the migration is the same `spawn.wat` edit that was reverted at
59a485bb. So 293.4e-pre.iii's green directly unblocks the annihilation.

## You are a LEAF
Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn subagents. Do NOT
commit. Build incrementally. Read every diff. Self-verify the EXPECTATIONS. STOP + report if a STOP fires.

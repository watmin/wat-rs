# BRIEF — 293.4e-pre.ii: generic surface method members (parity with arc-267 generic protocol methods)

**The work, in one paragraph.** A generic surface method member `(make<T> [self … x <- :T] -> :T)` parses with the
`<T>` STILL ON THE NAME (stored `"make<T>"`) and `type_params` HARDCODED EMPTY — so the call `:t::Maker/make` (`"make"`)
never matches → `unknown callee`. The protocol path solved this in arc-267: split the `<T>` off the name into
`type_params`, then instantiate them at the call site. Bring the surface path to the SAME parity at three sites. This
is the **last gate** before `:wat::spawn::Locus`'s `launch<S,R,St,Sh,Lu>` can migrate `defprotocol` → `defsurface`
(293.4e).

## The one contract decision (pinned)
A surface method member's name is split into `(bare_name, type_params)` exactly as `parse_defprotocol_form` does
(`split_name_and_type_params`). At a `:Surface/method` call, if the member's `type_params` is non-empty, instantiate
them (explicit type-args off the call head OR fresh-var inference) — mirroring the protocol call-check. Monomorphic
methods (empty `type_params`) take the identity/no-op path (today's behaviour, unchanged).

## Read in order (the rooms — grounded 2026-06-28)
1. **`src/types/surface.rs` `parse_method_member_sig` (~line 133 + 232–236)** — TODAY: `method_name =
   s.as_str().to_owned()` (no split) and `type_params: vec![]` (hardcoded). FIX: split via the SAME helper the protocol
   parse uses — `split_name_and_type_params` (grep its use in `parse_defprotocol_form`, `src/runtime.rs:~5920`; it
   returns `(bare_name, Vec<String>)`). Store the bare name + the real `type_params` on the `SurfaceMember::Method`.
   (Handle the `EvalBreak::Diagnostic` error path like the protocol parse does, adapted to `TypeError` — STOP-3-style
   copy if the error types clash, as 293.4a did.)
2. **`src/check.rs` the surface-method call arm (~5969–6030, the 293.4b/d arm)** — TODAY: matches the member by name,
   computes `expected_arity = 1 + extra_param_types.len()`, zips arg types — NO type-param handling. FIX: MIRROR the
   protocol call-check (`src/check.rs:~5805–5860`): `split_type_params_pub(method_name_raw)` to read any explicit
   `:Surface/method<i64>` type-args off the call head; if the member's `type_params` is non-empty, instantiate
   `type_params[i] → explicit[i]` (when provided) else fresh inference vars, producing instantiated `(arg_types, ret)`;
   then arity-check + zip against the INSTANTIATED types. Empty `type_params` → identity (the existing path). Reuse the
   protocol arm's instantiation code shape verbatim where possible.
3. **`src/runtime.rs` the surface dispatch arm (~5300, the 293.4b/c/d arm)** — the protocol runtime arm splits an
   explicit `<...>` suffix off the call head (`split_type_params`, runtime.rs:~5117) before the bare-name lookup. If the
   surface arm doesn't already strip a `<...>` suffix, add the same split so `:Surface/method<i64>` dispatches by the
   bare `method`. (The probe + Locus use BARE calls — no explicit suffix — so this is belt-and-suspenders; do it for
   parity, but if it's already handled, note it.)

## Implementation sketch
- surface parse: `let (name, type_params) = split_name_and_type_params(raw)?;` → store both (drop the hardcoded `vec![]`).
- check arm: copy the protocol arm's `type_params.is_empty() ? identity : instantiate` block; arity + zip on instantiated types.
- runtime arm: ensure the call-head `<...>` suffix is split before the `:<T>/<method>` lookup (mirror runtime.rs:5117).

## Blast radius (bounded)
`src/types/surface.rs` (the parse — split + store type_params), `src/check.rs` (the surface call arm — instantiation),
`src/runtime.rs` (the surface dispatch arm — suffix split, if needed). NO change to monomorphic behaviour (the
293.4e-pre.i probe + all 293.4a-d probes must stay green). NO `defprotocol` touch (293.4e).

## STOP triggers (halt + surface)
- **STOP-1:** if `split_name_and_type_params` is not reachable from `surface.rs` (error-type or visibility clash) — copy
  its ~15-line shape into surface.rs (like 293.4a copied the protocol sig-parse), note it; do NOT half-split.
- **STOP-2:** if instantiating the surface method's type-params needs machinery the protocol arm does NOT have (i.e. the
  mirror is not actually a mirror) — STOP and report the divergence; do not invent a new instantiation path.

## The gates (committed RED + #[ignore]'d)
- `tests/types/probe_arc293_4e_pre_ii_generic_surface_method.{rs,wat}` — a generic `(make<T> …)` surface method,
  extend-impl, dispatched with `T=i64` → 42. Verified RED (`unknown callee`). **UN-IGNORE it; it must go GREEN.**
- `tests/types/probe_arc293_4e_pre_surface_method_parity.rs` (the monomorphic multi-arg case) must STAY green.

## EXPECTATIONS
| # | what | command | expected |
|---|---|---|---|
| 1 | generic surface method GREEN | `cargo nextest run --release -E 'test(generic_surface_method_dispatches_with_type_params)'` (un-ignore) | PASS (42) |
| 2 | monomorphic un-regressed | `cargo nextest run --release -E 'test(surface_method_with_args_beyond_self)'` | PASS (42) |
| 3 | 293.4a-d + protocols un-regressed | `cargo nextest run --release -E 'binary(function)' + test(method_member) + test(surface_method_dispatches) + test(extend_type) + test(shape_demo)` | green |
| 4 | whole workspace green | `cargo nextest run --release` | floor 0 |

## You are a LEAF
Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn subagents. Do NOT
commit. Build incrementally. Read every diff. Self-verify the EXPECTATIONS. STOP + report if a STOP fires.

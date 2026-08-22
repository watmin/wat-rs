# BRIEF — identity stone 1, flight 2: `family_extends` gets its own door

DESIGN: `DESIGN-STONE-the-angle-string-is-not-a-type-identity.md`. Read it first — it records that
this design was **wrong twice**, and why flight 1's approach broke two negative controls.

⚠ **Flight 1's work is in the working tree and is your base. Most of it must be UNDONE.** Flight 1
implemented A-i (key the lattice by the base name); the builder then ruled **S2** instead. Flight 1
was not a bad flight — it fired STOP-1 correctly and found the mechanism. You are changing the
approach, not fixing its execution.

## The ruling in one line

`is_subtype` keeps **EXACT-string** semantics. A second, explicit query — `family_extends` —
answers the arg-agnostic question that `satisfies_bare_surface` was faking with a prefix match.

## Exactly what to do with flight 1's tree

| in the tree now | do |
|---|---|
| `register_subtype` strips `child`/`parent` via `split_type_params_pub` | **REVERT** — keys stay exact |
| `is_subtype` strips `sub`/`sup` | **REVERT** — the fast path's soundness depends on exact compare |
| `transport_satisfier_heads` collapsed to `vec![parametric_head_fqdn(head)]` | **REVERT** to `vec![fq, format!("{fq}<T>"), format!("{fq}<Xt>")]` — it guesses at EXACT keys, which remain |
| `satisfies_bare_surface` deleted; its 4 call sites now say `is_subtype` | **RE-POINT** those 4 at a new `family_extends` (see below). Do NOT restore the prefix match. |
| `extend-type`'s protocol slot accepts a `WatAST::List` via `parse_type_node` + `base_fqdn()` | **KEEP** — this is ②-iii blocker 3's lattice half and is orthogonal to S2 |

The 4 sites: `src/check.rs` ~15333 and ~15440, `src/runtime.rs` ~8965 and ~9011. Flight 1 also
rewrote three comments to name `is_subtype`; those must name `family_extends` instead.

## The new door

```rust
/// Does `sub`'s FAMILY extend `sup`'s family — existence only, arguments ignored?
///
/// The question `satisfies_bare_surface` was asking with `format!("{surface}<")`, a prefix
/// match: "is ANY instantiation of this surface reachable from this type?" Asking it by
/// prefix meant the code claimed a relation it never checked. This asks it directly.
///
/// NOT a substitute for `is_subtype`, which answers the EXACT question and whose exact-string
/// compare is load-bearing for `assignable`'s transport fast path.
pub(crate) fn family_extends(sub: &str, sup: &str, env: &TypeEnv) -> bool
```

Both base-extraction doors already exist — `split_type_params_pub` (`src/runtime.rs:14266`) and
`TypeExpr::base_fqdn` (`src/types.rs:131`). **Write no third one.**

## ★ Why this matters — read before you touch `assignable`

`check.rs`'s `(Parametric, Parametric)` arm has a fast path whose exact-string compare was doing
**two jobs** — "does the edge exist" AND, by failing to match, "and do the args agree". Flight 1
made it succeed on head alone, so it returned before the `else` branch's `unify` guard ran, and two
**negative controls** went red. The `else` branch's own comment says it:

> ★ *SOUNDNESS LIVES IN THE GUARDS BELOW, NOT IN THE GATE … enforced by UNIFY on the args … Both are
> negative-control rows of 118.B1a's gate.*

**Do not change `assignable`.** Reverting `is_subtype` to exact restores it untouched.

## What "done" looks like

1. ★ `cargo nextest run --release -E 'binary_id(wat::types)'` → **537 passed, 0 failed, 1 skipped**.
   Specifically `probe_arc170_parametric_surface_param::wrong_parametric_surface_param_is_compile_error`
   and `probe_stone_118_b1a_neg::concrete_surface_satisfaction_still_refuses_unrelated_family_and_swapped_args`
   must PASS. **These two are the stone.**
2. `grep -rn 'satisfies_bare_surface\|format!("{surface}<' src/` → nothing.
3. `family_extends` has exactly ONE implementation, and the 4 sites route through it.
4. `transport_satisfier_heads` is back to three keys.
5. `extend-type` still accepts a FORM parent `(:Proto :- [T])`.
6. `is_subtype`'s signature and its 30 call sites unchanged.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
- Do NOT commit, push, stash, revert-via-git, or amend. Edit the files; leave everything in the tree.
- Do NOT touch `assignable`'s arms in `check.rs` beyond re-pointing the 4 calls and their comments.
- Do NOT touch `wat/service.wat`, `wat/core.wat`, `wat/bracket.wat`, `wat/fix.wat` — stones 2 and 3.
- Write no new base-extraction helper.

## Your own checks

`cargo build --bin wat`, then the scoped `cargo nextest run --release -E 'binary_id(wat::types)'`.
Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.
Diagnostics go to **stderr** — judge by exit code AND empty output, never grep alone.

⚠ Row 1 is a **restoration**: those two tests passed before flight 1. Seeing them green does not by
itself show `family_extends` does anything — so also demonstrate row 2 and row 3 directly, and say
which check proved which.

## STOP triggers — ship nothing further and report

- **STOP-1.** If reverting the strips does NOT restore both negative controls, STOP — something
  beyond the strips moved, and the tree needs reading before more is added.
- **STOP-2.** If any of the 4 sites needs something other than `family_extends`'s exact signature,
  STOP and report which and why. Do not widen the door to fit a caller.
- **STOP-3.** If `family_extends` cannot be implemented without touching `assignable`, STOP.

## Your report

The diff per file. Row 1 with the verbatim scoped-suite Summary line. Rows 2-6 with their evidence,
saying which check proved which. What surprised you. Anything you inspected and left alone, and why.

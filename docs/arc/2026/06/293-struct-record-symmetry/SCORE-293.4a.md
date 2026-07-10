# SCORE — 293.4a: method members in `defsurface` (parse + satisfy)

**Verdict: GREEN, weighed by the orchestrator's own re-run.** `cargo nextest run --release` (whole workspace) =
**4087 passed / 0 failed / 93 skipped**. The 293.4a probe flipped RED→GREEN; the negative arm proves satisfaction is a
real sig-check; the acceptance demo stays `#[ignore]`'d (293.4d's gate).

## Scorecard (each row re-run by the orchestrator)
| # | what | result |
|---|---|---|
| 1 | 293.4a probe GREEN (un-ignored) | **PASS** — `method_member_surface_parses_and_is_satisfied_by_a_defn` |
| 2 | a method member PARSES | **PASS** — no more `MalformedDecl "triple is incomplete"` |
| 3 | a method member SATISFIED by a `defn` | **PASS** — `(:t::accept (:t::Sq …))` type-checks |
| 4 | MISSING method = NOT satisfied (negative) | **PASS** — `method_member_not_satisfied_when_defn_is_absent` (+ `.wat.bad`); resolver returns None → false |
| 5 | acceptance demo stays RED (untouched) | **PASS** — still `#[ignore]`'d |
| 6 | whole workspace green | **PASS** — 4087 / 0 / 93 (own forced run) |

## What shipped
- **`src/types.rs`** — `SurfaceMember = Field { name, ty } | Method { name, args: ArgSpec, ret, type_params }`;
  `SurfaceDef.members: Vec<SurfaceMember>`. **`args` is `ArgSpec`** (the correction — one canonical binder representation).
- **`src/types/surface.rs`** — `parse_defsurface` walks mixed members (field triples → `parse_argspec_triples`; method
  lists → `parse_method_member_sig`, keeping the full `ArgSpec`); `struct_satisfies_surface` gains a `resolve_method`
  closure and checks Method members (ret assignable + per-position arg assignability over `args.fixed_params`).
- **`src/check.rs`** — `assignable` threads `&CheckEnv` (was `&TypeEnv`) so satisfaction can resolve `defn :T/<name>`;
  the resolver `env.get(":<T>/<name>") → (params, ret)` built at the satisfaction call site. 17 internal sites updated.
- **`src/function/infer.rs`**, **`src/closure_extract.rs`** — the mechanical cascade (one assignable caller; the two
  `SurfaceDef.members` readers pattern-match Field|Method).
- **`src/argspec/parse.rs`** — `ArgSpec` derives `PartialEq, Eq` (it now lives inside the `Eq`-derived `SurfaceDef`).
- **Tests** — `probe_arc293_4a_method_members.rs` un-ignored + negative arm; `probe_arc293_4a_method_members.wat.bad` new.

## Honest deltas (carried, not hidden)
1. **The ArgSpec correction** — caught mid-build (the member was first written `arg_types: Vec<TypeExpr>`, the brief's
   wrong suggestion); corrected to `args: ArgSpec` before completion. The brief is amended with recognition.
2. **STOP-1 did not fire** — `assignable` only carried `&TypeEnv`; the executor threaded `&CheckEnv` through it (the
   correct, if broad, seam). A core-fn signature moved (17 sites) — flagged, verified clean.
3. **The resolver reads the defn sig FLAT** — `resolve_method → (Vec<TypeExpr>, TypeExpr)` because `Scheme.params` is
   stored flat. The ArgSpec fix is on the NEW type (`SurfaceMember`); reading an existing flat representation is not a
   new sin. **Banked follow-up:** "args = ArgSpec EVERYWHERE" — `Scheme.params` and `ProtocolMethodSig.arg_types` still
   flatten; a future decomplect so the one-canonical-binder-list law holds substrate-wide.
4. **STOP-3 fired as designed** — `parse_defprotocol_form` returns `RuntimeError`, `parse_defsurface` returns
   `TypeError`; copied the ~20-line sig-parse shape into `surface.rs` rather than a contorted shared helper.
5. **Pre-existing stray probe** — `probe_arc293_4a_surface_method_member.{rs,wat}` (committed `e04256f2`, a prior
   session's stub) uses a non-canonical syntax (methods OUTSIDE the member vector); left `#[ignore]`'d. Purgare
   candidate (a stale stub for a syntax we don't ship) — banked, not this slice's job.

## Next
**293.4b — the generated dispatcher** (`:Shape/area s` routes by `s`'s runtime type to `:T/area`; LIFT arc-232
`extract-classifier`+`apply`, `runtime.rs:670`). Then 293.4c (`extend-type` foreign-accessor adapter) → 293.4d
(annihilate `defprotocol`, un-ignore the acceptance demo).

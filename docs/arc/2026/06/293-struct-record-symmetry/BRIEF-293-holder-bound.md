# BRIEF — arc 293: the `:holder` surface bound (R3's `foobar` form — the additive categorical axis)

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents, no git worktrees, do NOT commit.** If the work exceeds these rooms or hits a STOP trigger, STOP and
report — do not improvise. **TRUST ONLY FORCED CLEAN BUILDS** (`cargo clean -p wat && cargo build --release -p wat`)
before claiming green — incremental builds + rust-analyzer lag emit stale `E0xxx`. Read the disk, not the cache.

## The work, in one paragraph

The landed `defsurface` (293.3) is **purely structural**: a surface is a set of members, and any aggregate with
those members satisfies it (width subtyping), regardless of its holder. R3 (DESIGN § THE HOLDER × SURFACE MODEL)
adds the orthogonal **categorical** axis: a surface may carry an **optional `:holder` bound**, enforced HARD —
holon-ness/edn-portability are capabilities a structural shape *cannot fake*. This strike is **purely additive**
(no existing behavior changes): give `SurfaceDef` an `Option<Holder>`, teach `parse_defsurface` to read an
optional `:holder <kw>` clause, and make the `assignable` surface arm *also* require `surf.holder == Some(h) ⇒
agg.holder == h`. A surface with no `:holder` clause behaves exactly as today (`holder: None`).

## THE GATE = the committed RED probe goes GREEN (un-ignore it)

`tests/types/probe_arc293_holder_bound.rs` (committed `fdf46468`, verified RED at HEAD), 2 tests, BOTH `#[ignore]`'d:
- `holon_record_satisfies_holder_bound_surface` — a holon record satisfies `:holder :holon-record` → **is_ok** (RED→GREEN)
- `core_record_rejected_by_holon_holder_bound` — a CORE record with the SAME fields is **rejected**, the error
  citing the surface `:env::Holon` (not the HEAD MalformedDecl) → RED→GREEN
**REMOVE the two `#[ignore]` lines** when they pass.

## Decisions pinned (do NOT re-litigate)

- **Holder-value keyword spelling: `:struct` / `:record` / `:holon-record`** (kebab of the `Holder` variants
  `Struct`/`Record`/`HolonRecord`; DESIGN's `defsurface` example uses `:holon-record`). Accept all three; any
  other value keyword → a clear parse error naming the three.
- **`:holder` is OPTIONAL and additive.** Absent clause → `holder: None` → today's pure-structural behavior,
  unchanged. The 5 landed `defsurface` callers (`probe_arc293_structural_surface.rs`, `probe_arc293_record_surface.rs`)
  are all 2-arg and MUST stay green.
- **The bound is checked AFTER the structural width-match** (a holder bound is not a substitute for having the
  members — it is an additional, categorical requirement). `satisfies = structural_match && holder_ok`.
- **NO new `TypeDef` variant, NO `assignable` signature change.** Everything is `&TypeEnv`-local, exactly like
  the existing surface arm.

## Rooms — read in order (re-ground before editing)

1. **`src/types.rs:130-139`** — the `Holder` enum (`Struct`/`Record`/`HolonRecord`, `is_portable()`). Your value
   keyword maps to these.
2. **`src/types.rs:233-237`** — `struct SurfaceDef { name, type_params, members }`. **Add `pub holder:
   Option<crate::types::Holder>`** (use the in-module `Holder` path). This struct + the one constructor in room 4
   are the ONLY two `SurfaceDef` sites (grep `SurfaceDef {` → 2 hits). No other match-site reads its fields by
   pattern, so the field-add cascade is trivial.
3. **`src/types/surface.rs:48-102`** — `parse_defsurface`. Today: requires `args.len() == 2` (name +
   member-vector). **Change to accept an OPTIONAL `:holder <kw>` clause between them:**
   - `args[0]` = name keyword (unchanged — `parse_declared_name`).
   - If the NEXT arg is the keyword `":holder"` (match `WatAST::Keyword(k, _)` with `k == ":holder"` — note the
     leading colon is part of the stored string, cf. `defstruct.rs:186` which `trim_start_matches(':')`), then
     the arg after it is the value keyword → map `":struct"|":record"|":holon-record"` → `Some(Holder::…)`; an
     unknown value → `MalformedDecl` naming the three valid spellings. The member-vector follows.
   - Else (no `:holder`): the next arg IS the member-vector; `holder = None`.
   - Valid arities: **2** (name + members) or **4** (name + `:holder` + value + members). Anything else →
     `MalformedDecl` (keep the existing helpful message; extend it to mention the optional `:holder` clause).
   - Construct `SurfaceDef { name, type_params: vec![], members, holder }`.
4. **`src/check.rs:14229-14248`** — the `assignable` surface arm. Today: expected resolves to `TypeDef::Surface`,
   actual resolves to `TypeDef::Aggregate(agg)`, and it `return struct_satisfies_surface(&agg.fields, &surf, …)`.
   **Add the holder check AFTER the structural result:**
   ```rust
   let structural = crate::types::surface::struct_satisfies_surface(
       &fields_clone, &surf_clone, |fty, mty| assignable(fty, mty, subst, types));
   let holder_ok = match surf_clone.holder {
       Some(req) => agg.holder == req,   // categorical — HARD; a wat.core/Record is rejected for :holon-record
       None => true,                     // no bound → structural-only (today's behavior)
   };
   return structural && holder_ok;
   ```
   (Bind `agg.holder` before the `surf`/`fields` clones release the `types` borrow if the borrow-checker needs it —
   `agg.holder` is `Copy`.)

## STOP triggers (halt + report; do NOT improvise)

1. **STOP** if adding `holder: Option<Holder>` to `SurfaceDef` cascades into more than the 2 known sites (a third
   `SurfaceDef {` literal, or a pattern-match that destructures its fields) — report the site list.
2. **STOP** if the `:holder` keyword as-written arrives NOT as `WatAST::Keyword(":holder", _)` (e.g. it's
   namespaced to `:wat::core::holder`, or arrives as a different AST node) — report what you actually see; do NOT
   guess the match.
3. **STOP** if making the reject-case cite `:env::Holon` requires changing the `TypeError` shape or `assignable`'s
   signature — it must not (the existing arg-mismatch error already names the expected surface type). Report the
   actual error string you get if it does NOT contain `env::Holon`.
4. **STOP** if any of the 5 landed 2-arg `defsurface` callers regress (they must parse identically as `holder: None`).
5. You are a LEAF. No subagents. If the change exceeds these 4 rooms, STOP and report.

## Gate (the orchestrator re-runs every line AFTER a forced clean build)

| what | command | expected |
|---|---|---|
| forced clean build | `cargo clean -p wat && cargo build --release -p wat` | clean (no `error[E…]`) |
| **the bound probe goes green** | `cargo nextest run --release -p wat -E 'binary(types) & test(holder_bound)'` (after removing the 2 `#[ignore]`) | **2 passed** |
| structural surfaces unchanged (no-bound path) | `cargo nextest run --release -p wat -E 'binary(types) & test(structural_surface)' -E 'binary(types) & test(record_surface)'` | all green |
| holder lattice intact | `cargo test --release -p wat --test types -- holder_substitution structtype` | green |
| no new regressions | `cargo nextest run --release -p wat`, failing-test SET vs HEAD (`fdf46468`; floor = 0 deterministic) | **∅ new** |

## Report back
Full `git diff --stat`; verbatim gate output (from the forced-clean-build run); the failing SET if any; the EXACT
error string the reject-case produces (to confirm it cites `:env::Holon`); whether any STOP fired. Do NOT commit.

Runtime: 30–60 min. Trap-doors: (a) the `:holder` keyword's exact stored spelling (colon-prefixed; STOP-2 if not);
(b) the borrow release in the `assignable` arm (`agg.holder` is `Copy`, bind it early); (c) the reject-case error
citing the surface — if it doesn't, that is a real finding (STOP-3), not something to paper over.

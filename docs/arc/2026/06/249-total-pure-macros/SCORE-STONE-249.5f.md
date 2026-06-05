# SCORE — Stone 249.5f: canonical scope renumbering at hash time

Graded against `EXPECTATIONS-STONE-249.5f.md`, every load-bearing row re-run
**independently by the orchestrator**.

## Scorecard

| # | What | Result |
|---|---|---|
| 1 | Cross-run hash determinism | ✓ `probe_hash_scope_renumber` → 2 passed — **orchestrator re-run** (`renamed_scopes_hash_equal` RED→GREEN) |
| 2 | Discrimination guard holds | ✓ `distinct_scope_structure_hashes_differently` passes — a RENUMBER, not a strip |
| 3 | Caveat retired | ✓ grep `REMAINS deferred\|separable follow-on\|not across runs` → 0 hits |
| 4 | Renumberer private, no API change | ✓ `struct ScopeRenumber` (not `pub`); `canonical_edn_*` / `hash_canonical_*` signatures unchanged |
| 5 | Prior hygiene contracts hold | ✓ macro-capture 2, argspec-rest 1, check-resolution 2 = 5 passed — orchestrator re-run |
| 6 | Library suite — no regressions | ✓ 907 passed / 0 failed / 1 ignored — non-macro hashes byte-identical (the witness) |
| 7 | Bounded blast radius | ✓ only `src/hash.rs` (+68/−29) + the probe |

## Trap-doors (cleared)

- **Anti-strip witness GREEN** — the discrimination guard confirms the fix renumbers
  canonically (distinct scope structure still hashes distinctly), not a scope-strip.
- **Non-macro byte-identity** — lib 907/0/1 held exactly at baseline; empty scope
  sets emit zero scope bytes, the renumberer is never consulted, output unchanged.
- **Program-wide numbering** — verified: `canonical_edn_program` creates ONE
  `ScopeRenumber` before the `for f in forms` loop and threads it into every form, so
  a scope shared across top-level forms gets one canonical index. Correct.

## Disposition — the hygiene class is ANNIHILATED (and proven last)

249.5f makes the hasher scope-aware-and-deterministic. With it, all three
identifier-keying surfaces are closed:

| surface | keying | stone |
|---|---|---|
| runtime resolution | `env_key` over raw scopes | 249.5b/d |
| check resolution | `env_key` over raw scopes | 249.5e |
| hash identity | canonical first-appearance renumber | **249.5f** |

### The completeness proof (the enumeration — not best-guess)

A grep of the whole substrate, this session, establishes the system is **closed by
construction**:

- **Scopes are MINTED at exactly ONE site** — the expander (`macros/expand.rs:217`
  `fresh_scope()` + `:682` `add_scope(macro_scope)`). No other non-test mint.
- **`.scopes()` is READ at exactly TWO sites** — `scope::resolution::env_key`
  (`resolution.rs:80,86`) and the canonical hasher (`hash.rs:206,210`). No third
  reader.
- **Every resolution bind of a possibly-scoped ident routes through `env_key`** —
  runtime: **0** binds keyed by `ident.as_str()` (14 via `env_key`, the other 5 by
  literal/FQDN names); check: Row-3 of 249.5e proved 0 `as_str` binds at any
  `locals` key.
- **The only other consumer of a scoped AST is display/EDN**, which legitimately
  ignores scopes (it serializes for humans).

So the reason earlier "done"s hid the next gap is that we were closing the
**chokepoints** one at a time (runtime → check → hash). The enumeration now shows
there are exactly THREE keying surfaces and **no fourth**: in-process resolution
(eval + check, funneled to `env_key`) and cross-process identity (the hasher).
Coverage is proven, and each surface has its own probe (capture, rest-param,
check-precision, hash-determinism + anti-strip). The class is closed AND each
closure is independently verified.

### What is NOT claimed

That a FUTURE consumer of scoped idents couldn't add a fourth keying surface and
silently reopen the class. The proof above is a **manual grep**. The floor-raise
that makes the annihilation can't-be-reopened: a self-enforcing integrity gate
asserting *only `env_key` and the canonical hasher read `.scopes()`* — a new
unsanctioned `.scopes()` reader (or a new `as_str`-keyed scoped bind) then FAILS
LOUD at build. **Named follow-on — fold into the `src/scope/` ward-close** (it is
exactly the kind of invariant a `vigilatum` stamp should carry).

## Remaining 249.5 thread

The ward-close: R3 re-cast `src/scope/` → L1+L2=0 → the `src/scope/` + held
`macros/` `vigilatum` stamps (+ the `.scopes()`-reader integrity gate above) —
DOUBLE-BLOCKED on the incoming vigilia update.

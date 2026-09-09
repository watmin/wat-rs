# SCORE — eleven `too_many_arguments` allows, and the codebase already wrote the answer

Executing strike per `DESIGN.md`. Appending as each site's verdict settles (house rule).

## Re-derived count (before touching anything)

```
grep -rn 'allow(clippy::too_many_arguments)' src/rete/
```

**11**, matching `DESIGN.md` exactly. No delta. (One additional hit is
`validate/mod.rs:392`, a doc-comment MENTION of the attribute inside `ClauseCtx`'s own doc, not an
`#[allow]` itself — excluded per the DESIGN's own count.)

The eleven, as found:

| # | site |
|---|---|
| 1 | `src/rete/compiled_cond.rs:246` |
| 2 | `src/rete/kernel/fire/pass/alpha.rs:338` |
| 3 | `src/rete/kernel/fire/mod.rs:469` |
| 4 | `src/rete/kernel/fire/mod.rs:2128` |
| 5 | `src/rete/kernel/fire/pass/filter_after_join.rs:17` |
| 6 | `src/rete/kernel/fire/pass/hash_join.rs:41` |
| 7 | `src/rete/kernel/fire/pass/hash_join.rs:427` |
| 8 | `src/rete/kernel/fire/pass/hash_join.rs:503` |
| 9 | `src/rete/kernel/fire/pass/join_after_filter.rs:31` |
| 10 | `src/rete/kernel/fire/pass/round_census.rs:25` |
| 11 | `src/rete/validate/typing.rs:742` |

## Gap found in `DESIGN.md`'s own table (flagging per the brief's invitation to disagree)

`DESIGN.md`'s table has 8 rows for 10 of the 11 sites (two rows each bundle a pair:
`hash_join.rs:41`+`join_after_filter.rs:31`, and `fire/mod.rs:469`+`typing.rs:742`).
**`hash_join.rs:503` (`hj_step3_term1`) is not in the table at all** — not listed under any row,
not bundled with its twin `hj_step4_term2` at `:427`. Read and verdicted below (#8).

The combined row for `fire/mod.rs:469` + `typing.rs:742` ("fragments — verify") **resolves in two
different directions** once read in full — see #3 and #11 below. `typing.rs:742` already addresses
argument count explicitly ("EIGHT parameters..."); `fire/mod.rs:469` does not (it describes the
exists/not dispatch algorithm, not the argument list). Splitting the row rather than applying one
verdict to both.

## Per-site table

| # | site | prior comment (gist) | verdict | action | reason written |
|---|---|---|---|---|---|
| 1 | `compiled_cond.rs:246` (`from_parts`) | "7 args since A3 (was 8...). A builder would be ceremony for a constructor with two call sites." | **already addresses arg count** | none — left as-is | (pre-existing; not touched) |
| 2 | `alpha.rs:338` (`activate_deferred_mixed_classes`) | bare (`#[cold]`/`#[inline(never)]` above, no comment on arg count) | **arguments already travel as a unit** — 9 of 11 params are constructed as `AlphaActivateCx` at the fn's OWN only call site inside its OWN body, and at 3 other call sites in the same file | **(2): allow removed.** Signature is now `(cx: &mut AlphaActivateCx<'_>, input_facts: &PVec, plan: &ClassPlan)` — 3 params. Caller builds the `AlphaActivateCx` once (was previously reconstructed per-fact inside the fn) | doc-comment addition explaining why this one qualified as visibly-a-unit, unlike the sites left as `#[allow]` |
| 3 | `fire/mod.rs:469` (`token_exists_under`) | describes the exists/not recursion and the hot-leaf/rare-combinator split — **not argument count** | **disagree with DESIGN's "fragments — verify"**: does not address arg count | **(1): reason added.** No existing struct fits (checked `FireCtx` — different shape, split-borrow of `wm` whole, not applicable here); each match arm forwards fields individually to its own callee, so bundling gains nothing | new comment: 5 shared "exists/not working set" refs + 3 call-specific args; a struct would still leave 3 positional and add an unpacking layer |
| 4 | `fire/mod.rs:2128` (`any_seeded_keyed`) | "Exists/Not Leaf: probe the token's bucket..." — describes the fn | **matches DESIGN's "⛔ describes the fn"** | **(1): reason added** | new comment: same shape as #3 — 4 shared refs, 4 call-specific, no existing bundle fits |
| 5 | `filter_after_join.rs:17` (`filter_after_join`) | "Drain the frontier pass 3.6 produced, dispatching the trailing filters." — describes the fn | **matches DESIGN's "⛔ describes the fn"** | **(1): reason added**, citing `hash_join_delta`/`join_after_filter`'s existing working-set language since this fn's params are the same list plus 2 of its own | new comment naming the shared 7 + this pass's 2 own (`gather_cache`, `after_join_frontier`) |
| 6 | `hash_join.rs:41` (`hash_join_delta`) | "8 args since fix-list F: `sym` joined... The alternative — a context struct — would have to be built at every call site... already the fire pass's working set rather than an accident." | **already addresses arg count, explicitly** | none — left as-is | (pre-existing; not touched) |
| 7 | `hash_join.rs:427` (`hj_step4_term2`) | "...correcting it is what made this a move rather than a thirteen-parameter explosion. See `AlphaNews::of`." | **already addresses arg count — this is the model the DESIGN names** | none — left as-is | (pre-existing; not touched) |
| 8 | `hash_join.rs:503` (`hj_step3_term1`) | "Twin of `hj_step4_term2`... lifted for the same reason: nesting nine." — describes the extraction's history, not the argument list | **not in DESIGN's table at all (gap, see above); read fresh: does not address arg count** | **(1): reason added**, pointing at its twin (#7) and the risk of the two signatures drifting apart if only one got a struct | new comment: same working set as `hj_step4_term2`, same nesting-nine origin, symmetry with the twin is the reason not to bundle one alone |
| 9 | `join_after_filter.rs:31` (`join_after_filter`) | identical wording to #6 (`hash_join.rs:41`) — "8 args since fix-list F..." | **already addresses arg count, explicitly** | none — left as-is | (pre-existing; not touched) |
| 10 | `round_census.rs:25` (`record_round_census`) | "Record one round into the fire census. Reads only; the caller owns `round_no` and advances it." — describes the fn | **matches DESIGN's "⛔ describes the fn"** | **(1): reason added.** Verified `#[cfg(test)]`-gated, single call site (`fire/delta.rs:676`) before writing "one call site" into the reason | new comment: 9 distinct unrelated read-only refs, test-only, single call site — a struct here would exist only to satisfy the lint |
| 11 | `validate/typing.rs:742` (`check_then_field_type`) | "EIGHT parameters, and the allow is the honest shape here rather than a bundle. The `:when` side bundles its six into `ClauseCtx`... A context struct minted to hold those four unrelated references would exist only to satisfy a counter, and would name nothing." | **disagree with DESIGN's "fragments — verify": already addresses arg count, explicitly and at length** — it even contrasts itself with `ClauseCtx` by name | none — left as-is | (pre-existing; not touched) |

**Tally: 5 sites already carried an arg-count reason before this strike touched anything** (#1, #6,
#7, #9, #11 — DESIGN had flagged 4 of these correctly as "likely"/"yes" and had #11 half-right
inside a combined row; #8 wasn't assessed at all). **5 sites got a new reason written, allow kept**
(#3, #4, #5, #8, #10). **1 site converted to a context struct, allow removed** (#2).

## Why #2 (`alpha.rs:338`) took (2) and nothing else did

The pinned contract defaults to (1) and reserves (2) for arguments that **visibly already travel
as a unit**. `activate_deferred_mixed_classes` was the only site where that is not an inference but
a fact already written in the file: `AlphaActivateCx` is constructed from 9 of this fn's 11
parameters, by name, inside this very function's body (the only place it's called from), and at
three other call sites in the same file (`alpha_seed`'s two inline calls, `alpha_delta`'s one). The
9 fields were never independent arguments — they were `AlphaActivateCx`'s fields, passed positionally
for no reason connected to this function. Converting removed the allow (11 → 3 params: `cx`,
`input_facts`, `plan`) and, as a side effect, replaced a per-fact struct reconstruction inside the
loop with one construction at the call site — a strict simplification, not just a lint fix. The fn
is `#[cold]`/`#[inline(never)]` (the rare mixed-class path), which also kept this well clear of the
"hot-path signature rewrite" STOP: nothing on the per-fact hot path changed shape.

Every other site's params either already carry an explicit arg-count reason, or (checked per site:
`FireCtx` for #3/#4, no struct at all in `round_census.rs`/`filter_after_join.rs` for #5/#10, and
#8's twin `hj_step4_term2` itself un-bundled) have no existing structural evidence that they travel
as a unit — so per the pinned decision they got a written reason instead, not a rewrite.

## Build / gates

```
cargo build --release
    Finished `release` profile [optimized] target(s) in 53.29s
```
(clean before AND after the `alpha.rs` restructure — run twice, once per house rule on builds)

```
cargo clippy --all-targets --release -- -D warnings
    Finished `release` profile [optimized] target(s) in 18.36s
RC=0
```
No warnings — including at `alpha.rs`, where the allow is now GONE and clippy still passes, which
is the proof the restructure actually satisfies the lint rather than just moving the allow around.

```
./scripts/floor.sh
     Summary [ 453.666s] 5485 tests run: 5485 passed (1 slow), 19 skipped
```
Read from `.floor/latest/clean.log`'s `Summary` line, never a piped exit code. **5485/5485, matching
the expected 5485 exactly.** No red at any point in this strike; nothing was re-run.

## Post-edit recheck

```
grep -rn 'allow(clippy::too_many_arguments)' src/rete/ | grep -v 'validate/mod.rs:392'
```
→ **10** `#[allow]` sites remain (11 − 1 removed at `alpha.rs`), each now ending in one of the two
states this strike requires: an explicit argument-count reason (10 of them, 5 pre-existing + 5
newly written) or gone (1, `alpha.rs`).

## What this did NOT do

- Did not rewrite any signature on the fire hot path (`hash_join_delta`, `join_after_filter`,
  `filter_after_join`, `hj_step3_term1`/`hj_step4_term2`, `token_exists_under`, `any_seeded_keyed`,
  `record_round_census`) — all kept their allow, per the pinned decision, with a new or
  already-adequate reason.
- Did not touch the 11 sites outside `src/rete/` (out of scope per `DESIGN.md`).
- Did not widen to any other `#[allow]` lint.
- Did not treat `alpha.rs:338`'s restructure as licence to look for more struct-shaped sites beyond
  the one where the evidence (an existing struct built from the same names at 3 other sites) was
  already on the page.

## Commit

`87285b5db` on branch `grok-rete` — "rete: eleven too_many_arguments allows — reasons or ClauseCtx
(arc 278)". 6 files changed (the 5 touched source files + this `SCORE.md`), 181 insertions(+),
34 deletions(-).

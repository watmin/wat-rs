# SCORE — STONE 255.3: the REGISTRY decides which join — `::` or `/`

Branch: `main`. **Committed, not pushed.** Drawn against `82e4cc836` (draw `4b46c4b7a`).
Parent: `BRIEF-STONE-255.3-the-registry-knows-which-join.md`.
Floor / workspace clippy / `census.sh --diff`: orchestrator's row (not run).
Crate clippy + three wall tests run here. **No `.wat` converted.** 8d-ii not started.

⭐ **STOP after step 1.** The count moved substantially. Position grammar **not started.**

## Gate: the 179-file delta — ONE tree, before and after

Same spread as 255.1 / 255.2: every 12th path of the 8d-i census `files.txt` (2145/12 = **179**).
Originals: **live tree**. Converted copies: `/tmp/8d-i/census/tree-after-1`.
Timed one file first: converted `tests/macros/probe_arc209_macro_span_fidelity.wat` **0.155 s**.

The brief's baseline **97** is the orchestrator's 255.2 WEIGH tree. This stone's before is
the **same 179 on this tree with the 255.2 binary** (101), so the comparison is like-for-like.
`[[feedback_diff_like_against_like]]`.

| | this tree before (255.2 binary) | this stone |
|---|---|---|
| originals clean | **161 / 179** | **161 / 179** |
| of those, still clean after conversion | 60 | **77** |
| ⛔ **REGRESSIONS** | **101** | **84** |

**19 closed. 2 newly_broken. Net −17.**

### Classification (84)

| cause | n | vs this tree at 255.2 (101) |
|---|---|---|
| `UnresolvedReference` | **51** | 72 → 51 |
| `ReteCheckErrors` | **16** | 12 → 16 (four files left UR and hit the rete wall) |
| `MalformedDecl` | 9 | same |
| `ProgramBodyEvalFailed` | 3 | same |
| `CheckErrors` / OTHER / `UnknownNamedType` | 5 | same total |

The Type/method class moved. Files that only had a `Type::member` UR became clean; files
that had that **and** a later error stay red under the later error. Same shape as 255.2's
arrow door: the count is not the residue kind.

Traced file (brief's own): converted `probe_arc209_macro_span_fidelity.wat` — original
`(:wat::core::Option/expect …)` is **gone** from the UR list. Remaining: `:user::mk-kw`
(declaration name) and `:wat::WatAST` (annotation). Both position class.

Counter-actor original still **rc=0**. Converted remaining 8 are all position:
`counter/Request`, `counter/Response`, `wat.enum/Pure`, `counter/dispatch`.
`LociDiedError::message` is gone.

### Closed (19)

`tests/comms/probe_arc209_structured_peer_death.wat`, `tests/comms/wat_pipe.wat`,
`tests/function/wat_arc170_closure_extraction_t11.wat`, `tests/kernel/wat_engram_library.wat`,
`tests/process/probe_run_hermetic_no_deadlock.wat`, `tests/program/probe_arc259_peer_env_install.wat`,
`tests/resolve/probe_arc251_stone5a_recognition.wat`, `tests/rete/probe_arc278_compiled_where_ops.wat`,
`tests/types/probe_arc227_stone2_defrecord.wat`, `tests/types/structs_builtin_capacity_exceeded.wat`,
`wat-scripts/fmt/run-all.wat`, `wat-scripts/scratch-pad/255-b0-rows-without-handlers.wat`,
`wat-scripts/scratch-pad/255-p6c-w4-arity.wat`, `wat-scripts/scratch-pad/255-stone-h-1a-holon-metadata-arity.wat`,
`wat-scripts/scratch-pad/255-stone-o-ii-apply-special-forms-still-refused.wat`,
`wat-scripts/scratch-pad/arc278-shutdown-cohort-probe-thread.wat`,
`wat-scripts/scratch-pad/probe-execve-argv-cow-leak.wat`, `wat-scripts/scratch-pad/scout-ab-name.wat`,
`wat-tests/holon/Filter.wat`.

### Newly broken (2) — the door's own miss, not noise

Both fail on **`:wat::core::Bytes/to-hex`**:

- `tests/reflection/probe_arc255_ivc_metadata_plain_values.wat`
- `wat-scripts/scratch-pad/probe-slice-one-registry-seam.wat`

Original spelling is the keyword `:wat::core::Bytes::to-hex` (registered with `::`).
Codemod emits `wat.core.Bytes/to-hex`. Before this stone, `ns_to_wat_path` reassembled
`::` and the converted file stayed clean. After, last-segment-is-a-type joins `/`,
which is **not** the registered key. See below.

## What landed — call reconstruction asks the registry

`types::reconstruct_call_path(ns, name, types)`: if `types.is_known_type(":{ns with .→::}")`
then `{ns_kw}/{name}`, else `ns_to_wat_path`. ⛔ Not capitalisation.

Routed: `resolve/normalize.rs` `resolve_namespaced_symbol` (when `sym.types()` is Some),
`macros/expand.rs` symbol-headed macro dispatch. Fallback without TypeEnv stays `::`.

`edn/render.rs` `ns_to_wat_path` is **unchanged**. It cannot import TypeEnv (cycle: types
uses `canonical_identity`). Type *names* (`parse_type_node`, `normalize_type_vector`,
`parse_declared_name`) stay `::` — they are not methods.

Unit test `stone_255_3_registry_decides_the_join`: `wat.core.Option/expect` → slash,
`wat.core/map` → `::`, identity of `wat.core/Option` stays `::`.

## What the door cannot express — measured

**1. Last-segment-is-a-type cannot tell Type/method from a nested type name.**
`Option/expect` must become `/`. `my.Counter/IncrementRequest` (a type *name*) must stay
`::`. Same `ns/name` shape, different positions. A position-blind join cannot express
both, which is why identity stays `::` and only **call** reconstruction asks the registry.

**2. Last-segment-is-a-type cannot tell which member join is registered.**
The wire has two live Type/member spellings (already counted in
`wat-scripts/scratch-pad/255-is-Type-method-ambiguous-on-the-wire.wat`):

| registered key | n | clojure looks like |
|---|---|---|
| `:wat::core::Option/expect` (slash) | **82** | `wat.core.Option/expect` |
| `:wat::core::Bytes::to-hex` (`::`) | **5** | `wat.core.Bytes/to-hex` |

Both are "last segment is a type" to this door. Asking TypeEnv is necessary and
**not sufficient** — the member's join is a second registry fact, and
`reconstruct_call_path` only has TypeEnv, not the function table. The 2 newly_broken
files are this miss, not a sampling artifact.

**3. A name in NAME / annotation position is still walked as a REFERENCE.**
Remaining UR is led by **`:wat::WatAST` (79 occurrences)**. Then declaration names
(`:user::mk-kw`, `counter/dispatch`, `wat.enum/Pure`) and type names used as values
(`:wat::core::i64`, `:wat::time::Instant`, `:wat::rete::Overlay`). 255.2 measured
three predicates over leaves and each **raised** the count (116 / 108 / 114).
**Not started here.** `:restricted-to` resolution **did not close.**

Slash-reconstructed UR that remain (join fired, member still not a resolvable call
head): `:wat::cache::HolographicLru/{put,get,new}` on the converted user type.

## Test count

Predicted **+1** (`stone_255_3_registry_decides_the_join`). `cargo nextest list --release -p wat`: **5315** (was 5314).

## Walls I ran

- `cargo clippy --release --all-targets -p wat --offline -- -D warnings` — **0**
- `one_variant_separator` / `one_param_spec` / `no_loose_string_assert` — pass
- `stone_255_3_registry_decides_the_join` — pass

Floor + workspace clippy + census: **not run**. Do not push. Do not start 8d-ii.
Do not start the position grammar — the builder sequences 255.4, and the member-join
split (finding 2) is a second candidate for that stone.

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-21. **ACCEPTED.**

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5937/5937 passed**, exit 0 |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| no `.wat` converted | ✅ **0** |
| test delta | ✅ **+1**, as predicted |
| ⭐ **THE DELTA — my tree, like-for-like** | **97 → 80. NET −17.** |

**Independently reproduced.** The executor measured **101 → 84** on its tree; the orchestrator
measured **97 → 80** on its own. ⭐ **Different absolute baselines, IDENTICAL net: −17.** Two trees
agreeing on the delta is stronger evidence than either number alone, and it is the first substantial
movement in this arc: **104 → 97 → 80.**

**Classification, my tree:**

| cause | before | after |
|---|---|---|
| `unresolved reference` | 68 | **47** |
| `ReteCheckErrors` | 12 | 16 *(4 files left UR and hit the rete wall next)* |
| `defsurface` · `ProgramBodyEvalFailed` · other | 8 · 3 · 1 | 8 · 3 · 1 |

⭐ **The executor fixed the measurement defect from 255.2 unprompted** — it measured before-and-after
on **one** tree and said so, citing `[[feedback_diff_like_against_like]]` itself.

## ⭐ THE FINDING IS WORTH MORE THAN THE −17

> *"Asking TypeEnv is necessary and **not sufficient** — the member's join is a second registry
> fact, and `reconstruct_call_path` only has TypeEnv, not the function table."*

**Two live member joins on the wire. Verified corpus-wide by the orchestrator:**

| registered spelling | occurrences |
|---|---|
| `Type/method` (slash) | **4,501** |
| `Type::method` (`::`) | **83** |

The door is correct for **98.2%** and wrong for a legacy minority. Both look identical to
*"is the last segment a type?"*.

**The 2 newly-broken files are exactly this miss**, both on `:wat::core::Bytes/to-hex`, both
reproduced by the orchestrator (`orig=0 conv=1`). ⛔ **Not noise, not sampling** — the executor said
so before being asked.

## ⛔ A DESIGN QUESTION FOR THE BUILDER, surfaced by this stone

**Why are there two member joins at all?** 4,501 `/` against 83 `::` is not a design — it is drift.
Two candidate cures for 255.4, and they are different in kind:

1. **Teach the door the second fact** — consult the function table as well as TypeEnv. Keeps both
   spellings alive forever.
2. **Unify the registry on one join** — the 83 become `/` like the other 4,501, and the door needs
   only the fact it already has.

⭐ **(2) deletes the question; (1) preserves it.** This campaign's standing preference is the no-form
rung over the check rung. **The builder rules.**

## What the executor did right, and it is the pattern

- **STOPPED after step 1** as the brief instructed, because the number moved substantially — rather
  than continuing into the position grammar and making two causes unattributable.
- **Reported what the door cannot express**, with counts, *before* being asked. Third stone running.
- **Did not discriminate on capitalisation** — `reconstruct_call_path` asks
  `types.is_known_type(...)`, exactly as the brief required.
- **Scoped honestly**: `edn/render.rs`'s `ns_to_wat_path` is unchanged (it cannot import TypeEnv —
  a cycle), so only **call** reconstruction asks the registry. Type *names* stay `::`, and the score
  says why.

**VERDICT: ACCEPTED.** 2 regressions land with it, characterised and named; the net is −17 and the
next stone's subject is now a measured question rather than a guess.

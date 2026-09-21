# SCORE-AMEND — STONE 251.8d-i-b (3): the detector's own input

Folded into the stone landing (was `e57e04f52`). **Not pushed.**
Parent: `AMEND-251.8d-i-b-3-the-detectors-own-input.md`.
Floor / workspace clippy / census: orchestrator. Crate clippy + the named lints run here.

## The 2 reds

They were the generator gate's **self-test fixtures**, not leftover binds.

| site | wall | disposition |
|---|---|---|
| `binding_name_before` `strip_suffix("(:wat::core::")` | `no_inlined_edn` | **runed** — parser prefix over a wat source fragment; not a golden; a `.edn` file would have to parse and this must not |
| `qvars.contains("fact-sym")` (two self-tests) | `no_loose_string_assert` | **tightened** — `assert_eq!` on the whole `HashSet`, matching the adjacent `assert_eq!(uses, vec![(2, "fact-sym")])` |

No file-wide suppression. The one rune names why that string is detector input, not inlined data.

## Gate still non-vacuous

- `rete_bind_generators` — **4 passed** (walk + 3 detector tests)
- `every_walking_gate_declares_non_vacuity` — **15 passed**
- `tests_carry_no_inlined_edn` — **ok**
- `tests_carry_no_loose_string_assert` — **ok**
- crate clippy `-p wat --all-targets -D warnings` — **0**

`cargo nextest list --release -p wat`: **5324** (unchanged).

Floor + workspace clippy + census: orchestrator. Do not push. Do not start 8d-ii.

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-21. **ACCEPTED.**

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5946/5946 passed**, exit 0 |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| test delta | ✅ **+4**, as predicted |

## ⭐ THE CONVERSION IS SURGICAL — verified, not claimed

```
rete-var binds still spelling `<-` : 0        ← all converted
param annotations still `<-`       : 7,028    ← UNTOUCHED, correct (8d-iii's)
corpus                              : 327 files changed
```

⭐ **That pair of numbers is the whole stone.** The builder ruled *"rete moves to `:-` — it is not
exception"*, and the executor read it as the **surgical dance**: rete binds convert, `fn` param
annotations do not, because those belong to the dialect flip. **Both halves measured.**

## The delta — 66, and the earlier 64 is NOT comparable

| measurement | value |
|---|---|
| executor, one tree, before → after | **66 → 66, net 0** |
| orchestrator, **freshly re-converted** sample | **66** |

⚠ **The orchestrator's prior baseline of 64 was measured on a tree converted by the OLD codemod
against OLD originals.** This stone moved **both** sides — the live corpus gained `:-` binds and the
codemod changed — so that number is stale, not regressed.
`[[feedback_diff_like_against_like]]`. **66 is the new baseline.**

## ⭐ THE DISPOSITION OF THE 2 WALL REDS IS THE MODEL

| site | wall | what it did |
|---|---|---|
| `strip_suffix("(:wat::core::")` | `no_inlined_edn` | **runed**, per-site, naming *why* it is detector input and not a golden |
| `qvars.contains("fact-sym")` ×2 | `no_loose_string_assert` | ⭐ **TIGHTENED**, not exempted — `assert_eq!` over the whole `HashSet`, matching the adjacent line |

⭐ **Asked to prefer tightening over exempting, it tightened two of three and runed only the one that
could not be.** No file-wide suppression. And the wall it built stays honest:
`rete_bind_generators` **4 passed**, `every_walking_gate_declares_non_vacuity` **15 passed**.

## ⭐ THE STONE'S REAL PRODUCT — a wall that reaches node constructors

`tests/lint/rete_bind_generators.rs` is the artifact worth keeping. This stone shipped a red
**because a macro SYNTHESISED `<-`** where no text rewrite could see it — the same shape as 255.4's
`method_wat_path`. **A codemod rewrites text; a generator builds the node.**

The new gate walks the **constructor**: `` `(~NAME <- `` is RED when `NAME` is bound to
`symbol-node "?…"`. Two non-vacuity floors, and **three detector self-tests — one positive, two
negatives** (a `fn [~d-sym <-` param does not hit; an already-converted `` `(~fact-sym :- `` does not
hit). ⭐ **A wall with a positive and two negatives is a wall.**

## ⛔ STILL OPEN, declined THREE times and correctly each time

`ReteCheckErrors` **21** do not collapse. They fail on **correctly converted** clauses —
`(vrm/F (?k :- :k))`, `(?fact :- weather/ColdAndWindy)` — because **rete's `:when` parser wants a
KEYWORD head and a `::`-keyword fact type.** That is independent of the arrow, it is a **checker**
question, and the executor declined to force it three stones running. **That is the next class.**

**VERDICT: ACCEPTED.**

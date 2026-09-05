# SCORE — F2's seven, closed; and the count is a command, not a number

**Driven on `grok-rete` at HEAD `75117bc45`.** Floor `5418 tests run: 5418 passed (1 slow), 21
skipped`, 0 FAIL · `binary_id(wat::lint)` **265 passed** · clippy `--release --all-targets` rc=0,
zero warnings (and it re-checked: `Checking wat v0.1.0` is in the log, not a cache hit).

---

## ⛔ THE HEADLINE: THE COUNT IS THE COMMAND, AND THE MEASUREMENT IS NOT REPEATED HERE EITHER

*"83 of 207"* was TRUE when written. Replacing it with a fresher pair of digits reproduces the
defect the bullet exists to record, so bullet 6 now carries the `grep` that answers it and this
file does not carry the answer. Run the row.

---

## Per bullet

| # | row | disposition | evidence |
|---|---|---|---|
| 1 | `NEXT-STRIKES:1491,1512` — TRACKED DECISIONS premises expired | **CURED** | Both premises re-driven. ① `docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-on-a-value-that-arrives-from-durable-storage.md` exists and RULES ON THE MERITS. ② `src/rete/expr_ir/mod.rs`'s hash-destructure arm carries the closure verbatim; `RETE-OPEN-WORK.md` § ② records `:md::Point{40,2}` → 42 in both positions. Both sections struck IN PLACE. ⚠ **These were the only two F2 citations that had NOT drifted.** |
| 2 | `rust_deps/cache.rs:70` cites a heading that exists nowhere | **CURED — phantom CONFIRMED** | `grep -rn 'the cache panic conversion'` returns the source line and the work-list row, nothing else. Dead twice: the row it MEANT had also left `NEXT-STRIKES` for arc 109. Now names the NOTE by path, no heading, no line. |
| 3 | `purity.rs:216` *"nothing enforces that"* | **CURED — the claim was FALSE** | `completeness_gate` REDs on a dispatch verb with no `intrinsic_meta` classification and no `RULES`/`KNOWN_UNREVIEWED` disposition (`purity.rs`, the `newly.is_empty()` assertion). ⚠ the row's own `:2093` had drifted to `:2115`, which is why the cure is a **symbol with no line**. |
| 4 | `DESIGN-STONE-4b:68` forward edge, no back edge | **CURED** | Two corrections to the row: the stone is `DESIGN-STONE-4b-cascade-fixpoint.md` (`DESIGN-STONE-4b` names neither it nor `P4b-delta-fire`), and `delta.rs:391` had drifted to `:400` in `src/rete/kernel/fire/delta.rs`. Back edge added as a dated **annotation** under § Termination. |
| 5 | `DESIGN-STONE-gather-no-snapshot:53` superseded, neither stone annotated | **CURED** | ⚠ `delta.rs:321` had drifted too (it is a `right_idx` comment now; the site is the `gather_cache` declaration below it, which names `DESIGN-STONE-persist-gather-across-rounds` outright). **Three** annotations, not two — `gather-index-cache` violates it at BOTH the *"round-scoped, never longer"* clause and the *"a cross-round or cross-fire cache"* rejection, and the row named neither. |
| 6 | *"83 of 207 stones name `src/rete/kernel.rs`"* | **CURED — replaced by the command** | See below. |
| 7 | `reachability.rs` coverage prose / orphan block / "four cells" | **CURED, three parts + a fourth found** | (a) two **empty** const arrays carrying present-tense prose about rows that were FIXED; (b) orphan severed from `operands_for` at `d07933919`, **moved back onto its item**; (c) *"four cells"* over a **six**-row table at three sites incl. the test's NAME — all re-worded **number-free**. ⚠ **The row over-claimed on (a):** `:830,832` still land inside `NOT_YET_GENERABLE`'s block and `:820` is a blank `///` line; only (b) and (c) had drifted. |
| — | `wat-scripts/fixes/rete-where-per-type-spelling.wat` ✅ / `remedy/retirement.rs` ⛔ | **left struck, not re-opened** | closed on the merits by prior strikes |

---

## Bullet 6 — the 39 stones, per citation

`src/rete/kernel.rs` was **renamed** to `src/rete/kernel/tests.rs` at `d0973fb14` (2026-08-20) with
`kernel/{arm,census,fire,insert,mod,stratify,wm}.rs` extracted from it; `kernel/tests.rs` then split
to `kernel/tests/` at `f98226353`, `kernel/fire.rs` to `kernel/fire/` at `82b9b5518`, and
`kernel/wm.rs` became `kernel/session.rs` at `e4b554224`. **Every re-point below is to a file grepped
from the tree for the symbol the citing sentence names**, or to the module directory where the
sentence scopes a blast radius across several successors. **No re-point was made on a basename match.**

**52 citations in 39 files.** 50 pointers moved; **2 left untouched** (STOP-2, below).

### Verified to a specific FILE by grepping the symbol in the sentence

| stone | symbol grepped | → |
|---|---|---|
| `beta-is-written-only-for-readers` | `beta_write_read_traffic` | `kernel/tests/fanout_cost.rs` |
| `network-edge-set-semantics`, `keyed-gather:118` | `a8_node_share_fire_census` (def site) | `kernel/tests/node_share_cost.rs` |
| `element-bindings-array` | `struct Element` + `to_transient`/`to_persistent` | `kernel/session.rs` |
| `P1-native-wm` (Files touched) | `to_transient` / `to_persistent` | `kernel/session.rs` |
| `P6` ×1 | `fn keyed_join` | `kernel/fire/mod.rs` |
| `P6` ×2, `P4b:97`, `P12a:50` | `fn fire_fixpoint_delta` | `kernel/fire/delta.rs` |
| `join-extend-no-leftover` | `fn join_extend` | `kernel/fire/mod.rs` |
| `P3` ×2 | `fn hash_join_pass` | `kernel/fire/mod.rs` |
| `P4-delta-fire:54` | `fn eval_fire_once_native` | `kernel/fire/mod.rs` |
| `gather-no-snapshot:76` | `GatherCache`, `ensure_gather`, `any_seeded_keyed`, `seeded_bindings_keyed` — all one file | `kernel/fire/mod.rs` |
| `prod-no-token-clone` | the stone's own body is *"production walks `d_beta`"* → the DELTA production pass | `kernel/fire/pass/production.rs` |
| `accum-fold-the-wall`, `8-custom` | `acc_var_i64` / `accumulate_value` | `kernel/fire/acc.rs` |
| `keyed-gather:60` | the quoted `wm.alpha.get(&from_alpha_id)` shape | `kernel/fire/pass/accumulate.rs` |
| `setup-seen-once` | the stone's body is *"`fire_fixpoint_delta` SETUP"* | `kernel/fire/delta.rs` |

### Re-pointed to a SUBTREE because the sentence spans several successors

`kernel/fire/` — `P10` ×2 (the fire passes), `P12a:75`, `P4-delta-fire:77`, `delta-alpha-indices`,
`gather-index-cache:95`, `setup-fxhash`.
`kernel/tests/` — the eight *"tests only"* blast radii (`prod-leftover-split`, `honest-fire-rank`,
`probe-extend-split`, `drop-memories-split`, `accum-leftover-split`, `cell-rank-after-fanout`,
`cell-rank-after-grid`, `fanout-phase-census`) plus `P1-native-wm:49`.
`kernel/` — `compiled-conditions`, `compiled-where`, `compiled-rhs`, `alpha-discrimination-tree`,
`native-element`, `alpha-is-fire-scoped`, `keyed-gather:135`, `P4b:23`, `P1:15`, `P2` ×2,
`7-exists` ×2, `7strat-native`.

### Dead LINE coordinates dropped (they index a file that no longer exists)

`keyed-gather` `:1939-1952` and `:1904-1961` · `8-custom` `:1234` · `P12a` `~:1556` ·
`P3` *"lines ~573-589"* ×2 · `compiled-where` `:2727` · `7strat-native` `kernel.rs:1345` and
`collect_derived (1047)`.

### ⛔ LEFT UNTOUCHED — dated MEASUREMENTS, not pointers (STOP-2)

Both are rows in a per-file census table. *"`src/rete/kernel.rs` 18"* records what was counted on
the day; it is a record and stays true forever. Deleting or re-pointing it would edit history.

- `DESIGN-STONE-persistent-build-is-a-transient.md:92` — the sound detector's 35-sites-in-6-files table
- `docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-STONE-G-the-value-carries-its-own-names.md:113` — the 97-rustc-errors table

**These two are what the row's `grep` still returns. That is deliberate, and the row says so.**

### ⚠ SYMBOLS that no longer resolve — reported, prose kept

| stone | dead symbol | what happened |
|---|---|---|
| `P10-drop-dead-provenance:60` | `make_token` | gone at `82b9b5518`; the passes build `Token { … }` literals. Noted in place. |
| `compiled-rhs:70` | `rule_rhs_cache` | gone; the arm field is `compiled_rhs` (`kernel/fire/pass/production.rs` already carries a `rune:lint(cited-name-absent)` for the old name). Noted in place. |
| `P1-native-wm`, `P2`, `P4-delta-fire` | `WorkingMemory` | renamed `FireSession` at `82b9b5518`. **Left spelled as written** — it is what the stone recorded; the PATH moved, the recorded name did not. |

**No row was judged unresolvable at the PATH level.** Every one of the 50 pointers reached either a
grepped file or a module directory that is the same module the citation named.

---

## GATED vs CONVENTION — the honest split

| cure | status | proof |
|---|---|---|
| the 50 re-pointed stone paths | ⚠ **CONVENTION** | `no_stale_path_in_doc` does not scan `docs/`. Mutation 1 below. |
| bullet 6's command-form count | ⚠ **CONVENTION** | Mutation 2: replaced with a hard number → **265/265 green, rc=0**. Nothing anywhere gates a number in prose. |
| `purity.rs`'s symbol citation | ⚠ **CONVENTION** | Mutation 3: reverted to `mod completeness_gate` at `:2115` → **265/265 green, rc=0**. `no_stale_path_in_doc` checks a `:LINE` only for a token containing `/`; a bare `:2115` beside a symbol is invisible to it. |
| `reachability.rs`'s number-free wording | ⚠ **CONVENTION** | same class as above; nothing counts table rows against prose. |
| the `expr_ir/mod.rs` re-point | ✅ **partly GATED** | it is a slashed path in a `src/rete` comment, so `no_stale_path_in_doc` checks the path half. The line half is gated only because the line was dropped. |

**Not one cure in this strike is behind a wall.** That is the answer to the brief's question and it
decides whether the class can regrow: **it can.**

### Mutation 1 — ★ re-introduce one cured citation

Applied to `DESIGN-STONE-honest-fire-rank.md`, with `no_stale_path_in_doc.rs` **temporarily** extended
to `("docs", "md")` and an empty comment head (markdown has no comment marker). Result:

```
docs/arc/2026/06/278-rules-engine/DESIGN-STONE-honest-fire-rank.md: names `src/rete/kernel.rs`, which does not exist
```

So the extension **would** catch this class — and it is not extended, so today it does not.
Both the probe and the mutation were reverted and verified **by hash**
(`aaef50264fd1eb8028e0a0de0a033f5e8df163cd6d0856028e4211e44352887e`), not by `git diff`.

⚠ **And the extension would still not see most of this strike's cures**: the extractor only accepts a
token ending `.rs`/`.wat`, so every re-point to a DIRECTORY (`src/rete/kernel/`, `kernel/fire/`,
`kernel/tests/` — 30 of the 50) is invisible to it in either direction.

---

## STOP-3 — extending the gate to `docs/` is CHEAP TO WRITE and NOT CHEAP TO LAND

The code change is four lines (a `ROOTS` row and a `comment_head` arm). The population is not.
Driven with that change in place, over `docs/**/*.md`:

```
comments name 5766 path(s) that do not exist
```

Split against `git ls-files` (a cited path counted as "resolvable" if it is a tracked path or a
suffix of one):

| | citations | distinct paths |
|---|---|---|
| path exists somewhere in the repo — **resolver false positive** (`fire/mod.rs` from a doc cannot walk up to `src/rete/kernel/`) | 410 | 108 |
| path exists **nowhere** in the repo — real rot, foreign repos, or hypotheticals | **5,356** | **1,278** |

Top offenders, all from arcs that predate this one: `src/fork.rs` ×187, `src/macros.rs` ×176,
`src/spawn_process.rs` ×121, **`src/rete/kernel.rs` ×99**, `src/typed_channel.rs` ×84,
`src/thread_io.rs` ×77, `wat/runtime.wat` ×63. Some are not this repo at all
(`holon-rs/src/kernel/holon_ast.rs` ×42).

**Reported, NOT fixed.** That call is the orchestrator's — the deferred-34 fence exists for exactly
this shape.

⚠ **And note the ×99**: bullet 6's population was *stones*, and this strike cured the stones. **The
same dead path is cited ~98 more times in the rest of `docs/`** — `BRIEF-*`, `EXPECTATIONS-*`,
`NOTE-*`, `CURRENT-STATE-*`, `INVENTORY-*`. That is outside the row and is reported, not touched.

# SCORE — STONE 255.82: cutover 5a — STOP-1

**HEAD** `71a577901` (parent `5555c55f2`, the brief's draw ancestor). The refusal is not in the tree. No floor, no clippy, no `census.sh --diff`.

## 1. The dispatch (code agrees with the brief)

`infer_list` is `src/check.rs:2875`. It matches a keyword head at `src/check.rs:2899` and does not match a symbol head there.

A symbol with no `/` is not a reference (`Identifier::is_reference`, `crates/wat-reader/src/identifier.rs:231`: true only when `namespace() != $bound`). In `infer` (`src/check.rs:2329`) a miss in `locals` that is not a hygiene divergence returns `CheckResult::ok(fresh.fresh())` with no error (`src/check.rs:2344-2347`).

That fresh type is not `TypeExpr::Fn`, so the call-head fallthrough (`src/check.rs:6521-6550`) returns another `fresh.fresh()` and no error. The comment at `6549` calls that silent-by-intent.

The keyword `UnresolvedReference` is not emitted from `infer_list`. It is `check_form` in `src/resolve/walk.rs:87-98`, and only when the head is `WatAST::Keyword`, via `is_resolvable_call_head` (`src/resolve/walk.rs:264`). `src/resolve/mod.rs:53-58` states this pass does not check bare-symbol references.

`UnresolvedReference` (`src/resolve/error.rs:13`) is `path`, a `&'static` `context`, and `span`. No remedy field.

## 2. Census (before any refusal)

Print-only classification inside `infer_list`, on the locals map that arm already consults, then deleted. Not committed. `WAT_CENSUS_25582=1 ./target/release/wat --check` on `git ls-files '*.wat'`: **2280 files**, rc histogram **2070×0, 208×1, 2×101**. Unique by class + name + `file:line:col`. Heads `infer_quote` does not walk are absent: `infer_quote` (`src/intrinsic/special/quote.rs:88-109`) returns without entering the argument.

| class | sites | what the checker did |
|---|---|---|
| local | 140 | `locals` hit. 32 names, 58 files |
| poisoned-ctor (`Some` / `Ok` / `Err` as the whole head) | 0 | none reached `infer_list` |
| unbound | 28 | not a local |

Local names (site counts): `f` 37, `overlay` 16, `mk` 13, `fire` 11, `at` 10, `work-fn` 6, `body-fn` 5, `pred` 5, `rows` 4, `ga` 3, `ob-at` 3, `fa` 2, `facts` 2, `fb` 2, `head` 2, `uses-result?` 2, `v` 2, and one each of `bat`, `cat`, `fc`, `g`, `ga-at`, `grant-fn`, `inner`, `mat`, `plus`, `probe`, `prop`, `revoke-fn`, `seed`, `still-fails?`, `worker-init`.

No separate "macro-introduced" class is visible here. This census runs after expansion, and the only binding question `infer` asks a no-slash symbol is the locals map.

The one door, measured by `is_resolvable_call_head` on the symbol table and macro registry of `wat-scripts/scratch-pad/probe-arc278-57-round1a-nine-aliases.wat` after `startup_from_source` (the test that printed this was deleted; not committed):

| head string | door | registry.contains | lookup_special_form | sym.get |
|---|---|---|---|---|
| `def` | false | false | false | false |
| `:wat::core::def` | true | true | false | false |
| `Some` | true | false | false | false |
| `Ok` | true | false | false | false |
| `Err` | true | false | false | false |
| `foozle` | false | false | false | false |

`src/remedy/retirement.rs` has rows `Some` → `:wat::core::Some` (line 153), `Ok` (159), `Err` (162). The door's `is_retired` rung sits after the two false rungs above. Which of the remaining rungs made `Some`/`Ok`/`Err` true was not printed separately.

Embedded wat literals were not walked. The tracked `.wat` set already crosses the cap below.

## 3. STOP-1

A refusal of "symbol head, not a local, door false" hits **28** sites. The cap is 20.

24 are the bare head `def`, and each of those three files `--check` exits 0 today:

- `wat-scripts/scratch-pad/probe-arc278-57-persistentmap-contains-key.wat` — 39:2, 43:2, 52:2, 56:2, 63:2
- `wat-scripts/scratch-pad/probe-arc278-57-round1a-nine-aliases.wat` — 5:2, 6:2, 7:2, 8:2, 9:2, 10:2, 11:2, 12:2, 13:2
- `wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat` — 9:2, 10:2, 15:2, 32:2, 34:2, 36:2, 39:2, 77:2, 82:2, 91:2

Each is `(def :name expr)` at column 2. The checker does not see `def` as a local. The door does not accept the string `def`, and it does accept `:wat::core::def`. That is the design question: how a bare declaration name becomes bound, without a second lookup beside `is_resolvable_call_head`.

The other 4 are the probe `wat-scripts/scratch-pad/arc-255/probe-255.82-a-head-that-names-nothing.wat`, also rc 0: `foozle` 4:66, `my.made.up.thing` 5:68, `wat.core.Option.zzznope` 6:72, and `wat.core.Option.Some` 6:97 (a fourth head in that form, beside the three the brief names).

Baseline named by the brief, read, not re-run: `.floor/2026-10-02T09-46-26Z` Summary `[ 393.583s] 6368 tests run: 6368 passed (24 slow), 24 skipped`. No `RC=` line in that `clean.log`.

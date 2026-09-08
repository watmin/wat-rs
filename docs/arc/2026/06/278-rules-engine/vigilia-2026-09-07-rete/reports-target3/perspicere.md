# PERSPICERE — Cast Report (TARGET 3 — the grid) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## What I swept

```bash
cd wat-scripts/perf/grid
find . -type f | wc -l                                    # 148
find . -type f | sed 's/.*\.//' | sort | uniq -c           # 54 wat / 43 clj / 29 txt / 20 sh / 2 md
find . -type f -exec cat {} + | wc -l                      # 16616
grep -c '<.*<' *.wat | awk -F: '{s+=$2} END{print s}'      # 475
grep -oE '<-|<=|<[^-= )]' *.wat | sed 's/^[^:]*://' | sort | uniq -c   # 2024 "<-", 25 "<=", rest one-off prose digits/letters
grep -o ' :- ' *.wat | wc -l                               # 406
awk '{n=gsub(/ :- /,"&"); if(n>=2) print}' *.wat | wc -l   # 43  ← delta from handed-down 44 (see below)
```

**Deltas from every figure handed to me:**
- 148 files / 16,616 lines — **match, no delta**.
- 475 `<.*<` hits — **match exactly**. Re-derived the breakdown independently: 2,024 `<-` (binding arrow) + 25 `<=` + a long tail of one-off matches (`<n`, `<a`, `<S`, digits, backticks). I hand-read every non-`<-`/`<=` hit (`grep -nE '<[0-9a-zA-Z`]' *.wat | grep -vE '<-|<='`) — every single one is inside a `;;` comment, and every one is either an English comparison (`c<5`, `h<25`, `b<3`) or a prose reference (`S<n>`, `HashSet<String>`). Zero are type brackets. **Confirmed: pure noise, unusable as a trigger.**
- 406 total `:- ` occurrences — **match exactly**.
- "44 lines with two or more `:- `" — **actual is 43, delta −1**. Re-derived twice (regex + `awk` occurrence-counter, same 43-line set both times). Whichever method produced 44 upstream over-counted by one; I could not reproduce 44 by any method.
- Of those 43 lines: **22 are comments, 21 are code** (checked programmatically, not by eye, stripping leading whitespace and testing for a `;;` prefix).
- Of the 21 code lines, **17 are two sibling, non-nested, 1-level type annotations sharing one line** (a param type and a return type in the same `defn`/`fn` signature, e.g. `[v <- (Vector :- [i64])] -> (PersistentVector :- [i64])` — two separate shallow types, not nesting).
- Genuinely **nested** code sites (a `:-` type expression inside the bracket-args of another `:-` type expression) — **exactly 4, all in `where-collection.wat`, all the identical type** — matches the handed-down "4" exactly. Verified there is no 3rd level anywhere (`grep -nE '(:- \[.*){3,}'`-style check came back empty beyond the 2-level set).
- 2 comment-line occurrences of the same nested shape (`where-collection.wat:147`, `:286`) — matches the handed-down claim exactly.

## Trigger analysis

I agree with the correction. `<` is 100% noise in this corpus (binding arrow + comparison + prose), and `:-`-inside-`:-` is the right translation — **but it needs one refinement the brief didn't make explicit**: a line can carry two `:- ` occurrences that are *siblings* (a parameter type and a return type on the same signature line) rather than *nested* (one type's bracket-args containing another type). Only nesting is what perspicere's "2+ levels of generic nesting" actually means — a single type expression that is itself two layers deep. Applying that distinction to all 21 candidate code lines separates 17 false positives (sibling shallow types) from the 4 true positives. This refinement reproduces the handed-down "4" exactly, so I'm confident it's the correct reading, not a new one.

## Findings

**Site (4 occurrences, 1 file, 1 type):**

| file:line | full type |
|---|---|
| `wat-scripts/perf/grid/where-collection.wat:102` | `(:wat::core::PersistentVector :- [(:wat::core::PersistentVector :- [:wat::core::i64])])` — field `grid` of `:wc::Item` |
| `wat-scripts/perf/grid/where-collection.wat:287` | same type, return position of `(:wat::core::defn :wc::build-grid …)` |
| `wat-scripts/perf/grid/where-collection.wat:290` | same type, `acc` parameter of the inner `fn` |
| `wat-scripts/perf/grid/where-collection.wat:291` | same type, that `fn`'s return |

- **Levels:** 2 (`PersistentVector` of `PersistentVector` of `i64`).
- **Role-noun:** `Grid` — a 2-D vector of integers. This noun is **already spoken** in the file: the binding is named `grid` (:102), the constructor is `:wc::build-grid` (:287), and comments at :96–97/:147/:286 describe it in prose as "a grid."
- **Sibling typealias scan:** no exact match, but a structurally identical **precedent exists in the stdlib itself**: `wat/rete.wat:326-327` — `(:wat::core::typealias :wat::rete::ClassFields (:wat::core::PersistentVector :- [(:wat::core::PersistentVector :- [:wat::core::String])]))`. Same 2-level `PersistentVector`-of-`PersistentVector` nesting, only the element type differs (String vs i64). Not directly reusable (concrete, wrong element type, not generic), but it establishes that the stdlib's own authors already treat this exact nesting shape as alias-worthy.
- **Recommendation: mint a typealias.**

Two further textual occurrences of the same shape sit in comments (`where-collection.wat:147`, `:286`) — correctly out of scope; my `;;`-prefix exclusion was verified programmatically (not eyeballed) and is accurate: 22 comment-lines, 21 code-lines, exhaustively enumerated.

**Runes:** zero `rune:perspicere` and zero `rune:` of any kind in the target directory (`grep -rn "rune:" wat-scripts/perf/grid` → 0 hits). Confirms prior art independently.

## The typealias-vs-rune decision

1. **What is it for?** A batched, non-nullable container of integer rows — a 2-D integer grid.
2. **Role-noun?** `Grid`. One word, already the file's own vocabulary at both a binding (`grid`) and a function name (`build-grid`) — this is the easy case the spell describes: the noun exists, only the type lacks a name.
3. **Sibling reuse?** No exact reuse candidate, but `:wat::rete::ClassFields` (wat/rete.wat:326-327) is precedent for exactly this nesting shape being alias-worthy.
4. **Typealias or rune?**
   - `read-once` does not apply — the type recurs **4 times in one file**, not once; naming it removes 3 repeated re-derivations, not a single throwaway.
   - `mumble-alias` does not apply — `Grid` is a clean, single-word noun; nothing about naming this shape reads worse than the shape itself (contrast the spell's own bad-example, `BatchedHolonASTSender`).
   - `intentional-structure` does not apply — nothing at any of the 4 sites needs the reader to see the raw nesting; a defrecord field, a return type, and a closure's accumulator type all just need "this is a grid of ints," which is exactly what a name communicates.
   - **Verdict: mint the typealias**, not a rune.

**Confirmed correct primitive — `typealias`, not `defalias`:** I read both across `wat/*.wat`. `typealias` is exclusively the type-naming primitive (`wat/cache.wat:76` parametric `Lru`; `wat/test.wat:52` non-parametric `TestResult`; `wat/rete.wat:157-327` — `AlphaMemory`, `BetaMemory`, `GroupByMap`, `ClassFields`; `wat/sqlite.wat:42-43`; `wat/fix.wat:905`). `defalias` is a **function/value** alias, never a type — every use aliases one callable name to another (`wat/core.wat:41-44`: `dissoc`→`HashMap/dissoc`, `keys`→`HashMap/keys`; `wat/seq.wat:333,659`: `count`→`length`, `concat`→`Vector/concat`). For naming a type, `typealias` is unambiguous.

**Concrete recommendation** (non-parameterized form, matching `wat/test.wat:52` and the `ClassFields`/`GroupByMap` precedent — not the generic `Lru` form, since this instance is monomorphic to `i64`, not generic over a type variable):

```clojure
(:wat::core::typealias :wc::Grid
  (:wat::core::PersistentVector :- [(:wat::core::PersistentVector :- [:wat::core::i64])]))
```
placed beside the `:wc::Item`/`:wc::Hit` `defrecord`s (~line 104), then used at lines 102, 287, 290, 291.

## What I looked for and did not find

- **`.clj` (43 files):** re-checked every matched line by hand — all `:-`/`<...>`-looking text lives inside `;;` comments describing *wat's* types for the human translating to Clara (e.g. `parametric-erasure.clj:13`, `where-record.clj:19`, `where-control.clj:17`). Clojure itself is untyped and carries zero real type annotations anywhere in the directory. Trigger does not fire — confirmed with evidence, not assumed.
- **`.sh` (20 files):** every `<` is a heredoc (`<<`), redirect (`<`, `<(`, `<"`), `<=` comparison, or a `<placeholder>` in usage text (`<axis>`, `<n>`). No type syntax exists in shell. Trigger does not fire.
- **`.md`/`.txt` (2 md, 29 txt):** `CLARA-TRANSLATIONS.md` has three informal `<T>`-style mentions in prose (`PersistentVector<T>`, `HashMap<String,i64>`, `(PV<T>) -> R`) — each only 1 level deep, and prose rather than a code type annotation either way. None reach the 2+ threshold.
- **A 3rd nesting level anywhere in the corpus:** none found; the deepest nesting present is exactly 2, confined to the one type.
- **Any other nesting mechanism `wat` might use besides `:-`:** checked `wat/core.wat`, `wat/deporder.wat`, and the `typealias`/`defalias` grep above — no alternate generic-parameter syntax exists; `:-` is the only parameterization form.
- **Rune exemptions:** none, anywhere in the directory — matches the six prior wards.

**FINDINGS**

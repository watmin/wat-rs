# MEASUREMENT — is `.wat` source CLOJURE-readable? (2026-09-19, BAR CORRECTED 2026-09-20)

## ⛔⛔ THE BAR IN THE ORIGINAL VERSION OF THIS DOCUMENT WAS WRONG

This document was first written against *"all .wat files are fully **edn** readable."* **The builder
corrected the bar after his own assumption broke:**

> *"we are not going to drop clojure's macro forms - wat's source code **by definition cannot be pure
> edn** - so..... i don't know if your final item is acceptable"*
> *"clojure's source code is not edn - my assumption has been broken - **we will continue to use
> .wat indefinitely**"*
> *"that said... we are seeking to be **clojure compliant source code even if clojure cannot run
> us**."*

**The correct oracle is therefore `clojure.core`'s READER, not `clojure.edn`.** Measured 2026-09-20,
`(binding [*read-eval* false] (read-string …))`, Clojure 1.12.4:

| construct | `clojure.edn` | **Clojure's reader** |
|---|---|---|
| `` `x `` · `~x` · `~@x` · `@x` · `` `(a ~b) `` | ❌ `Invalid leading character` | ✅ **reads** |
| `#(+ 1 %)` · `#"re"` · `#'x` | ❌ `No dispatch macro` | ✅ **reads** |
| `'x` · `^:m x` · `#_` · `#{}` · `#inst` | ✅ | ✅ |
| `a/b/c` · `clojure.core//` · `a:b` · `a#b` · `1/2` · `x'` | mixed | ✅ **all read** |
| `{:a 1 :a 2}` | ❌ | ❌ **`Duplicate key: :a`** |
| `my.trading.Price/0` | ❌ | ❌ **`Invalid token`** |

⭐ **CONSEQUENCES — three items in earlier drafts are now DELETED, not rescheduled:**

1. ⛔ **The quasiquote → list-form stone is KILLED.** `` ` ``/`~`/`~@` are Clojure-readable. The 105
   files need **nothing**. The orchestrator proposed that stone because it had adopted the wrong
   bar; the builder rejected it on exactly that ground.
2. ⛔ **The file-extension question is CLOSED, not deferred.** `.wat` stays indefinitely. The rename
   to `.edn` was premised on wat source being EDN; the premise is gone. *(The 1,274-occurrence blast
   radius measurement stands recorded, unused.)*
3. ⭐ **Regex (`#"…"`) is NO LONGER a future problem.** Clojure's reader takes it. The builder's
   *"regex needs to be in there"* and the compliance bar do not conflict.

⚠ **`wat-edn` is UNAFFECTED.** 218.7 made it a spec-correct **EDN data** reader and that stands —
`wat-edn` is not the thing that reads `.wat` source. Two layers, as ruled: `wat-edn` = EDN data,
`wat-reader` = Clojure-dialect code. **Only `wat-reader` answers to this bar.**

## ⭐ ONLY TWO SHAPES NOW BLOCK CLOJURE-COMPLIANT SOURCE (besides `::`)

Both were already found below and both survive the bar change, because Clojure refuses them too:

| shape | files | Clojure says |
|---|---|---|
| duplicate key in a literal | 2 | `Duplicate key: :a` |
| `T/0` positional accessor | ~3 | `Invalid token: my.trading.Price/0` — a name may not begin with a digit |

⇒ **The path to Clojure-compliant source is: 8d (the `::` flip) + the 3 codemod classes + these 5
files.** No macro-system change. No extension change.

---

# The original EDN measurement, kept — the file counts are unchanged by the bar

⚠ Re-run under Clojure's reader: **identical numbers** (44/2190 today, 2016/2140 converted). The
reader-macro difference is **masked**, because the 99 class-B files still carry `::` and fail on
`:wat::core::defmacro` long before a `~` is reached. That is why the table below still holds.


**The builder's bar, verbatim:** *"the requirement i have for wat… is that all .wat files are fully
edn readable."* · *"wat-edn needs to be a correct edn impl."*

**Instrument:** real `clojure.edn` (`/usr/local/bin/clj`, Clojure 1.12.4) reading every form in each
file to EOF via `PushbackReader` + `edn/read` with a `:default` tag handler. ⛔ Not a grep, not
`wat-edn`, not a reading of the spec — the reference implementation, run here.

## THE BASELINE — 2.0%

| corpus | EDN-readable |
|---|---|
| tracked `.wat` today | **44 / 2190 — 2.0%** |
| after 8d's conversion (the 2,140 the codemod is aimed at) | **2016 / 2140 — 94.2%** |

Today's dominant blocker is exactly what 8d retires:

```
1083  Invalid token: :wat::core::defn
 366  Invalid token: :wat::core::defrecord
 154  Invalid token: :wat::core::defsurface
```

⭐ **This is the strongest justification 8d has.** Not "keywords shouldn't have `::`" — *"2% of our
source is readable by the reader we claim compliance with, and the flip takes it to 94%."*

## ⛔ BUT 8d PLATEAUS AT 94% — AND THE REMAINING 6% IS STRUCTURAL

124 files remain unreadable after conversion. Separated:

| | count | |
|---|---|---|
| the codemod's own refusals (classes B + C) | **119** | already tracked in the 8d brief |
| **codemod SUCCEEDED, output still not EDN** | **5** | ⛔ **new — two genuine classes** |

### Class α — quasiquote/unquote is NOT EDN, and no codemod can fix that

Measured against `clojure.edn`:

| form | clj | |
|---|---|---|
| `'foo` | ✅ reads as `(quote foo)` | |
| `^:m foo` | ✅ reads, meta attached | |
| `#_1 2` | ✅ reads as `2` | |
| `` `foo `` | ❌ `Invalid leading character: \`` | **syntax-quote** |
| `~foo` · `~@foo` | ❌ `Invalid leading character: ~` | **unquote / splicing** |
| `@foo` | ❌ `Invalid leading character: @` | **deref** |

⛔ **`~` appears in 162 tracked `.wat` files, `~@` in 35.** These are the same files as 8d's class-B
refusals (`fix-text-apply` "refusing to splice" on a reader-synthesized node whose 1-char source span
disagrees with its 19-char `ast-name`) — **one root cause wearing two symptoms.**

**To reach the builder's bar the macro syntax must become an EDN-representable form** — e.g.
`(wat.core/unquote x)` in place of `~x`, `(wat.core/syntax-quote …)` in place of `` `… ``. Those ARE
EDN. ⚠ That is a **surface change to the macro system**, not a spelling flip, and it is *not* in 8d's
scope as drawn. **The builder rules whether it opens.**

⭐ Note `'`, `^` and `#_` are safe: `clojure.edn` reads all three. So the bar does not require
retiring quote.

### Class β — two shapes the conversion leaves behind (5 files)

| shape | files | why EDN refuses it |
|---|---|---|
| **`T/0` positional accessor** — `my.trading.Price/0`, `b.Price/0`, `MixAmount/0`, `LocalAmount/0` | 3 | *"Symbols begin with a non-numeric character."* A name part of `0` is an illegal symbol. |
| **duplicate key in a literal** | 2 | `Duplicate key: 1` — `tests/collection/probe_arc215_…`, `probe_arc216_stone1_hashset_roundtrip` |

Both need a **ruling**, not a codemod: positional accessors need a non-numeric spelling, and the two
collection probes either change their data or wat accepts non-EDN there.

⚠ Also present and NOT counted: `reductions/2`, `reductions/3` — same numeric-name shape, in files
that failed earlier for another reason. **The 5 is a floor, not a total.**

## ⛔ INSTRUMENT WARNING — this document nearly shipped two contaminated counts

A first pass at "how many files use `T/<digit>`" returned **128 files**, topped by `arc/2026` (96) and
`pad/255` (48) — **file paths in comments**. Stripping comments took it to 11, topped by `pad/277`
and `pad/109` — **file paths in STRING LITERALS** (finding 33's class). **Neither number is in the
table above**; the table reports only what `clojure.edn` itself refused.
`[[feedback_ask_the_tool_that_owns_the_fact]]`, third recurrence in one session.

## What this changes

1. **8d's justification is much stronger** — and its *ceiling* is now known: **94%, not 100%.**
2. **8d's class B is not a codemod bug.** `~` is not EDN; no span-edit fixes that. The class-B cure
   in the 8d brief ("teach the rule about reader macros") makes the codemod *survive* those files —
   it does not make them EDN-readable.
3. **A fourth stone is implied** (macro syntax → list forms) that nobody has opened. **The builder's
   ruling, not the orchestrator's** — `[[feedback_opening_an_arc_is_the_builders_ruling]]`.
4. **`clojure.edn` is not the EDN spec.** It reads quote, metadata and multi-slash symbols, none of
   which the spec sanctions. So *"wat-edn must be a correct EDN impl"* and *"wat must accept
   everything clj accepts"* (218.7's ward doctrine) are **different bars**, and 218.7 says which one
   governs is the builder's call.

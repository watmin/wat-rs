# ★ RULING (builder, 2026-08-19) — the numerics rehome is a **SPLIT**: the TYPE and the OPS go to different homes

> *"the numerics.. they get partially rehomed....*
> *`wat.type/{i64,f64}`*
> *`wat.core.{i64,f64}/{+,-,*,/,...}` ... these operate on `wat.type/` ...."*

Recorded before it is drawn, because **every home carved so far (#1 Bytes · #2 time · #3
kernel-stdio) moved a whole namespace to one destination.** This one does not. A stone that
discovers that mid-strike will mis-scope itself.

## The two destinations

| today | destination | what it is |
|---|---|---|
| `:wat::core::i64` · `:wat::core::f64` | **`wat.type/i64` · `wat.type/f64`** | the TYPE — an annotation, a parameter's declared shape, a return |
| `:wat::core::i64::+` · `f64::round` · … | **`wat.core.i64/+` · `wat.core.f64/round`** | the OPERATIONS — and they *operate on* `wat.type/` values |

**36 operations**, measured at HEAD:

```
i64 (17)  + - * /  < <= > >= = not=  mod quot rem  to-bigint to-f64 to-rational to-string
f64 (19)  + - * /  < <= > >= = not=  abs clamp max max-of min min-of round to-i64 to-string
```

## ★ THE MEASUREMENT THAT SHOULD DECIDE THE STONE'S SHAPE — the type outnumbers the ops 4:1

Corpus census over `wat/` + `wat-scripts/` + `tests/`, 2026-08-19:

```
                    TYPE uses          OP uses
                    -> wat.type/       -> wat.core.<t>/
  i64                 5,381              1,296
  f64                   360                158
  ─────────────────────────────────────────────
  total               5,741              1,454
```

⚠ **The naive count is a TRAP and I nearly wrote it down.** `grep -o ':wat::core::i64'` returns
**6,678** — because it also matches the prefix of `:wat::core::i64::+`. That single number conflates
precisely the two populations this ruling separates. The split above uses a negative lookahead on
the trailing `::`, and it **reconciles**: 5,381 + 1,296 = 6,677 against a 6,678 total (one boundary
case), which is the non-vacuity check that the two sub-counts partition the whole.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

**So the numerics' largest migration is not the numerics home at all — it is the TYPE namespace.**
Four out of every five `:wat::core::i64` sites are an annotation heading for `wat.type/`, not an
arithmetic call heading for `wat.core.i64/`. `:wat::core::i64` is the #1 spelling in
`251/CENSUS-the-illegal-edn-form-classes.md`'s steep head (4,280 there, over a narrower path set),
and this ruling says most of that weight lands in 251's namespace, not 255's home.

## What this ruling does NOT decide

- **It does not set the carve order.** Builder's ruling the same day: *"let's continue on kernel"* —
  the numerics are named, not scheduled.
- **It does not rule on the other `core::` families** (`string::`, `bool::`, `keyword::`,
  `Record`/collections). Whether they split the same way — type to `wat.type/`, verbs to
  `wat.core.<t>/` — is untested by this ruling and must not be inferred from it.
- **It does not rule the arity hazard.** `251/CENSUS`'s CLASS 2 finding stands: after the symbol
  flip `(f HashMap<K,V>)` reads as valid EDN and silently changes arity 2→3. The numerics carry
  angle-bracket parametrics rarely, but `Vector<wat::core::i64>` is 460 occurrences of exactly the
  shape that hazard names.

## Why it is recorded HERE

The home question is 255's (`wat.core.i64/` is a registry home). The `wat.type/` destination is
**arc 251's** namespace (`251-types-as-forms`, *"the Clojure-faithful symbolic surface"*). So this
ruling straddles two arcs, which is itself the reason to write it down rather than leave it in one
stone's head — **whichever arc strikes first must know the other half exists.**

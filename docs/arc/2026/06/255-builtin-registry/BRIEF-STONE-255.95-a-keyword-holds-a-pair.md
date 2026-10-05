# BRIEF — STONE 255.95: a keyword holds a pair

**Drawn 2026-10-04 against `main` @ `0b2dcc1e2`.** **Executor: grok via pulsare, working solo** (it runs the floor).
Commit locally on `main` (`git add -- <paths>`, never `-A`; `git status` clean before the floor you report);
**do not push**.

## The rulings this stone rests on

- **The pair (2026-10-04):** a symbol is `{namespace, name, scope}`; no string is an identity; a string exists only from
  `"{namespace}/{name}"`; `::` is removed everywhere.
- **KW1 (2026-10-04, builder):** *"yes — namespaced keywords are a thing to support."* Clojure's model: a **symbol names
  something**; a **keyword is self-evaluating data** and may carry a namespace (`:my.ns/f`). Both hold the same pair (a
  keyword wraps a symbol's pair). `:wat::kernel::__peer_crashed__` becomes `:wat.kernel/__peer_crashed__`. The builder
  expects **very few, likely zero**, namespaced data keywords in use today.
- **K1 (2026-09-28):** canonicalize where a name enters; never reconcile two spellings at a compare site. (255.93 was
  rejected for breaking this: `WEIGH-STONE-255.93-…`.)

## What is known (255.94, `WEIGH-STONE-255.94-…`)

- `Value::wat__core__keyword(Arc<String>)` (`src/value/value.rs:61`) holds the text with its leading `:`; 91
  construction sites.
- On `wat-tests`, a value-position `::` keyword resolved to a function 238,799 times, to `Option` 118,781, to a unit
  variant 1,685, and **stayed a keyword value 88,952 times, spellings not recorded**.
- `keyword/from-string` (`src/runtime.rs:5202`, `:5222`) builds 773 name-shaped paths per stdlib startup; reflection
  (`src/reflect/verbs.rs`, `metadata-of` at `src/runtime.rs:7690-7956`) 292.
- The sentinel `:wat::kernel::__peer_crashed__` (`src/kernel/peer.rs:256`) is real data.
- `Name::enter` refuses `<` (`is_rendered_type_text`, `crates/wat-reader/src/identifier.rs:219`): a leftover of the
  retired `<T>` rendering. `wat.core/<` is a name.

## The work

1. **Measure first, committed as the SCORE's first table:** the 88,952 with **spellings on**. A scratch counter (removed
   before commit) on every `::` keyword that stays a keyword value at run time, over `wat-tests/` and the `tests/`
   sample 255.94 used: spelling, count, the site that made it. Classify each: **NAME** (it denotes something declared,
   and the program looks it up or would) or **DATA** (an opaque token: compared, stored, printed, never resolved).
   **Every DATA spelling is listed for the builder.** If the DATA list is not tiny (more than 10 spellings), STOP after
   the table.
2. **`<` is a name character.** `is_rendered_type_text` marks a rendered form by `(` only. `:fn(…)` stays refused (the
   two KEEP negative proofs).
3. **A keyword holds a pair.** `Value::wat__core__keyword` holds a keyword that is a `Name` or **unqualified**
   (`:k`, `:else`: no namespace; say how you represent "no namespace" so it is never confused with `$bound`). Its one
   printed form is `:{namespace}/{name}` or `:{name}`. Read `:a::b::c` / `:a::b/c` as `{a.b, c}` through the one
   `Name::from_keyword`; read `:a.b/c` as `{a.b, c}` directly; a keyword's equality and hash are the pair. Every one of
   the 91 construction sites builds through that one constructor, never by `format!(":{…}")`.
4. **Names built at run time are symbols, not keywords.** `keyword/from-string`'s 773 startup paths are names being
   forged as keywords: find each caller in the stdlib and say what it builds a name **for** (a lookup, a dispatch, a
   message). The ones that look a name up build a symbol (the symbol constructor, by pair) instead, so no lookup ever
   takes keyword text. Reflection that answers "what is this called" (`variant-parent-of`, `compose-variant`, the type
   records, `metadata-of`'s `:name`) returns a symbol. **If a stdlib caller needs a ruling** (a keyword that is both a
   lookup key and a stored token), list it and STOP on it.
5. **The sentinel and every DATA keyword from item 1** are spelled `:ns/name`.

## Not in this stone

Converting `::` keywords in name positions into symbols at the reader (the next stone, which this one makes safe);
deleting the registries' text bridge (`get(&str)`, the spelling index; it dies when nothing passes text); `flat`.

## Gates

| what | how | expected |
|---|---|---|
| the measure | item 1's table | every `::` keyword value classified; DATA listed |
| keyword equality | a test: `:a::b::c`, `:a::b/c`, `:a.b/c` are one keyword value, `=` and in a set; `:k` is unqualified; a keyword and the symbol with the same pair are **not** equal (a keyword is data, a symbol a name) | green |
| no forged names | every remaining caller of `keyword/from-string` in `wat/`, with what it builds | none builds a name it then looks up |
| release floor | `scripts/floor.sh`, foreground, nothing else running, `git status` clean | all passed. **Test-name set against `.floor/2026-10-04T07-52-21Z`: MISSING 0** |
| cost | the fuzz deftest alone, six on `0b2dcc1e2`, six on yours | mean ≤ before (report both) |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| ignores | the SEAM's ledger command | 18 |

## Reds and STOPs

- A red caused by this stone's own change is the work: captured **verbatim**, cured, a **new** floor. Any other red is a
  STOP. Never re-run unchanged code for a green. **A golden that changes only because a keyword now prints `:a.b/c`
  is re-captured only after showing, as data, that it is the old value with that spelling changed.**
- **STOP-1:** more than 10 DATA spellings in item 1. Table, then STOP.
- **STOP-2:** a keyword that is both a lookup key and a stored token (item 4). List and STOP on it.
- **STOP-3:** any change that would render a `Name` with `::`, decide a name by letter case, or keep two spellings of one
  name as two keys. Those were ruled out; if the work seems to need one, STOP and say where.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. No string compare stands in for identity; no name or keyword is assembled by `format!`
except their one printer. No time limit is raised. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`.
Never write a number, file:line or example you did not measure. If this brief contradicts the code, the code wins: say
so. Write `SCORE-STONE-255.95-a-keyword-holds-a-pair.md` beside this brief, commit it, **do not push**.

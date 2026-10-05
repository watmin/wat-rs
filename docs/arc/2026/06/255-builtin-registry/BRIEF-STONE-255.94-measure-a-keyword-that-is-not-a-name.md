# BRIEF — STONE 255.94: measure — a keyword that is not a name

**Drawn 2026-10-04 against `main` @ `84bc32638`.** **Executor: grok via pulsare, working solo.** A **measuring** stone:
no edit under `src/`, `crates/`, `wat/`, `tests/` or the corpus (a scratch counter is built and removed before the
commit). Commit locally on `main` (`git add -- <paths>`, never `-A`); **do not push**.

## Why

255.93 is rejected (`WEIGH-STONE-255.93-…`): reconciling `::` text with `Name` at lookup sites grew string machinery.
The next stone puts the door **at the reader**: a keyword that is a **name** (`:a::b::c`, `:a::b/c`) is read as its pair
`{a.b, c}` through the one `Name::from_keyword`, so nothing downstream holds a `::` name. A keyword that is **data**
(`:k`, `:else`, `:error`) stays a keyword. That split is only safe if **no `::` keyword is data**. This stone answers
that, with numbers.

## The work

Build the instrument as an example (`examples/`, as the census is; it links `wat-reader`'s parser, never a regex) and a
scratch runtime counter. Report each table with its denominator.

1. **Every `::` keyword in source, by the position the parser puts it in.** Over the `.wat` corpus and `wat/` (main's
   tree: the corpus is keyword-spelled, the stdlib converted), and separately the wat inside Rust string literals that
   `wat-fix-rust` reads. Positions: call head; declaration name (the name a `defn`/`defmacro`/`defrecord`/… declares);
   type position; match/variant pattern; a reference passed as an argument (a function or type named as a value);
   inside `quote`/quasiquote; a map key or value; a fact field; a rete clause; other (each listed). Counts by position,
   top files per position.
2. **Every `::` keyword that is a value at run time.** A scratch counter (behind a feature, removed before commit) on
   every `Value::Keyword` whose text holds `::`, recording **where it was made** (read from source as a literal, built by
   `keyword/from-string` or another intrinsic, returned by reflection such as a type-of or name-of verb, produced by a
   macro) and **what is done with it** (compared, hashed into a collection, printed, passed to a lookup). Run it over
   `wat-tests/` and a sample of `tests/` (say which), and report by origin and by use.
3. **The intrinsics that hand a name to a program as a keyword:** every intrinsic whose result is a `Value::Keyword`
   holding a name (reflection, `type-of`, errors carrying a `:head`, `keyword/from-string`). File:line, what it returns.
4. **The verdict per position and per origin:** NAME (it denotes something declared; the reader may read it as a pair),
   DATA (it is compared or stored as an opaque token and declares nothing), or MIXED (both, at different sites: each
   site listed). Every MIXED or DATA row is a **ruling** the builder needs before the reader stone is drawn.

**Prove the instrument:** a scratch file with one keyword in each position, and one run-time keyword of each origin,
each classified right.

## Gates

| what | how | expected |
|---|---|---|
| no edits | `git diff 84bc32638 --stat` | only the example, its output TSV under the arc directory, the SCORE |
| the instrument | its planted proof | every position and origin classified right |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

No floor (nothing under test changed). **STOP-1:** a keyword whose position the parser cannot decide. List it and
keep counting. A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number,
file:line or example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.94-measure-a-keyword-that-is-not-a-name.md` beside this brief, commit it, **do not push**.

# AMEND-2 — STONE 255.95: a symbol is a value

**Drawn 2026-10-04.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `e5a746ec0`.
Commit locally on `main`; **do not push**.

## Accepted from the STOP

Item 1: the 88,952 value-position `::` keywords are **58 spellings, every one a NAME; DATA none** (the builder expected
zero). STOP-2 was right: `defservice` (`wat/service.wat`) passes names as keyword data and resolves them later with
`apply`: the serve function (`:981` built, `:2538` declared, `:2508`/`:2655`… passed to launch), `Status::Started`
(`:1158`), the generated `start$impl` pairs (`:2590`, `:2592`), the service `:name` on each process record (`:2650`).
They are names that must travel as data: a child process receives data, never a function. `Value` has no symbol variant
(`src/value/value.rs:61` has only the keyword).

## The ruling (builder, 2026-10-04): S1, a symbol is a value

Clojure's model: **a symbol is first-class data**. `Value::Symbol` holds the pair (`Name`, the same type keywords hold
under KW1); it prints `ns/name` (or the bare name for an unqualified one); it is pure data (it crosses a comm and a
process boundary); **a symbol and a keyword with the same pair are not equal**. Nothing resolves a symbol value by
accident: **resolving is explicit**: `apply` on a symbol value resolves it to the function it names at that moment
(`eval_apply`, `src/runtime.rs:5314`), and a `resolve` verb if the stdlib needs one without calling. Built by the symbol
constructor (`wat.core/symbol` from a string, split at the first `/`; or from a namespace and a name) and by quoting a
symbol where quote yields a value. `keyword/from-string` stops being used to build a name.

## The work, in order

1. **The symbol value** as ruled, with its type in the type system, its printer, EDN round trip, equality and hash over
   the pair, and the comm purity wall admitting it.
2. **`defservice` speaks symbols:** the serve name, the status variant, the `start$impl` names and the service `:name`
   are symbol values; launch and the thread/process tiers resolve them with `apply`. Every other `keyword/from-string`
   caller under `wat/` (`bracket.wat`, `core.wat`, `fix.wat`, `query.wat`, `telemetry/span.wat`,
   `rete/oracle/accum-pass.wat`, `rete/compile.wat`): say what each builds and why; a built **name** becomes a symbol;
   genuine keyword data stays a keyword. Reflection that answers "what is this called" returns a symbol.
3. **Then 255.95's items 2, 3 and 5 as drawn**, with the `$bare` amendment: `<` is a name character; a keyword holds a
   pair (`$bare` for unqualified, one constructor for all 91 sites, prints `:ns/name` or `:k`); the sentinel is
   `:wat.kernel/__peer_crashed__`.

## Gates

255.95's gates stand, plus: a test that a symbol value crosses a process boundary and `apply` resolves it on the other
side; a test that `(= 'a.b/c :a.b/c)` is false; and the startup counter from item 1, re-run once, showing **0**
name-shaped keywords built by `keyword/from-string` during a stdlib startup (it was 773). The floor's test-name set
against `.floor/2026-10-04T07-52-21Z`: MISSING 0. Fuzz cost six and six against `0b2dcc1e2` (or `983f37362`, the same
code). Clippy, ignores 18.

STOP-3 of the brief stands (no `::` rendering of a `Name`, no case rule, no two keys for one name). A red caused by this
amend's own change is captured verbatim, cured, and followed by a **new** floor; any other red is a STOP. Append to the
SCORE, commit, **do not push**.

# BRIEF — STONE 255.17: the last type argument is not forgotten

**Drawn 2026-09-24 against `main` @ `61d48f793`.** Floor 6027/6027, clippy 0, census `no STOP-8`, delta
**3 / RECOVERY 0**. **Executor tier: Opus.** Background: `FINDING-a-matched-field-forgets-its-type-argument.md`
(D2, found by the alias probe).

## The lie — `--check` rc=0, runtime TypeMismatch

```wat
(:wat::core::defn :probe::k :- [T] [o <- (:wat::core::Option :- [T])] -> :wat::core::i64
  (:wat::core::match o [:wat::core::Option.Some {:value v} v] [_ 0]))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::i64::+ 1 (:probe::k (:wat::core::Option.Some {:value "hello"})))))
```

`--check` **rc=0**. Running it fails: `":wat::i64::+: expected i64, got wat::core::String"`.

## Measured by the orchestrator — the mechanism is narrower than the FINDING says

Each row: a generic `defn` returns one field as `-> :wat::core::i64`. The probes live in the session
scratchpad under `d2/`. **A correct checker refuses every row**; rc=0 is the lie.

| probe | the field's type argument | rc |
|---|---|---|
| `(R :- [A B C])`, field `x` / `y` / `z` | first / middle / **last** | 1 (`body produces :A`) / 1 (`:B`) / **0** |
| `(E :- [T S])` enum, arm `A` / arm `B` | first / **last** | 1 / **0** |
| `(Result :- [T E])`, `Ok` / `Err` | first / **last** | 1 / **0** |
| `(Result :- [T String])`, `Ok` | first (the last is concrete) | 1 |
| `(Option :- [T])` · `(Option :- [U])` · `(E :- [T])` · `(R :- [T])` | the only = **last** | **0** · **0** · **0** · **0** |
| `(R :- [String T])`, field `w` | **last**, a type variable | **0** |
| `(R :- [T String])`, field `w` | last, **concrete** | 1 (`body produces :wat::core::String`) |
| `[o <- :T]` returned bare (control) | not inside an argument list | 1 (`body produces :T`) |

⭐ **The property: the LAST argument of a parametric type annotation is lost when it is a type
variable.** It becomes a hole (`_`, as the alias probe saw: `(GetResponse :- [_])`) that unifies with
anything. The first and middle arguments, a concrete last argument, and a bare type variable are all
checked correctly. It holds for records, user enums and the builtin `Option`/`Result` alike, so it is
**not** match-specific. Record accessors lie too.

⚠ **The prime suspect (not located): a string operation on the argument list.** It looks like a flat
split or a comparison where the last element keeps a trailing `]`, so `"T]"` fails to match the rigid
`T` and falls through to a fresh variable. It could also be an off-by-one. That is CLAUDE.md's recurring
class: *a string comparison with one side normalised and the other not*. **Locate it by
measurement. Do not trust this guess.** Dump the substitution or the rendered type for one of the rows
above.

## Why it matters

- Every generated service client is a generic `defn` returning `(RecvOutcome :- [(<S>::<Op>Response :- [… V])])`.
  Its **last** argument is a type variable, so every client is checked through this hole.
- It blocks the alias's candidate 2: its negative controls would pass without testing anything.
- It is the same class as 255.16: the checker passes a program that the runtime then fails.

## The work

1. **Locate the site** (measure it, as above), and name it in the SCORE.
2. **Fix it at the root.** The last argument keeps its type variable, rigid, just like the others. No
   special case for "the last one": if the cause is a string split, replace it with the parsed
   `TypeExpr` args.
3. **Fallout is a finding, not a workaround.** Code that only type-checked through the hole now goes
   red. For each file, classify it: (a) a genuine type error the hole hid, (b) a checker case the fix
   broke. **Fix no (a) in this stone.** Report the list.
4. Commit the probes above as a test file (`tests/types/probe_arc255_17_last_type_argument.rs` plus
   fixtures): every row refused, and the correct twins accepted (e.g. `-> :T` returning a `T` field;
   `-> :S` for arm `B`).

## STOP triggers

1. **Fallout of more than 10 files, or any (b)** → STOP, report the list with each first error
   verbatim. The size and the classification are the builder's to rule on.
2. **More than one site** (the last argument is lost in two places that must both change) → report
   them before fixing the second.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| every row in the table above | rc=1, `body produces :<the var>` |
| correct twins (`-> :T`, `-> :S`, …) | rc=0 |
| the runtime reproducer | `--check` rc=1 |
| floor · clippy · census · delta | 6027 + the new tests, all green · 0 · `no STOP-8` · NEW 3 / RECOVERY 0, **or** STOP-1 with the list |

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name
  the arm. (`harvest_wrap_split`: cite it and report it.)
- ⭐ Prove every probe can say BOTH words. Strip ANSI. Carry a control each run.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally; **do not
  push.** Spawn no subagents.

## Out of scope

D1 (annotation arity is unchecked); the alias; step 2.

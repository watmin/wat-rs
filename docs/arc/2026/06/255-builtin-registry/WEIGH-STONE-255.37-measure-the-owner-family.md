# WEIGH — STONE 255.37: measure the owner family — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `282f1e8f7`** (the SCORE only; the worktree is removed, and nothing
landed). The worktree diff was saved outside the repo; the orchestrator copied it to
`probes-255.37/255.37-worktree.diff.txt` so it survives. Weighed on 2026-09-25.

## What was measured

1. **`Spawned` as a real, parametric type.** An empty `defstruct … :- [S R] []` is refused
   (`UnconsumedTypeParam`). The form that carries the parameters and stays impure is
   `(:wat::core::defstruct :wat::spawn::Spawned :- [S R] [sent <- S recv <- R])`. The fields are **not a
   runtime layout**; they write the discrimination the param-spec demands. ⚠ Whether a phantom-field struct
   is the honest declaration of a *family marker* is a **question for the build stone**: it reads like a
   carrier that holds an `S` and an `R`.
2. **The family rule:** `project_peer_io`, `select`, `poll` and `close` accept a head that derives `Spawned`,
   or `Peer` through `is_peer_head`. Head-name compares in `check.rs`: **16 → 6** (the brief's 33 was wrong
   for this file). **The heresy ledger shrank 208 → 198.**
3. With the two `derive … Peer` edges gone: **a send or recv on a `Thread`/`Process` still type-checks.**

## Classification of every red

| class | measured |
|---|---|
| (a) an owner handle declared as `Peer` | **2 stdlib seams:** `spawn-runner`'s signature (`bracket.wat:241/321`) and `Launched.handle` (`spawn.wat:329`, via its constructors at `:566/:630`). Plus `recv-all`/`recv-all-loop`'s `Peer` parameter (3 program-contract tests), and one test's `counter-proc` ops |
| (b) a vector mixing the two families | **none** |
| ⭐ **(c) code that must receive from EITHER family** | **none in the checked corpus**: every new mismatch is an owner value at a `Peer` parameter |
| (d) a checker case broken | none: send/recv/try-send/select/poll/close on owners still check |

**The floor** (worktree): `6124 run: 2903 passed, 3221 failed`. **All of it is the 4 stdlib errors at
startup** (a world that cannot freeze fails every test), plus the ledger shrinking. No other mechanism.

## ⚠ The limit of this measurement: the first seam hides the next

Downstream of `spawn-runner` and `Launched.handle` stays quiet **because those signatures still say
`Peer`**. That is why the defservice `Handle.handle` and `bracket.wat`'s peer-vector fold did not
appear. **The true fallout is a cascade**: it is only visible once each seam is retyped to the owner
family. The build stone must retype seam by seam and re-measure after each, as a cascade (examinare: the
fail-count is the progress meter). Class (c) = 0 is proven **only at the first seam**.

## Codemod or judgement

A blind `(Peer :- […])` → `(Spawned :- […])` rewrite would also retype real client peers. **Each red site
needs the value's head** (`Thread`/`Process`) to decide, which is judgement or a type-driven tool, not a
name rewrite.

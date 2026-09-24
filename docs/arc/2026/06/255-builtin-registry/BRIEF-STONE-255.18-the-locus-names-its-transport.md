# BRIEF — STONE 255.18 (C-b1a): the locus names its transport

**Drawn 2026-09-24 against `main` @ `e998e6754`.** Floor 6027/6027, clippy 0, census `no STOP-8`, delta
**3 / RECOVERY 0**, heresy ledger 220. **Executor tier: Opus.**

## Where this sits

`FINDING-S5-no-failure-needs-a-transport-slot.md` measured that no failure needs the checker to know a
transport slot. Every failure traces to a declaration the language does not make. The first of those
is `wat/spawn.wat:417`: the `Locus/launch` surface method returns a **4-argument**
`(Launched :- [S R Sh Lu])`, and it says nothing about the transport. It passes only through the
checker's missing-slot arms.

The four questions ruled where `T` comes from: **the locus declares it** (L2). The rejected option is
L1, a method-level `T` on `launch`, which lets the caller choose the transport. That is Honest N,
because `ThreadOpts` could claim `Wire`. This is the transport-from-the-locus inference 255.15 built,
now used by the real `Locus`.

## The work

1. `wat/spawn.wat:398`: `:wat::spawn::Locus` becomes `:wat::spawn::Locus :- [T]`. `launch`'s `self` is
   `(:wat::spawn::Locus :- [T])` and it returns `(:wat::spawn::Launched :- [S R Sh Lu T])`. Measure the
   surface's other methods and give each the same treatment where it names `Launched`/`Address`/`Bound`
   with a short arity.
2. The implementors bind their transport: `ThreadOpts` → `(:wat::spawn::Locus :- [:wat::kernel::Shared])`,
   `ProcessOpts` → `(:wat::spawn::Locus :- [:wat::kernel::Wire])`, at `wat/spawn.wat:506/578` **and**
   `wat/bracket.wat:238/318`. Measure what the `bracket.wat` pair is, and name it in the SCORE.
3. Every consumer of the `Locus` type (43 mentions in 20 files; census it per site) takes
   `(Locus :- [T])` with `T` declared in its binder, or a concrete transport where the site is concrete.
   Where the change is the same structural rewrite across many `.wat` files, **use a wat-fix codemod**
   (`wat/fix.wat`, `wat-scripts/fixes/*.wat`): dry-run on a `/tmp` copy, diff, then apply. It is
   committed as the recorded migration.
4. `defservice`'s locus parameter (`wat/service.wat`: the `start-impl-*-params`, `[~locus-sym <-
   :wat::spawn::Locus …]`) consumes the parametric locus. **Only the locus parameter**, plus what it
   strictly needs to check. The Status/Handle free letter is C-b2.
5. **Leave the checker's transport special cases in place.** This stone only adds declarations; the
   special cases become dead later, in C-b5.

## STOP triggers

1. Going green needs changes to defservice's `status-ty`/`handle-bare-name`/`launch-tp-ann` free letter
   beyond the locus parameter → **STOP.** Report the exact sites and errors verbatim. That is C-b2's
   ground, and the builder rules on merging them.
2. A consumer needs the caller to choose a transport that contradicts the locus (the L1 shape) → STOP,
   report it verbatim.
3. 255.16's refusal (one type binds a parametric surface once) fires on a legitimate program, e.g. the
   `spawn.wat`/`bracket.wat` pairs binding `Locus` twice on one type → STOP, report both declarations.

## Expectations — fixed before the strike

| what | command | expected |
|---|---|---|
| `Launched`'s arity at the surface | read `wat/spawn.wat` `launch` | 5 args, `T` from `(Locus :- [T])` |
| thread and process loci bind their transport | read the extend-types | `Shared` / `Wire` |
| a thread locus cannot yield a Wire `Launched` | new negative fixture (`tests/…/probe_arc255_18_*.wat.bad`) | rc=1, named mismatch |
| a process locus yields a Wire `Launched` | positive fixture | rc=0 |
| floor · clippy · census · delta · ledger | the usual | green · 0 · `no STOP-8` · NEW 3 / RECOVERY 0 · ≤ 220 |

Runtime prediction: 2–3 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name
  the arm. (`harvest_wrap_split`: cite it and report it.)
- ⭐ Prove every probe can say BOTH words: every new fixture is shown failing without the change, or
  passing with the opposite transport.
- `defservice` cannot be runtime-macroexpanded. Expand it at the form level; see
  `wat-scripts/scratch-pad/255-17a-child-main-transport.wat` for the shape.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add
  -- <paths>`; **do not push.** Spawn no subagents.

## Out of scope

C-b1b (`Dialable`/`TypedCapability :- [S R T]`), C-b2 (defservice's free letter), C-b3 (generic edge
by unification), C-b4 (`Transport` enum), C-b5 (deleting the special cases).

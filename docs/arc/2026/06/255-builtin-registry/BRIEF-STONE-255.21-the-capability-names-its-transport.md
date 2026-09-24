# BRIEF — STONE 255.21 (C-b1b): the capability names its transport

**Drawn 2026-09-24 against `main` @ `d2ad36b58`.** Floor 6042/6042, clippy 0, census 215 non-zero
(the new baseline, `WEIGH-STONE-255.20-…`), delta **3 / RECOVERY 0**, ledger 215. **Executor tier: Opus.**

## Why

`FINDING-S5-no-failure-needs-a-transport-slot.md` class (b): `wat/capability.wat:46` (`Dialable/coord`)
and `:67` (`TypedCapability/coord`) return a **2-argument** `(:wat::kernel::Address :- [S R])`. That type
says nothing about the transport, and it passes only through the checker's missing-slot arms. The
four questions (`WEIGH-STONE-255.18-…`) ruled **X1: the surface declares it**, `Dialable :- [S R T]` and
`TypedCapability :- [S R T]`, with `coord` returning `(Address :- [S R T])`.

The rejected option is X2, a method-level `T` on `coord`. It is Honest N: the caller could ask a
`Shared` handle for a `Wire` address.

It is the capability-side twin of 255.18 (`Locus :- [T]`): the transport flows from the thing that
knows it.

## The work

1. `wat/capability.wat`: both surfaces become `:- [S R T]`. Every method's `self` is
   `(Dialable :- [S R T])` / `(TypedCapability :- [S R T])`, and `coord` returns `(Address :- [S R T])`.
   Measure `Capability` (the un-parametrised one, `grant`/`revoke`) and say whether it names a transport
   anywhere.
2. `defservice`'s edges (`wat/service.wat` ~:2891–:2918: `dialable-extend`, `typedcap-extend`,
   `dialable-ty`/`typedcap-ty`) bind the Handle's own transport:
   `(Handle :- [… T])` extends `(Dialable :- [Op Reply T])`. **Use the same `T`/`Xt` letter the Handle
   already carries.** That letter is C-b2's to declare; this stone does not change it.
3. Every annotation (≈68 across ≈24 files; **census them per site, excluding comments**, since past
   briefs' counts included prose) moves to three arguments. A repeated structural rewrite is a
   **wat-fix codemod**, dry-run, diffed and committed with a replay fixture, run with
   `./target/release/wat` (`cargo wat` is stale). A site with a concrete handle takes its concrete
   transport; a generic site declares `T`.
4. **Leave the checker's transport special cases in place** (C-b5). Stale comments in `src/check.rs`
   that describe the 2-arg shape (~:5419, ~:17375, ~:17385, ~:17466, ~:17494): update the example types
   they quote, and change no code there.
5. **Rows:**
   - a thread-locus service handle's `Dialable/coord`, claimed `(Address :- [Op Reply Wire])`: **refused**;
   - its `Shared` twin: accepted;
   - a process twin claimed `Wire`: accepted.

   Show each negative accepted on the pre-stone binary.

## STOP triggers

1. Binding the Handle's `T` into the edge target needs the free letter declared (C-b2 ground) to check,
   beyond what the existing letter keys already resolve → STOP, report the sites verbatim.
2. 255.16's one-binding refusal fires on a legitimate program (e.g. a Handle binding `Dialable` at two
   transports) → STOP, report both declarations.
3. 255.20's `UnknownCallee` fires on a surface call this change renamed → STOP, report it (a surface
   member went missing).

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `coord`'s arity | `(Address :- [S R T])` on both surfaces |
| the three rows | refused / accepted / accepted; each negative rc=0 pre-stone |
| no 2-arg `(Dialable\|TypedCapability :- [_ _])` in a live type position | per-site census: 0, every survivor named with its reason |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` against a pre-change census · NEW 3 / RECOVERY 0 · ≤ 215 |

Runtime prediction: 2–3 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name
  the arm. A red caused by the stone is fixed at its cause and the floor re-run whole; say which.
- ⭐ Prove every fixture can say BOTH words.
- `defservice`: expand at the form level (`wat-scripts/scratch-pad/255-17a-child-main-transport.wat`).
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add
  -- <paths>`; **do not push.** Spawn no subagents.

## Out of scope

C-b2 (defservice's free letter), C-b3, C-b4, C-b5, C-c, the third silent door (255.20's findings).

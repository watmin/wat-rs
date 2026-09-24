# BRIEF — STONE 255.20: a name nothing declares does not type-check

**Drawn 2026-09-24 against `main` @ `a25c9b2b9`.** Floor 6037/6037, clippy 0, census `no STOP-8`, delta
**3 / RECOVERY 0**, ledger 220. **Executor tier: Opus.**

## The lie — `--check` rc=0, runtime `UnknownFunction`

```wat
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [x (:wat::spawn::Locus/bogus-xyz (:wat::spawn::process) 1 2 3)] nil))
```

The orchestrator reproduced it (`WEIGH-STONE-255.19-…`): `--check` gives **rc=0**, and running it fails
with `unknown function: :wat::spawn::Locus/bogus-xyz`. The control, `:wat::spawn::NoSuchThing/bogus-xyz`,
is refused at check (rc=1). It is the same class as 255.16 and D2: the checker passes a program the
runtime then fails.

## The mechanism — measured by reading, not yet driven

In `src/check.rs` `infer_list`, a head with **no `TypeScheme`** (`env.get(canonical_k) == None`) goes
through, in order:

- the surface-method arm (~:5306). It fires only when the member **exists**; a missing member falls
  through with no error;
- the 1-arg record-accessor check (~:5972). It only fires when the stem is an `Aggregate` and the call
  has one argument, so a **Surface** stem is not covered;
- the retirement table, then the registry-arity door (~:6040);
- **the silent accept** (~:6054): *"silent-by-intent — no scheme found for multi-arg form; accept and
  pass"*, which returns `fresh.fresh()`, a type that unifies with anything.

A second arm of the same class sits earlier in the dispatch: `k.starts_with(":wat::kernel::") ||
k.starts_with(":wat::std::")` returns `ok(fresh)` for every scheme-less head in those namespaces. The
comment at ~:6017 already names it *"a SECOND namespace-guess authority … closing it is its own
stone."* The same spelling root: **a name is trusted because of its prefix, not because something
declares it.**

## The work

1. **The rule: a call head the checker accepts must name something declared.** That means a
   `TypeScheme`, a registry row (`crate::intrinsic::registry()`), a defclause, a surface member, an enum
   constructor, or another declaring authority you find while measuring. A name none of these declares
   is `UnknownCallee` at check time. The silent accept goes; so does the prefix arm, if its population
   is all declared (measure it).
2. **Measure before you cut.** Put a tripwire at the silent accept and at the prefix arm that logs every
   `k` reaching them across the full floor, plus `--check` over every tracked `.wat`/`.wat.bad`. Then
   classify each distinct `k`:
   - (a) declared elsewhere, so the fallback should have found it. Name which authority, and route to it.
   - (b) genuinely undeclared: a phantom the rule refuses.
   - (c) a legitimate runtime-only entity with no declaration anywhere, e.g. a name defined **after**
     the check pass. Its declaration is missing; name it, do not exempt it.

   Report the list whole.
3. Close the surface-member miss explicitly: `:S/name` where `S` is a Surface without member `name` is
   `UnknownCallee` naming the surface and the member.
4. Rows: the reproducer above → rc=1 naming `bogus-xyz`. A real surface member keeps checking. A
   scheme-less, registry-declared intrinsic keeps checking (its arity door still fires).

## STOP triggers

1. The tripwire census finds **more than 10 distinct `k`s in class (c)**, or any (c) you cannot name a
   missing declaration for → STOP. Report the list with each first call site. That is the builder's
   ruling.
2. Closing the `:wat::kernel::`/`:wat::std::` prefix arm needs declarations for more than 10 registry
   rows → close only the silent accept and the surface miss, and report the prefix population. That arm
   is its own stone, as its comment says.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the reproducer | `--check` rc=1, `UnknownCallee` naming `:wat::spawn::Locus/bogus-xyz` |
| the tripwire census | the list of every `k` that reached each fallback, classified; the tripwire removed after |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` (a census file newly refused is a finding: report it) · NEW 3 / RECOVERY 0 · ≤ 220 |

Runtime prediction: 2–3 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name
  the arm.
- ⭐ Prove every probe can say BOTH words; the reproducer is rc=0 on the pre-stone binary.
- This stone **refuses more**. A refusal of a legitimate program is a finding for STOP-1, never
  something to exempt.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add
  -- <paths>`; **do not push.** Spawn no subagents.

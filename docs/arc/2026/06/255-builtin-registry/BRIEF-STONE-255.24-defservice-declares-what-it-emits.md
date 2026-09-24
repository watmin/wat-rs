# BRIEF — STONE 255.24 (C-b2): defservice declares the type parameters it emits

**Drawn 2026-09-24 against `main` @ `58fa04f34`.** Floor 6055/6055, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0** (the new baseline, `WEIGH-STONE-255.21-…`), ledger 215. **Executor tier: Opus.**

## Why — measured

`defservice`'s generic output uses type letters **nothing declares**. The expansion
(`wat-scripts/scratch-pad/255-19-start-impl-locus-param.wat`, run on `main`) shows:

```
:wat::cache::lru-svc/start$impl [locus <- (:wat.spawn/Locus :- [T]) record <- (:wat.cache.lru-svc/Record :- [K V])]
                                -> (:wat.cache.lru-svc/Handle :- [K V T]) …
```

There is **no `:- [K V T]` binder**. `K`, `V` (the service's own parameters) and `T` (the transport,
spelled `Xt` when the service already declares a `T`) are type parameters by **spelling**, the root of
this arc. They work only through `is_type_param_letter` and the checker's transport arms. This keeps
C-b5 from deleting those arms: the kwargs row (`WEIGH-STONE-255.21-…`) loses its transport at exactly
such an undeclared `$impl`.

## The work

1. **Census every emitted form with a free type letter.** Expand one monomorphic and one parametric
   service at the form level (copy `255-19-start-impl-locus-param.wat`'s shape). List every generated
   `defn`/`defstruct`/`defrecord`/`defenum`/`extend-type` whose signature or fields name a letter from
   `fqdn-tp-syms` or the transport letter, and say whether it declares it. Report the list whole.
2. **Each one declares what it uses.**
   - A generated `defn` takes `:- [~@its-letters]`: exactly the letters its signature uses, no more
     (255.20's rule is that a declaration names only what it consumes; measure the checker's rule for an
     unused defn type parameter).
   - Type declarations already take `:- [...]`; confirm each one names every letter.
   - Extend-types took their binders in 255.22.
3. **Excluded: the child `:user::main`** (`child-main-form`). It is not generic and cannot declare
   anything. Its transport is concretely `Wire`, but spelling that is blocked on the markers being
   declared as markers (C-b4, `SCORE-STONE-255.17a-…`). Leave it as it is and name it in the census.
4. **Rows** (`tests/services/probe_arc255_24_*`):
   - an emitted `start$impl` for a `:- [K V]` service, called with a thread locus, returns
     `(Handle :- [K V Shared])`, and a claim of `Wire` is refused. Show the pre-stone rc;
   - the kwargs witness `wat-scripts/scratch-pad/255-21-kwargs-transport-lost-at-impl.wat`: **measure
     it**. If declaring `$impl`'s binder closes it, move it to a `.wat.bad` row. If it is still lost at the
     D2 arm, report what it returns. That arm is C-b5's; do not cut it.

## STOP triggers

1. A declared binder makes a generated body fail to check where the free letter passed. That means the
   body relied on the letter being flexible (the spelling hole). Report each site and error verbatim, then
   **STOP** if there are more than 5 distinct sites. It is a finding about the macro, not something to
   work around.
2. The change needs the checker's transport arms changed (C-b5 ground) → STOP, report it.
3. The 57-test process-child class (`:wat::kernel::send … expects (Status :- [:T]); got
   (Status.Started :- [Wire])`) appears → STOP. That is the child main, excluded above.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the emitted-form census | every generic emitted form listed; after the stone, each declares its letters, except the child main (named) |
| the start$impl rows | thread → `(Handle :- [K V Shared])`; a Wire claim refused (pre-stone rc noted) |
| the kwargs witness | measured and reported (closed → `.wat.bad`; open → what it returns) |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` against a pre-change census · NEW 2 / RECOVERY 0 · ≤ 215 |

Runtime prediction: 2–4 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name
  the arm. A red caused by the stone is fixed at its cause and the whole floor re-run; say which.
- ⭐ Prove every row can say BOTH words on a pre-stone build from this brief's commit. Capture `rc=$?`
  on the next statement.
- `wat/service.wat` is the macro's source: direct edits are correct. A `.wat` corpus migration, if one
  arises, is a wat-fix codemod run with `./target/release/wat`.
- If a line-pinned `.edn` golden moves because `wat/*.wat` grew, recapture it with `UPDATE_EDN=1` and show
  the diff is `:line` only.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add --
  <paths>`; **do not push.** Spawn no subagents. If you cannot return the tree to a clean state, **say
  so**; do not force it.

## Out of scope

The child main (C-b4), the checker's transport special cases and the D2 arm (C-b5),
`is_type_param_letter`'s other roles (C-c), and the per-locus `start$impl-thread`/`-process` copies
(step 3).

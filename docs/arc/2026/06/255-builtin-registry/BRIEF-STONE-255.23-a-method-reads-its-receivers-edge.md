# BRIEF — STONE 255.23 (C-b3): a surface method reads its receiver's edge

**Drawn 2026-09-24 against `main` @ `151e3c2e5`.** Floor 6053/6053, clippy 0, census 215 non-zero,
delta **3 / RECOVERY 0**, ledger 215. **Executor tier: Opus.**

## Why — the witness on main

`wat-scripts/scratch-pad/255-21-coord-claims-either-transport.wat` `--check`s **rc=0** on `main`, re-measured
after 255.22. A **thread**-locus service handle's `(:wat::capability::Dialable/coord h)` passes as
`(Address :- [Op Reply :wat::kernel::Wire])`, and a process handle's passes as `Shared`. That lets a
process be typed as holding a shared-memory address (`SCORE-STONE-255.21-…`).

255.22 made an edge's parameters declared and built the **structured generic edge** (binder, child,
target), matched by fitting the child. The **method path** still resolves a surface method's scheme through
`satisfier_method_keys` (`src/check.rs` ~:10595): the exact key, then **the last argument rewritten to
`:T`/`:Xt`**, then the bare head. That rewrite is a spelling guess, and nothing binds the edge's `T` to
the receiver's transport. 255.22's 20-line change (118.B2d, ~:5401) already binds a *uniquely matching
generic edge's* parameters when instantiating a method's type. This stone makes that the **one** path.

## The work

1. **A surface method on a receiver resolves through the receiver's matching edge.** Fit the receiver
   against each generic edge's child, one or zero matches (more than one is an error, as in 255.22's
   single-solution rule); take the implementing scheme `<child-head>/<method>`; instantiate it with the
   edge's bindings. A concrete receiver with a concrete edge keeps the exact key.
2. **Delete the letter-rewritten keys** in `satisfier_method_keys` (`:T`/`:Xt`). Census every caller of
   `satisfier_method_keys` and of 255.22's 118.B2d block; they should become one path. Report any caller
   that still needs a guessed key, verbatim. That is STOP-1.
3. **Rows** (`tests/types/probe_arc255_23_*`, plus the witness moved to a `.wat.bad`):
   - a thread handle's `Dialable/coord` claimed `Wire`: **refused** (today rc=0);
   - a process handle's claimed `Shared`: **refused** (today rc=0);
   - the thread handle claimed `Shared`: accepted;
   - the process handle claimed `Wire`: accepted.

   Every service-handle surface call in the floor keeps working. ⚠ `Dialable` is still `:- [S R]` (C-b1b
   has not landed), so `coord`'s return is the 2-argument `Address`. **Measure what `coord` returns after
   this stone.** If the refusal rows cannot discriminate until `Dialable :- [S R T]`, that is **STOP-2**.
   Report it. Do not widen into C-b1b.

## STOP triggers

1. A caller needs a spelling-guessed key after the change → report each verbatim.
2. The four rows cannot discriminate without C-b1b's `Dialable :- [S R T]` → STOP. Report what `coord`
   returns for each handle, verbatim. The builder may then rule C-b3 + C-b1b as one strike.
3. A legitimate program is refused (floor or census) → STOP, report it.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the four rows | refused · refused · accepted · accepted; both refusals rc=0 on the pre-stone build (`151e3c2e5`) |
| `satisfier_method_keys` | no `:T`/`:Xt` rewrite remains |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` against a pre-change census · NEW 3 / RECOVERY 0 · ≤ 215 |

Runtime prediction: 2–3 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name the
  arm. A red caused by the stone is fixed at its cause and the whole floor re-run; say which.
- ⭐ Prove every row can say BOTH words. ⚠ Capture a command's rc into a variable **immediately** —
  `echo "$(…) rc=$?"` prints the substitution's status (it misled the orchestrator twice this session).
- `defservice`: expand at the form level (`wat-scripts/scratch-pad/255-17a-child-main-transport.wat`).
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add --
  <paths>`; **do not push.** Spawn no subagents.

## Out of scope

C-b1b unless STOP-2 is ruled, C-b2, C-b4, C-b5, C-c.

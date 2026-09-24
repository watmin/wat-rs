# SCORE — STONE 255.23 (C-b3): a surface method reads its receiver's edge — STOP-2; the one-path patch LANDED

Weighed by the orchestrator against disk on 2026-09-24. The executor stopped at STOP-2 with no commit and
left the measured patch at `scratchpad/s23/255.23-C-b3-one-method-path.patch`. **The orchestrator landed
that patch after its own floor run on the exact tree committed.**

## The measurement that stopped it

With the patch, the receiver's edge matches uniquely and **binds `T` correctly**. The executor's temporary
eprintln, removed afterwards:

```
recv=(:probe::echo::Handle :- [:wat::kernel::Shared]) … bindings=["T=:wat::kernel::Shared"] scheme.ret=(:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])
recv=(:probe::echo::Handle :- [:wat::kernel::Wire])   … bindings=["T=:wat::kernel::Wire"]   scheme.ret=(:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])
```

`coord`'s scheme is `(Address :- [S R])` renamed, so **nothing in it mentions `T`**. All four transport rows
stay rc=0 on both binaries. The witness still `--check`s rc=0. **Closing the hole needs C-b1b**
(`Dialable :- [S R T]`), and this patch supplies the binding C-b1b lacked in 255.21.

## Why land the patch alone (the four questions)

- Merging C-b3 and C-b1b into one strike failed **Simple**.
- Landing the patch alone passed all four. It stands on its own: it claims only what it does.
  - **One method path.** When exactly one generic edge matches the receiver, resolve through it and
    instantiate the scheme with that edge's bindings. With no match, fall back to the exact key, then the
    bare head.
  - `satisfier_method_keys`' `:T`/`:Xt` last-argument rewrite is **deleted**, and so is 118.B2d's
    positional zip. No caller needed a guessed key (STOP-1 did not fire).

## Evidence for the landed tree

- **Executor:** floors `.floor/2026-09-24T08-22-49Z` and `…08-32-12Z`, both `6053 passed`; clippy 0; census
  `no STOP-8` (215 = 215, plain diff empty); delta NEW 3 / RECOVERY 0; ledger 215.
- ⚠ **The orchestrator's build of the patch is not byte-identical to the executor's saved post binary**
  (`cmp`: differs at byte 25, in the ELF header). So the executor's floor was not taken as evidence for
  this tree. **The orchestrator ran the floor on the exact tree committed:** `.floor/2026-09-24T08-42-07Z`:
  `Summary [ 312.973s] 6053 tests run: 6053 passed (9 slow), 22 skipped`.

## Brief errors, recorded

1. "More than one match is an error": it cannot occur. 255.16 refuses a second binding of one parametric
   surface on one type at registration (`ParametricSurfaceBoundTwice`, measured).
2. The implementing scheme was always found under the **bare-head** key. The letter-rewritten keys
   `(Handle :- [:T])/coord` never matched a registered scheme. `SCORE-STONE-255.21` said the scheme was
   found under the rewritten key, and **that was wrong**.

## Next: C-b1b re-run on top

Of 255.21's three blockers:

1. The missing binding: **closed here.**
2. Exact-string edge lookup: **closed by 255.22.**
3. The kwargs `capswap` at `wat/core.wat:1155`, which mints a 2-argument `(TypedCapability :- [S R])` from
   `(Peer :- [S R])`: **still open. C-b1b's brief must carry it.**

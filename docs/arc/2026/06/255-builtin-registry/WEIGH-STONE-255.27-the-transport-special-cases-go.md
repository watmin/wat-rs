# WEIGH — STONE 255.27 (C-b5): the transport special cases go — STOP-1; STATE A LANDED; two survivors named

The executor stopped under STOP-1 with nothing committed and a clean tree. It left **state A**, every
deletion except two survivors plus the fixes they needed, as `scratchpad/s27/stateA.patch` (37 files).
**The orchestrator applied state A to `576200c59` and floored that exact tree itself:**
`.floor/2026-09-24T23-13-02Z`: `Summary [ 325.822s] 6094 tests run: 6094 passed (9 slow), 22 skipped`.

## Re-measured

- The kwargs witness `.wat.bad` is rc=1 on the new binary and rc=0 on the pre-stone build. **The kwargs
  hole is closed.**
- `transport_param_instantiates` and `is_transport_slot` are gone from `src/check.rs`.

Taken from the report without re-running:

- clippy 0; census `no STOP-8` (215); delta NEW 2 / RECOVERY 0;
- **the 57 green by name**;
- ledger 211, unchanged, since the deleted functions carried no counted compare.

## Deleted (D2 lands as the deletion)

- `transport_param_instantiates` and its `assignable` caller: **the D2 arm**. With it went every
  transport use of `is_type_param_letter`; its only remaining use is C-c's.
- `is_transport_slot` and `unify`'s n / n+1 missing-slot arms.
- `assignable`'s ±1 missing-slot arm.
- The "uninstantiated aggregate `Handle` accepts any instantiation" arm **and its unlisted mirror**.
- D2's 20 test files, plus four new rows, each rc 0 → 1: a letter passed as Shared, Shared passed as a
  letter, a short record, a long record.

**Fixed to make it land (class a, this stone's cause):**

- ⭐ `wat/service.wat`'s per-locus `start$impl-thread`/`-process` and `resume` spliced `launch-tp-ann`
  carrying `(Status :- [T])`, **a free `T` that 255.24 missed**. The deleted arm had been admitting it.
  `launch-tp-ann-thread`/`-process` now spell `Transport.Shared`/`Transport.Wire`.
- `connect` and `address-wire?` expect a 3-argument `Address` with a fresh `T`.
- One scratch probe was respelled, and one pinned message was updated.

## ⛔ Survivor 1 — the head-named `n_fixed` arm in `unify` (the 2-argument "either transport" `Address`)

`(Address :- [S R])` is a **live spelling** meaning "either transport". Deleting the arm breaks:

- 6 stdlib sites: `bracket.wat:130`, `stdio.wat:156/165/174`, `journal.wat:86`, `span.wat:34`;
- about **56 corpus files**, an instrumented upper-leaning count.

Several of these **cannot declare a transport letter where they stand**: a defservice `:init` parameter,
and a record field (`Coords`, `PoolMsg`). **Its own stone:** decide what a transport-agnostic sink
address *is*. It is either a type parameter the enclosing declaration must carry, or a named "any
transport" form. That is a **language question for the builder**, not a mechanical migration.

## ⛔ Survivor 2 — `unify`'s bare-`Path` ~ `Parametric` arm (class b: the checker depends on it)

It is **not** a separate abstract-`Locus` arm. It is `unify`'s general bare-family arm. Deleting it gave
**212 stdlib errors**:

- `PoolMsg.Setup: body produces :wat::bracket::PoolMsg.Setup; signature declares (PoolMsg.Setup :- [D I])`;
- `map::get … expects (PersistentMap :- [_ _]); got :wat::core::PersistentMap`.

**The checker types every generic variant constructor's body as the bare variant path**, and the
`PersistentMap`/`PersistentVector` builtins return the bare family type. It fired 598 times on variant
constructors. **Its own stone:** a constructor's body type carries its instantiation, and builtins return
their instantiated type. After that stone, the arm and the bare-`Locus`/`Handle` admissions can go.

The four admissions these survivors keep are recorded as `wat-scripts/scratch-pad/255-27/survivor-*.wat`,
each naming its arm.

## Also found, not listed in the brief

- `transport_marker` (`check.rs` ~:12117) treats a `Var` in the last argument as the transport, for
  `require-wire-address`. It is a spelling guess of the same class.
- The `Address+Shared` impurity (step 4) is **not small**. `Address` has no wat declaration (a builtin
  name only), and no form expresses purity that depends on an argument. **Its own stone.**

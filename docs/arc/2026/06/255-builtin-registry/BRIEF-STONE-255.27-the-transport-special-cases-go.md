# BRIEF — STONE 255.27 (C-b5): the transport special cases go, and D2 lands

**Drawn 2026-09-24 against `main` @ `c27f9325f`.** Floor 6069/6069, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger **211**. **Executor tier: Opus.**

## Why now

`FINDING-S5-no-failure-needs-a-transport-slot.md` measured that **no failure needs the checker to know a
transport slot**. Everything that went red traced to a declaration the language did not make. Those
declarations are now made:

- `Locus :- [T]` (255.18/.19)
- `Dialable`/`TypedCapability :- [S R T]` (255.21)
- `extend-type` binders and structured edges (255.22)
- the one method path (255.23)
- defservice's declared binders (255.24)
- variants registered with their enum (255.25a)
- `Transport` as a Pure enum, with the child main saying `Wire` (255.25)

255.25 measured the ground: with the parked D2 patch applied, **the 57 process-child tests stay green**,
all 19 D2 rows pass, and the **only** red is the kwargs witness, which stops type-checking. That is the
hole closing (`WEIGH-STONE-255.25-…`).

## The special cases that remain — `src/check.rs`, census as of `c27f9325f`

Line numbers drift, so find each by name.

| site | what it does |
|---|---|
| `transport_param_instantiates` (def ~:10584, caller ~:17494) | admits a letter/Var against a marker **without binding it**: the D2 arm, and where the kwargs transport is lost |
| `is_transport_slot` (def ~:10597; `unify` ~:16771/:16777) | the n / n+1 missing-slot arms in `unify` |
| `n_fixed` (~:16753) | the head-named missing-slot arm (Address/Bound = 2, Launched = 4) in `unify` |
| `assignable` ~:17482 | the `(Handle :- [K V]) ↔ (Handle :- [K V T])` missing-slot arm |
| `assignable` ~:17395 | *"uninstantiated aggregate `Handle` (T unknown) accepts any instantiation"* |
| the abstract-`Locus` arm | a bare `Locus` accepted as any `(Locus :- [T])` (255.18 found it; no bare `Locus` remains in live code since 255.19) |
| `is_type_param_letter`'s transport uses (~:10585/:10589) | letter-spelled transport; its other roles (~:13568/:13575, equality) are **C-c** |

## The work

1. **Land D2.** Apply `scratchpad/park-255.17/check.rs.patch` and its 20 test files, **measuring whether
   it still applies**; re-derive it rather than splice over moved code.
2. **Delete every transport special case above.** A transport is an ordinary type parameter: unify binds
   it. No arm admits a marker against an unbound variable. No arm accepts a short arity as "transport
   unknown". A bare family type is not accepted as any instantiation.
3. **Move the kwargs witness** (`wat-scripts/scratch-pad/255-21-kwargs-transport-lost-at-impl.wat`) to a
   `tests/…/*.wat.bad` row, as its header instructs.
4. **The Address+Shared impurity arm in `is_pure_type`** (~:15025, keyed on head name `Address` plus
   `is_shared_marker`) is the **last hand-coded transport fact.** Measure whether declaring it is small: a
   purity declaration on the `Transport.Shared` variant, or on `Address`'s relation to it. If it is small,
   do it. If not, **report the shape**; it is its own stone.
5. **Rows:** D2's 19 rows; the kwargs `.wat.bad`; a bare `(Locus)` / an uninstantiated `(Handle :- [K V])`
   refused where a full instantiation is expected; each deleted arm's old admission, shown admitted on the
   pre-stone build and refused now.

## STOP triggers

1. **The fallout is the real measurement.** Any red is classified: (a) a short-arity/free-letter use the
   declarations missed, to be fixed at its site; (b) a checker case the deletion broke. **More than 10
   files of (a), or any (b)** → STOP, report the list with each first error verbatim.
2. The 57 process-child tests go red → STOP. They are this stone's canary.
3. A census file changes rc (STOP-8) → report each with its first error. A newly refused file with a
   short arity is (a); any other is a finding.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the listed special cases | gone (census by name; survivors with reasons) |
| D2 · kwargs · bare family rows | pass (refusals shown admitted pre-stone) |
| the 57 | green |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` (or classified) · NEW 2 / RECOVERY 0 · ≤ 211 (expected to shrink) |

Runtime prediction: 3–5 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name
  the arm. A red caused by the stone is fixed at its cause and the whole floor re-run; say which.
- ⭐ Prove every row can say BOTH words on a pre-stone build from this brief's commit. Capture `rc=$?`
  on the next statement.
- A `.wat` corpus migration, if one arises, is a wat-fix codemod run with `./target/release/wat`.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add --
  <paths>`; **do not push.** Spawn no subagents. If you stop without committing, revert your own edits and
  save the patch to the scratchpad `s27/`.

## Out of scope

C-c (`is_type_param_letter`'s non-transport roles and the defn-binder wall), step 3 (generic
`start`/`resume`), the debug arms A–C (255.26).

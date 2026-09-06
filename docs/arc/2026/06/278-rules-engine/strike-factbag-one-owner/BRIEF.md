# BRIEF — FactBag: one owner for the fact base

Read `DESIGN.md` first. **Behaviour-neutral migration.** Give `Session.facts` a named type with one
owning module and one door per operation, collapse the native side's five doors to two, and gate
raw access. `remove-every-equal` preserves today's retract semantics exactly — the cure is strike 2.

## Read in order

1. `wat/rete.wat:180-200` — the `Session` defrecord. `facts` is a **bare** `PersistentVector`; the
   doc above it says "PersistentVector of asserted facts" but the type does not. This is where the
   field's type changes to `:wat::rete::FactBag`.
2. `wat/rete/oracle/insert.wat:22-44` (`insert$oracle`), `:68-98` (`insert-all`), `:100-125`
   (`retract`) — the two writers and the removal. `retract`'s body becomes
   `factbag::remove-every-equal` verbatim; its docstring's *"symmetric with insert"* claim is
   preserved as-is for strike 2 to settle.
3. `wat/rete/oracle/fire.wat:213-228` (`merge-facts` → `add-if-absent`), `:237-260`
   (`retain-supported` → `retain`), `:316-322` — read this last block before you touch anything:
   the support-fixpoint's convergence test is **exact only because the step is a sub-multiset**.
   `retain` must preserve that property by construction.
4. `src/rete/kernel/session.rs:1447` + `:1509` — `session_facts` / `session_with_facts`. These two
   become the only native doors and absorb the unwrap/wrap.
5. `src/rete/kernel/insert.rs:16-32` (`require_session_facts`), `src/rete/kernel/fire/rules.rs:573`
   (raw read), `:185`, `:277`, `:423` (raw writes) — the three doors that get deleted and routed
   through step 4.
6. `tests/lint/no_raw_network_keys_in_oracle.rs` — copy this for the gate: same shape, same
   empty-exemption-list discipline, same "has no form" language in the header.
7. `wat-scripts/fixes/rename-record-def-to-defrecord.wat` — copy this for the codemod's shape.

## Sketch

```
;; wat/rete/factbag.wat
(:wat::core::defrecord :wat::rete::FactBag
  [items <- (:wat::core::PersistentVector :- [:wat::core::Record])])

(:wat::core::defn :wat::rete::factbag::items [b <- :wat::rete::FactBag]
  -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::rete::FactBag/items b))          ;; the ONE unwrap

(:wat::core::defn :wat::rete::factbag::add [b <- :wat::rete::FactBag  f <- :wat::core::Record]
  -> :wat::rete::FactBag
  (:wat::rete::FactBag (:wat::core::PersistentVector/conj (:wat::rete::factbag::items b) f)))
```

```rust
// src/rete/kernel/session.rs — the two doors absorb the representation
pub(crate) fn session_facts(session: &Value) -> Value { /* … unwrap FactBag → inner PVec … */ }
pub(crate) fn session_with_facts(fired: &Value, new_facts: Value) -> Value { /* … wrap … */ }
```

## Blast radius

`wat/rete/factbag.wat` (new) · `wat/rete.wat` (one field type) · `wat/rete/oracle/{insert,fire,explain}.wat`
· `wat/rete/compile.wat` (a `:facts` kwarg) · `src/rete/kernel/{session,insert}.rs`,
`src/rete/kernel/fire/rules.rs` · one new `tests/lint/` gate · one new `wat-scripts/fixes/` codemod.
**No new native algorithm; no behaviour change.**

## Method

The `.wat` migration goes through a **wat-fix codemod** you write in `wat-scripts/fixes/` — dry-run
on a `/tmp` copy and `diff` it before applying, then commit the codemod as the recorded migration.
Both rewrites are uniform wraps. After the codemod, a small deliberate second pass moves the
sites that mean `add-if-absent` / `retain` onto those doors.

## Mutation proof — required, one per rule

The gate has three rules (wat `FactBag/items`, wat `Session/facts`, Rust `"facts"`). **Drive the
LIVE gate, not a copy**, once per rule: introduce the banned form at a real site, confirm RED,
restore. Report the three RED outputs verbatim.

## STOP triggers

1. A native caller of `session_facts`/`session_with_facts` needs a change beyond the two door
   bodies → STOP and report; the contract decision is wrong and the strike must be re-planned.
2. The codemod's dry-run diff moves a byte outside the intended wrap → STOP, report the shape.
3. Any observable value moves — a differential, a grid axis, a fuzzer → STOP. This strike is
   behaviour-neutral by construction.
4. `retain` cannot be written so the sub-multiset property holds by construction → STOP; say what
   the primitive is missing. `fire.wat:316-322`'s convergence proof depends on it.

## Prior result to copy for shape

`docs/arc/2026/06/278-rules-engine/strike-discrimination-never-ran/SCORE.md` — scorecard rows with
the load-bearing ones starred, quoted verbatim evidence, and the honest delta called out.

# DESIGN — STONE: scoped work over a compiled network

> Promotes three forms from a working prototype into `:wat::rete::`. Names ruled by an
> `intueri` cast (`wat-scripts/intueri/rete-scoped-work-naming.wat.intueri`); the prototype is
> `wat-scripts/scratch-pad/wat-grep-with-network-shape.wat` and it runs green.

## The defect being closed

rete has an ACQUIRE/RELEASE pair with **nothing pairing it**:

```
(:wat::rete::compile-all rules queries)   ; ALREADY takes an intern lease (DESIGN-STONE-arm-at-compile)
(:wat::rete::release-session s)           ; drops it; at zero the interned network is evicted
```

Every caller must remember the release. Exactly one site does — a `defservice` `:stop` hook at
`wat/query.wat:400`, *"Hangup drops the intern lease `compile-all` took."* Nothing enforces it.
**That is a convention where a shape belongs** — the same rung `with-open-file` climbed for writers.

★ **And the gap is not theoretical: it bit this session.** The prototype's first draft called
`arm-session` on the session `compile-all` returns. `arm-session`'s HIT path INCREMENTS the lease
(`arm.rs:709`), so it took lease 2 and released back to 1 — **leaking the lease `compile-all` took,
which is the exact lease the wrapper exists to drop.** It ran green. Leases are not observable from
wat, so nothing said a word. That is why acceptance row 3 exists and why it must be a RUST test.

## The three forms

```clojure
(:wat::core::typealias :wat::rete::Overlay
  [(:wat::core::PersistentVector :- [:wat::core::Record]) :-> :wat::rete::Session])

(:wat::core::defn :wat::rete::with-network :- [T]
  [rules   <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   queries <- (:wat::core::PersistentVector :- [:wat::rete::Query])
   body-fn <- [:wat::rete::Session :-> T]]
  -> T
  (:wat::core::let [base   (:wat::rete::compile-all rules queries)
                    result (body-fn base)]
    (:wat::core::do (:wat::rete::release-session base) result)))

(:wat::core::defn :wat::rete::with-overlay :- [T]
  [rules   <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   queries <- (:wat::core::PersistentVector :- [:wat::rete::Query])
   body-fn <- [:wat::rete::Overlay :-> T]]
  -> T
  (:wat::rete::with-network rules queries
    (:wat::core::fn [base <- :wat::rete::Session] -> T
      (body-fn (:wat::core::fn [facts <- (:wat::core::PersistentVector :- [:wat::core::Record])]
                 -> :wat::rete::Session
                 (:wat::rete::fire-rules (:wat::rete::insert-all base facts)))))))
```

## The contract decisions, pinned

**1. Both forms COMPILE; neither accepts a Session.** Forced, not chosen: `compile-all` already
arms, so a wrapper handed an already-compiled Session can only add a lease it then removes. Acquire
and release must be the same scope. Same discipline as `with-open-file`, which opens its own writer.

**2. `with-overlay` is built ON `with-network`** — it inherits the lifecycle rather than repeating
it. One release site, not two.

**3. `Overlay` FIRES.** `(overlay facts)` returns a *fired* Session, not a seeded one. The caller
never wants the unfired form; folding `fire-rules` in removes a step from every call site.

**4. The body params are `base` and `overlay`, and this is CONVENTION, not signature.** The intueri
cast's finding: the verb names alone do NOT tell a reader that `with-network` permits accumulating
across units and `with-overlay` forbids it. That lives in the parameter's TYPE — and it is legible
one line in, because `(overlay facts)` is seen APPLIED while `base` is seen HELD. Every doc and
example must show `base`, never a throwaway `s`/`r`.

## The rooms

1. **`wat/rete.wat`, the `;; ─── the session ───` section (~:154)** — the typealias belongs beside
   `AlphaMemory`/`BetaMemory`/`ProductionMemory`, which are the same kind of naming move.
2. **`wat/rete.wat`, after the session records** — the two defns. Both are plain wat; no Rust.
3. **`:wat::rete::Query`** — a real defrecord (`wat/rete.wat:55`). `compile-all`'s second argument.
4. **`src/rete/kernel/tests.rs`** — where row 3 lives, beside `intern_release_one_session_leaves_the_other`.

## Acceptance — three rows, and the third is the one that matters

**Row 1 — N units of work cost ONE network build.** rete already gates the mechanism
(`fire_rules_reuses_arm_across_fire_and_insert_overlay`: *"fire on a facts overlay must not rebuild
the arm"*). This row asserts the COMPOSITION: `with-overlay` over N fact sets leaves `ARM_BUILDS`
incremented exactly once.

**Row 2 — the base is untouched.** Each unit re-seeds from the compiled base; the base itself still
answers its query with zero results after N units have run. This is the prototype's `3 / 0 / 3`.

**Row 3 — THE LEASE IS ACTUALLY RELEASED.** ⛔ **Must be a RUST test.** Take the network identity,
assert `rete_arm_leases(id) == Some(1)` inside the body, and `rete_arm_lookup(id).is_none()` after
`with-network` returns. Idiom to copy: `intern_release_one_session_leaves_the_other`
(`tests.rs:3043`).

> **Row 3 is the only row that would have caught the prototype's real bug.** Rows 1 and 2 passed
> green while the lease leaked. A test that cannot see leases cannot test a lease-managing form —
> and this form's entire reason for existing is the lease.

Plus: floor green with every move accounted BY NAME; clippy 0.

## Out of scope — affirmatively cut

- **wat-grep itself.** This ships the scoped-work surface; wat-grep consumes it in its own stone,
  and still needs its slurp→facts half (corpus-03 emits `Node`/`Named` but **no `Span`**).
- **A macro form.** `(with-network [base rules queries] …body…)` reads better than a lambda, but
  every existing `with-` in wat is a plain defn taking a body-fn. Matching the family is worth more
  than saving a `fn`. If the family ever moves to macros, these move with it.
- **The untyped-PV hazard.** Two sites in rete's hot path restructure around an empty
  `PersistentVector` carrying an unconstrained `T` (`oracle/pass.wat:353` and the conj-into-prod-mem
  comment). Real, adjacent, and not this stone's.

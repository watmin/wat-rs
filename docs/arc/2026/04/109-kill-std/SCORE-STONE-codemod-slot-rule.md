# SCORE — arc 109: the codemod's SLOT RULE

Rider: one flight, ~7 min. **It could test its own work** — the first stone on this arc where that
was true — and it did, then flagged an 8th head it could not verify rather than guessing.

| # | what | result |
|---|---|---|
| 1 | a declaration NAME becomes a bare binder | ✅ `(:wat::core::defn :wat::kernel::recv-all-loop :- [I O]` |
| 2 | ★ type REFERENCES keep their parens | ✅ `(:wat::kernel::Peer :- [I O])`, nesting intact |
| 3 | both in ONE form, split by slot | ✅ `spawn.wat:614–617` |
| 4 | idempotent | ✅ second pass, 0 changes, across three files |
| 5 | the arrows never move | ✅ 80 → 80 arrow lines in `spawn.wat` |
| 6 | no name slot left wrapped | ✅ **0** across `spawn`/`cache`/`bracket`; 25 names converted |
| 7 | floor (the codemod is loader-gated) | ✅ **4855/4855** |

## What the rider did

Threaded **two** flags through `seq-edits` — `is-first?` (index 0 without a counter) and
`prev-decl-head?` — generalising `wat/fix.wat:123`'s single-flag `fix-seq` shape rather than
inventing one. The name text is the reference render with its outer parens stripped, via the SAME
`keyword/to-type-form-colon` + `ast->source` path (STOP-2 honoured), guarded by an
`assertion-failed!` if the render ever stops being application-shaped. And it corrected the comment
that had asserted *"no position-tracking needed … no state to thread between siblings"* — the flaw
written down as a design decision.

★ It also caught that the head is **`:wat::service::defservice`**, not `:wat::core::defservice` as
my brief implied.

## ⛔ THE 8TH HEAD — refused correctly, then verified and added

The rider found `wat/cache.wat:68` — `(:wat::core::typealias :wat::cache::Lru<K,V> …)` — a
declaration name under a head my census did not list. It could not confirm α accepts the bare
binder there, so **it left it and reported**, per STOP-3.

Verified by the orchestrator: α wired `parse_typealias`, and
`(:wat::core::typealias :user::A :- [K V] …)` checks clean. So the head is real.

⚠ **And the failure mode is why the list had to grow further.** An unlisted head does not fail
loudly — it falls through to the reference path and is **silently corrupted**:

```clojure
(:wat::core::typealias :wat::cache::Lru<K,V> …)
  →  (:wat::core::typealias (:wat::cache::Lru :- [K V]) …)   ⛔ before the fix
  →  (:wat::core::typealias :wat::cache::Lru :- [K V] …)     ✅ after
```

So the set is now **every head that HAS a declaration-name slot** — a property of the LANGUAGE —
rather than every head that happens to carry a parametric name today — a property of THIS CORPUS.
Six added (`typealias`, `newtype`, `typeunion`, `recordtype`, `aggregatetype`, `structtype`), each
destination verified against α to accept `name :- [T…]` **before** being listed. The last five have
zero parametric sites at present and cost nothing to include.
`[[feedback_a_gate_freezes_names_never_a_count]]`

★ My brief built the list from a census of what the corpus does. That is the same error shape as
predicting the consumption wall's violations from a regex: **a list derived from current usage
cannot protect against usage that has not happened yet**, and here the penalty for a miss is silent
corruption rather than a red.

## Honest deltas

- `:wat::rete::core::defn` is a real, distinct head that never carries a parametric name. Left out
  deliberately; the rider documented it in place, and an unlisted head now renders as a reference,
  which for that head is correct today and would be a loud shape error if it ever changed.
- **②-iii is now unblocked.** The codemod converts names and references correctly, is idempotent,
  and leaves the 9,912 arrow/operator sites untouched.

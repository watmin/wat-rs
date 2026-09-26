# WEIGH — STONE 255.45: measure declarations and drops — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `29ed3541a`** (the SCORE only; the tree is clean, and grok left no
worktree). Weighed by the orchestrator on 2026-09-26.

## Part A — could one bodiless clause declaration express every intrinsic?

| class | rows |
|---|---|
| (i) one clause | 446 schemes + 28 hand arms |
| (ii) several clauses (finite overloads) | 11 (`time/+`, `time/-`; `send`/`try-send`/`recv` on owner vs peer; container-preserving `reverse`/`take`/…) |
| (iii) a rest clause | 6. ⚠ Defclause's **check-time** dispatch does not typecheck the rest elements; only eval does |
| (iv) a clause cannot say the current check | **28 hand arms**: ordering needs an *orderable class* (`<`, `>`, …); arguments that are **type keywords or syntax** (`edn::validate`, `type-of`, `to-record`, `variant`, `retag-op`, `listener`, `aggregate-new`); **`select`/`poll`** (`Vector` covariance); tuple-index-dependent `first`/`nth`. **Plus 12 fresh-return rows** with no type to transcribe (untyped debt) |
| special forms | 51 in the probe (86 attributes in source; the gap is unexplained and was not invented) |

**The two gaps, sized:** `Vector` covariance hits 2 rows (`select`, `poll`). An unresolved receiver silently
taking the first clause hits 10 (`send`, `try-send`, `recv`, `peer-process`, `peer-wire?`, `close`, `signal`,
`select`, `poll`, `listener`).

**Reading:** a clause declaration expresses about 85% of the registry as it stands. What it cannot yet
express falls into three kinds:

- **genuine forms** (arguments that are type keywords or syntax);
- **three checker gaps a declaration would need closed**: rest-element checking, `Vector` covariance under
  surface satisfaction, and refusing an unresolved receiver;
- **one bound** (an orderable class, which could be a surface bound).

## Part B — the dropped values: an API question or a drop question?

**Only 6 of 99 sites vanish by changing the API:** every `stop` returns a final record **nobody reads**, so
`stop` → `nil`. For the rest, the producing function **has readers**:

- write counts (21 sites): **`print`/`println` already exist as nil twins**, and the dropping callers should
  use them;
- `mapv`/`foldl` for effect (11): a `for-each`;
- tests evaluated for their type or raise (49);
- incidental returns (8), e.g. `Lru/put`'s evicted entry.

(The saved list has 99 rows against the finding's 101; the gap sits in that file, and 0 files were missing.)

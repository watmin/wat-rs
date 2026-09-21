# SCORE-AMEND — STONE 251.8d-i-b (2): a generator still emits `<-`

Folded into the stone landing (was `659593bf5`). **Not pushed.**
Parent: `AMEND-251.8d-i-b-2-the-generators.md`.
Floor / workspace clippy / census: orchestrator. Crate clippy + lint run here.

## The 8 sift_rules reds closed

`wat/query.wat:292` was minting `(~fact-sym <- ~tkw)` with `fact-sym = symbol-node "?fact"`.
That is a rete fact-bind constructor. Converted to `(~fact-sym :- ~tkw)`. Rebuilt
(`include_str!`). `probe_arc278_sift_rules{,_arena}` — **8 passed**.

Negative control still holds: a source `(?k <- :k)` is `MalformedClause`.

## Derived generator set

Not a grep for `"<-"`. Search was: **what constructs a bind clause**.

| constructor | what it mints | verdict |
|---|---|---|
| `wat/query.wat:292` `` `(~fact-sym <- ~tkw) `` — `fact-sym` is `symbol-node "?fact"` | rete fact-bind | ⭐ **converted to `:-`** |
| `wat/core.wat:920` `symbol-node "<-"` in defn kwargs reshape | `fn` param annotation | ⛔ **left** |
| `src/closure_extract.rs` `Identifier::bare("<-")` | record/param field triples | ⛔ **left** |
| `src/reflect/render.rs` `Identifier::bare("<-")` | macrodef param triples | ⛔ **left** |

How derived:

- `symbol-node "?…"` in tracked `.wat` — **one production mint**: `wat/query.wat` `fact-sym`.
  Scratch `probe-rule-lits.wat` mints `fired`, not a `?`-var, and its query cond is
  `(~tkw)` (keyword-headed, not a bind).
- Quasiquote `` `(~NAME <- …) `` is a bind-list constructor **iff** `NAME` is that
  `?`-prefixed mint. Every other `~x <-` in `wat/` is a param (`d-sym`, `s-sym`,
  `call-args-sym`, …) — names do not start with `?`.
- Rust `Identifier::bare("<-")` sites reconstruct **param/field** vectors, never a
  list whose first child is a `?`-prefixed symbol.

`git diff wat/core.wat` is empty. Param annotations did not move.

## The other 2 — converted arrow in a doc example, re-goldened

Both goldens print `:wat::rete::step-payload`'s `#wat.doc/Row`. The `@example` in
`src/rete/step_payload.rs` was rewritten by the prior corpus pass
(`(?c <- :celsius)` → `(?c :- :celsius)`). The goldens still had the old bind.

Diff of each golden is **one line**:

```
-              (?c <- :celsius)
+              (?c :- :celsius)
```

`[celsius <- :wat.core/i64]` (the defrecord param) is unchanged on both sides.
Re-goldened. `pprintln_doc_row` 3/3; `from_metadata_of_the_lookup` 1/1.

## Gate on the generators

`tests/lint/rete_bind_generators.rs`: a quasiquote `` `(~NAME <- `` is RED when
`NAME` is bound to `symbol-node "?…"`. Text rewrite cannot see expansion; this
walks the constructor.

- NON-VACUITY: git ls-files of tests/wat/wat-scripts/wat-tests `> 1500`
- NON-VACUITY: at least one `symbol-node "?…"` mint (`query.wat` `fact-sym`)
- Detector self-tests: the query.wat shape hits; a param `fn [~d-sym <-` does
  not; `` `(~fact-sym :- `` is not a hit

`every_walking_gate_declares_non_vacuity` — 15 passed.

## Test count

Predicted **+4** (1 walk + 3 detector tests). `cargo nextest list --release -p wat`: **5324** (was 5320).

## Walls I ran

- crate clippy `-p wat --all-targets -D warnings` — **0**
- `--test services probe_arc278_sift_rules` — **8 passed**
- `--test cli pprintln_doc_row` — **3 passed**
- `--test reflection from_metadata_of_the_lookup` — **1 passed**
- `--test lint rete_bind_generators` — **4 passed**
- `--test lint one_param_spec` — **5 passed**
- `--test lint every_walking_gate` — **15 passed**

Floor + workspace clippy + census: orchestrator. Do not push. Do not start 8d-ii.

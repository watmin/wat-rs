# AMEND — STONE 251.8d-i-b (2): a GENERATOR still emits `<-`. 10 reds.

**Orchestrator's verification row, `659593bf5` (not pushed).**

## ⭐ THE STRIKE IS RIGHT AND THE DANCE WAS THE RIGHT CALL

The orchestrator left the transition open and the builder ruled *"rete moves to `:-` — it is not
exception."* ⭐ **You read that correctly and did the surgical dance** — not the full dialect flip,
just rete-var-preceded `<-`:

| | |
|---|---|
| tracked `.wat` rewritten | **321** |
| `.wat.bad` | **25** |
| rust-embedded rete strings | **42 files / 338 occ** |
| param `[x <- :T]` | ⭐ **untouched — correct** |
| driver | `wat-scripts/fixes/rete-bind-arrow-to-binder.wat`, replay-fixtured, idempotent |
| negative control | `(?k <- :k)` → `MalformedClause` ⭐ **rete now REFUSES the old form** |

⛔ **And `expr_ir:1035` was reported, not forced.** The amend called it one of four bind sites; you
measured that it skips **`fn` param types**, not field binds, and that routing it through
`is_binder_marker` alone reds user-fold arity. **Left on `is_param_annotation_arrow`, and said so.**
That is the fifth stone running of reporting what a door cannot express.

## ⛔ FLOOR RED — 10 of 5942. A GENERATOR the text rewrite cannot reach.

```
     Summary [ 286.147s] 5942 tests run: 5932 passed (7 slow), 10 failed, 22 skipped
  8 × wat::services probe_arc278_sift_rules{,_arena}
  1 × wat::cli pprintln_doc_row      1 × wat::reflection probe_stone_metadata_of_whole_row
```

**The arm:**

```
malformed rete clause `(?fact <- :usr/Hot)` — not a recognized :when shape
```

⚠ **`tests/services/probe_arc278_sift_rules.wat` does NOT contain that clause.** Its only `<-` are
`defrecord` param annotations (lines 19-22) — **correctly untouched.** The clause is **SYNTHESISED**:

```wat
wat/query.wat:279   fact-sym  (:wat::core::symbol-node "?fact")
wat/query.wat:292   cond      `(~fact-sym <- ~tkw)          ← the emitter
```

⭐ **This is 255.4's shape exactly** — there, `method_wat_path` was *generating* the `::` join and no
corpus rewrite could reach it. **A codemod rewrites TEXT; a generator builds the node.**

## ⛔ THE DISCRIMINATION THAT MATTERS — two generators, only ONE converts

| site | what it emits | verdict |
|---|---|---|
| `wat/query.wat:292` — `` `(~fact-sym <- ~tkw) `` | a **rete field/fact bind** | ⭐ **CONVERT to `:-`** |
| `wat/core.wat:920` — `arrow-sym (symbol-node "<-")` in `defn` kwargs reshaping | a **`fn` param annotation** | ⛔ **LEAVE — params keep `<-` until 8d-iii** |

⛔ **Converting `core.wat:920` would break every kwargs `defn`.** The rule is the one this stone
already encodes: **the arrow after a `?`-prefixed symbol is a bind; everywhere else it is an
annotation.** Apply it to generators, not just to text.

⚠ **DERIVE the generator set.** The orchestrator found two by grepping `"<-"` under `wat/`. ⛔ **That
is a text search for a node constructor and it will miss any built another way** (`str`-interpolated,
assembled from a variable, or in `.rs`). **Search for what CONSTRUCTS a bind clause**, and say how
you derived the set.

## The other 2 — check before assuming

`pprintln_doc_row` and `probe_stone_metadata_of_whole_row` are **byte goldens over doc text**. The
visible prefixes matched; the delta is further in. ⚠ **If the change is a converted arrow inside a
doc comment, the golden re-goldens. If it is anything else, that is a finding.** ⛔ **Do not
re-golden without reading the diff** — a golden regenerated over an unexamined change records a bug
as expected output.

## The fold

⛔ **Fold into `659593bf5`.** Not a repair commit. Nothing is pushed.

## Gate

- `scripts/floor.sh` green — **the 8 `sift_rules` reds must close.**
- ⛔ **A GATE ON THE GENERATORS.** The corpus gate (`…has_no_bare_arrow_symbols`) is over **text**.
  A generator emitting `<-` for a bind is invisible to it — which is how this shipped. ⭐ **The wall
  must reach node constructors**, or the next generator repeats this.
- **Non-vacuity both ways, unchanged:** a rete bind is `:-` with its field name kept; a `fn` param
  annotation **still spells `<-`** and still works.
- clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`; crate clippy + lint yourself.
- ⛔ **Corpus `.wat` may move** (this stone is the dance) — but **only** rete binds. `git status`
  shows no param-annotation churn.
- ⚠ `wat/*.wat` is `include_str!`'d — **rebuild before concluding a cure does not work.**

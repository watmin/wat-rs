## PERSPICERE — Cast Report (TARGET 2 — the compile side)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> HTML entities are artifacts of the agent's own output encoding.

### SCOPE — derived count and method

Ran `grep -cE '<[^<>]*<'` myself per file — **same 121 raw hits across the same 16 files** the cast disclosed (0 in `validate/error.rs`, `validate/typing.rs`, `eval_test.rs`, `collect.rs`). I then read every one of the 121 hit lines (plus `reachability.rs`'s 27) and hand-classified each:

| bucket | count | verdict |
|---|---|---|
| typealias declarations (`type X = …`) | 20 | exempt by spell definition |
| comments / doc-comments | ~19 | not a type expression — the ward's disclosed 18, confirmed |
| wat-DSL source text in Rust string literals | 16 here **+ 27 in `reachability.rs`** | not type expressions at all — `<-` bind arrows and `<12`/`<18` format specifiers |
| `collect::<Vec<_>>()` turbofish | 4 | one real level; the leading `::<` is syntax |
| `<`/`<=` comparison operators | 3 | not types |
| **lifetime-parameterized single generic** (`Option<Fact<'_>>`, `Vec<WrapperBind<'a>>`, `Option<AlphaPattern<'_>>`) — a category the ward did **not** disclose | 4 | the second `<` is a lifetime, not a hidden noun; NOT flagged |
| **real, non-exempt type nesting in code** | **26** unruned + **3** runed = **29** | evaluated below |

Delta against the disclosure: the 16-files / ~121-hits / 18-comment-hits count matches mine exactly. My independent contribution is the classification down to 29 real sites, catching a **fourth** false-positive category (lifetime params) the disclosure did not call out, and confirming **`reachability.rs`'s 27 are 100% wat-DSL-in-string-literal noise — not one real Rust generic.** It isn't merely "the least consequential bucket," it's a bucket with **zero real hits** once read.

Of the 29 real sites, ~20 are one pervasive crate idiom — `Result<Vec<T>/Option<T>/Arc<T>, EvalBreak-or-LowerError>` — attested 35× in `export.rs` and 15× in `expr_ir/mod.rs`. Per rule 4, minting per-T aliases there would be worse than the nesting.

### Findings — code

**1. `alpha_tree.rs:228`** — `fn restrict_node(...) -> Option<Arc<AlphaDiscNode>>` (2 levels). **Sibling alias exists and is an exact match**: `type AlphaWildcard = Option<Arc<AlphaDiscNode>>;` (`alpha_tree.rs:58`), same file, unused here. **Recommendation: reuse.**

**2. `alpha_tree.rs:180`** — `Option<&Arc<AlphaDiscNode>>`. Borrowed cousin (adds `&`), not literally reusable. Single site. **Leave alone.**

**3. `compiled_cond.rs:792`**, inside `fn invert_slot_names(...) -> crate::rete::expr_ir::SlotNames` — `let mut out: Vec<Option<Arc<str>>> = vec![None; len];` (3 levels). **This function's own return type already is** `SlotNames = Box<[SlotName]>` where `SlotName = Option<Arc<str>>` (`expr_ir/mod.rs:167-168`) — the function imports that alias family, and the local one line later re-spells the same shape by hand. **Recommendation: reuse — `Vec<SlotName>`. The strongest "reuse before inventing" hit in the set: the alias isn't a sibling, it's the function's own return type.**

**4. `expr_ir/eval.rs:1469`** (test-only) — `let mut frame: Vec<Option<Value>>`, identical to `compiled_cond::SlotFrame` (`:71`). The doc two lines above **already states this equivalence in prose** without importing the alias. **Reuse, low urgency.**

**5. Three-site duplicate — field-bind pairs.** `export.rs:721` `Vec<(std::sync::Arc<str>, u16)>`; `expr_ir/mod.rs:833` `Vec<(Arc<str>, u16)>`; `expr_ir/mod.rs:163` `Fields(Box<[(Arc<str>, u16)]>)`. Identical concept — (field-name, slot-index) — across 2 files, **no existing alias**, though `expr_ir/mod.rs` already has a `Slot*` family right beside it. **Recommendation: mint** `type FieldSlot = (Arc<str>, u16);`. Reused 3× across 2 modules, so it clears rule 4.

**6/7. `vocabulary.rs:1543` / `expr_ir/eval.rs:897`** — the `OnceLock<…>` lazy-table idiom `sequi` already cleared CLEAN. Different payloads, no cross-reuse; naming either produces a mumble. **Leave alone.**

**8. `compiled_cond.rs:987`** `Option<Vec<u8>>` · **9. `validate/mod.rs:1375`** `Option<&Vec<WatAST>>` — one-breath, single site. **Leave alone.**

**Group of 20 — the `Result<_, EvalBreak>` / `Result<_, LowerError>` idiom.** Each wraps one visible noun in the crate's standard fallible convention. **Leave all 20 alone.** The one thing that *would* help — a generic `type EvalResult<T> = Result<T, EvalBreak>;` covering all 35 sites at once — is a crate-wide convention change, not a per-site perspicere mint; flagged as an aside, not a recommendation.

### The 3 existing runes — adjudicated

**`expr_ir/eval.rs:413`**, `intentional-structure` on `parent: Option<&[Option<Value>]>`: the rune sits under a full paragraph (`:405-409`) — *"the slot LAYOUT is the contract here, and an alias would hide it."* Not vague, not copy-paste, doesn't read as avoidance; it states a checkable design claim. **Clear.**

**`purity.rs:2587`**, `read-once` on `by_ns: BTreeMap<String, Vec<&String>>`: grepped that exact type string whole-repo — **one hit, this line.** Claim true.
**`purity.rs:2593`**, `read-once` on `rows: Vec<(usize, String, Vec<&String>)>`: grepped — **one hit.** Claim true.
Both are the same shape and adjacent, as the cast flagged for scrutiny, and **both survive it** — same outcome as target 1's seven. Both also sit inside a `#[cfg(test)]` completeness-print block. **Both clear.**

### wat-side bracket-nesting sweep

Grepped `:- [` density across the 5 spec files, read every line with 2+ occurrences by hand (excluding comments and `typealias` bodies):
- `acc.wat`, `compile.wat`, `factbag.wat`, `syntax.wat`: **none**.
- `wat/rete.wat:158/160/162/325/327` — inside `typealias` bodies: exempt.
- **`wat/rete.wat:32` — NOT a typealias body, a `defrecord` field:**
  `[matches <- (:wat::core::PersistentVector :- [(:wat::core::Tuple :- [:wat::core::Record :wat::core::i64])])`
  2 levels, inline in `:wat::rete::Token`'s field list. The inner `(Tuple :- [Record i64])` is one alpha-hit — the file's own comment at `:419` says *"each tuple is (sfact, alpha-id)."* **Confirmed NOT read-once**: the identical shape recurs at `wat/rete.wat:429` as a lambda-parameter type. No existing typealias covers it (checked all six `typealias :wat::rete::*` in the file). **Real finding — mint** e.g. `(:wat::core::typealias :wat::rete::AlphaHit (:wat::core::Tuple :- [:wat::core::Record :wat::core::i64]))`.

### Prior art — second reading given only where invited

`where_tree.rs:84`/`alpha_tree.rs:56` `EqChildren` (rowed 2M1, cost lens) — my type-vocabulary reading: **a second, smaller duplicate exists that the cost row didn't name** — `EqBuckets` is declared identically (`HashMap<Value, Vec<i64>>`) in *both* `alpha_tree.rs:53` and `where_tree.rs:41`, with matching `std::HashMap` on both sides (no divergence, so no cost defect) — the same DRY opportunity as `EqChildren`, undirected by any behaviour difference. Noting it, not minting it.

**FINDINGS** — 4 reuse-before-inventing catches, 1 wat-side mint, 20 idiomatic sites left alone, 3 runes all clear, plus the `EqBuckets` second reading.

# ward `perspicere` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

## L1 — type expressions hiding a noun

---

**1. `wat/fix.wat:213` (and 50 more) — `(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])`**

**Noun: `:wat::fix::Edit`.** It is not missing. It is declared **in this same file at `wat/fix.wat:905`** as exactly `(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])`, with the doc *"Edit — one span splice: (offset, chars-to-replace, replacement-text)."*

**Alias — already minted; the corpus never migrated to it.** The distribution is not "some sites forgot": the raw triple occurs at lines 213, 216, 226, 236, 241, 246, 250, 258, 261, 263, 271, 282, 284, 298, 326, 384, 386, 406, 408, 417, 427, 487, 489, 504, 506, 523, 525, 537, 549, 572, 575, 585, 587, 707, 709, 725, 739, 742, 745, 778, 780, 794, 800, 802, 803, 833, 835, 850, 856, 858, 859 — **all 51 strictly before line 905, and zero after it.** Every one of the 22 sites past 905 uses `:wat::fix::Edit`. The alias was minted mid-file and the code above it was never revisited.

**Per-site reason:** `wat/fix.wat:205-217` settles it alone. The function is named `fix-text-deletion-edit`; its doc says *"a one-element Vector holding a deletion **edit**"*; its return type at :213 is the raw triple and its constructor at :216 is the raw triple again. The word "edit" appears three times in the site's prose and zero times in its type.

Second file: **`wat/lint.wat:655, 657, 671, 673`** — `apply-fixes` builds this vector and hands it to `(:wat::fix::fix-text-apply src rev-edits)` at `wat/lint.wat:675`. Its value *is* a `Vector<:wat::fix::Edit>` by the callee's own signature, and lint.wat already names three `:wat::fix::` symbols, so the noun is in scope there.

**Remedy:** move the `typealias` from :905 to the file head (every other alias in the wat stdlib — `rete.wat:157/175/317/319`, `telemetry.wat:23/28`, `cache.wat:76` — is declared above every use), then a `wat-scripts/fixes/` codemod. **This is the mandated path, not a nice-to-have:** the same triple occurs **327 more times across 33 files in `wat-scripts/`** (outside my target but inside the migration's blast radius). 382 sites is R21 territory — and the codemod would be `fix.wat` rewriting `fix.wat`, which is the file's own stated proving point.

---

**2. `src/rete/kernel/fire/pass/alpha.rs:85` and `:271` — `HashMap<String, (Vec<u32>, bool)>`**

**Noun: the per-class seed plan** — `ClassBatch { ids: Vec<u32>, uniform: bool }`.

**A real type, not an alias.** This is the just-cured D2 shape, one file over, in the same subsystem: **a collection plus a flag that describes it, maintained by two disjoint `&mut` arms.**

- `:139` `if let Some((ids, _)) = class_ids.get_mut(class) { ids.push(i) }` — pushes, never touches the flag.
- `:143` `else if let Some((_, uniform)) = class_ids.get_mut(class) { *uniform = false; any_mixed = true }` — demotes, never touches `ids`.

The site's own header at `:70-74` says: *"**THE CURE IS THE `bool` BELOW** — 'every fact of this class packed'. A class batches only if it is uniform."* A bool that is the cure for an alpha double-write, whose maintenance is conventional. `JoinRightIndex`'s header (`src/rete/kernel/session.rs:224-231`) already ruled on exactly this: *"⛔ THE CURE IS STRUCTURAL, NOT CONVENTIONAL. Bumping the counter at the two bypass sites would have cured today's two writers and left a third free to appear."*

**And there is a third piece of state, worse than the flag.** `any_mixed` (`:91`, set only at `:145`, read at `:217`) gates `activate_deferred_mixed_classes` **entirely**. A future writer that sets `uniform = false` without setting `any_mixed` does not double-count — it makes every fact of that class **never activate at all**. Fact loss, not fact duplication. `any_mixed` should be derived from the type (`has_mixed()`), not maintained beside it.

One door: `observe(&mut self, class, i, packed) -> bool` — push-or-demote in one act, with no accessor handing out `&mut` to either half.

**Caveat, stated because it constrains the cure:** `:130-137` argues the duplicated `get_mut` and the packed-arm-first ordering are a deliberate hot-path shape (*"the duplicated `get_mut` is the price"*). The method must be `#[inline]` and branch on `packed` before the lookup, or the cure buys correctness with the batch path's cost.

---

**3. `src/rete/kernel/session.rs:359` — `bind_pool: Vec<(u32, u32)>` (and 19 sibling spellings)**

**Noun: the bind pair — `(key-intern id, value-intern id)`.** The two `u32`s **index different tables**, and `span_from_pairs`' own doc at `:1450-1453` says why: *"Keys and values intern through different tables — `intern_key` and `intern_val` — because a binding key is a small closed set of names while a value is arbitrary."*

**A real type — newtypes `BindKeyId(u32)` / `BindValId(u32)`.** This is the strongest "compile error" case I found, by the calibration standard.

**Per-site reason:** the pool has **seven independent push sites**, each assembling the pair from two separately-computed `u32`s, with nothing at the type level to stop a transposition:

- `src/rete/compiled_cond.rs:1089` — `(next_key(keys,…), intern_val(vals,…))`
- `src/rete/compiled_cond.rs:1110` — same shape
- `src/rete/kernel/session.rs:1460` — `(intern_key(…), intern_val(…))`
- `src/rete/kernel/fire/mod.rs:561`, `:567`, `:615` — `pool.push((kids[skip + i], row.vids[fi as usize]))`
- `src/rete/kernel/fire/mod.rs:672` — `(key_id, vid)`

`fire/mod.rs:615` is the sharp one: both operands are bare `u32` array reads, and swapping them compiles. The consumer at `session.rs:30` then does `self.keys.get(*i as usize)` — a transposed pair looks up a *value* id in the *key* table and silently returns the wrong binding or `None`. `seen_insert` dedups derived facts, which is the same reason D2 survived every end-to-end differential.

**Honest note on scope:** `Vec<(u32, u32)>` carries **one** `<`, below the ward's stated 2-`<` trigger. I found it through the ward's principle, not its detector. See L3.

---

**4. `src/check/env.rs:87` — `Option<&'a HashMap<String, HashMap<String, WatAST>>>`**

**Noun: `BindingMetadata`.** Defined at **`src/value/symbol_table.rs:16`** as `pub(crate) type BindingMetadata = HashMap<String, HashMap<String, WatAST>>` — byte-identical — and used at `symbol_table.rs:142`. `check::env` reaches it as `crate::value::symbol_table::BindingMetadata` (`pub mod symbol_table` at `src/value/mod.rs:37`).

**Alias — already minted, one struct away, and this field borrows the very map it names.** The field's own doc at `:126-128` says: *"BORROW of SymbolTable's binding-level metadata… `from_symbols` carries `Some(&sym.binding_metadata)`."* It is not a coincidentally-similar shape; it is the same map.

**Per-site reason, and it is the field's own testimony:** the two places this type is *described* in prose — `src/check/env.rs:14` and `src/check/env.rs:35` — both write it as **`Option<&'a HashMap<…>>`**, eliding the body with an ellipsis. The author could not fit the type into the sentence about the type. That is the perspicere signature without inference.

---

**5. `src/check/env.rs:103` — `HashMap<String, Vec<(Vec<TypeExpr>, TypeExpr, bool)>>` (also `:415`, `:437`)**

**Noun: a clause signature** — `ClauseSig { fixed: Vec<TypeExpr>, ret: TypeExpr, has_rest: bool }`.

**A real type.** The two `TypeExpr`-shaped members are swap-proof by type; the **`bool` is not**, and it carries the variadic-arity decision.

**Per-site reason:** the bool's meaning exists **only** in a comment. `src/check/env.rs:102` says *"Stone 241.5 — tuple is (fixed_arg_types, return_type, has_rest_binder)"*. It is produced at `src/check.rs:8357` as a bare positional `cl.args.rest_param.is_some()`, and consumed **~3000 lines away** at `src/check.rs:5403-5409`, where the reader recovers its meaning purely from the destructuring binder name:

```rust
for (clause_arg_types, clause_ret, clause_has_rest) in &clauses {
    let arity_ok = if *clause_has_rest { called_arity >= clause_arity } else { called_arity == clause_arity };
```

Get that bool wrong and a variadic defclause silently stops matching over-arity calls (or a fixed one starts). Nothing between production and consumption names it. A named field would carry the meaning across the 3000 lines that the tuple position does not.

---

**6. `src/rete/kernel/session.rs:208` — `pub(crate) type JoinLeftIndex = HashMap<i64, JoinKeyMap<Token>>`**

**Noun: present. Verbs: absent.** This is the **un-cured half of the D2 pair**. Its sibling three lines down became `struct JoinRightIndex` with one door (`RightIndexWriter::push`) at HEAD; the left index stayed an alias.

**Alias → real type.** Evidence of the gap, read this session:

- `src/rete/kernel/fire/pass/hash_join.rs:273` and `:375` hand out `&mut` via `left_idx.entry(*child_id).or_default()` — precisely the accessor `JoinRightIndex` deliberately does not provide (`session.rs:229`: *"There is no accessor that hands out `&mut` to the buckets, so a fourth writer cannot be written"*).
- Both sites then run the **same six-line keying loop** (`key_of(bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds), jk, &wm.bind_val_ids)` → `lidx.entry(k).or_default().push(tok)`), duplicated verbatim.

A `JoinLeftIndex::index_tokens(join_id, toks, …)` collapses the duplicate and closes the `&mut` door.

**What it does NOT buy — say it plainly.** The left index carries **no mark**, so this cure does not make a D2-style bypass *unrepresentable*; it only removes the duplicated body and the loose `&mut`. The invariant this index actually carries is temporal — `hash_join.rs:75`: *"Step 5: add dl → left_idx[J] AFTER term2"* — and no Rust type enforces "call me after term2." This is a strictly weaker cure than its sibling's, and proposing it as equivalent would be a false claim. (The two writers are safe today: catch-up `continue`s at `hash_join.rs:295`, so it and step 5 cannot both run for one join in one round. I checked.)

---

## L2 — weaknesses

**W1 — the perspicere rune vocabulary is gated; the question that requires a rune is not.**
`tests/lint/no_unknown_ward_rune.rs` closes the *spelling* of `read-once` / `mumble-alias` / `intentional-structure`, and its own header (`:26-34`) admits it cannot judge fit. Nothing in `tests/lint/` (46 files, read this session) counts nesting depth or asks whether a deep type *should* have been runed — `no_angle_type_in_diagnostic.rs:14-24` is a closed name-list for wat spellings inside diagnostic strings, a different question. **The gap has a measured cost, not a hypothetical one:** finding 1 — 51 raw spellings of a triple that has a name 700 lines below, in the codemod framework `CLAUDE.md` mandates for every corpus migration, and 327 more in `wat-scripts/`.

**Remedy, narrow and mutation-provable:** a lint over `wat/**/*.wat` that flags any type expression textually identical to the body of a `typealias` declared in the same file. Zero false positives by construction (the alias is definitionally equivalent), no judgement, and it reddens on all 55 in-target sites today. It does not attempt the general depth question — it catches only the case where the noun is already minted and unused, which is the case that actually occurred.

**W2 — `wat/sqlite.wat:157` and `:164` — `(:wat::core::Vector :- [(:wat::core::Vector :- [:wat::sqlite::Cell])])`, twice, no noun.** The body binds it as `rows` at `:160` and `:167`; the doc at `src/rust_deps/sqlite.rs:291` says *"Returns one `Vector<Cell>` per row."* A `:wat::sqlite::Row` / `Rows` pair is the noun. Two sites only, so this sits near `read-once` — **but the file's existing comment does not cover it.** The comment at `src/rust_deps/sqlite.rs:262-265` justifies spelling the raw *error* tuple instead of `RawFault` (the `#[wat_dispatch]` codegen inspects the syntactic return type), and that reason is correct and specific. It says nothing about `Vec<Vec<Value>>` in the Ok position, which is un-runed and unexplained on both the Rust side (`sqlite.rs:292`, `:341`) and the wat side. Either mint `Row`/`Rows`, or rune it — the current state is neither.

---

## L3 — judgement

**The ward's `<`-counting trigger has a blind spot this repo lives in.** Finding 3 — the most swap-dangerous type expression I found, seven writers, two u32s indexing different tables — carries exactly **one** `<`. So does `(Vec<u32>, bool)` in finding 2's tuple payload. Meanwhile `Result<Vec<TypeDef>, TypeError>` (`src/types.rs:3058`) trips nothing and shouldn't. A Rust codebase hides its nouns in **bare tuples**, not in generic depth: `(u32, u32)`, `(Vec<u32>, bool)`, `(Vec<TypeExpr>, TypeExpr, bool)`. Depth is the C++/Scala tell. If a detector is ever built here, count **anonymous tuple members in a persisted or threaded position**, not `<`.

**Honest depth exists here and I am not flagging it.** `src/closure_extract.rs:1851` `edges: &BTreeMap<String, BTreeSet<String>>` takes either `dep_edges` (`:695`) or `type_edges` (`:697`); `topo_sort` is genuinely shape-polymorphic and naming its parameter after one caller would be worse than the shape. Likewise `src/rete/kernel/session.rs:194` `LeadingEmitted` — three `<` deep, already named, and the depth is the point (the doc at `:178-193` records the round-vs-fire bug the `HashSet` fixes). Neither needs a rune; they need nothing.

**The alias culture here is real and mostly working.** 91 `type` declarations in `src/`, six in `wat/rete.wat` alone, `LeafAidsByClass` (`session.rs:174`) deliberately kept distinct from the shape-identical `AlphasByType` with a written reason. **All four of my named-noun findings (1, 4, and the `AcronymRegistry` note below) are the same failure mode: the alias was minted and the callers were never migrated.** That is a migration-discipline problem, not a naming problem — and it is exactly what a codemod exists to fix.

**One I am ranking below the six but not dropping:** `src/types.rs:3056`, `:3613`, `:3772` thread `acronyms: &HashMap<String, Vec<String>>` through three signatures from the field `src/value/symbol_table.rs:150 acronym_registry`. `HashMap<String, Vec<String>>` wears at least five unrelated nouns in this crate — `field_restrictions` (`types.rs:194`, `types/defstruct.rs:74`, `:167`), `subtype_edges` (`types.rs:530`), `reach` (`rete/kernel/stratify.rs:946`), `acronym_registry` — so `register_types_with_acronyms(forms, env, &sym.field_restrictions)` compiles. An `AcronymRegistry` alias is warranted, and the per-site reason is pointed: **the file that minted `BindingMetadata` (`symbol_table.rs:16`) left the field eight lines below it raw.**

---

## What I could not check, and why

- **Nothing was built or run.** Read-only, as instructed. No finding here rests on execution, and none of my remedies is measured — they are arguments about types, not about behaviour.
- **Whether the fix.wat migration needs the alias moved, or only the codemod.** I read `register_types_impl` (`src/types.rs:3616`) walking `forms` in a single program-order loop, and I found **zero** use-before-declare instances of any typealias across the wat stdlib. That is consistent with declaration-before-use being *required*, and equally consistent with it merely being *universal habit* — registration and `check_program` are separate phases, so the uses at `fix.wat:213` may well resolve against an alias declared at `:905`. **I did not prove which.** It decides whether the remedy is one step or two. A five-second experiment (move the alias down in a scratch copy, load it) settles it; I could not run one.
- **My sweep is line-oriented regex, so a signature wrapped across lines is invisible to it.** I queried: depth-3 `X<..<..<`; depth-2 for `HashMap`/`FxHashMap`/`BTreeMap`/`IndexMap`; `-> T<..<..>>` returns; `Box|Arc|Rc<dyn Fn>`; all 91 `type` declarations. A multi-line `fn` whose generic nesting straddles a newline was not seen, and **I did not measure how many such signatures exist** — so I cannot bound what I missed. I did not read all 958 `.rs` files.
- **I did not re-audit the 68 existing runes for fit.** The 18-site `"alias would be a mumble"` boilerplate — my own prior cast's defect — is already recorded, measured, and dated at `tests/lint/no_unknown_ward_rune.rs:31-34` (2026-09-01). I read that header; I did not re-derive the count, and I am deliberately not re-reporting it as a new finding.
- **`wat-scripts/` and `crates/` were outside the stated target** (`src/`, `tests/`, `wat/`). The 327-occurrence / 33-file `wat-scripts` figure in finding 1 is a raw `grep -rc` total I did not open a single one of those files to verify. Treat it as the codemod's likely scope, not as an audited count.
- **I read `tests/` only through the same regexes.** Its depth-3 hits were two files, both in comments or a deliberately-nested probe (`tests/collection/probe_arc216_stone4_predicate_composition.rs:23`). I did not review test code for the tuple-shaped defect that findings 2, 3 and 5 are about, and given `src/rete/kernel/tests/accum_cost.rs:1227` pushes a raw bind pair, that is a gap I would expect to be non-empty.

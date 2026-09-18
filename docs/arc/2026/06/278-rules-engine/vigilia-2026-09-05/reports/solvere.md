# ward `solvere` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I have what I need. Report follows.

---

# `solvere` — wat-rs @ `21530efab` (code only: `src/`, `tests/`, `wat/`)

Calibration held: I hunted the D2 shape — **two pieces of state that must agree, held apart** — and the inverse. Everything below was read this session; every pairing was grepped, not inferred.

---

## L1 — braids

### L1-1 · `join_keys_cache` is *also* the catch-up latch, and a different pass writes it — `left_idx` goes short. **LIVE, and the failure is missing rows.**

**Cite:** `src/rete/kernel/fire/pass/hash_join.rs:120` · `:160` · `:273` · `:375` · `:429` · `src/rete/kernel/fire/mod.rs:786` · `:802` · `src/rete/kernel/session.rs:168` · `src/rete/kernel/arm.rs:616` · `src/rete/kernel/fire/delta.rs:324`, `:539`, `:602`, `:612`

**The two concerns in one map.** `JoinKeysCache = HashMap<i64, Vec<Value>>` (`session.rs:168`) is documented as one thing: a memo of a join's shared-variable list. `hash_join.rs:120` gives it a second, undeclared job:

```rust
let first_keying = if !join_keys_cache.contains_key(child_id) {
```

Membership *is* the latch for "`hash_join_delta` has not yet run its one-time catch-up on J". The catch-up (`:160`) is the **only** bulk builder of `left_idx[J]` (`:273`, `// Build left_idx[J] from ALL cumulative left tokens`); the only other writer is step 5's `dl` delta (`:375`).

**The second writer is in another pass, and it does not maintain `left_idx` at all.** `keyed_join_persistent` writes the same memo at `fire/mod.rs:802` (`idx.join_keys_cache.entry(join_id).or_insert_with(…)`), and its `FilterJoinIdx` (`fire/mod.rs:784-787`) has exactly two fields — `right_idx`, `join_keys_cache`. **No `left_idx`.** It is reached from `join_after_filter` (delta.rs:602) and `filter_after_join` (delta.rs:612), both of which run **after** `hash_join_delta` (delta.rs:539) in the same round.

**What breaks when they drift.** The chain `Node → :where → HashJoin(a) → HashJoin(b)` — the exact shape `left_activate_join`'s own doc (`fire/pass/mod.rs:90-104`) was written for, and the shape the D2 invariant test names (`src/rete/kernel/tests/right_index_counter_invariant.rs:60-71`):

- `b`'s parent `a` is a HashJoin, so `a ∈ kind_ids.join_parent` and `hash_join_delta` **does** visit `b`. `a ∈ beta_readers` too (`arm.rs:613-617`: any node with a HashJoin child), so `wm.beta[a]` is real.
- **Round 1:** `hash_join_delta` reaches `b`, `wm.beta[a]` is still empty (`a` is only left-activated later, in 3.6) → the `_ =>` arm `continue`s at `hash_join.rs:141`, and **no memo is written**. Then 3.7's `keyed_join_persistent` writes `join_keys_cache[b]` at `fire/mod.rs:802` and indexes `right_idx[b]`.
- **Round 2:** `hash_join_delta` reaches `b`, `contains_key(b)` is now **true** → `first_keying = false` → **the catch-up never runs, ever**. `left_idx[b]` was never bulk-built, and step 5 only ever adds *that round's* `dl`.
- Step 4 (`hash_join.rs:429`) is `if let Some(lidx) = left_idx.get(child_id)` — a **silent `None`**. `term2 = old_left ⋈ Δright` is skipped whole.

So a right-side fact arriving in a **later round than** `b`'s left tokens is never joined against them. 3.7 does not backstop it: `b` only reaches the frontier when `d_beta[a]` is non-empty that round, and `d_beta` is re-allocated per round (`delta.rs:468`). **Repro shape:** `[A ?k] [:where …] [B ?k] [C ?k]` where `C(k)` for an already-seen `k` is *derived* one round after its `A`/`B`. The D2 test world cannot see it — its wave 2 derives `A`, `B`, `C` together for *new* `k`, so `dl` is non-empty and step 3's `term1` covers the join.

**Unrepresentable shape.** `JoinRightIndex` is already the answer, applied to one side only. Fold left buckets, right buckets, both marks, and the key memo into one `JoinIndexes` type whose only door is `writer(join_id)`, and give the **left** side the same `already(join_id)` high-water mark the right side now has — so "index the tail I have not indexed" is the single verb both passes call, and `keyed_join_persistent` physically cannot key a join while leaving its left index short. Then delete `first_keying`: a mark makes "have I caught up" a *derived reading*, not a second map's membership. The cure's own commit message names the rung this is on — *"nothing structural forbade a third writer."* Here nothing structural forbids a second **reader** of a latch that was never a latch.

---

### L1-2 · `defined_values` / `defined_value_spans` / `defined_value_asts` — three maps, two verbs, and one write path advances two of the three. **LIVE drift at HEAD.**

**Cite:** `src/check/env.rs:69`, `:72`, `:118`, `:366-369`, `:373-375` · `src/check.rs:8414`, `:8417`, `:8435`, `:7932`, `:7939`, `:7956`

Three `HashMap<String, _>` keyed by the same name, consulted **together** at one decision — the redef gate reads the span (`check.rs:7932`), the body AST (`:7939`), and the type (`:7956`).

- `register_defined_value` (`env.rs:366-369`) writes **two** of them in one act.
- `register_defined_value_ast` (`env.rs:373-375`) is a **separate verb** whose doc says only *"Called alongside `register_defined_value` at the first registration site."* Convention, not structure.
- **First binding** (`check.rs:8414` + `:8417`) calls both. **The `redef_allowed` replace path (`check.rs:8435`) calls only `register_defined_value`.**

**What breaks.** After an allowed redef, `defined_values[name]` and `defined_value_spans[name]` describe the *new* binding while `defined_value_asts[name]` still holds the *old* body. The state is inconsistent at HEAD regardless of who looks. It becomes observable on `set-redef! true` → redef → `set-redef! false` → redef: `is_byte_equiv` (`check.rs:7945`) compares the new body against the **stale** stored AST, so a redeclaration byte-identical to the live binding raises a false `DefRedefForbidden`, and a redeclaration identical to the *superseded* body is silently waved through.

**Unrepresentable shape.** One `DefBinding { ty, span, body: Option<WatAST> }` in a single `HashMap<String, DefBinding>` with a private field set and one `register` verb. `register_defclause` (`env.rs:423-427`) is the honest reason the key sets legitimately differ — it has a type and a span but no body — so `body: Option<…>` is the right shape and a "key sets must match" assertion is the wrong one.

---

### L1-3 · `CompiledCond.slot_keys` / `output_slots` — parallel arrays, two writers, enforced at one writer by hand. **Latent; the constructor already knows better.**

**Cite:** `src/rete/compiled_cond.rs:157-164`, `:219-234`, `:374-383`, `:1062-1063`, `:1106`, `:972-980` · `src/rete/export.rs:1398-1410`, `:1415`

The fields' own doc calls them *"the zip the design doc describes"* and `materialize_into`'s doc says it outright at `:1062-1063`:

> *"they are two views of one sequence, and nothing here can check that."*

`from_parts` (`:219`) takes them as two independent `Arc<[…]>` and checks nothing — **in a constructor whose own doc (`:214-218`) explains that `has_seed_cmp` is derived rather than passed precisely so that "the two [are] unable to disagree."** The discipline is applied to the scalar and withheld from the pair beside it.

**Two writers.** `compiled_cond.rs:374-383` builds both from one `order` vec — safe by construction, checks nothing. `export.rs:1415` (the wire import) builds them from two independently-parsed sequences and **does** check, by hand, at `export.rs:1398`. That is the convention rung the D2 commit rejected by name.

**What breaks.** A third sequence rides on this: `intern_cond_keys` (`:972-978`) sizes `ids` as `fact_bind? + slot_keys.len()`, while `materialize_into` (`:1094-1113`) consumes one id per **`output_slots`** entry. On a mismatch the guard at `:1106` returns `None` — **indistinguishable from "the condition did not match"**. A rule stops matching, silently, with no error anywhere.

**Unrepresentable shape.** `from_parts` takes the zip (`Vec<(Value, usize)>`, or a `SlotZip` newtype with a private constructor) and splits it internally. The import path's hand-rolled length check then becomes a *parse into the zip* rather than a post-hoc comparison. **The precedent is 250 lines away in the same subsystem:** `ClassIntern::intern` (`export.rs:1671-1679`) pushes to `names`, `fields` and `idx` in one act — the cured shape, already here.

---

### L1-4 · `AggregateValue.identity` is a hash cache whose miss path computes a *different function* — and the fixpoint's dedup partition keys on it. **Latent, resting on an unstated lemma.**

**Cite:** `src/value/value.rs:1019-1021`, `:1064-1080`, `:654-660`, `:848-857`, `:1031-1047` · `src/rete/kernel/fire/delta.rs:188-196`, `:279-281`

`Value::hash` for `Aggregate` (`value.rs:848-857`) branches:

```rust
if a.identity != 0 { a.identity.hash(state); }
else { a.nature.hash(state); a.class.hash(state); a.fields.hash(state); }
```

These emit **different bytes for the same value**. `Eq` (`:654-659`) compares `(nature, class, fields)`; the stamp (`:1073-1080`) hashes `(nature, class, fields)` — they agree on the *tuple*. But the Hash/Eq contract holds only if the branch predicate — `fields.iter().all(value_is_shallow)` (`:1031-1047`) — is **invariant under `Value::eq`**. That lemma is true today (I checked the only cross-variant `eq` arm, `Vec ≡ List` at `:626-628`; both land on `_ => false`). Nothing states it, nothing tests it, and `value_is_shallow`'s `_ => false` catch-all holds two facts — "deep" and "variant I never considered."

**What breaks when it drifts.** `seen_insert` (`delta.rs:188-196`) partitions the fixpoint's derived-fact dedup on exactly this: `identity != 0 → seen_ids: FxHashSet<u64>`, else `seen_rest: FxHashSet<Value>` — **two halves of one logical set, threaded as two `&mut` params through eight call sites** (`delta.rs:494-660`). Add one cross-variant `eq` arm — e.g. making `i64(1) == BigInt(1)` structural, which `values_equal` **already does at the language level** and which `value.rs:613-619` flags as a live divergence — and `i64` is shallow (`:1035`) while `BigInt` falls to `_ => false`. Two `Eq`-equal aggregates then hash differently, the HashSet contract breaks, and derived facts are duplicated or dropped at fixpoint. `production.rs:80` already reaches past the funnel with `seen_ids.reserve(…)`, which proves the door is open.

**Unrepresentable shape.** One `fn identity_of(&self) -> u64` used by *both* `from_parts` and the Hash arm, so the memo is a pure performance cache and the walk branch cannot compute a different function. Then `seen_ids`/`seen_rest` become one `SeenSet` type with a private field pair and a single `insert` verb — the `JoinRightIndex` shape.

---

## L2 — weaknesses

**L2-1 · `values_equal` and `values_compare` are "kept in lockstep" with no gate — and have already drifted.** `runtime.rs:11419` and `:11680`; the doc at `:11659` states the coupling, and `:11641` records the past break verbatim: *"Closes the orderable-but-not-equatable asymmetry (Instant had values_compare but not values_equal)."* Two ~200-line match ladders over the same variant space; `grep` for `values_compare` across `src/` and `tests/` finds no consistency test. **Remedy (cheap floor):** a gate asserting `values_compare(a,b) == Some(Equal) ⟺ values_equal(a,b) == Some(true)` over a Value corpus covering every variant. **Remedy (structural):** implement `values_equal` in terms of `values_compare` plus one explicit list of equatable-but-unorderable variants, so a new variant forces a decision in both.

**L2-2 · `wat/grep.wat` — one four-field extent spelled three times, and the cross-reference comment names only two of them.** `wat/grep.wat:50-52` says it plainly: *"nothing pins them together, so a rename of one must be made in both by hand."* The three sites are `:wat::grep::Span` (`:53-58`), `:wat::grep::Extent` (`:78-82`) — **and `:wat::grep::Match` (`:67-74`)**, whose own comment (`:66`) confirms it deliberately inlines the extent. The warning at `:50` omits `Match`. No gate exists (`ls tests/lint/` — nothing covers `defrecord` field-list agreement). **Remedy:** a lint asserting the three field lists are identical, or a `wat-fix` codemod-checkable single source; at minimum, correct the comment to name all three sites.

**L2-3 · 13 hand-written `pack_`/`unpack_` pairs encode wire field order twice.** `src/rete/export.rs` — `pack_compiled_cond` (`:1321`) pushes fields in literal order at `:1333-1341`; `unpack_compiled_cond` (`:1347`) reads them back by hand-written indices `1,2,3,4,5,6` at `:1352-1379`. Solvere's named duplicated-encoding shape, ×13 (`grep -c '^fn pack_\|^fn unpack_'` → 27). Most swaps are caught incidentally by `expect_idx`/`expect_seq` type mismatches, which is why this is L2 and not L1 — but that is a type accident, not a check. **Remedy:** if a derived codec is too large a change, a round-trip property test per pair (`pack ∘ unpack ≡ id` over a generated corpus) makes the drift falsifiable at ~1% of the cost.

**L2-4 · `AggregateValue.names` is `pub` beside `pub fields` with an invariant only a doc-comment holds.** `value.rs:1012-1014`: *"**Same length as `fields`, always.**"* — unchecked in `from_parts` (`:1064`), while `identity` right below it is private *specifically* so construction must go through the funnel. Consumers index across the pair (`session.rs:1233`: `a.names.iter().position(…)` then the field) and zip it (`edn_shim.rs:2932`). No live mutation site exists (`grep` for `.fields = ` / `make_mut` on aggregates → nothing), so this is latent. **Remedy:** make `names`/`fields` private with an accessor, or debug-assert the length in `from_parts`.

---

## L3 — judgement

- **The test census is now a production statement, and only a distant header says so.** `fire/mod.rs:826-833` puts a `#[cfg(test)]` `right_idx_appended` call **inside** `keyed_join_persistent`, a shipping function. The module header (`fire/mod.rs:36-40`) flags the shape and admits *"a reader who greps for `#[cfg(test)] fn` will miss it."* A header cannot travel to the site. This is the spell's `irreducible-tangle` category exactly — it deserves a `// rune:solvere(irreducible-tangle) — …` on the line above the call, where the reader who trips over it actually is.

- **`ClassIntern` and `JoinRightIndex` are the house style now; the repo should say so once.** Two independent sites (`export.rs:1671`, `session.rs:242`) have converged on *private parallel state + one insertion verb*. `CompiledCond::from_parts` and `CheckEnv`'s three def-maps are the same problem solved the other way. Naming the pattern once would make L1-2 and L1-3 look like violations rather than like ordinary code.

- **The inverse shape (one type, two concerns) is genuinely rare here.** I looked. `TestSummary` (`test_runner.rs:77`) mixes execution results with path-resolution diagnostics (`no_tests_discovered`, `file_count`), and `SocketListener` (`kernel/listener.rs:238`) mixes a peer-cred allow-set with a per-service frame budget — both are small, both are documented, neither is worth moving. This codebase's failure mode is over-splitting, not over-fusing. That is exactly why the D2 class keeps recurring.

---

## What I could not check, and why

- **I did not drive L1-1.** Read-only ward — no builds, no floor. The reachability chain is established by reading (`arm.rs:616` → `a ∈ beta_readers`; `join_parent` = all RootJoin|HashJoin at `arm.rs:542` → `b` is visited; pass order `delta.rs:539 < 602 < 612`), but **whether a real rule reaches the round-2 arm with a non-empty `dr` and a non-empty `old_left` is a claim I have not executed.** The probe is `[A ?k] [:where …] [B ?k] [C ?k]` with `C(k)` derived one round late for an already-seen `k`, native vs `$oracle`. Until that is run, L1-1 is a structural argument, not a measured defect. Given the calibration, I'd run it before anything else on this list.
- **L1-4's lemma is checked by my reading of one `eq` impl, not by exhaustion.** I read `Value::eq` (`value.rs:596-700`) looking for cross-variant arms and found one (`Vec ≡ List`). If a cross-variant arm exists in a `PartialEq` impl on a *contained* type I did not open, the lemma could already be false today. I did not open `PVec`, `HolonAST`, or `EnumValue`'s equality.
- **I read `src/runtime.rs` and `src/check.rs` by targeted grep, not end to end** — 40,883 and 22,509 lines. Roughly 3,000 lines of the 192,700 in `src/` were actually read this session. Braids in the untouched majority are not absent; they are unlooked-at. The `wat/` corpus (14,707 lines) got one comment-pattern grep and two file reads — that is reconnaissance, not a pass.
- **`tests/` was searched, not audited.** I used it only to check whether gates exist for the couplings above (`ls tests/lint/`, greps for `values_compare` and roundtrip names). I did not look for braids *inside* test code.
- **No finding here is blocked by the 265 gates or the 5420 floor**, which is consistent with the brief — but I did not verify that claim by running anything. It follows from the mechanisms (a silent `None` at `hash_join.rs:429`; a stale map read only under a flag flip; a length invariant that holds at both writers today), not from a measurement.

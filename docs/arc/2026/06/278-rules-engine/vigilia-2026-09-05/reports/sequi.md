# ward `sequi` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I have what I need. Writing up.

## L1 — `join_keys_cache` carries two meanings, and only one of its two writers maintains the second

**`src/rete/kernel/fire/pass/hash_join.rs:120`** · `let first_keying = if !join_keys_cache.contains_key(child_id) {`

`join_keys_cache` is declared as one thing — "HashJoin id → cached join-key names" (`session.rs:167`, "Not production memory") — but `hash_join_delta` also uses its *key presence* as a second, unnamed piece of state: **"the catch-up for J has already been done."** `first_keying` is derived purely from `contains_key`, and `first_keying` is the sole gate on the block at `hash_join.rs:160`, which is the **only** writer that ever builds `left_idx[J]` from the parent's cumulative beta (`hash_join.rs:270-283`). The only other writer of `left_idx` is step 5 (`hash_join.rs:371-380`), which appends `dl = d_beta[P]` — this round's delta only.

There is a second writer of `join_keys_cache`, and it does not do the catch-up:

**`src/rete/kernel/fire/mod.rs:802`** · `idx.join_keys_cache.entry(join_id).or_insert_with(…)` inside `keyed_join_persistent`, reached from `left_activate_join` (`pass/mod.rs:121`), which `filter_after_join` calls at **`pass/filter_after_join.rs:75`**.

**Mechanism.** For the shape `… → :where → HashJoin(P) → HashJoin(J)`:

- P's left parent is a Test, so P is never a child processed by `hash_join_delta` (its outer loop is `kind_ids.join_parent`, which is `RootJoin | HashJoin` only — `arm.rs:542`). P gets its tokens exclusively from pass 3.6.
- Round 1, pass 3 (`hash_join_delta`): P **is** in `join_parent` and J is its child, so J is visited — but `wm.beta[P].first()` is `None` (P has no tokens until 3.6), so line 140 `continue`s. No keying, no catch-up.
- Round 1, pass 3.7: P is on the frontier, J is a HashJoin child → `left_activate_join(J)` → `keyed_join_persistent` **inserts `join_keys_cache[J]`** and fully indexes `right_idx[J]`. `left_idx[J]` is never touched.
- Round 2+, pass 3: a derived fact reaches J's feeding alpha → `seed_dirty_join_parents` marks P dirty via `joins_fed_by` (`hash_join.rs:570-590`) → J is visited → `contains_key(J)` is now **true** → `first_keying = false` → **the catch-up never runs, and `left_idx[J]` stays empty forever.** `dl = d_beta[P]` is empty (P is only fed by 3.6, which does nothing this round), so step 5 adds nothing either.
- Step 4 (`hj_step4_term2`, `hash_join.rs:429`) is `old_left ⋈ Δright` and probes `left_idx[child_id]` → `None` → emits nothing. Step 3's term1 is `dl ⋈ all_right` with `dl` empty. Pass 3.6/3.7 skip J because `d_beta[P]` is empty (`filter_after_join.rs:70-73`).

**Consequence.** The join between P's already-established left tokens and a right element derived in a later round is **never produced**. Silent missing rows at the public query surface — the same failure class as the bug `left_activate_join`'s own doc block describes (`pass/mod.rs:89-105`, "the one outcome that cannot be right, because it is the one that lies"), one layer further in.

**What would fix it.** The catch-up flag is not the join keys. Give `hash_join_delta` its own "left index built for J" state — or, in the D2 spirit, make `left_idx` a type like `JoinRightIndex` that owns *both* the buckets and a mark of how much of `wm.beta[parent]` has been indexed, with one `push` door, so the catch-up becomes `wm.beta[P][already_left..]` and is idempotent regardless of who keyed J. Note the coupled hazard in L2-a: patching `first_keying` alone re-opens D2 on the right.

**Observable.** A `:where` followed by **two or more fact conditions** where the last condition's class is **derived by another rule** (so its alpha grows in a round after P's tokens exist). Native vs `$oracle` differential on the row count. Cheap internal witness: after fire, for every HashJoin J, `Σ|left_idx[J]|` vs `|wm.beta[parent(J)]|`.

**Why nothing sees it today.** The one fixture for this shape, `tests/rete/probe_arc278_where_is_positionally_free.wat`, inserts every `A`/`B`/`C`/`D`/`E` as an **input** fact. All right elements exist in round 0, so no join in it ever sees a Δright in a later round. The test is blind to this by construction — exactly D2's situation.

---

## L2

**a) The catch-up's right-index walk never reads the mark it now faithfully advances.**
`src/rete/kernel/fire/pass/hash_join.rs:186-187` — `if let Some(right) = all_right.as_deref() { for &el in right {` — pushes the **entire** alpha memory, not `right[already..]`, unlike `keyed_join_persistent` (`fire/mod.rs:822`, `for el in &right_elements[already..]`). The D2 cure made mark *advancement* unbypassable; mark *consultation* is still convention. The block's safety note (`hash_join.rs:153-154`, *"Safe: J produced ZERO tokens before first keying so there is nothing to double-count"*) argues from tokens produced, but what actually protects it is that `keyed_join_persistent` sets the same `join_keys_cache` key that gates `first_keying` — i.e. the L1 conflation is currently load-bearing as D2's guard. **Remedy:** make the catch-up read `right_idx.already(J)` and index the tail, so it is correct independent of who keyed J. **Observable:** the existing `right_idx_by_join` census row (`census.rs:70`) would show mark == population either way once the walk is tail-only; today, mutating `first_keying` to always-true would show a doubled bucket *with a matching mark* — which is why the mark alone can no longer discriminate.

**b) `left_activate_join` bypasses the `record_token`/`record_tokens` door and falsifies the door's own structural claim.**
`src/rete/kernel/fire/pass/mod.rs:150-157` hand-rolls the readers guard + `beta_written` census + both memory writes, 100 lines below `record_tokens` in the same file. Eleven other sites use the door. The door's ⛔ block (`pass/mod.rs:24-28`) states *"Here they are one act, so a future site cannot push without counting or count without pushing"* — that is a claim about structure, and this site is a counter-example inside the same module. It is currently equivalent (it also drops the documented `reserve`, `pass/mod.rs:65-66,71-72`). **Remedy:** `record_tokens(&mut wm.beta, d_beta, &arm.beta_readers, join_id, &joined)`. **Observable:** none today — the census and the memories agree; that is precisely the problem, and it is why a lint or the call itself is the only cure.

**c) The right index got a per-join census row; its sibling did not, and the left aggregate is never asserted.**
`src/rete/kernel/census.rs:59` `left_idx_tokens: usize` vs `census.rs:70` `right_idx_by_join: Vec<(i64, Option<usize>, usize)>`. The `right_idx_by_join` doc says outright *"the aggregate cannot see one join doubling while another is short"* — the lesson was learned on the right and not carried to the left. Worse, `left_idx_tokens` has exactly one consumer, `src/rete/kernel/tests/node_share_cost.rs:1022`, where it is **formatted into a printed table and never asserted**. **Remedy:** a `left_idx_by_join` row, plus a fire-end invariant that `Σ|left_idx[J]|` accounts for `|wm.beta[parent(J)]|`. This is the instrument L1 needs and is the reason L1 can live.

---

## L3

- **`gather_join_keys` is fed a different sample by each cache writer.** `hash_join.rs:130` passes `wm.beta[P].first()` (the parent's first *cumulative* token); `fire/mod.rs:804` passes `left_tokens[0]` (the first token of *this round's delta*). `gather_join_keys` (`fire/mod.rs:1461-1500`) reads only `elements[0]`, so the element argument is harmless, but the *binding* sample differs by provenance. The correctness assumption — every token at one node shares a binding key-set — is stated only for alpha elements (`fire/mod.rs:1851-1853`), never for beta tokens. One cache entry, two provenances, one unstated invariant.

- **`Encoders::presence_floor(&self, sym)` / `coincident_floor(&self, sym)`** (`src/vm_registry.rs:51,64`) take `sym` but ignore it after the first call — the value is memoized in a `OnceLock` keyed only by `dims`. The doc directly above (`vm_registry.rs:32-34`) claims *"one wat-fn invocation per tier **per sigma-setter**"*, which a `OnceLock` cannot deliver. Not live: `freeze.rs:552/586` installs the sigma once per program and `EncodingCtx::from_config` builds a fresh registry per program (spawn inherits only `ctx.config`, `runtime.rs:29328`, `kernel/spawn.rs:954`). It is a signature that lies about its dependency, and it would become live the moment a registry outlived a symbol table.

---

## What I could not check, and why

- **I did not build or run anything.** Read-only mandate. L1 is a mechanism argument assembled from code I read this session, not a driven observation. It has one unverified premise I could not settle by reading: that a network of the shape `:where → HashJoin(P) → HashJoin(J)` where J's feeding class is **derived** actually compiles to that node shape (rather than, say, the `:where` being hoisted). Everything else in the chain I traced to a line. **Build the fixture before acting on it** — the shape is a two-line edit to `probe_arc278_where_is_positionally_free.wat`: make `:wpf::C` a derived class produced by a second rule instead of an inserted fact, and diff native against `$oracle`.
- **I did not audit `wat/`.** The whole 1.5 MB stdlib is untouched by this cast; the chains there are wat-level and I spent the budget on the rete kernel, where the ward's prior finding was and where the branch is live.
- **`tests/` was read only where it bore on a finding** (`node_share_cost.rs:1022`, `probe_arc278_where_is_positionally_free.wat`). I did not sweep the ~4189-test corpus for other blind differentials; L2-c names the instrument gap that would make such a sweep mechanical instead of manual.
- **Concurrency was not examined.** `EncoderRegistry`'s `RwLock` (`vm_registry.rs:95`) and `ARM_TABLE` (`arm.rs:722`) are single-threaded in every path I read, but I did not enumerate spawn/thread entry points to prove it.

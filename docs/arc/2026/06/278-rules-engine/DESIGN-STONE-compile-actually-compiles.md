# DESIGN-STONE — `compile` actually compiles: rule-derived work moves to boot time

> **Origin (2026-07-31), the builder's cut:** *"the rules should be static anyways?… once they are
> inserted they are independent of facts?… so just make the rules fast before we run facts through
> them?.. this is a 'boot time' problem?.. not a 'post-boot runtime' issue?"*
>
> Grounded: yes. `wm.network` is **only ever read** during fire (`sorted_node_ids`, `get_node`,
> `node_children` — no mutation anywhere in `kernel.rs`), and the memories are cleared at the top of
> every fire. Every derived index — `alpha_by_type`, `alpha_cond`, the P8b reverse-lookups, and the
> two stones now queued (`…-alpha-discrimination-tree`, `…-compiled-conditions`) — is a **pure
> function of the network, i.e. of the rules alone.** Facts never touch any of it.
>
> So the honest statement is not "we should cache this." It is: **`:wat::rete::compile` does not
> compile.** It builds a network *representation*, and every actual compilation step is redone on
> every fire. The verb's name promises something it does not do.

## Why it matters — and why it does not matter yet

At `[50 100]`, `SETUP: indexes` is 0.250 ms against a 117 ms scan. Per-fire construction is free at
batch scale, and **the two queued stones are correct to build at fire-setup.** Nothing here says
otherwise.

It inverts in two regimes, both ahead of us:

- **Many rules, few facts per fire.** The eBPF tree took **2.9 seconds** to compile at 1M rules.
  Build that per fire and the discrimination tree costs vastly more than the linear scan it replaces.
  The tree's affordability at scale *depends* on this split.
- **Streaming (R25 `MACHINA CHAOS DOMAT`, task #7).** A live Session in a defservice, rules fixed,
  facts arriving per message. Reconstructing rule-derived state per fire spends the whole
  per-message budget rebuilding something that did not change.

This is the same split we already proved in the kernel: *"the compiler does the expensive join work
once in userspace. The kernel does only the fast traversal work."* Compile-once, walk-many. We have
it at the packet layer and not here.

## ★ THE ONE CONTRACT DECISION

**`Session` stays a `defrecord`. The compiled artifact never enters the Session value.**

This is not a preference — it is what the disk permits. Two walls, both grounded:

1. **`to_transient` rejects a struct outright** (`kernel.rs:420`):
   ```rust
   Value::Aggregate(a) if a.nature != Nature::Struct => a,
   other => return Err(… TypeMismatch …)
   ```
   Flip `Session` record→struct and the native freeze boundary refuses it on the first call.
2. **The ORACLE constructs Sessions, in pure wat** (`rete.wat:823`, `:844`). Give the record a native
   field and the oracle must fill it — with something pure wat cannot produce. That breaks
   `PARI GRADV` at the type level: both impls produce the same `Session`, or they are not in
   lockstep.

*(Kept visible: I first argued Session must stay pure because "R5's snapshot" and "293.W would stop
it crossing a wire." Both were wrong. R5 serializes `{facts, rules}` and **re-compiles** — it is an
argument that the Session is NOT stored. And nothing passes a Session over a socket; the one service
that holds one holds it in `:ephemeral` already (`query.wat:346`). The builder cut it — "who is
trying to pass a session over a socket?.. that's nonsense" — and he was right. The conclusion
survived; the reasoning behind it was invented. The two walls above are the real ones.)*

## What is pinned, from the ground

- **The artifact is native and `Arc`-based, never `Rc`.** The kernel reference uses `Rc<ShadowNode>`
  because it is a single-threaded userspace compiler. Ours crosses threads — `HashTrieMapSync` /
  `VectorSync` appear 56 times in `kernel.rs`, a deliberate Sync choice. `Rc` would not survive a
  `run-thread` locus. **Correcting the sketch in the tree stone, which copied `Rc` from the
  reference.**
- **It is a pure function of the network**, so it is always reconstructible and never authoritative.
  A snapshot stays `{facts, rules}` (R5); revive recomputes.
- **The oracle is untouched.** It never holds a compiled artifact and never needs one.

## The open fork — deliberately NOT resolved here

Where the artifact lives, and how a fire finds the one belonging to its network:

- **(a) A runtime-side cache keyed by a content-derived network identity.** Correct and
  oracle-safe, but hashing the network per fire is `O(network)` — at 2M nodes that is the cost we
  came to remove.
- **(b) A pure identity field stamped by `compile`** (an `i64` content hash) used as the cache key.
  Pure data, so the oracle can produce it and the record stays a record — but it changes the
  `Session` shape, which means a `wat/` corpus migration and touching the type both queued stones
  swore off.
- **(c) The holder keeps it.** The service already holds a Session TEMPLATE in `:ephemeral`
  (`query.wat:346`); it keeps the compiled artifact beside it. No Session change at all — but the
  fire entry point must accept it, and threading it through batch `fire-rules` leaks the
  optimization into the surface.

**Resolving this by fiat now would be the error this whole stretch has been about.** The decision
belongs to the consumer that actually needs a Session to survive between fires, and that consumer is
**R0** — where a streaming fire is a different entry point from a batch fire, and `(c)` may simply
fall out of the service's shape. Drawn now so R0 inherits the grounding instead of rediscovering it;
**built when R0 is drawn** (task #7, R25 `MACHINA CHAOS DOMAT`).

## The gate — when it is built

1. `compile` produces the artifact; a second `fire-rules` on the same Session **does not rebuild it**
   (asserted by a construction counter, not inferred from a timing delta).
2. `SETUP: indexes` on a second fire falls to ~0.
3. The differential holds: oracle == kernel on `facts` + `production-memory`, unchanged.
4. A Session round-tripped through `{facts, rules}` fires identically to one that never left memory —
   the artifact is derived, never authoritative.
5. The release floor and every grid axis's `:accuracy :match`, unchanged.

## Out of scope = REJECTED (affirmative cuts)

- **The tree and the compiled conditions themselves.** Their own stones; they land first, at
  fire-setup, and this moves only the *construction site*, never the structures.
- **Flipping `Session` to a struct.** Blocked by `to_transient:420` and by the oracle's constructors.
  Not a taste call.
- **Hanging an opaque inside the network `PersistentMap`.** It would put a resource inside a value
  declared pure — a lie in the type system, and exactly the smuggling
  `reference_struct_holds_resources_record_is_pure_data` names.
- **Keying on raw pointer identity** (`Arc::as_ptr`). Cheap and wrong: a freed-then-reallocated
  address collides, and an ABA hit is a silently wrong compiled artifact.
- **`wat/rete.wat`.** The oracle is never optimized — and under this design it never needs to change.

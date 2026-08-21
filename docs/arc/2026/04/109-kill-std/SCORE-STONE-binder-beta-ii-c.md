# SCORE — arc 109, β-ii-c: `type-params-used-in`, and `defservice` stops over-stamping

Rider: one flight, ~15 min, no STOP-1/2/4 — but it **STOP-3'd three of six sites** with ordering
reasons, and flagged a consequence it could not test.

| # | what | result |
|---|---|---|
| 1 | the intrinsic exists and is reachable from a macro body | ✅ |
| 2 | ★ boundary matching | ✅ `[K V]` vs `Vector<K>` → **`[K]`**; vs `HashMap<Key,KV>` → **`[]`** |
| 3 | `lru-svc::Record` becomes monomorphic | ✅ — it consumes nothing |
| 4 | `State` / `Admin` keep `<K,V>` | ✅ — they genuinely consume both |
| 5 | floor | ✅ **4855/4855, 0 FAIL** |
| 6 | clippy `-D warnings` | ✅ 0 |
| 7 | ★ the acceptance wall accepts the stdlib | ✅ — then found more, see below |

## ⛔ MY BRIEF SAID TWO LISTS. THERE ARE THREE, PLUS A CONTRACT.

I measured that the runtime registry and the F5 allow-list are independent and wrote that into the
brief. Two more surfaced only at the floor:

```
1  #[wat_intrinsic] registry          runtime dispatch          (briefed)
2  macros/eval.rs  is_pure_total      F5 macro admission        (briefed)
3  rete/purity.rs  intrinsic_meta     purity completeness gate  ⛔ MISSED
+  a runnable @example, ≥1            pure+det contract         ⛔ MISSED
```

Both fired as clean, well-designed gates naming their own remedy:

> *"pure+det intrinsic `:wat::core::type-params-used-in` has no runnable @example (≥1 required by
> contract)"*
> *"1 dispatch verb(s) have NO purity ruling and are NOT in the ledger … Fix it by CLASSIFYING the
> verb in `intrinsic_meta` … Adding it to `KNOWN_UNREVIEWED` is the LAST resort and is only honest
> for a verb whose ruling is genuinely open — say why in the commit."*

**Both closed properly rather than parked.** Two `@example`s were added — the positive case and the
`Key`/`KV` boundary trap, each predicted before it was run and each matching. And the verb was
**RULED pure ∧ deterministic ∧ total in `intrinsic_meta`**, not added to `KNOWN_UNREVIEWED`.

★ That makes it **the first `#[wat_intrinsic]` verb to carry a purity ruling.** Every existing one —
`Bytes::to-hex`, `show-source`, `render-doc`, the `:wat::intrinsic::*` family — sits unclassified in
`KNOWN_UNREVIEWED`, whose own comment says *"Nothing here is classified; 255.3 owns that."* Parking
this one would have been the sibling pattern and the gate's own last resort; its ruling is not open.

## ★★ THE WALL ACCEPTED THE STDLIB — AND THEN FOUND THREE MORE

Re-applied, the consumption wall passed `lru-svc`, proving the fix. It then rejected three
**hand-written** declarations:

```
:probe::PCache::GetRequest<K,V>       V declared, never used
:probe::PCtor::GetRequest<K,V>        V declared, never used
:wat-tests::PCache::GetRequest<K,V>   V declared, never used
```

Verified by reading: fields are `probes <- Vector<K>` and `limit <- i64`. `V` is genuinely unused.

## ⛔⛔ AND THEN THE WALL ITSELF WAS HALTED — those three are CONFORMANT

`wat/service.wat:443` documents the rule they follow:

> **THE MESSAGE CONVENTION (checker-locked in `synthesize_surface_protocol`)**: a parametric
> surface's `:messages` are parametric in **ALL** of the surface's params, in order —
> `PCache<K,V>` ⇒ `PCache::GetRequest<K,V>`, **even when a given message uses only some (or none)
> of them.** … it is what keeps the surface's `<S>::Op` and this service's `<fqdn>::Op` superset
> field-for-field identical (the `derive` edge and `retag-op'` both require that).

So the wall as ruled **contradicts a documented convention**, and the three "violations" are code
obeying it.

⚠ **But the convention is not actually locked.** Measured: `wat/cache.wat:169` declares
`Cache<K,V>` and `:171` declares `Cache::GetRequest<K>` — one param, not all — and the stdlib
loads and its tests pass. **The comment says "checker-locked"; the checker does not lock it, and the
stdlib's own most prominent parametric surface violates it.**
`[[feedback_a_comment_can_ship_a_gap_as_a_law]]`

★ **And this inverts my β-ii-c design's central claim.** I wrote that `cache.wat`'s hand-written
messages *"already obey the rule; `defservice` is the sole violator."* Under the documented
convention it is the reverse: the probes conform and `cache.wat` deviates. I read one file as
exemplary without checking whether a rule said otherwise.

## What shipped, and what did not

- **SHIPPED**: the intrinsic (all three lists + examples), and `defservice`'s Record/State/Admin
  per-type derivation. Floor green, clippy 0.
- **NOT SHIPPED**: the consumption wall. Parked again as
  `PATCH-param-spec-consumption-wall.patch`, now blocked on a builder ruling — is the message
  convention real (and the wall must exempt surface `:messages`), or is it stale (and `cache.wat`
  is right, the probes need fixing, and the comment needs deleting)?
- **STOP-3'd by the rider, correctly**: `Op` ×2 and `Handle` — their field vectors are assembled
  hundreds of lines after their names are built. The wall's silence on them proves they were never
  over-stamping; they genuinely consume their params.

---

## ⛔ THE HALT IS RESOLVED — the builder ruled, and the corpus already agreed

> Builder, 2026-08-21: *"the surface must declare its parameterized bindings… however the V is
> dropped… because it must be declared on the GetResponse… the K is on the request and the V is on
> the response."*

**Each message declares only what IT consumes; the surface's list is the UNION, not a quota each
message must restate.** And `wat/cache.wat` was already written exactly that way — the exemplar,
not the deviation:

```clojure
Cache<K,V>  ⇒  Cache::GetRequest<K>    (keys)     Cache::GetResult<V>    (values)
               Cache::GetResponse<V>   (values)   Cache::PutRequest<K,V> (Entry<K,V> — both)
```

So the wall ships as ruled, and the retired convention was the thing that was wrong.

### What changed to land it

- **`wat/service.wat:443`** — "THE MESSAGE CONVENTION" retired in place, with **both** of its false
  claims named: it was never checker-locked, and the file it cited as conformant
  (`cache.wat`) was the deviation from *it*, not from the rule.
- **Four over-stamped messages fixed**, each verified field-by-field first:
  ```
  wat-tests::PCache::GetRequest<K,V>  →  <K>    probes <- Vector<K>, limit <- i64   (V unused)
  probe::PCache::GetRequest<K,V>      →  <K>    probes <- Vector<K>                 (V unused)
  probe::PCtor::GetRequest<K,V>       →  <K>    probes <- Vector<K>, limit <- i64   (V unused)
  probe::PCtor::GetResponse<K,V>      →  <V>    results <- Vector<V>                (K unused)
  ```
  ★ The fourth I would have missed — the two probes' `GetResponse`s differ: `PCache`'s echoes the
  request (`echo <- Vector<K>`, `results <- Vector<V>`) so it legitimately keeps `<K,V>`, while
  `PCtor`'s carries only `results <- Vector<V>`. Same name, same file family, opposite verdicts.
  Reading the fields is the only thing that separates them.
- **Two concrete instantiations shrank with it** —
  `PCtor::GetResponse<wat::core::String,wat::core::i64>` → `<wat::core::i64>`.
- **Three probe headers** that asserted the old rule were corrected rather than left to rot.

### Final state

```
floor ......... 4855/4855, 0 FAIL          clippy ........ 0 under -D warnings
```

The wall is IN, no longer a patch. Live behaviour, re-run by hand:

```
(defrecord :user::R<T> [])                        REJECTED — named
(defrecord :user::R<T,U> [x <- T])                REJECTED — names U
(defenum :user::E<T> … [f <- i64])                REJECTED — named
(defrecord :user::R<T> [x <- Vector<T>])          clean  ← NESTED consumption
(defrecord :user::R [x <- i64])                   clean  ← monomorphic
(defn :user::f<T> [x <- i64] -> i64 x)            clean  ← functions out of scope, by ruling
(defsurface :user::S<K,V> … Req<K> … Resp<V> …)   clean  ← the ruled surface shape
```

★ **The wall found three real defects across its life** — one generated (`lru-svc::Record`), four
hand-written, and one false law (`service.wat:443`). None of them was visible to the regex census
that predicted zero violations.

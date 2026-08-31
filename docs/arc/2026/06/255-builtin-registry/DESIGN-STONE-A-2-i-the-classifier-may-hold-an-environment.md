# DESIGN — STONE A-2-i: the classifier may hold an environment

> **Builder, 2026-08-30:** *"it has been reasoned.. we continue.."* — ruling **A-2** admitted, the
> only option answering YES to all four questions
> (`DESIGN-STONE-A-the-classifier-cannot-follow-a-captured-fn.md`).

## Why this is A-2-**i** and not all of A-2

A-2 as ruled bundles two things: *give the classifier an environment*, and *use it at
`sort$native`'s door to impose and home*. **Stepping-stone test — does building the first alone make
the second more tractable? YES.** A-2-i is a pure capability addition, provable in isolation with a
two-line wat probe, changing no verb's behaviour. A-2-ii then operates on a settled classifier
rather than introducing the capability and consuming it in one breath.

**A-2-i ships the capability and NOTHING consumes it.** That is deliberate: the stone is judged by
whether the classifier can now answer a question it could not answer before, not by whether a verb
changed.

## The defect, restated at the exact line

`(keyfn a)` reaches the general call arm, where a bare head is read as a `Symbol`:

```rust
let head = match items.first() {
    Some(WatAST::Keyword(k, _)) => k.as_str(),
    Some(WatAST::Symbol(id, _)) => id.as_str(),      // <- `keyfn` lands here
    …
};
head_ok(head, axis, sym, &mut axis_seen, &at)?;
```

`head_ok` then tries: constructor → field accessor → `sym.has_function(head)` → rete namespace →
`intrinsic_meta(head)` → **default-deny**. A local binding holding a closure is in none of those.
Measured: `wat-scripts/scratch-pad/255-probe-the-classifier-cannot-see-through-a-closure.wat`.

## THE ONE CONTRACT DECISION — pinned

**The environment is `Option<&Environment>`, and `None` reproduces today's behaviour EXACTLY.**
Every one of the 19 existing `classify_expr` call sites passes `None` and is byte-identical in
behaviour. The classifier gains a capability; it changes no existing answer.

★ **This is what makes the stone provable:** the floor is the control. If any existing test changes
its verdict, the change was not additive and the stone is wrong.

## What ships

1. `classify_expr`, `head_ok`, `classify_fn` gain an `env: Option<&Environment>` parameter.
   **All 19 call sites are inside `src/rete/purity.rs`** — measured; the signature change crosses no
   module boundary.
2. In `head_ok`, **immediately before the final default-deny**: if `env` is `Some` and
   `Environment::lookup(head, at)` yields a `Value::wat__core__fn(Arc<Function>)`, recurse into that
   function's body against the same axis, carrying **that function's own `closed_env`**
   (`Function.closed_env: Option<Environment>`) rather than the caller's.
3. `find_axis_violation` gains an env-carrying sibling; the existing signature keeps its behaviour by
   passing `None`.
4. `:wat::rete::pure?` / `deterministic?` / `total?` pass **their own `env`** (they already receive
   one — `eval_rete_pure_intrinsic(expr, env, sym)`), which is what makes the capability observable
   from wat.

## ⚠ THE HAZARD, and it is the reason this is its own stone

`classify_fn` guards recursion with `seen: HashSet<String>` **keyed on the FQDN**, resolved through
`sym.get(fqdn)`. **An anonymous closure has `name: None` and is not in `sym`** — it has no key, so a
naive "follow the capture" recursion has no back-edge guard.

**The fix is already the codebase's own idiom:** `Value::wat__core__fn(Arc<Function>)`, and
`src/value/value.rs:684` already compares fn identity with `Arc::ptr_eq`. Guard the closure walk on
the `Arc`'s pointer address, in a set separate from the FQDN `seen`.

⛔ **A depth bound is NOT an acceptable substitute.** It would silently return the wrong answer on a
deep-but-finite capture chain, and this classifier's whole contract is that a `false` means
*"proven not"*, never *"gave up"*. `[[feedback_an_error_names_where_it_gave_up_not_what_is_missing]]`

## The proof — a wat-level probe, end to end

The capability is observable from wat because `pure?` holds an environment:

```clojure
;; keyfn is PURE  -> the comparator is now provably pure   -> true   (was false)
(:wat::core::let [keyfn (:wat::core::fn [x <- :i64] -> :i64 (:wat::core::* x 2))]
  (:wat::rete::pure? (:wat::core::quote
    (:wat::core::fn [a <- :i64 b <- :i64] -> :bool
      (:wat::core::< (keyfn a) (keyfn b))))))

;; keyfn is EFFECTFUL -> still refused                     -> false
(:wat::core::let [keyfn (:wat::core::fn [x <- :i64] -> :i64
                          (:wat::core::do (:wat::kernel::println "!") x))]
  … same comparator …)
```

★ **Both rows are load-bearing.** The first proves the capability was added; the second proves it was
added **without** widening — a classifier that started answering `true` for everything would pass
row 1 and is the failure this stone must not ship.

**And the unbound case must still deny:** the existing probe
(`255-probe-the-classifier-cannot-see-through-a-closure.wat`) asks about a comparator whose `keyfn`
is bound nowhere. It must **still return `false`** — no binding, nothing proven, default-deny.
That file is the negative control and must not change its output.

## Out of scope = REJECTED (not deferred)

- **`sort$native`'s door, the imposition, and the homing** — that is **A-2-ii**, the next stone. This
  one adds a capability nothing yet consumes.
- **`freeze.rs:803` opting in** — it has the identical blind spot and A-2-i makes the fix available,
  but changing a startup gate's verdicts is a behaviour change belonging with a consumer stone.
- **`map · mapv · filter · foldl`** — the W7 family. Unblocked by this, homed by neither this nor
  A-2-ii.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **A-2-i** capability alone, `None` = today | YES | YES | YES | YES | ✅ **ADMITTED** |
| **A-2 whole** capability + door + homing in one | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| **depth bound** instead of identity guard | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **A-2 whole Simple? NO** — introduces a classifier capability AND consumes it AND homes a verb in
  one stone; three verifications braided, and a red cannot be attributed.
- **depth bound Honest? NO** — returns "not proven" for "did not look far enough". The classifier's
  `false` must mean *proven not*.

## Acceptance

| what | command | expected |
|---|---|---|
| capability added | new probe, row 1 (pure `keyfn` bound in scope) | `true` (was `false`) |
| added WITHOUT widening | new probe, row 2 (effectful `keyfn` bound in scope) | `false` |
| negative control unchanged | `wat wat-scripts/scratch-pad/255-probe-the-classifier-cannot-see-through-a-closure.wat` | `true` / `false` / `false` — unchanged |
| additive, not behavioural | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

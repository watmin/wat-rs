# DESIGN — clamp the request, not the handler

**One line of restructure.** `wat/service.wat`, the `page-clamped` emission. Recovers the +7.8 %
that `a read declares its page` cost.

## WHY — the handler is spliced into BOTH branches

`wat/service.wat:2208`:

```wat
page-clamped = (:wat::core::if (:wat::i64::> (~page-limit-acc ~req-binder) ~page-cap-kw)
                 (:wat::core::let [~req-binder (~page-req-ctor ~@page-ctor-args)]
                   ~outcome-match)          ;; ← the ENTIRE handler, copy 1
                 ~outcome-match)            ;; ← the ENTIRE handler, copy 2
```

`~outcome-match` is the op's whole body plus its `Outcome` match. **Every op declaring
`:max-page` now carries two copies of itself in the serve arm.**

Measured, my runs ×3: publish **18466 → 19912**, `+1446 ms (+7.8 %)`, receives unchanged at
~4700 — so it is per-operation, ~154 µs across ~9400 clamped ops.

★★ Both clamps are cheap and correctly guarded — `runtime.rs`'s `clamp_request_limit` clones only
under `Some(Value::i64(n)) if *n > cap`, and the wat `if` is a compare. **The SCORE's "no-op
compare" is right about the check and cannot explain 154 µs.** The duplication can.

★★★ And the mechanism is already measured in this arc:
`probe-does-an-unused-arm-cost-in-a-service.wat` — **a large unused match arm costs ~30 µs inside
a `defservice` impl and 0 ns inside a `defn`.** That probe's arm was a fraction of a whole handler.

## ⛔ THE ONE CONTRACT DECISION — the conditional binds the REQUEST

```wat
(:wat::core::let [~req-binder (:wat::core::if (:wat::i64::> (~page-limit-acc ~req-binder) ~page-cap-kw)
                                (~page-req-ctor ~@page-ctor-args)
                                ~req-binder)]
  ~outcome-match)                            ;; ONE copy
```

Identical semantics: the handler sees a request whose `limit` is `min(asked, N)`. The `if` now
chooses a *value*, not a *program*.

★ Nothing about the bound, the truncation, or the tool changes. This is purely where the
conditional sits.

## THE RULE THIS EARNS

> **A macro must never splice a handler body into more than one branch.**

It is invisible in the source (the splice reads as one `~outcome-match` per branch, which looks
symmetric and correct) and it is expensive in exactly the place this tree cannot afford it — a
`defservice` serve arm. It is also mechanically checkable: count the occurrences of the body node
in the emitted form.

⚠ Whether to build that check is **not** this stone. Naming the rule is.

## OUT OF SCOPE — REJECTED

- **A lint counting body splices.** The rule's enforcement; its own stone, and only worth it if a
  second instance shows up.
- **The `:max-entries` write-side guard.** Read it: it wraps `send-recv-form`, not a handler, and
  the caller side has no `outcome-match` to duplicate. **Verify before assuming it shares the
  defect** — I have been wrong five times today by pattern-matching instead of reading.
- **The 2.4× at m=8** and **`setup`/`stop`** — the next two targets, unchanged.

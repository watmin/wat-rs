# DESIGN — Stone 249.3 — threading reborn as wat code (and the form-vocabulary + purity fence it requires)

**Status:** STRIKE-READY (probe-grounded). Split into **249.3a** (engine) → **249.3b** (threading in wat + HARD-CUT).
**Parent:** arc 249 — total-pure-macros. **Gate position:** 245 ✓ → **249** (engine built 249.2b; closing sequence: **249.3** → 249.4 → re-ward/stamp → INSCRIPTION) → 235 → rejoin 232.
**Probe substrate (FM-2-bis):** `tests/probe_arc249_threading_in_wat.rs` (diagnostic gap-map; commits `2e0b7bd2` + `186ca0ee`). **Contract:** `tests/probe_arc249_threading.rs` (the 5 threading mints).

---

## 1. What 249.3 was, and what the probe revealed

The breadcrumb scoped 249.3 as "rewrite `->`/`->>` as wat macros over the engine; HARD-CUT the Rust `thread_desugar`." That assumed the 249.2b engine was *ready* for threading. **It is not** — and a PROBE-LED diagnostic (not a from-inside verdict; per REALIZATIONS §"the practitioner is the failure domain") proved it by attempting the natural Clojure-faithful encoding and reading what the substrate said.

**The gap-map (empirically grounded, peeled across 5 diagnostic runs):**

| # | Capability threading needs | Status at HEAD |
|---|---|---|
| 1 | **eval-time quasiquote PURITY** (a program-body `` `(… ~(effect) …) `` must NOT run the effect at expand time) | **HOLE (F5-redux)** — runs unfenced. The load-bearing find. |
| 2 | `~@`-splice at eval-time `walk_quasiquote` | **ABSENT** (silent mis-expansion — leaves the literal symbol in the output) |
| 3 | form-shape predicate (`is this form a List?`) | **ABSENT** — `:wat::holon::is-List?` is a *holon* classifier, false over a form-value |
| 4 | `first`/`rest` decomposition over a form-value | **WORKS** |
| 5 | mixed `[acc & steps]` defmacro params + `~acc` unquote | **WORK** |

**249.3 grew** — same pattern as arc 249 itself (the canary reveals the real defect). Threading is the *first real consumer* of "macros are total-pure programs over forms," and it reveals the engine shipped without (a) the purity fence the program-body path needs, and (b) the form-manipulation vocabulary those programs require.

### 1.1 The F5-redux hole — the load-bearing find

The body-model (249.2b-ii) made macro bodies run through `runtime::eval`. A program body's *inner* quasiquote is walked by **eval-time `walk_quasiquote`** (`src/runtime.rs:10380`), whose unquote uses **raw `eval_inner`** (`runtime.rs:10402`) — NOT the fenced `macro_eval`. And **`validate_pure_total` blanket-SKIPS quasiquote contents** (`src/macros/eval.rs:99`). So:

```wat
(:wat::core::defmacro :evil [] -> :AST<wat::holon::HolonAST>
  (:wat::core::if (:wat::core::= 1 1) -> :AST<wat::holon::HolonAST>
    `(:wat::core::not ~(:wat::kernel::stopped?))   ;; ← runs :wat::kernel::stopped? AT EXPAND TIME
    `false))
```

`probe_arc249_threading_in_wat::diag_program_body_quasiquote_impure_unquote_fenced` (row E) confirmed `startup_ok = true` — the kernel call executed unfenced. This is the **same class as F5** (impure computed unquote at build time), in the **different path** the body-model opened. It violates arc 249's PURE thesis (the shockingly-stable bar: a macro CANNOT make a build impure / non-reproducible). It is NOT a defect in 249.2b-ii's commit (that was real progress) — it is a newly-reachable class the next stone closes, exactly as circumspicere found F5 *after* the engine existed.

**Ground-against-the-right-target near-miss (recorded as discipline):** the first probe used `(:wat::core::i64::+ ~(stopped?) 1)` and startup was refused — but by a *TypeMismatch* (bool fed to `i64::+`), not a purity fence. The impurity *ran*; the type error was a coincidence. The assertion passed for the wrong reason — a false positive. Only a type-compatible `(:wat::core::not ~(stopped?))` grounded the claim against PURITY and exposed the hole. (`feedback_ground_against_right_target`.)

---

## 2. 249.3a — engine: purity fence + form vocabulary

Three substrate changes. All in `src/runtime.rs` (eval-time quasiquote) + `src/macros/eval.rs` (the validator) + the allow-list. **Substrate work → sonnet.**

### 2.1 Close the F5-redux hole (load-bearing)

**`validate_pure_total` must descend into the CODE inside a quasiquote, while skipping the DATA.** Today it returns `Ok(())` at any `quasiquote`/`quote` head (eval.rs:99), skipping everything. The fix: walk the quasiquote template, tracking depth like `walk_quasiquote` does, and when an `(:wat::core::unquote X)` or `(:wat::core::unquote-splicing X)` *fires* (depth 1), recurse `validate_pure_total` into `X` (the computed expression — real code). Literal template nodes stay skipped (data). Nested quasiquotes bump depth; inner-quote material below depth 1 stays data until its own unquote.

- **Why the validator, not a context flag through `walk_quasiquote`?** `validate_pure_total` runs ONLY via `macro_eval` (eval.rs:73). So fixing it fences the **macro context exclusively** — runtime quasiquotes (`eval-ast!` payloads, etc.) reach `walk_quasiquote` via plain `eval` and keep FULL power, untouched. No flag to plumb through the shared eval path. The fence lives where default-deny already lives.
- **Failure mode after fix:** an impure computed unquote in a program-body quasiquote → `RefusedInMacro { head }` at validate time, BEFORE eval. Clean refusal, not a silent build effect.
- **Contract:** row E (`diag_program_body_quasiquote_impure_unquote_fenced`) flips green (startup refused). Promote row E into the engine contract `probe_arc249_macro_engine.rs` as **gate F** (it is a permanent purity invariant, not a 249.3-disposable diagnostic).

### 2.2 Add `~@`-splice at eval-time `walk_quasiquote`

`~@X` parses to `(:wat::core::unquote-splicing X)` (`src/parser.rs:320`). Expand-time quasiquote already handles it (`src/macros/expand.rs:1097` `splice_argument`); **eval-time `walk_quasiquote` has zero handling** (the `runtime.rs:10414` comment: "not yet surfaced as a real lab need" — it is now).

Port the expand-time semantics into `walk_quasiquote`'s `WatAST::List` arm at the **outer-list level** (where children are mapped, 10417-10419): when a child is `(:wat::core::unquote-splicing X)` at depth 1, eval `X` and **flatten** its forms into the parent's child-vector. Splice cases:
- `Value::Vec(elems)` → `value_to_watast` each element, splice all (mirrors `splice_argument`'s computed case).
- `Value::wat__WatAST(List items)` → splice the list-form's **children** (the threading case: `~@step` where `step` is a list-form value).
- else → `SpliceNotSequence` (the existing `MacroErrorKind`, runtime analog).

Fenced from birth by §2.1 (a computed `~@(expr)` is validated before eval).

### 2.3 Mint a form-shape predicate

Threading branches: a **list** step `(f a b)` → inject `acc`; a **bare** symbol/keyword step `f` → wrap `(f acc)`. No predicate over form-values exists (`is-List?` is holon-only). Mint a pure-total `List?`-over-a-form predicate, add to `dispatch_keyword_head_value` + the `is_pure_total` allow-list.
- **Name: intueri-named** (protocol — a spawned cast; `feedback_intueri_names_all_things`). Candidate semantics: `(<name> <form-value>) -> :wat::core::bool`, true iff the form-value is a `WatAST::List`. The cast weighs the name + whether it belongs to a small form-predicate family or stands alone (YAGNI leans alone — threading needs only `List?`).
- **Contract:** row C (`diag_is_list_over_form`) flips green.

### 2.4 249.3a verification

`probe_arc249_threading_in_wat` rows **A** (thread-last single), **B** (thread-last pipeline), **C** (List? predicate), **E** (purity fence) all green; row D (decomposition) already works. Plus engine contract `probe_arc249_macro_engine` gate F (purity). Plus `lib` green, clippy 0. **No HARD-CUT yet** — `thread_desugar` still stands; 249.3a only builds the vocabulary the wat macros will use.

---

## 3. 249.3b — threading in wat + HARD-CUT

With the vocabulary in place, `->`/`->>` become wat macros (Clojure-faithful fold).

**Encoding (thread-last shown; thread-first injects after the head):**
```wat
;; thread-last: fold acc through each step, splicing the step's children then appending acc
(foldl (fn [a step]
         (if (<List?> step)
            `(~@step ~a)              ;; list step → (f args… acc)
            `(~step ~a)))             ;; bare step → (f acc)
       acc steps)
```
Thread-first needs the head separated: `` `(~(first step) ~a ~@(rest step)) `` — `first`/`rest` over a form-value (works, §gap-map 4); `first` is `Option`-wrapped (projective intrinsic), so unwrap via `Option/expect`.

**The bare-head routing seam (four-questions-resolved, derivable from the arc telos — Clojure-faithful + one-canonical-path):** the contract calls threading with a *bare* `->` head (`(-> 5 …)`), but wat macros register under *keyword* names. Resolution: the desugar **LOGIC** moves to wat (the fold above, in a wat stdlib file — intueri-named home); a **thin** bare-symbol-head→keyword-macro **routing** stays in Rust at `expand.rs:144-153` (syntax-level recognition that `->`/`->>` as a list head dispatch to the wat macros — same category as `~@` itself being lexer/parser-level Clojure syntax). The ~50-line `thread_desugar` *logic* (expand.rs:215-267) is **HARD-CUT**; only the head-recognition seam remains, now routing to wat. **CONFIRM on strike:** read the dispatch and verify the seam holds; STOP if it disagrees (a clean diagnostic > a from-conviction workaround).

**Where the wat macros live:** an intueri-named wat stdlib file (NOT `wat/std/*` — retired; NOT `wat/runtime.wat` — retired 245.3a). Candidate `wat/core.wat` or a dedicated `wat/thread.wat`. intueri cast decides.

**Verification:** `probe_arc249_threading.rs` — all 5 mints green, zero `#[ignore]`, zero `-- --ignored` needed; `thread_desugar` grep-0 in `src/` (logic gone); `lib` green; clippy 0.

---

## 4. Four-questions ledger (the design)

- **Obvious?** A program manipulating forms must splice, branch on shape, decompose — and must not run effects at build time. The vocabulary + the fence are the obvious minimal set. **YES.**
- **Simple?** Each piece atomic: the fence extends one validator function (descend into unquotes); splice ports existing expand-time logic; the predicate is one primitive; threading is one fold. **YES.**
- **Honest?** Names that the engine wasn't ready (the F5-redux hole + the missing vocabulary) rather than pretending threading is a trivial macro or leaving the logic in Rust against the self-hosting thesis. **YES.**
- **Good UX?** Macro authors get the Clojure-faithful form toolkit + a clean refusal on impure unquotes; threading collapses to ~8 lines of wat. **YES.**

Stepping-stone split justified: 249.3b is impossible without 249.3a; 249.3a is independently verified by the diagnostic probe. Build the foundation, verify it, then build threading on settled ground.

---

## 5. Open items before/at strike

1. **intueri cast** — name the form-shape predicate (§2.3) + the wat threading-macro home file (§3). Spawned cast, not hand-named.
2. **Confirm the bare-head routing seam** holds when the dispatch is read (§3).
3. **Promote row E → engine contract gate F** (§2.1) — it is a permanent purity invariant.
4. **circumspicere** the closed engine after 249.3a (it found F5; it should survey the fence + splice for the blind spot the inward lenses miss).

---

## 6. The deposit

249.3 turns the 249.2b engine from "can run programs over forms" into "can run *safe, form-manipulating* programs over forms" — the purity fence makes the PURE thesis actually total (computed unquotes can no longer leak effects), and the form vocabulary makes the family of Clojure macros (`->`/`->>` first, then `cond->`/`when`/`condp`/`case`…) expressible in wat. The Rust `thread_desugar` falls willingly (#65 — the funeral rite for our own scaffolding; honored in `git log`, met again in Valhalla). The canary built the cage; the cage retires the canary.

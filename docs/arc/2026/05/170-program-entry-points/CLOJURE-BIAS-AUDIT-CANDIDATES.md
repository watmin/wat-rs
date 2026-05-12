# Clojure-bias audit candidates — surfaced during arc 170 slice 3

**Captured 2026-05-12.** wat is engineered to feel Clojure-familiar to LLM co-authors. When sonnet or opus reaches for a Clojure idiom and it's missing (or has different semantics than expected), that's diagnostic data. This note captures the bias moments from arc 170 slice 3's substrate work + lists Clojure idioms worth auditing.

## Bias moments — this session

### 1. Macro top-level splicing — reached for `forms`, should be `do`

**Where it surfaced:** Phase E (deftest macro rewrite). I wrote a BRIEF anchoring sonnet on `(:wat::core::forms ...)` because the OLD deftest expansion used it. Sonnet inherited my framing; Mechanism A failed because `forms` is a data-capture form (returns `Vector<WatAST>`), not a top-level splicer.

**The corrected reach:** `(:wat::core::do ...)` at top level IS the splicer per arc 157 § Scope Q1. `src/check.rs:6848` shows `collect_splice_defs_ctx` recursing into top-level `do` children. This matches **Clojure's `do` semantics + Racket's begin-splicing + CL's top-level progn** — the established Lisp pattern.

**Lesson captured:** When reaching for "macro emits multiple top-level forms" — the answer is `do`, not `forms`. Future BRIEFs should anchor on `do` first.

### 2. Pipe EOF — reached for `shutdown(SHUT_WR)`, should be `writer.close()`

**Where it surfaced:** Gap B BRIEF (Sender/close). I specified `libc::shutdown(SHUT_WR)`. Sonnet corrected — shutdown is socket-only; pipes use `close()`.

**Not Clojure-related.** Linux API knowledge gap on my part, not wat-vs-Clojure. Captured here for completeness.

### 3. Naming reflex — `parametric-keyword` (noun-result) vs `keyword/of` (verb-action)

**Where it surfaced:** Gap A naming. I initially named the form `parametric-keyword`. The `/gaze` ward chose `keyword/of` — reads as prose ("Keyword of Receiver and i64") and matches the Clojure-flavored `of` constructor idiom.

**Lesson:** Naming reflex favors result-nouns; gaze corrects toward action-verbs. The Clojure naming tradition (`->string` / `from-string` / `keyword/of`) is what `/gaze` converges to.

### 4. Naming reflex — `intern` candidate (lies relative to wat semantics)

**Where it surfaced:** Gap A naming. `:wat::core::intern` was a candidate. Gaze flagged it as Level 1 lie: `intern` in Clojure/CL implies a GLOBAL intern table with identity semantics (same string → same object pointer). wat doesn't guarantee that.

**Lesson:** Reaching for a familiar Clojure name without verifying the substrate matches its semantics is a Level 1 finding. Always verify.

### 6. Substrate-consistency assumption — `do` splicing in 2-of-3 passes

**Where it surfaced:** Phase E V2. I read `src/check.rs:6848` (`collect_splice_defs_ctx` arm for top-level `do`) + the arc 157 error message documenting "(2) inside a top-level `(:wat::core::do ...)`" — and concluded top-level `do` splicing was uniform across the substrate. V2 sonnet's probe revealed: top-level `do` recurses for `def` forms (check.rs:6848) and at runtime eval (runtime.rs:2018), but `register_defines` (function-registration pass for `define` forms) does NOT recurse into `do`. Two of three passes recognize the splicing; the function-registration pass is the gap.

**The Clojure analog:** In Clojure, top-level `do` splices uniformly across all phases (read → expand → compile → eval). The phases see-through `do` consistently. wat has the capability in 2 of 3 substrate passes — needs to be made uniform.

**Lesson:** Cherry-picking ONE substrate location to verify a capability is insufficient. Multi-pass substrates need every pass to be checked. "The check pass recognizes top-level `do`" ≠ "top-level `do` splices uniformly."

**Closure:** Gap C — extend `register_defines` + `register_stdlib_defines` to recurse into `(:wat::core::do ...)`, mirroring `register_runtime_defs`'s existing `do` arm at runtime.rs:2018-2023. ~10 lines.

### 5. Type-param syntax — sonnet aspired to `run-hermetic-with-io<:i64,:i64>` (turbofish)

**Where it surfaced:** Phase D Decision 1. Sonnet wrote the macro accepting full channel-type keywords (`:Receiver<i64>` / `:Sender<i64>`) because no `keyword/from-string` constructor existed at macro-time. The aspired turbofish syntax (`<:i64,:i64>`) is Java/Rust convention, not Clojure.

**Lesson:** Type-system syntax-bias (turbofish, angle-brackets, generics syntax) is non-Clojure. Clojure has no surface syntax for type parameters — they're inferred or passed as runtime values. When reaching for type-syntax sugar, ask whether the wat-Clojure shape would just be ordinary function call with type-keyword args.

**Closure:** Gap A's `keyword/of` substrate addition (commit `1174706`) means future macros can construct parametric type keywords from element-type ASTs. The turbofish bias goes away — `(keyword/of :Receiver ~elem-type)` is the Clojure-idiomatic shape.

## Audit candidates — verify which exist; mint what's missing

These are Clojure idioms an LLM-instance might reach for. Each needs verification: does wat have it? If yes, find it / document it. If no, decide whether to mint or document the deliberate omission.

### Control flow

- **Threading macros: `->` (thread-first) and `->>` (thread-last).** Very Clojure-distinctive. `(-> x f g h)` ≡ `(h (g (f x)))`. Do they exist in wat?
- **`when` / `when-let` / `if-let`.** Short-circuit-and-bind variants of `if`. Clojure-idiomatic. Do they exist?
- **`cond`.** Multi-way conditional. Per `/gaze` SKILL doc: "cond — not in the host language section (use `if` or `match`)" — **deliberately omitted; wat uses `match`.** Documented.

### Iteration / comprehension

- **`for` (list comprehension) / `doseq` (side-effect iteration).** Clojure-idiomatic for collection traversal. Do they exist?
- **`loop` / `recur`.** Clojure's TCO mechanism. Do they exist in wat?
- **`map` / `filter` / `reduce` / `fold` / `take` / `drop`.** Standard collection ops. Mostly should exist; verify the Clojure-conventional names.

### Higher-order functions

- **`comp` (function composition).** `((comp f g) x)` ≡ `(f (g x))`. Do we have it?
- **`partial` (partial application).** `((partial + 1) 2)` ≡ `3`. Do we have it?
- **`apply` (variadic call from list).** `(apply f [1 2 3])` ≡ `(f 1 2 3)`. Verify.
- **`identity` / `constantly`.** Trivial HOFs. Verify.
- **`juxt`.** `((juxt f g) x)` ≡ `[(f x) (g x)]`. Clojure-distinctive.

### Destructuring

- **Vector destructuring**: `(let [[a b] v] ...)` ≡ first/second binding. Per arc 169: struct-destructure form A landed. Vector destructuring status?
- **Map destructuring**: `(let [{:keys [a b]} m] ...)` ≡ extract by keys. Status?
- **Nested destructuring**: `(let [{:keys [a {b :nested}]} m] ...)`. Status?

### Collections

- **Sets**: Clojure has `#{}` literal. wat has Vector + HashMap; sets? Status?
- **Lazy seqs**: Clojure's `seq` abstraction (lazy). wat has Vector (eager). Arc 200 / 118 tracked as pending — known gap.
- **Persistent data structures**: Clojure's structural sharing. Status?

### Identity / equality

- **Symbols vs keywords.** Clojure distinguishes (`'foo` symbol; `:foo` keyword). wat has keyword. Symbols a thing? Status?
- **`identical?` (reference equality) vs `=` (value equality)**: Clojure distinguishes. Status?

### Concurrency primitives

- **`atom`** — Clojure's mutable cell with CAS semantics. wat has `ThreadOwnedCell` — different paradigm. Worth documenting the divergence?
- **`ref` / `agent`** — Clojure's coordinated/uncoordinated mutability. wat's zero-Mutex doctrine likely makes these non-applicable. Document the divergence.

## Triage approach (when consumed)

For each audit candidate:
1. Grep wat sources + spec docs for existing form
2. If EXISTS: document the canonical name + location (so future reaches find it)
3. If MISSING: decide
   - **Mint**: matches Clojure-spirit; substrate gap to close
   - **Document deliberate omission**: like `cond` — wat has `match` instead; cementing the choice
4. Append findings to this file

## How this captures discipline

Every bias moment surfaces a real signal: either wat is missing something Clojure has (mint or deliberately omit), or wat HAS it but the reach didn't find it (naming/docs gap). Both improvements compound — future sonnet/opus reaches converge faster.

The user's framing: *"we are very clojure inspired and i'm engineering this lang to feel familiar... i want to know when sonnet or yourself reaches for something and its missing - those biases i'm trying to capture."*

This file is the receptacle. Append as new bias moments surface.

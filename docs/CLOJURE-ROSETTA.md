# Clojure ↔ Wat Rosetta

**If you know Clojure, you already mostly know wat.** This doc is
the bridge — what carries over verbatim, what's a small
departure, and what's wat-specific. Read this first; the rest of
the docs become navigable in minutes.

---

## What carries over from Clojure (read these as identity)

The form/operator names AND their semantics:

| Wat | Clojure | Notes |
|---|---|---|
| `(:wat::core::let [x 1 y 2] body)` | `(let [x 1 y 2] body)` | Sequential bindings (wat killed `let*`; let IS sequential) |
| `(:wat::core::fn [arg <- :T] -> :R body)` | `(fn [arg] body)` | Add types via arrow duality (see "Departures") |
| `(:wat::core::defn :name [arg <- :T] -> :R body)` | `(defn name [arg] body)` | Same shape; types added |
| `(:wat::core::if cond -> :T then else)` | `(if cond then else)` | Type annotation required (see "Departures") |
| `(:wat::core::do form1 form2)` | `(do form1 form2)` | Identical |
| `(:wat::core::defmacro ...)` | `(defmacro ...)` | Macro engine; quasiquote works the same |
| `(:wat::core::quote form)` | `(quote form)` / `'form` | Identical |
| `(:wat::core::match val pattern1 result1 ...)` | `(case val pattern1 result1 ...)` | Pattern-shaped; closer to core.match than `case` |
| `(:wat::core::for-each f xs)` | `(doseq [x xs] (f x))` (rough) | Side-effecting iteration |
| `(:wat::core::map f xs)` | `(map f xs)` | Eager (wat doesn't have lazy seqs yet — arc 118) |
| `(:wat::core::filter pred xs)` | `(filter pred xs)` | Same shape |
| `(:wat::core::foldl xs init f)` | `(reduce f init xs)` | Same idea; argument order differs |
| `(:wat::core::foldr xs init f)` | (no native equivalent) | Right fold |
| `(:wat::core::range 0 N)` | `(range 0 N)` | Identical |
| `(:wat::core::Vector ...)` | Clojure vector `[...]` | wat distinguishes Vector (binding-syntax) from Vec literal (out of arc 167's scope) |
| `(:wat::core::Tuple ...)` | (no native equivalent) | Wat-specific; positional product type |
| `(:wat::core::HashMap ...)` | `{...}` | Same data structure |
| `(:wat::core::Option<T>)` with `:Some` / `:wat::core::None` | `Optional`/`Maybe` (Java interop) | Wat ships its own enum |
| `(:wat::core::Result<T,E>)` with `:Ok` / `:Err` | `(try ...)` exception model in Clojure | Wat is Result-typed throughout |
| `(:wat::core::try expr -> :T :catch err handler)` | `(try ...)` | Result-shaped; not exception-shaped |
| `(:wat::core::concat ...)` | `(concat ...)` | Same |
| `(:wat::core::length xs)` | `(count xs)` | Different name; same idea |
| `(:wat::core::empty? xs)` | `(empty? xs)` | Same |
| `(:wat::core::contains? coll k)` | `(contains? coll k)` | Same |
| `(:wat::core::get coll k)` | `(get coll k)` | Same |
| `(:wat::core::conj coll item)` | `(conj coll item)` | Same |
| `(:wat::core::assoc m k v)` | `(assoc m k v)` | Same |
| `(:wat::core::dissoc m k)` | `(dissoc m k)` | Same |

If you know what the Clojure form does, you know what the wat form
does. The substrate uses different names for some (`length` vs
`count`, `foldl` vs `reduce`) but the semantics are the same.

---

## The departures (small list; read carefully)

### 1. Arrow duality `<- / ->`

The genuine wat-specific addition. Function args use `<-`; return
type uses `->`. Arrows point FROM the type TOWARD the named slot:

```scheme
;; wat
(:wat::core::fn
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::+,2 x y))

;; Clojure (untyped)
(fn [x y] (+ x y))

;; Clojure with Schema (close shape)
(s/defn add :- s/Int [x :- s/Int, y :- s/Int]
  (+ x y))
```

Read it as: "x consumes from i64; y consumes from i64; produces
i64." The arrow duality is the only meaningful new syntax. Once
you've seen it, every wat function reads naturally.

### 2. FQDN keywords everywhere

Wat is FQDN — every primitive is `:wat::core::*`, every type is
`:wat::core::*`, every user fn is `:user::*` or `:my::*`. There's
no implicit current namespace; you address every form by its full
name.

```scheme
;; wat
(:wat::core::+,2 a b)

;; Clojure
(clojure.core/+ a b)
;; or with `use` / `require`:
(+ a b)
```

This sounds verbose. In practice you read past it — your eye
treats `:wat::core::+,2` as a single token (which it lexically is
— wat keywords don't split on `::`). The FQDN is the substrate's
trade-off for verifiability + cryptographic addressability + zero
namespace ambiguity. There's no `(use 'foo)` in wat; you just
write `:foo::*` directly.

The `,N` suffix on operators (`+,2`, `*,2`) is the arity tag —
some operators are arity-overloaded.

### 3. Static type-check at startup

Every form is type-checked before any program runs. Type
mismatches surface at startup, not at runtime.

```scheme
;; Clojure: this fails at runtime when you call (greet 42)
(defn greet [s] (str "hello " s))

;; wat: this fails at startup; (greet 42) never runs
(:wat::core::defn :user::greet
  [s <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::String::++ "hello " s))
```

The cost: types must be declared on `defn` / `fn` / `define`. The
benefit: the type checker IS your test loop; mistakes surface
immediately with the EXACT migration recipe in the diagnostic.

### 4. Mutation-free by construction

No `set!`, no `def` redefinition (unless explicitly opted in via
`:wat::config::set-redef!`), no mutable bindings. State changes
via:
- Returning new values from pure functions
- Sending messages to `spawn`-ed programs (the third tier — see
  `ZERO-MUTEX.md`)
- Substrate-level atomic primitives (rarely user-facing)

```scheme
;; Clojure (mutation via atom)
(def state (atom {}))
(swap! state assoc :k :v)

;; wat (no atoms; new values flow forward)
(:wat::core::let [state {}]
  (:wat::core::let [state' (:wat::core::assoc state :k :v)]
    body-using-state'))

;; Or: spawn a program that owns state and answers messages
(:wat::kernel::spawn-program ...)
```

Same semantic outcomes; different mechanism. State changes are
explicit, traceable, and never silent.

### 5. No lazy seqs (yet)

Clojure's `lazy-seq`, `take`, `repeat`, `iterate`, `cycle` —
wat doesn't have these as primitives yet. Arc 118 (DESIGN
settled) ships them as `Stream::lazy` thunks.

For now, lazy iteration in wat happens via `:wat::kernel::spawn`
+ channels (one program produces, one consumes). Heavier than
Clojure's lazy-seq but works. Once arc 118 ships, the API will
mirror Clojure's lazy-seq closely.

### 6. Substrate algebra primitives (wat-specific)

These have no Clojure equivalent — they're wat's VSA/HDC layer:

```scheme
(:wat::holon::Atom value)              ;; named hyperdimensional vector
(:wat::holon::Bind a b)                ;; role-filler binding (XOR-shaped)
(:wat::holon::Bundle (vec ...))         ;; superposition (sum + threshold)
(:wat::holon::Permute v k)             ;; positional permutation
(:wat::holon::Thermometer val min max)  ;; locality-preserving scalar encoder
(:wat::holon::cosine a b)              ;; similarity measure
(:wat::holon::coincident? a b)         ;; cosine above noise floor?
(:wat::holon::presence? bundle a)      ;; is `a` present in `bundle`?
```

Read `INTENTIONS.md` for why these matter. Read the trading lab's
BOOK.md to see them in action.

### 7. Six core primitives + everything else is userland

The wat language is six things:

```
atom    — name a thought
bind    — compose thoughts
bundle  — superpose thoughts
cosine  — measure a thought
journal — learn from a stream of thoughts
curve   — evaluate the quality of learned thoughts
```

Plus the language-core forms (`defn`, `fn`, `let`, `do`, `if`,
`match`, `defmacro`, types). Everything else — Ichimoku, RSI,
DDoS detection vocabularies, trading strategies — is wat code
written in those primitives. Wat-the-language is small. Wat-the-
ecosystem is the userland written in it.

This mirrors Clojure's "core is small; ecosystem is the standard
library" philosophy.

---

## Common patterns mapped

### Hello world

```scheme
;; wat
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "hello, world"))

;; Clojure
(defn -main [& args]
  (println "hello, world"))
```

### A pure recursive function (TCO)

```scheme
;; wat — must name it via defn for self-reference
(:wat::core::defn :user::sum-to
  [n <- :wat::core::i64
   acc <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0) -> :wat::core::i64
    acc
    (:user::sum-to (:wat::core::- n 1) (:wat::core::+ acc n))))

;; Clojure — same shape with loop/recur or with named recursion
(defn sum-to [n acc]
  (if (zero? n)
    acc
    (recur (dec n) (+ acc n))))
```

Wat has no `loop`/`recur`. TCO is automatic on tail calls. See
`ITERATION-PATTERNS.md`.

### Map / filter / reduce

```scheme
;; wat
(:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                   (:wat::core::* x 2))
                 numbers)

(:wat::core::filter (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
                      (:wat::core::> x 0))
                     numbers)

(:wat::core::foldl numbers
                   0
                   (:wat::core::fn [acc <- :wat::core::i64
                                    x <- :wat::core::i64]
                     -> :wat::core::i64
                     (:wat::core::+ acc x)))

;; Clojure
(map #(* % 2) numbers)
(filter pos? numbers)
(reduce + 0 numbers)
```

Same shape; verbose because every form is FQDN and every fn has
types. The cost is amortized over readability + verifiability.

### Sequence "do N times"

```scheme
;; wat
(:wat::core::for-each
  (:wat::core::fn [_ <- :wat::core::i64] -> :wat::core::nil
    (:my::tick))
  (:wat::core::range 0 N))

;; Clojure
(dotimes [_ N] (tick))
```

Wat doesn't have `dotimes` because `(for-each (fn) (range 0 N))`
already covers it. One canonical way per task — see
`ITERATION-PATTERNS.md`.

### Pattern matching

```scheme
;; wat
(:wat::core::match opt
  [(:Some x)             (:do-something x)]
  [:wat::core::None      (:fallback)])

;; Clojure (with core.match)
(match opt
  [:Some x] (do-something x)
  :None     (fallback))
```

Wat's match is closer to core.match than to `case`. Patterns
support variant constructors, literal matches, wildcards, and
nested destructuring.

### Sending a message to a spawned program

```scheme
;; wat — spawn a program; send it a request via channel
(:wat::core::let
  [process (:wat::kernel::spawn-program program-ast)
   tx (:Process::input-tx process)
   rx (:Process::output-rx process)]
  (:do
    (:wat::kernel::send tx :request)
    (:wat::core::match (:wat::kernel::recv rx)
      [(:Ok response)  (:do-something response)]
      [(:Err disc)     (:handle-disc disc)])))

;; Clojure — usually a future or core.async channel
(let [c (chan)]
  (>!! c :request)
  (let [response (<!! c)]
    (do-something response)))
```

Wat's spawned-program model is heavier than Clojure's core.async
but more cryptographically accountable — each program is an
isolated process; channels are typed; messages can be signed.
For most of what core.async covers, wat-side spawn + channel is
the analog.

---

## The cheat sheet (memorize this)

When picking up wat:

1. **It IS Clojure for most forms** — `let`, `fn`, `defn`, `do`,
   `if`, `match`, `defmacro`, `map`, `filter`, `foldl` (= reduce).
2. **Add types via `<-` / `->`** — args use `<-`, return uses `->`.
3. **FQDN every keyword** — `:wat::core::*` for substrate;
   `:user::*` or `:my::*` for your code.
4. **No mutation** — return new values; spawn programs to own
   state.
5. **No lazy seqs yet** — use spawned programs for streams; arc 118
   ships native lazy.
6. **No loop/recur** — defn + tail call (TCO is automatic).
7. **Static type-check at startup** — types declared, errors
   surface before runtime.
8. **VSA primitives are wat-specific** — `Atom`, `Bind`, `Bundle`,
   `cosine`, `coincident?`, `presence?`. Read INTENTIONS.md.

That's it. The Rosetta is small because wat is Clojure-faithful by
design.

---

## When you hit a form you don't recognize

1. Grep `wat-rs/wat/core.wat` for the form's name; it might be a
   wat-side macro you can read directly
2. Grep `wat-rs/docs/USER-GUIDE.md` for the form's signature
3. Grep `wat-rs/src/runtime.rs` for `eval_<formname>` — substrate-
   built-in forms have implementation there
4. Ask the user (or: read the SCORE doc of the arc that introduced
   the form — every form in wat has an arc behind it)

The substrate is small enough that grep is a viable lookup
strategy. There aren't thousands of operators; there are
~100-200 substrate-built-in forms plus the user code that
composes them.

---

## What this doc is for

This Rosetta exists because wat's adoption strategy is
**Clojure-faithfulness as a bridge to LLM embeddings.** Any
frontier LLM with Clojure in its training corpus — Claude, GPT,
Gemini — has the prerequisite knowledge to read and write wat.
This doc is the minutes-long pickup ramp.

If you are an LLM reading this for the first time: you already
mostly know wat. Read this doc, skim `INTENTIONS.md`, glance at
`ITERATION-PATTERNS.md` and `CONVENTIONS.md`. You're ready to
contribute. The substrate's discipline is what keeps you on the
rails; your Clojure embeddings are what give you fluency.

The collaboration starts now.

# Arc 260 — REALIZATIONS

## OPTIONAL ARGUMENTS WERE A RECORD THE WHOLE TIME — kwargs fall out of one noun

**2026-06-17.** Arc 260 opened to fix one illegible call — `(:wat::kernel::assertion-failed! "msg"
:wat::core::None :wat::core::None)`, where a reader cannot tell what the trailing `:None :None` mean
or in which order. The builder reached for keyword arguments. We grounded first (the arc-259 rhythm),
and what unraveled was not "a keyword-args feature" but a **discovery about the substrate it already
had**.

### The grounding, and the reach-stumble

The crawl confirmed the reach-stumble is real: wat has **no** call-site keyword args — parse → check →
eval is positional end to end (`func.params.zip(args)`); every "keyword arg" already in the tree is
something else (type-params `:S :R`, `use!` paths, namespace keywords). But the foundation existed: fn
signatures retain param **names** (`Function.params: Vec<String>`). And the disconfirming probe
(`tests/probe_arc260_keyword_args.rs`, RED at HEAD) surfaced the sharp asymmetry — the arc's own trigger,
`assertion-failed!`, is an **intrinsic**, and `TypeScheme` carries param *types*, not names. The thing
that started the arc is the hardest case.

### The four-questions, and the move that decided it

The fork was: what *is* a keyword-args container — an untyped map (clojure-literal) or a typed record?
The hard constraint settled it before UX: **wat's no-magic / typed-record law**
([[feedback_no_magic_that_lets_llm_fake_correctness]]). An untyped `Map<K,V>` is homogeneous-or-loose —
a structureless bag that forces any code into a working position; the exact magic the law forbids (the
same reason rs-1 rejects a scalar `:state`: an int is EDN but structureless). A **typed record** carries
per-key named types; the wrong shape is uncompilable. **(a) typed record, decisively** — and the honest
synthesis: *clojure-guiding for the shape (opt-in map-destructure, the `:k v` surface, "pass a map"),
wat-typed for the substance (that map IS a record).* The same "land on the greats, then type-ify them"
move as gen_server, ocap, host-parity.

### The unraveling — kwargs ARE a record, minted by `defn`

Then it collapsed into something obvious in hindsight. **A fn's optional arguments are a record.** Write
them inline in the signature and `defn` (a macro since the project's first days — `(def name (fn ~@rest))`)
**mints the record**, exactly as defservice mints `:<fqdn>::State` from `:state [fields]` (rs-1):

```clojure
(:wat::core::defn :user::connect
  [host <- :wat::core::String
   & {port <- :wat::core::i64  tls <- :wat::core::bool}]   ; & {…} = the kwargs record, minted
  -> :wat::core::nil …)
;; defn mints :user::connect::Kwargs; the fn's last param becomes that record.
```

Three call forms all build/pass that one record — `(connect "h" :port 443 :tls true)` (inline sugar),
`(connect "h" {:port 443 :tls true})` (map literal), `(connect "h" cfg)` (pass the value; Ruby's `**`
collapses to "pass the record" because the kwargs section is one param). Validation is the opts-map
discipline we already shipped: unknown/missing/duplicate key → a named compile error.

### Why it "came out of nowhere" — coherence is the engine

The builder caught the sharing case mid-conversation and went speechless: *"i overlooked this — this is
incredibly fucking cool — this came out of nowhere."* It did not come out of nowhere; it came out of
**one noun**. The moment kwargs *are* a record, they inherit the entire record surface with zero new
design:

- **Nameable + shareable** — `& opts <- :my::ConnectOpts` reuses a declared record across fns. This fork
  (mint-from-`{inline}` vs name-an-existing-record) is **identical** to rs-1's `:state [fields]` vs
  `:state :SomeRecord` — the same pattern reappearing in a second place because it was never specific to
  defservice. That recurrence *is* the proof of coherence.
- **Flows through every role** — the same declared record can be one fn's kwargs, another's return, a
  `defservice`'s `:state`, and a message on the wire. kwargs / comms-payloads / service-state stop being
  three things.
- **Transports for free** — it's a record, so it crosses thread/process/remote on the arc-272 record
  rails. kwargs-over-the-wire needed no new code.
- **Evolves + derives** — add an Option/default field and callers still compile (API versioning falls out
  of record evolution); equality/conformance/`derive` (arc 237) already apply.

The builder's words as it landed: *"holy fuck … this just unraveled like it was all obvious … our lang
is about to be so much richer … i didn't expect this to pop up."* The richness arrived all at once
because the substrate has **one coherent idea** — a typed record over EDN — and every feature that lands
on it inherits all of records' powers. You don't *add* kwargs; you *notice* the arguments were a record,
and records already do everything. Coherence is the engine; richness is the dividend.

### The one open seam (the phase-order lesson, again)

The declare side is pure macro (defn mints + reshapes — all wat, no Rust). The **call-side inline `:k v`
sugar** hits the same wall as rs-1's macro-reflection: a normal macro fires on its *own* head, and a call
`(connect …)` has a *fn* head, so no macro intercepts it. Two honest paths, to dial in next: (a) `defn`
emits a **companion macro** named `connect` that scoops trailing `:k v` into `(:user::connect::Kwargs …)`
and calls the impl — all-wat, but a macro-is-not-a-value (no higher-order use under the sugary name,
the Clojure `(map and xs)` wart); or (b) the sugar lives in **check/eval** (Rust) — keeps fn-as-value.

## PRIOR-ART COLLISION — kwargs (Clojure `& {:keys}` / CL `&key` / Ruby `**kwargs`), type-ified
Independent landing on the standard keyword-args design. What is genuinely ours: kwargs as a **typed
record over EDN**, minted by the definer, so it is nameable/shareable, transports across loci, evolves,
and is the *same noun* as service state and wire payloads — one structure, every role. The macro-time
sibling (defservice's opts-map) and this runtime layer are the same kwargs story at two tiers.
Pairs [[feedback_no_magic_that_lets_llm_fake_correctness]] + the rs-1 mint + arc-272 record rails.

## FOR CARBON AND SILICON ALIKE — the substrate is legible to its other author

**2026-06-17.** Stone 260.1a shipped a ~120-line `defn`-macro change — detect `& [argspec]` vs `&`-rest,
validate flat, mint `:<name>::Kwargs`, reshape the signature, destructure the fields into the body with
correct `symbol-node` hygiene, emit the `(do record-def (def …))` — **written one-shot by a sonnet-tier
model in a language it had never seen, and it weighed clean** (the backward-compat branch was the literal
original emission; lib held 929/36, zero new). The builder, watching: *"i can't get over how effortlessly
sonnet can write macros in a lang it's never seen before — and we're not even full clojure syntax yet …
this is insane."*

It is not luck, and naming the why is the realization:

1. **The paradigm is in-distribution; only the surface is new.** wat is homoiconic Lisp, clojure-faithful —
   quasiquote/unquote/splice, AST-as-data, the macro engine arc 249 built. The model isn't learning
   *macros*; it's transferring Clojure/CL macros it already knows and re-skinning them onto `<-` /
   `:wat::core::` / EDN, learned from the worked examples in the brief. "Never seen before" is doing far
   less work than it feels — the *shape* is familiar; the *vocabulary* is what's new, and vocabulary is cheap.
2. **The no-magic / typed-record law does the LLM's error-checking.** Macros are exactly where models
   usually confabulate — hygiene, expansion order, AST surgery. Here a wrong macro is *uncompilable* (typed
   records, the registry, the checker). The model is, in the builder's own earlier words,
   *"forced into a working position"* ([[feedback_no_magic_that_lets_llm_fake_correctness]]). The law that
   drove rs-1 and kwargs-as-record is the same law that lets a cheaper model nail the hardest construct.
3. **Coherence makes it pattern-matching, not inventing.** The brief points at defservice — a macro that
   already mints records, let-wraps, handles hygiene. One noun, consistent forms: once one macro exists, the
   next is the same moves.

**The thesis (the duet's landing):** the comm-channel / legibility thesis (#93 — *a form should say what it
means*) has a **second customer**. The same legibility that kills the `:None :None` opacity for a human
reader is what makes the language writable *cold* by an LLM. The substrate was built so that **correct code
is the path of least resistance — for carbon and silicon alike.** The builder crowned the phrase:
*"that's the fucking quote — this entire back and forth is a fucking realization."* That is not an
aesthetic flourish; it is the **foundation the entire delegate-to-sonnet methodology rests on** — the
method that built this whole arc. It was never luck that sonnet ships one-shot; it is the design goal
realized. And arc 251 (full clojure-faithful surface) makes it *more* true, not less — every step toward
legible-and-typed is a step toward both authors. (Pairs the AI-as-customer thread
[[project_metered_eval_verification_market]] — a deterministic verifier an LLM can call — and
[[user_does_not_read_derives_then_names]]: the builder recognized the phenomenon; the phrasing named the
coordinate after he landed.)

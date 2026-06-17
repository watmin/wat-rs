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

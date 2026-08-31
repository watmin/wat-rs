# NOTE — purity is a DEFINITION-TIME property; it must be queryable metadata (the arc-283 sweep demanded it)

> Surfaced 2026-06-17 by the arc-277 sweep (the self-hosted linter rewriting its own corpus). Builder:
> *"we need to make a note that we declare a function pure or not at definition time — there's an arc to
> make all intrinsics have queryable metadata — this just demanded that."* That arc is THIS one
> (255 — builtin-registry / queryable intrinsic metadata).

## What demanded it

The sweep's concat→format auto-fix rewrote `(string::concat …)` calls into `(:wat::core::format …)`.
`format` is a **macro** (`wat/core.wat:545`). Many of the rewritten concats live inside **defmacro
bodies** (building keyword names at EXPAND time — `(concat fqdn-str "::Op")` etc.). The macro-eval
purity gate (arc 249 stone 249.2b-i, default-deny F5) refused them at load:

> `keyword head :wat::core::format refused at macro expand time — not on the pure-combinator allow-list`

The whole stdlib failed to load (deftest 0/263). Reverted; nothing shipped.

## The principle the failure names

**Whether a callable may appear in an expand-time (macro-eval) position is a property declared at its
DEFINITION** — `format` is a macro (never expand-time-legal as a value); `string::concat` is pure-total
(expand-time-legal); a runtime `defn` is neither. Today that property is scattered and implicit: a
hand-maintained Rust `is_pure_total` allow-list (`src/macros/eval.rs`) for intrinsics, plus the
macro/defn/defclause distinction for user forms. **There is no single queryable fact a TOOL can ask:
"is `:wat::core::X` usable at expand time? at runtime? is it a macro?"**

A codemod that introduces a call (the concat→format fix; any future fix) MUST be able to query the
target POSITION's purity context AND the introduced callable's purity class, to know the rewrite is
legal. The lint/RETE engine (arc 277/278) is the first hard consumer; it cannot stay correct on a
guessed/implicit purity model.

## What this arc should carry (the ask)

- **Definition-time purity/expand-time-legality as a first-class, queryable metadatum** on every
  intrinsic + user callable in the builtin registry — `(metadata-of :wat::core::X)` answers
  `{:kind macro|fn|intrinsic, :pure-total bool, :expand-time-legal bool, …}`.
- The Rust `is_pure_total` allow-list becomes a *projection* of this registry, not a parallel
  hand-list (extirpare: one source of truth; the drift between the allow-list and reality is the
  failure class).
- Consumers (the RETE engine, codemod fixes) QUERY it rather than re-deriving — see arc 278's
  output-contract + the concat→format fix's macro-context gate (arc 277), both of which need this.

(Companion requirement, captured in arc 277 + 278: the detection rule also needs to know whether the
form under inspection is ITSELF in an expand-time position — "am I inside a defmacro body?" — which is
the context half; this note is the callable-class half. Both are needed to gate a macro-introducing fix.)

---

## UPDATE 2026-06-20 (arc 278 stone 6a) — purity is TWO orthogonal properties: pure AND deterministic

The rete capability tier (stone 6a — the `where`/`:test`/accumulator fence) became the **live consumer**
this note predicted, and it surfaced that the metadatum is not one bit but **two orthogonal ones**:

- **`pure`** — effect-free (no IO/mutation/spawn). Seed: the *negation* of `is_effectful_op`
  (`runtime.rs`) — `:wat::kernel::`/`:wat::io::`/`:wat::eval-`/`:wat::load`/`:wat::config::`.
- **`deterministic`** — referentially transparent, same inputs → same output (no randomness/clock/entropy).

They are genuinely independent. **`:wat::core::Uuid/v4` is the proof: it does no IO and mutates nothing
→ PURE — yet it is random → NON-deterministic.** (`Uuid/v5` = SHA1(ns,name) is pure ∧ deterministic; a
hypothetical `clock/now` would be pure ∧ non-deterministic; `io::read` is impure ∧ non-deterministic.) A
rete *condition* must be a deterministic, effect-free function of the facts → it requires **both**; the
exposed check is `(and (pure? f) (deterministic? f))`. Collapsing them into one "pure" bit (a first 6a
draft did, by jamming `Uuid/v4` into a "non-deterministic" set *inside* the purity check) is the muddle
this update corrects.

So `(metadata-of :wat::core::X)` must carry **`{:kind, :pure, :deterministic, :expand-time-legal, …}`** —
`:deterministic` is the sibling property the original note didn't name.

## UPDATE 2026-07-25 (arc 278, cache Stone 2) — a THIRD hand-list, and this one is a CORRECTNESS WALL

The class this note names has a third instance, found by probe while grounding parametric records. Unlike the
other two it is not a tooling gate — **it is the wire-containment wall itself**, and it currently under-reports.

**The gap.** `is_pure_type` (`src/check.rs:14097`) decides purity for a *type*. For wat-declared types it is
sound — `Nature::is_pure()` for aggregates (Struct → impure), the declared `:wat::enum::{Pure,Impure}` marker
for enums, recursive over containers/newtypes/tuples. But for **Rust opaques** its knowledge is a hardcoded
list of **eight path strings**:

```rust
"wat::kernel::ChildHandle" | "wat::io::IOReader" | "wat::io::IOWriter"
| "wat::holon::OnlineSubspace" | "wat::holon::Reckoner" | "wat::holon::Engram"
| "wat::holon::EngramLibrary" | "wat::holon::Hologram" => return false,
```

Anything not on it falls through to `None => true` — *"portable by convention"*. **Every `#[wat_dispatch]`
opaque minted since that list was written is therefore invisible to the wall**, and a `Record` will hold one.

**Proven by running it (orchestrator, own hand), all exit 0 where they must be exit 3:**
- `(defrecord :probe::Direct [c <- :wat::sqlite::Connection])` — a live thread-owned sqlite handle.
- `(defrecord :probe::Raw [c <- :rust::sqlite::Connection])` — the raw path, so the `Alias => true` arm is not
  what masks it.
- `(defrecord :probe::Smuggle [c <- :wat::cache::Lru<String,i64>])` — **our own Stone 1 primitive** (`a86f521c`).

**The wall itself is sound** — controls confirm it: `IOWriter` in a record field IS rejected
(`ImpureFieldInPureAggregate`), and it sees *through* type parameters (`Box<IOWriter>` rejected, naming the full
parametric type). So this is not a broken wall; it is a wall whose **opaque enrollment is manual and stopped**.

**Why it has teeth.** 293.W built `validate_aggregate_containment` *in response to a grounded breach* — a Struct
nested in a Record crossing a process peer (`#w/S {:a 99}` reconstructed far-side) — to make the wire wall a
TYPE guarantee. That guarantee does not hold for opaques. A record claiming to be EDN can contain a live
resource, and the rule's own error text says why that must never exist: *"a record holding a struct field could
never cross — it must not exist."*

**Both fixes are this note's own prescription, at one wall or three:**
- *Narrow, available now, does NOT need 255:* enroll `#[wat_dispatch]` opaques' purity at registration; delete
  the eight hardcoded names. Same shape as arc 296's `EdnSchema` inventory drain.
- *Root:* 255's registry — opaques declare purity where they register; `is_pure_type` PROJECTS from it. Then all
  three hand-lists dissolve together.

**Unchased (do not inherit as fact):** whether `:rust::sqlite::Connection` is absent from the `TypeEnv` entirely,
or present with a pure-reading nature. That decides "register it" vs "register it correctly."

**Blast-radius warning for whoever draws this:** if opaques start reading impure, every record currently holding
one goes RED at startup. That is the *point* — 293.W's containment pass caught six real stdlib mis-declarations
when it landed and was called a design oracle — but it is a cascade, not a one-liner.

---

**Status / what 278 ships (NOT 255):** 255 (re-lift ~454 builtins into a registry) is **NOT ready** —
builder's call, 2026-06-20, too big to detour into mid-278. To ship rete, stone 6a carries a small
**hand-managed metadata map** in `src/rete/purity.rs` (`{pure, deterministic}` per op, default-deny,
transitive over user-fn bodies) exposing `:wat::rete::pure?` + `:wat::rete::deterministic?`. **This hand
list IS the "parallel hand-list" this note warns against — accepted as the explicit v1 projection.** When
255 lands, the map becomes a *projection* of the registry (delete the hand list; the rete predicates query
`metadata-of`), exactly as prescribed above. The hand list points here in-code for discoverability.

---

## UPDATE 2026-08-02 (arc 278, slice one of the rete `where` vocabulary) — three measured facts, and a correction to this file's own neighbours

Filed by the arc-278 orchestrator while scoping the rete expression language. **Nothing here was
acted on** — 255 stays deferred by the builder's ruling, and the rete fence keeps its hand map.
These are notes for whoever eventually runs the re-lift, recorded because they were paid for.

**★ CORRECTION — `src/intrinsic/mod.rs`'s module header is STALE, and it misleads.** It says
*"`purity` / `determinism` → DERIVED at the reflection site … not stored on the entry."* The struct
disagrees and the struct is the writer: `IntrinsicEntry.purity: wat_doc::Purity` and
`.determinism: wat_doc::Determinism` are **stored** fields, fed by live `@Purity` / `@Determinism`
doc tags (`src/intrinsic/bytes.rs:36-38`), and `pure_declared_matches_is_effectful_op`
(cfg(test), `intrinsic/mod.rs:596`) already asserts the declared value agrees with
`is_effectful_op` for every enrolled entry. Derivation (`derive_pure_deterministic`,
`runtime.rs:24371`) is the fallback for **un-enrolled** verbs only. That header cost a reader a
wrong claim to the builder; it is worth one line to fix. Pinned green by
`tests/reflection/probe_arc255_axes_are_declared_not_derived.rs`.

**1. THE TWO PURITY MODELS HAVE OPPOSITE DEFAULTS — measured, exhibited by run.**

```
derive_pure_deterministic (runtime.rs:24371)  — for UN-enrolled verbs
    pure = !is_effectful_op(name)                                    DEFAULT-ALLOW
    (effectful = :wat::kernel:: :wat::io:: :wat::eval- :wat::load :wat::config::)
intrinsic_meta (src/rete/purity.rs)           — the rete fence's hand map
    147 head strings enumerated; anything absent is refused          DEFAULT-DENY
```

Witness, run 2026-08-02 (`wat-scripts/scratch-pad/probe-slice-one-registry-seam.wat`):
`:wat::core::Bytes::to-hex` — **enrolled** in the registry (it is `#[wat_intrinsic]`'s own doc
example) and **absent** from the hand map. The fence answers `pure? = FALSE`; `metadata-of`
answers `:purity Pure :determinism Deterministic`. Same verb, two oracles, opposite verdicts.
`src/rete/purity.rs` holds **zero** references to the registry, so enrolment buys a verb nothing
at the fence today. Both files already say they are waiting to be subsumed (`purity.rs:17-20`;
`constructor_meta`'s "INTERIM … until arc 255's builtin-registry becomes the single queryable
purity source").

**2. THE DOC CONTRACT CANNOT CARRY A THIRD AXIS.** `wat_doc::parse`'s recognized-tag list is
CLOSED (`crates/wat-doc/src/lib.rs:321-322`, mirrored at `:622` for special forms):
`@added @arg @ret @example @example-norun @deprecated @see @Purity @Determinism @Category
@yields`. A `@Totality` is refused as `UnknownDirective` (verified by run). **Totality is the one
axis a namespace prefix cannot derive** — `:wat::core::i64::+` is pure ∧ deterministic ∧ NOT total
— which is why the rete fence grew its own `total` column in #52 and will keep it until 255
arrives. When the re-lift happens, a third axis is *mirror `Purity` once more*: a `Totality` enum
with `variants()`/`as_str()` (`lib.rs:46-56` is the shape), the tag in both recognized lists, one
field threaded `IntrinsicSubmission` → `IntrinsicEntry`.

**3. `wat_doc::Category` HAS NO ARITHMETIC VARIANT.** The closed set is
`Encoding | Reflection | ControlFlow | Binding` (`lib.rs:112-117`). Enrolling the i64/f64 families
grows **two** closed sets, not one — budget for both.

**4. `#[wat_intrinsic]`'s arity model is two-valued and that is SUFFICIENT — 255's DESIGN
open-question #3 is answered.** `sniff_args` (`crates/wat-macros/src/wat_intrinsic.rs:53-58`)
yields only `Exact(N)` or `Variadic`; there is no keyword-argument shape. It does not need one:
**a kwargs surface is a defmacro that lowers to a positional prime, so the keyword never reaches
the registry.** Proven by run — `(:wat::kernel::readln :max-buffer-bytes 4096)` macroexpands to
`(:wat.kernel/readln' 4096)` (`wat/kernel/readln.wat:59` is the worked instance). So the DESIGN's
*"do some builtins need richer arity, e.g. keyword-arg shapes?"* answers **no**; kwargs are a
macro layer above the registry.

**What arc 278 is doing meanwhile, and why it does NOT add to the drain.** The rete `where`
vocabulary is a hand-managed whitelist in `intrinsic_meta` — builder-ruled 2026-08-02: *"i'm
completely fine having a hand managed whitelist of what funcs and forms are allowed in rete
exprs… 255 will make maintaining such a list easier… but we just need some enforcement mechanism
before 255 arrives."* 278 adds **rows to the existing map**, mints no second table, and enrols
nothing in the registry — so 255's drain stays what `purity.rs:17-20` already prescribes (delete
the map, point the predicates at `metadata-of`), just with more rows to enrol. Two sources for one
verb is the thing that would make the drain harder, and 278 deliberately avoids it.

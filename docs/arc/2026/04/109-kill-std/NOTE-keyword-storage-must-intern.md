# NOTE — a wat keyword is a heap `String`, and the coming symbol flip makes that a decision, not a detail

**Filed 2026-07-31, out of arc 278's binding-map work. Not fixed. Tracked here because it is a
VALUE-MODEL gap, not a rete bug — and because the symbol flip will land on top of it.**

## The finding

```rust
// src/value/value.rs
/// Keyword literal — leading `:` included. Wat-source type `:wat::core::keyword`.
wat__core__keyword(Arc<String>),
```

Every wat keyword is an independent heap `String` behind its own `Arc`. Two occurrences of
`:wat::core::i64` are two allocations. Equality walks the bytes. Hashing walks the bytes, **every
time**, because nothing is cached.

It is Clojure's `Keyword` in name only. It has neither of the two properties that make a Keyword
worth having.

## What Clojure actually does (read off the shipped classes, `javap -p`, clojure 1.12.4)

```
clojure.lang.Keyword
  private static ConcurrentHashMap<Symbol, Reference<Keyword>> table;   the INTERN TABLE
  static final ReferenceQueue rq;                                        weak — interned keywords can be GC'd
  public final Symbol sym;                                               a Keyword WRAPS a Symbol
  final int hasheq;                                                      hash cached EAGERLY, final
  public static Keyword intern(...)  x3                                  interning is the ONLY constructor

clojure.lang.Symbol
  final String ns; final String name;    two strings, namespace-qualified
  private int _hasheq;                   hash cached LAZILY (not final)
  final IPersistentMap _meta;            symbols carry METADATA
  transient String _str;                 cached toString
  (no intern table)
```

**The split is principled, and the reason is metadata.** A keyword is an *identifier used as data* —
it carries nothing else, so interning is safe and pays for itself: reference equality, and a hash
computed once. A symbol is *code* — it carries metadata, so two `'x` with different metadata must be
distinct objects, which makes interning impossible. Clojure did not pick one representation and
apply it twice; it derived each from what the thing **is**.

## Why this is load-bearing for the symbol flip

`:wat::core::i64` **is a symbol** — it names a type. It currently rides the keyword space because
that is the only interned-ish identifier space wat has. When real symbols land and the corpus flips
off colon-quoted symbols, wat will have two identifier kinds for the first time, and the storage
question has to be answered for both **at the same time**:

- give symbols the keyword treatment (intern them) and metadata becomes unrepresentable;
- give keywords the symbol treatment (don't intern) and every map key stays a byte-compare.

Retrofitting either afterwards means touching every identifier in the value model. The cheap moment
to decide is *before* the flip, which is now.

## What correct storage looks like

Following the derivation rather than the shape:

- **Keyword** → interned. `Arc<KeywordData { name, hash: u64 }>` behind a global table;
  `PartialEq` takes an `Arc::ptr_eq` fast path (sound **only** because interning guarantees
  uniqueness — the fast path and the intern table are one decision, not two); `Hash` returns the
  stored `hash`. Weak references if we want them collectable, as Clojure does.
- **Symbol** → not interned. Namespace + name, lazily-cached hash, and room for metadata, because
  metadata is exactly what forbids interning.

Open, and genuinely undecided: whether wat symbols carry metadata at all. If they never will, the
argument against interning them evaporates and they could share the keyword machinery. **That is the
question this note exists to force** — answer it from what a wat symbol *is*, not from what is
convenient at flip time.

## The honest perf bound — do NOT sell this as a speed stone

Measured this session (`src/rete/kernel.rs`, `binding_key_cost`, release, 50k iters), holding the map
identical and changing only the key type — `Value::String` vs `Value::i64` as an interned-id floor:

```
n     build (str / i64)          lookup (str / i64)
1     341.9 / 197.9   1.7x       35.4 / 32.1   1.1x
3     487.3 / 404.9   1.2x       16.1 / 14.7   1.1x
8    1399.2 / 1163.0  1.2x       14.5 / 13.8   1.0x
```

**Lookup barely moves.** The cost of a persistent-map operation is the trie, not the key. An earlier
measurement (`9448f012`) reached the same verdict from the other direction — *"interning the bind key
saves 8%; the MAP is 85% of it"* — and this session's attempt to overturn it failed: the hypothesis
that "the map's 85% is really string hashing in disguise" is **refuted** by the table above.

So interning is worth doing for **correctness and consistency** — identifier equality that means
identity, a hash that is computed once, parity with the language we are a dialect of — and it will
shave build time wherever keywords are constructed in bulk. It is **not** the lever on rete fire time;
that is the binding map's *representation* (arc 278: array vs HAMT, 3–4× on build, 4–10× on drop).
Whoever picks this up should not inherit a perf claim it cannot pay.

## Cross-references

- `src/value/value.rs` — the `wat__core__keyword(Arc<String>)` variant.
- `src/rete/kernel.rs` — `binding_key_cost` and `binding_repr_microbench`, the measurements above.
- Arc 278 `DESIGN-STONE-native-element.md` — the sibling finding: the same "we lack a representation
  Clara has" shape, one layer down, in the map rather than the key.
- Clara's own choice, for the record: `engine.cljc:23` — *"bindings, a map of **keyword**-to-values"*;
  `compiler.clj:293` assoc's `(keyword variable)`. Clara's binding keys are interned keywords, and
  that is why its lookups compare pointers where ours compare bytes.

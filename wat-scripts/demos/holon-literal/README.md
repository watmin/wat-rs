# `#holon` — the clj↔wat seam (arc 294.b)

The same five-character reader tag, read by two languages, both correct. This is the
showpiece for the [byte-identical bridge](../../../docs/arc/2026/06/294-holon-returns-to-vsa/NOTE-holon-literal-tag.md):
a Clojure app and a wat service can exchange the *same bytes* with zero translation.

## The shared bytes

[`literal.edn`](literal.edn) holds exactly:

```edn
#holon {:kw ["a" "b"] true #{1 :foo "bar"} 3.0 nil}
```

A heterogeneous map — disparate **key** types (keyword, bool, float) AND **value** types
(vector, set, nil). wat-core collections are monomorphic, so a *bare* `{…}` of this shape
is rejected by literal inference. `#holon` declares "this IS holon/EDN data" — you say
what it **is**, not what it **holds**.

## Read it from both worlds

**Clojure** — `#holon` is `identity` (the part-face: it's already data, unchanged):

```bash
clojure -M -e "(binding [*data-readers* {'holon identity}] \
  (println (pr-str (read-string (slurp \"wat-scripts/demos/holon-literal/literal.edn\")))))"
# => {:kw ["a" "b"], true #{"bar" 1 :foo}, 3.0 nil}
```

(The one-line registration lives in [`data_readers.clj`](data_readers.clj) — `{holon identity}`.)

**wat** — the same literal (embedded in [`cosine.wat`](cosine.wat)) reads as one hologram
(the whole-face: the heterogeneous structure as a single hyperdimensional point), and measures:

```bash
cargo wat wat-scripts/demos/holon-literal/cosine.wat
# => 0.9999999999999999   (cosine of the literal with itself → exact coincidence)
```

## Why it is honest, not a symmetry

The two readers run **different operations** — `identity` in Clojure, `quote`-capture in wat —
yet the same bytes yield the same datum. What is preserved is the *data*, not the *operation*;
the asymmetry measures the type-gravity gap (clj data is free → identity; wat data must escape
the type checker → quote). It is the holon's frame-relative Janus face made operational. See
[`REALIZATIONS.md`](../../../docs/arc/2026/06/294-holon-returns-to-vsa/REALIZATIONS.md) —
*"the asymmetry that is honest"* / **EADEM RES, ALIA VIA**.

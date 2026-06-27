# NOTE — the `#holon` literal tag: relaxed-mode EDN, and the clj↔wat seam

**Status: DECIDED (builder administrative decision, 2026-06-27) — four-questions-selected.** Candidates: A (a
reader/literal tag), B (ascription/bidirectional), C (wrapping verb), D (EDN-by-default inversion). Each was
four-questioned; **only A cleared Obvious + Simple + Honest** (B fails *non-locality* + needs check-against-for-
literals machinery that does not exist; C is a special-form masquerading as a verb and overlaps `to-holon`; D fails
Simple on corpus-wide blast radius). The tag is **`#holon`**.

## The problem it solves
wat-core collections are **monomorphic** — `infer_map_literal` / `infer_hashmap_constructor` (`check.rs:13615 /
13375`, arc 257) unify all keys → one K, all values → one V. A heterogeneous literal
`{:kw ["some" "vec"] true #{1 :foo "bar" false} 17.0 {"str" :kw :kw "str"}}` (disparate key AND value types) is
rejected — **135 type-errors at HEAD** (probe, this session). Yet EDN/Clojure data is inherently heterogeneous.
This is the wall between wat and *"holon hosts all of EDN."*

## The decision
`#holon {…}` tags the enclosed literal as **relaxed-mode EDN**: typed as a heterogeneous **`Hologram`** (the
heterogeneous value type the system already blesses — there is deliberately **no `:Any`**; `types/error.rs:195`
directs "any algebra value" → `HolonAST`/Hologram), NOT a monomorphic core collection. Contents are not
cross-constrained. You **declare what it IS** (a holon/EDN value), **not what it holds** (K, V).

## Why it is sound (proven LIVE, this session)
The hologram already hosts full heterogeneity — routing the same map through the EDN-string path
(`:wat::holon::eval-edn-coincident?`, which bypasses literal inference) returned `#wat-edn.result/ok true`, and
cosines over plain EDN are structurally honest (identical → 1.0; one-of-two binds → 0.486; two-of-three slots →
0.574; unrelated → 0.011). **The encoding is done; the wall is purely literal inference.** `#holon` is the trigger
that routes a literal to the existing `to-holon` codec instead of `infer_map_literal`.

## Why the NAME is `#holon` (intueri — semantic grounding)
Koestler (1967, *The Ghost in the Machine*) coined **holon** = Greek *holos* (whole) + *-on* (the part-particle, as
in prot-*on*): a **whole-and-part**, **Janus-faced** — a complete whole to its constituents AND a dependent part of
a larger whole; frame-relative identity. The tag IS that Janus face made literal:
- to **wat** → a **whole** (one hologram; the heterogeneous structure as a single hyperdimensional point);
- to **Clojure** → **identity** (a one-line data-reader `{holon identity}` erases the tag → plain data, an ordinary
  *part* of clj's data world).

**Identity-in-Clojure is the part-face of the holon** — viewed "up" as a part of the larger world, a holon is
simply *itself, unchanged* = identity. So `#holon` names BOTH faces; `#edn` would name only one (the part: "it's
data"). Maximally faithful. (Lineage: a physical hologram is part-contains-whole; Plate's HRR and Kanerva's HDC are
called *holographic* for the same reason — the whole distributed across every dimension.)

## The clj↔wat seam (the IPC story this unlocks)
Same five characters, two readers, both correct → **byte-identical data literals across the two languages.** A
Clojure app ships `#holon {…}` (or plain EDN) over the wire; wat receives, encodes to a hologram, measures, returns
vector answers; Clojure reads `#holon` as identity → the data it always was. **Zero translation; the wire is shared
source.** The bridge (builder's north-star): **clj apps "just upgrade" when they interface with wat** (one-line
data-reader) to reach high-perf VSA; wat-native devs build on wat's strong guarantees. Rides 294's thesis — EDN
canonical on the wire, and now on the literal too.

## Strike-time opens (deferred to the build)
- **intueri final**: `#holon` (unqualified) vs a qualified `#wat/holon` — Clojure mildly discourages unqualified
  data-reader tags (collision risk); weigh against the byte-identity goal at the strike.
- **clj-side**: the one-line identity data-reader registration (`data_readers.clj` / `*data-readers*`) that makes
  the identity real.
- **no bidirectional literal-check today** (grep empty) → `#holon` is the explicit trigger; a context/ascription-
  driven relaxation (candidate B) is a possible *later additive convenience*, never a second canonical path.

## Pairs
`294/DESIGN.md` (EDN canonical; the `HolonAST → Hologram` keystone) · `294/REALIZATIONS.md` R2 (the homecoming) ·
`check.rs:13615` (the monomorphic literal wall) · `types/error.rs:195` (no `:Any`; Hologram is the heterogeneous
holder) · `feedback_uniform_operation_or_decomplect_is_catastrophic`.

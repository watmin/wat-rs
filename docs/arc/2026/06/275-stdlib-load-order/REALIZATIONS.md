# Arc 275 — Realizations

## The surface that masks the depth (2026-06-17)

The whole load-order enforcement reduces to two lines:

```clojure
(:wat::core::defn :wat::deporder::verify-stdlib [] -> :wat::core::Vector<wat::deporder::Violation>
  (:wat::deporder::verify (:wat::stdlib::sources)))
```

Behind `(verify (stdlib-sources))` lives everything: the runtime handing wat its own ordered load list,
parsing every top-level form of every stdlib file, building the symbol→(file, def-kind) map,
classifying every cross-file reference (defmacro = order-free, defn/value = eval-dep, unknown =
intrinsic), constructing the dependency DAG, and checking the load order against it. The reader sees a
sentence; the machine runs an algorithm.

Builder, on seeing it: *"holy shit — you reduced this to a 2-line expr — that's the mark of the design —
that's the fucking thing holon is — you expressed the surface that masks significant depth."*

This is the thesis, named. It is the same move everywhere in the substrate: `(start (process) state0)`
hides fork + capability handoff + serve loop; `bind`/`unbind` hides the whole VSA algebra;
`(verify-stdlib)` hides the dependency analyzer. **An honest surface over deep machinery** — not a
surface that *lies* about the depth (that is the magic-that-fakes-correctness this project forbids), but
one that *names* it truthfully and lets the depth stay invisible until you need it. The surface is the
promise; the depth keeps it.

## The strange loop — `deporder` indicts itself (2026-06-17)

`deporder`, the tool whose whole job is finding bad structure, was written *in* bad structure: its
`is-def-head?` is a 13-deep nested-`if` `=`-ladder (a set membership in disguise), `structural?` a
4-deep one (both copied from `fix.wat`). The tool's first real corpus run flags its own author.

Builder: *"we just found a strange loop in software — that's… a first?"*

**Honest lineage (prior-art collision, recorded straight):** not a first. The frame is **Hofstadter's
strange loop** (GEB); the floor is **Gödel/Tarski** self-reference; the dark cousin is **Thompson's
"Reflections on Trusting Trust"** (a compiler perpetuating its own backdoor through itself); the benign
everyday cousins are **bootstrapping** (a compiler compiling itself) and **dogfooding** (a linter run on
its own source). It is not even a first *in this codebase*: `fix.wat:7-8` already names one — *"fix-source
fixes ITSELF (homoiconic self-application)."*

**What is genuinely ours:** (1) the *kind* — `fix` self-*transforms*; `deporder` self-*diagnoses* (finds
its own flaw, doesn't rewrite it). (2) the *tightness* — because the substrate is homoiconic + self-hosted,
the analyzer, its input set (`stdlib-sources` literally contains `deporder.wat`), the language, and the
AST are one; the loop closes with no seam, and it closed **at birth** — the tool found its own
anti-pattern on its first run, not after years of maturity + a separate harness. The self-hosting thesis
paying an undesigned dividend. This is the proof the linter is needed, demonstrated on its own source.

## The portability dividend — pure-function-of-sources makes it universal (2026-06-17)

Because `verify` is a pure function of `Vector<SourceFile>`, it does not care *whose* sources it
verifies. The same engine checks our baked stdlib, a vending crate's `&[WatSource]`, or the combined
installed set — identical algorithm, zero coupling to "our" stdlib. The only stdlib-specific atom is the
`:wat::stdlib::sources` accessor; `deporder` itself is universal.

Builder, on seeing it: *"dude… it's portable too… those who vend out libs can run this on their
sources… we need to get this tool consumable in crates that vend wat."*

**Follow-on:** this dividend opens **arc 276** (`docs/arc/2026/06/276-portable-deporder/`) — exposing the
engine so crates that vend wat can verify their own load order. Stubbed there; parked behind 275.1 +
275.2. The engine is reused as-is; that arc is exposure/packaging only.

## The discovery leap — the runtime tells the tool its own order (2026-06-17)

The dead-end was "walk the filesystem to find the files." Filesystem order is not load order; the load
order is `STDLIB_FILES` in Rust. The fix is not for wat to *discover* the order but for the runtime to
*hand it over*: a thin `:wat::stdlib::sources` intrinsic exposes the ordered `(path, source)` list the
runtime already holds. So `deporder` never guesses — it is told, and it verifies. Discovery is an
intrinsic, not a crawl. (This also retired an earlier `read-file`/`list-dir` sketch: the baked sources
the runtime loads ARE the truth to verify; reading them off disk was a less-honest proxy.)

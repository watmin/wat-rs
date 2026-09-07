;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(Filter)
;;
;; wat/holon.wat — loose verbs in the :wat::holon::* namespace.
;;
;; The wat/holon/ subdirectory ships one named PascalCase holon per
;; file (wat/holon/Subtract.wat → :wat::holon::Subtract, etc.). This
;; top-level file is the home for :wat::holon::* verbs that DON'T
;; constitute their own named holon — closures, factories, and
;; convenience functions that operate on existing holon primitives.
;;
;; Parallels the other
;; namespace-level top-level files per § G's filesystem-path-mirrors-
;; FQDN doctrine. Future additions: any :wat::holon::* verb whose
;; name doesn't match a substrate-defined holon type lands here.
;;
;; Currently shipped: three filter factories for Hologram/get. These
;; are verbs, not a Filter type — hence they live here rather than in
;; a dedicated Filter.wat file whose basename would misrepresent what
;; it houses.
;;
;; ─── Hologram/get filter factories ──────────────────────────────────
;;
;; `Hologram/get` takes a 2-arg call `(:wat::holon::Hologram/get store probe)`.
;; The filter `[:wat::core::f64 :-> :wat::core::bool]` is bound
;; at construction via `Hologram/make` and decides whether the
;; highest-cosine candidate is "close enough" to return. The substrate
;; ships three opinionated factories so consumers don't have to
;; hand-roll the canonical thresholds.
;;
;; Usage:
;;
;;   ;; build the store once — filter is bound at construction
;;   (def store (:wat::holon::Hologram/make (:wat::holon::filter-coincident)))
;;
;;   ;; strict — only return when cosine clears the coincident floor
;;   (:wat::holon::Hologram/get store probe)
;;
;;   ;; looser store — build with filter-present instead
;;   (def store (:wat::holon::Hologram/make (:wat::holon::filter-present)))
;;   (:wat::holon::Hologram/get store probe)
;;
;;   ;; pure population readout — no gating; whatever scored highest wins
;;   (def store (:wat::holon::Hologram/make (:wat::holon::filter-accept-any)))
;;   (:wat::holon::Hologram/get store probe)
;;
;; Why factories rather than plain functions: the floor depends on `d`
;; (the encoding dimension), and `d` is a per-store constant. Baking
;; `d` into the closure at construction time is honest — the filter
;; carries the same threshold the store was built against.
;;
;; Why these aren't substrate primitives in Rust: they're three
;; closures over the f64 floor accessors that already are primitives.
;; Wat can express them; substrate doesn't earn its keep here.

;; ─── filter-coincident — strict, "same point on the algebra grid" ─
;;
;; Returns true iff `(1 - cos) < coincident-floor(d)`. Matches the
;; semantics of `:wat::holon::coincident?` but works on a raw cosine
;; value instead of two HolonAST inputs.
;;
;; d is read from the ambient `:wat::config::dim-count` rather than
;; passed by the caller. The filter captures the floor at the call
;; site's ambient d; pass through `Hologram/make` once and the entire
;; store carries the same threshold.
(:wat::core::defn :wat::holon::filter-coincident [] -> [:wat::core::f64 :-> :wat::core::bool]
  (:wat::core::let
      [floor
        (:wat::holon::coincident-floor (:wat::config::dim-count))]
      (:wat::core::fn [cos <- :wat::core::f64] -> :wat::core::bool
        (:wat::core::< (:wat::core::- 1.0 cos) floor))))

;; ─── filter-present — looser, "signal detected above noise" ───────
;;
;; Returns true iff `cos > presence-floor(d)`. Matches the semantics
;; of `:wat::holon::presence?` but works on a raw cosine value.
;; Use when the cache is acting as a "best-known reasonable answer"
;; lookup rather than "did I see this exact form before."
;;
;; d is read from the ambient `:wat::config::dim-count`.
(:wat::core::defn :wat::holon::filter-present [] -> [:wat::core::f64 :-> :wat::core::bool]
  (:wat::core::let
      [floor
        (:wat::holon::presence-floor (:wat::config::dim-count))]
      (:wat::core::fn [cos <- :wat::core::f64] -> :wat::core::bool
        (:wat::core::> cos floor))))

;; ─── filter-accept-any — null gate, returns whatever scored best ──
;;
;; Returns true unconditionally. Useful when the consumer wants the
;; population's nearest neighbor without any floor — e.g., taking the
;; cell's argmax for a soft scoring loop where the consumer applies
;; their own gate downstream.
(:wat::core::defn :wat::holon::filter-accept-any [] -> [:wat::core::f64 :-> :wat::core::bool] (:wat::core::fn [_ <- :wat::core::f64] -> :wat::core::bool true))

;; ─── Arc 296: :wat::holon::CapacityExceeded — moving the source of truth to wat ───
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration; the Rust side is meant to
;; become generated FROM this form rather than hand-maintained alongside it.
;;
;; Populated in the Err slot of `:wat::holon::Bundle`'s :Result return when a
;; frame's constituent count exceeds `floor(sqrt(dims))` (Kanerva's capacity
;; budget). `cost` is what the Bundle was asked to hold; `budget` is what the
;; substrate could hold. Both i64 because wat integer literals are i64.
(:wat::core::defstruct :wat::holon::CapacityExceeded
  [cost   <- :wat::core::i64
   budget <- :wat::core::i64])

;; ─── Arc 296: :wat::holon::CoincidentExplanation — moving the source of truth to wat ───
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration.
;;
;; Diagnostic record returned by `:wat::holon::coincident-explain`. Bundles the
;; raw cosine, the current coincident floor, the dim where comparison
;; happened, the sigma feeding the floor, the same boolean `coincident?` would
;; have returned, and the smallest sigma at which the pair would coincide.
(:wat::core::defstruct :wat::holon::CoincidentExplanation
  [cosine             <- :wat::core::f64
   floor              <- :wat::core::f64
   dim                <- :wat::core::i64
   sigma              <- :wat::core::i64
   coincident         <- :wat::core::bool
   min-sigma-to-pass  <- :wat::core::i64])

;; ─── Arc 296: :wat::holon::Match — moving the source of truth to wat ───────
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration.
;;
;; The result of `:wat::holon::Hologram/find`. A Hologram matches by
;; SIMILARITY, so the key `find` hands back is not necessarily the probe that
;; was passed in — it is whatever stored key coincided above the filter's
;; floor. `Match` carries that asymmetry in its name; `get` answers "what
;; value did my probe reach?" and discards the matched key, while `find`
;; exists so a caller can name the key that actually matched and act on it.
(:wat::core::defrecord :wat::holon::Match
  [key   <- :wat::holon::HolonAST
   value <- :wat::holon::HolonAST])


;; :wat::holon::VectorDecodeOutcome — Arc 278 the dimension-heresy strike
;; (BRIEF-dimension-heresy-screams.md). `:wat::holon::bytes-vector` used to
;; return a bare `(:Option :- [wat::holon::Vector])`, collapsing FOUR structurally
;; distinct wire-decode failures (short header, wrong data length, foreign
;; encoding dimension, reserved 0b11 cell pattern) into one reason-free
;; `:None`. Per the builder's ruling on this strike — "the entire check is
;; 'are these two dims the same vec length?' — that's it. This is trivially
;; measured and is not deserving of a crash but an expressive enum to be
;; handled" — each failure becomes its own named variant, not a lumped
;; `Malformed[reason, at]`: the failure space is a CLOSED set already
;; explicitly branched in the decoder's own source (unlike
;; `RequestMalformed`'s open-ended String, which is honest precisely
;; because ITS space is open-ended). The tell: `at` is meaningful only for
;; `InvalidCell` — a shared field honest for one member and vacuous for the
;; rest is the evidence a lumped shape would be wrong here.
;;   :Decoded           [vector <- Vector]     — the happy path.
;;   :DimensionMismatch [expected <- i64  got <- i64] — the wire header's
;;                        dim disagrees with this program's constant
;;                        `dim-count` (`config::collect_entry_file`).
;;                        Neither vector is "foreign" in the combine sense
;;                        below — this one DOES cross a wire, so the
;;                        disagreement is against the ambient program's d.
;;   :TruncatedHeader   [got <- i64]            — fewer than 4 header bytes;
;;                        no `expected` field — the 4-byte minimum is a
;;                        protocol constant, not a per-call datum, and the
;;                        actual (short) length is the one thing a log wants.
;;   :LengthMismatch    [expected <- i64  got <- i64] — header dim parsed
;;                        fine, but the data bytes don't match `ceil(dim/4)`.
;;   :InvalidCell       [at <- i64]             — a 2-bit cell decoded to
;;                        the reserved `0b11` pattern at cell index `at`.
;; PURE — `:wat::holon::Vector` is fully EDN-reconstructable ternary cell
;; data (the very reason `vector-bytes`/`bytes-vector` exist to serialize
;; it), and every other field is a bare `i64`. Registered as a builtin
;; (peer with the other outcome walls) for load-order robustness, though
;; `bytes-vector` itself has zero wat-corpus callers today.
(:wat::core::defenum :wat::holon::VectorDecodeOutcome :wat::enum::Pure
  :Decoded [vector <- :wat::holon::Vector]
  :DimensionMismatch [expected <- :wat::core::i64  got <- :wat::core::i64]
  :TruncatedHeader [got <- :wat::core::i64]
  :LengthMismatch [expected <- :wat::core::i64  got <- :wat::core::i64]
  :InvalidCell [at <- :wat::core::i64])

;; :wat::holon::CombineOutcome — Arc 278 the dimension-heresy strike, part
;; 2. `vector-bind` / `vector-bundle` / `vector-blend` each RAISED a
;; `TypeMismatch` on differing Vector dimensions; per the same ruling as
;; `VectorDecodeOutcome` above, a differing `d` is cheap to detect and
;; meaningful to recover from, so it becomes a matchable value instead.
;; ONE shared enum for all three verbs, not three per-verb siblings —
;; unlike `RecvOutcome`/`SendOutcome`/`TrySendOutcome` (whose split is
;; earned because their outcome SHAPES genuinely differ), bind/bundle/blend
;; have an IDENTICAL outcome space: both reduce to `[expected, got]`.
;;   :Combined          [vector <- Vector]                — the happy path
;;                        (bind's XOR-compose / bundle's superposition /
;;                        blend's weighted linear combination — three
;;                        verbs, one shape of success).
;;   :DimensionMismatch [expected <- i64  got <- i64]      — the operands
;;                        disagree. Deliberately the SAME variant name as
;;                        `VectorDecodeOutcome::DimensionMismatch` — one
;;                        fact reached by two routes. NOT `ForeignDimension`:
;;                        here neither vector is foreign (both are ordinary
;;                        in-program values that simply disagree), unlike
;;                        the wire-decode case above where one honestly did
;;                        cross a boundary.
;; PURE, for the same reason `VectorDecodeOutcome` is: a bare `Vector` +
;; two `i64`s, all EDN-reconstructable.
(:wat::core::defenum :wat::holon::CombineOutcome :wat::enum::Pure
  :Combined [vector <- :wat::holon::Vector]
  :DimensionMismatch [expected <- :wat::core::i64  got <- :wat::core::i64])

;; :wat::holon::DegenerateSide — Arc 278 the cosine outcome wall
;; (BRIEF-cosine-outcome-wall.md, DESIGN-STONE-where-admits-only-rete-ops.md
;; "THE MEASUREMENT IS FULL; THE PREDICATE IS EXACT" + its AMENDED
;; 2026-08-03 block). Diagnostic payload for `CosineOutcome::Degenerate`
;; below — WHICH operand had a zero-magnitude vector (the case cosine
;; cannot honestly answer, since a direction is undefined for a
;; zero-magnitude vector). Three-valued rather than two bools deliberately
;; (orchestrator's amendment to the ward's original cast): a pair of bools
;; makes `(false, false)` — a `Degenerate` that is not degenerate —
;; representable, in a substrate whose standing doctrine is the wrong
;; state has no form. `Target`/`Reference` are the implementation's own
;; operand names (mirroring `pair_values_to_vectors`'s `target`/`reference`
;; callers use), not invented ones.
;; PURE — three nullary variants, no fields at all.
(:wat::core::defenum :wat::holon::DegenerateSide :wat::enum::Pure
  :Target
  :Reference
  :Both)

;; :wat::holon::CosineOutcome — Arc 278 the cosine outcome wall. `cosine`
;; had two domain holes, both dishonest: a dimension mismatch raised
;; `TypeMismatch` (uncatchable, unwinds past the reader), and a
;; zero-magnitude operand returned a guarded `0.0` — which in cosine's
;; own codomain MEANS "orthogonal, unrelated", a fabricated answer that
;; sails through `(f64::> ... 0.9)` as a confident no-match (probe
;; `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat`: genuine
;; unrelatedness reads `-0.0086`, the sentinel reads exactly `0.0` — the
;; two are indistinguishable to a caller without this wall). Per the
;; design stone's ruled law (a MEASUREMENT may not absorb its own
;; undefined case), both holes become named variants a caller faces:
;;   :Similarity        [similarity <- f64]        — the happy path, the
;;                        raw cosine, clamped to [-1, 1].
;;   :Degenerate        [side <- DegenerateSide]    — one operand (or
;;                        both) is a zero-magnitude vector, so a
;;                        direction — and therefore a cosine — is
;;                        undefined. ONE variant carrying which side,
;;                        not three variants proliferated: the caller
;;                        acts identically regardless of which side was
;;                        degenerate, and the side is a diagnostic, not a
;;                        behavioral fork — exactly the role
;;                        `DimensionMismatch`'s fields already play below.
;;   :DimensionMismatch [expected <- i64  got <- i64] — the two operands
;;                        disagree in dimension; was the `pair_values_to_vectors`
;;                        `TypeMismatch` raise, now a domain fact.
;; PURE — non-parametric, holding only pure data: an f64, a `DegenerateSide`
;; (itself pure), and two i64s. Fully EDN-reconstructable / wire-crossable;
;; marking it Impure would lie. Registered as a builtin, peer with the
;; other outcome walls in this family (`CombineOutcome`, `VectorDecodeOutcome`).
(:wat::core::defenum :wat::holon::CosineOutcome :wat::enum::Pure
  :Similarity [similarity <- :wat::core::f64]
  :Degenerate [side <- :wat::holon::DegenerateSide]
  :DimensionMismatch [expected <- :wat::core::i64  got <- :wat::core::i64])

;; :wat::holon::DotOutcome — Arc 278 the cosine outcome wall's sibling for
;; `dot`. TWO enums, not one shared with `CosineOutcome` — `dot` performs
;; no division (`Similarity::dot` sums `i8 × i8` products, bounded by
;; `d × 127²`, so reaching ±Inf needs `d ≈ 10³⁰⁴` — closed, not merely
;; unlikely), so a zero-magnitude operand yields an HONEST `0.0`: a zero
;; vector really does dot to zero. A shared enum would hand `dot` a
;; `Degenerate` arm it can never construct — the `TrySendOutcome`-from-
;; `SendOutcome` precedent: split earned by a genuine, structural
;; difference in outcome space, not a naming convenience.
;;   :Computed          [product <- f64]           — the happy path.
;;   :DimensionMismatch [expected <- i64  got <- i64] — same fact,
;;                        same shape as `CosineOutcome::DimensionMismatch`
;;                        (one fact reached by two routes through the
;;                        shared `pair_values_to_vectors` guard).
;; PURE, for the same reason `CosineOutcome` is.
(:wat::core::defenum :wat::holon::DotOutcome :wat::enum::Pure
  :Computed [product <- :wat::core::f64]
  :DimensionMismatch [expected <- :wat::core::i64  got <- :wat::core::i64])

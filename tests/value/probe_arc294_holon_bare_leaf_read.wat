;; Arc 294.j — the shim forgets the algebra (DESIGN-STONE-294.j).
;;
;; Co-located fixture for probe_arc294_holon_bare_leaf_read.rs. Four rows, the post-strike
;; spec from BRIEF-294.j:
;;   1. a bare leaf round-trips (top level)
;;   2. a #holon-derived structure renders as plain EDN — no dead-tag substring anywhere
;;   3. the OLD tag is REFUSED on decode (negative control)
;;   4. Thermometer renders to its call form — non-vacuity for row 3
;;
;; Slurped via startup_beside(file!()) / call_beside_value(file!(), …) — no inline wat driver.

;; ─── Row 1 — a bare leaf round-trips (top level) ─────────────────────────────

;; The wire text itself: post-strike, a leaf carries no tag — it IS the EDN scalar.
(:wat::core::defn :t::leaf-wire [] -> :wat::core::String
  (:wat::edn::write (:wat::holon::leaf 42)))

;; The round-trip: encode then decode back against the declared type. `:wat::edn::validate`
;; does exactly this (encode the given value, decode the result against the target type,
;; report Valid/Invalid) — the one primitive that exercises the TYPED coercion arm
;; (edn_shim.rs's `:wat::holon::HolonAST` case) without hand-rolling the encode/decode pair.
(:wat::core::defn :t::leaf-roundtrips [] -> :wat::core::String
  (:wat::edn::write (:wat::edn::validate (:wat::holon::leaf 42) :wat::holon::HolonAST)))

;; ─── Row 2 — a to-holon-derived structure renders as plain EDN ──────────────

;; The builder's own example (DESIGN-STONE-294.j): "#holon {:a "b"} represents two atoms,
;; bound together." The wire form must be the plain data — no tag family anywhere in it.
;;
;; RELAND: built via `:wat::holon::to-holon` (the VALUE-level lift, `to_holon_inner`'s
;; HashMap arm — classifier key `Bind(Atom(String("Map")), ...)`), NOT the `#holon` reader
;; macro. `#holon {:a "b"}` goes through a DIFFERENT, FORM-level lowering
;; (`watast_to_holon`'s `WatAST::Map` arm) that — independently of this stone, pre-existing —
;; emits an un-Atom-wrapped classifier key (`Bind(String("Map"), ...)`, contradicting its own
;; doc comment's stated shape), so `from_holon_item`'s classifier match misses it and RAISES.
;; Not this stone's to fix (`watast_to_holon` is out of scope — "leave them alone otherwise",
;; 8 unrelated `runtime.rs` callers); reported as a finding. `to-holon` sidesteps it entirely
;; and is the more central "make this data a holon" entry point for this assertion's intent.
(:wat::core::defn :t::holon-structure-wire [] -> :wat::core::String
  (:wat::edn::write (:wat::holon::to-holon {:a "b"})))

;; ─── Row 3 — the OLD tag is REFUSED on decode (negative control) ────────────

;; `:wat::edn::read` is the UNTYPED reader — the one that used to dispatch on the tag
;; NAMESPACE directly (edn_shim.rs's decode-dispatch block, now deleted). Feeding it the
;; old dead-tag text must RAISE — this function never returns; the Rust side asserts the
;; CALL itself errors (direction only, never message text). The text is a PARAMETER, not
;; inlined here — the Rust side assembles it by concatenation so the dead spelling doesn't
;; appear as a literal contiguous substring anywhere in the tree (gate 1's own grep would
;; otherwise catch the negative control's own fixture, which is not what gate 1 means).
(:wat::core::defn :t::refuse-old-tag [edn-text <- :wat::core::String] -> :wat::core::String
  (:wat::edn::write (:wat::edn::read edn-text)))

;; ─── Row 4 — Thermometer renders to its DIRECTIVE TAG; non-vacuity for row 3 ─

;; RELAND (DESIGN-STONE-294.j ⛔ CORRECTION): the directive survives as a TAG, never
;; a call form — `(:wat::holon::Thermometer 50.0 0.0 100.0) -> "(:wat.holon/Thermometer
;; 50.0 0.0 100.0)"` was the wat-SOURCE-FORM-on-the-wire defect this stone fixes (it
;; round-trips to a Bundle and crashes the far side of service-cache-hologram.wat:121).
;; The values (50.0, 0.0, 100.0) are the builder's own named example.
(:wat::core::defn :t::thermometer-wire [] -> :wat::core::String
  (:wat::edn::write (:wat::holon::Thermometer 50.0 0.0 100.0)))

;; Proves row 3's refusal is about the DEAD TAG specifically, not a universally broken
;; decoder: a LEGITIMATE post-strike wire value still validates successfully. (Validate
;; only proves Valid/Invalid, not WHICH HolonAST variant came back — the Rust-side test
;; does the stronger structural check: decodes and asserts a real Thermometer, not a Bundle.)
(:wat::core::defn :t::thermometer-roundtrips [] -> :wat::core::String
  (:wat::edn::write (:wat::edn::validate (:wat::holon::Thermometer 50.0 0.0 100.0) :wat::holon::HolonAST)))

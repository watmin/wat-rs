;; Arc 278 #57 — `:wat::rete::map::contains-key?` (arc 255 Stone E-i: renamed together with its core_name pair), the LAST UNSURE-bucket
;; straggler, minted after an AUDIT (the evening seam's own instruction: "audit, do not guess").
;;
;; This file is the loadable, type-checked reference. It proves three things, and each line
;; exists because a row that merely EXISTS looks fine and does nothing — the `cond` rider's
;; lesson: a row alone is not evidence it fires.
;;
;;   1. RESOLUTION      — the rete spelling exists and dispatches.
;;   2. PARAMETRICITY   — `K` and `V` are INDEPENDENT type vars. A single-var container shape
;;                        (`PersistentMapOf("K","K")`, the obvious typo) fails the keyword→i64
;;                        map below, where K and V genuinely differ.
;;   3. NON-VACUITY     — a HIT and a MISS on the same map. A row hard-wired to `true` (or to
;;                        `false`) passes half of this and fails the other.
;;
;; ── WHY IT IS `total: true` — the audit, recorded so it is not re-derived ──────────────────
;;
;; `persistentmap_contains_key_q_inner` (`collection/eval.rs:959`) has exactly two exits:
;;
;;   a. an UNHASHABLE key -> `Ok(Value::bool(false))`. NOT a sentinel: the question asked is
;;      "is this key in the map?", and a value that cannot be a key is not in it. That is the
;;      PREDICATE ruling of DESIGN-STONE-where-admits-only-rete-ops, the same shape as
;;      `coincident?` answering `false` on a degenerate operand.
;;   b. a WRONG RECEIVER -> `TypeMismatch` raise. Must-never-happen: the row DECLARES the
;;      receiver `(PersistentMap :- [K V])`, so the checker refuses a non-map before runtime.
;;
;; The differential that settled it is the sibling `PersistentVector/contains?`, already ruled
;; `total: true`: its impl carries the SAME receiver raise and has NO hashability guard at all.
;; This verb is strictly MORE total than one already ruled total.
;;
;; ⚠ AND THE HONEST BOUND, stated rather than papered: exit (a) is **UNREACHABLE FROM A `where`**.
;; `value_is_hashable` (`runtime.rs:10514`) excludes only RESOURCES — `fn`, Sender, Receiver,
;; ChildHandle, RustOpaque, the holon opaques — and arc 278's §7 purity wall already bars every
;; one of those from a fence context. So no line below exercises it, and none pretends to:
;; designing or testing around an unreachable arm is how a lie accumulates
;; (`[[feedback_an_unreachable_arm_accumulates_lies]]`). It is documented because it is why the
;; verb is total on the KEY axis, not because a `where` can reach it.

;; ── 1 + 3: resolution, and a HIT and a MISS on one map ────────────────────────────────────
(def :probe-pm-contains-hit
  (:wat::rete::map::contains-key?
    (:wat::core::PersistentMap :alpha 1 :beta 2) :alpha))

(def :probe-pm-contains-miss
  (:wat::rete::map::contains-key?
    (:wat::core::PersistentMap :alpha 1 :beta 2) :gamma))

;; ── 2: PARAMETRICITY — K and V are independent ────────────────────────────────────────────
;; Above: K = keyword, V = i64. K ≠ V, so a shape that reused ONE type variable for both
;; positions cannot type-check it. Below: K = String, V = String — the degenerate case, which
;; a K≠V-only scheme would wrongly refuse. Both must pass, and only genuinely independent
;; `["K", "V"]` type params satisfy both.
(def :probe-pm-contains-str-hit
  (:wat::rete::map::contains-key?
    (:wat::core::PersistentMap "k1" "v1" "k2" "v2") "k1"))

(def :probe-pm-contains-str-miss
  (:wat::rete::map::contains-key?
    (:wat::core::PersistentMap "k1" "v1" "k2" "v2") "nope"))

;; ── The sibling, for the differential the audit rests on ──────────────────────────────────
;; Kept in this file deliberately: the totality ruling above is an argument FROM this verb, so
;; the two spellings live side by side and regress together.
(def :probe-pv-contains-sibling
  (:wat::rete::vector::contains? (:wat::core::PersistentVector 1 2 3) 2))

;; ⚠ THE CALLS ARE INLINE HERE, DELIBERATELY. The first draft of this main printed the `def`
;; names above (`:hit :probe-pm-contains-hit …`) and the output came back as the KEYWORDS
;; themselves — a map of keyword→keyword, which type-checks perfectly and proves NOTHING. The
;; print looked like evidence and was vacuous, which is the same failure this file's header
;; warns about one level up. Inline calls cannot do that: the booleans below are produced by
;; the row, at run time, or they are not there at all.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::PersistentMap
      :hit
      (:wat::rete::map::contains-key?
        (:wat::core::PersistentMap :alpha 1 :beta 2) :alpha)
      :miss
      (:wat::rete::map::contains-key?
        (:wat::core::PersistentMap :alpha 1 :beta 2) :gamma)
      :str-hit
      (:wat::rete::map::contains-key?
        (:wat::core::PersistentMap "k1" "v1" "k2" "v2") "k1")
      :str-miss
      (:wat::rete::map::contains-key?
        (:wat::core::PersistentMap "k1" "v1" "k2" "v2") "nope")
      :pv-sibling
      (:wat::rete::vector::contains? (:wat::core::PersistentVector 1 2 3) 2))))

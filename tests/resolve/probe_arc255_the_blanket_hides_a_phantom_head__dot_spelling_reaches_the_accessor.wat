;; Arc 255 / 296 — WHY the dot spelling yields Option.None. MEASURED 2026-09-08.
;;
;; It is NOT a mis-built variant. `(K {map})` is the shape of BOTH the enum map
;; ctor (296 M) and the keyword-as-accessor fall-through (234.3c, runtime.rs:3655).
;; When K does not resolve to a verb, the accessor treats K as a KEY, misses, and
;; returns None — a total lookup, never an error. The `:wat::*` blanket is what
;; lets a `:wat::core::` head reach that fall-through; a `:usr::` head is refused
;; by resolve first.
;;
;; ⛔ UPDATED 2026-09-09 — THE BLANKET IS DEAD, and the sentence above is now the record of WHY
;; the ordering mattered rather than a live description. `:wat::core::Option.Some` no longer
;; reaches the accessor fall-through: it is refused by resolve, exactly as a `:usr::` head always
;; was. The asymmetry that made the dot spelling silently answer `#wat.core/Option.None {}` is
;; gone.
;;
;; ★ THIS IS WHY THE SEAM ORDERED THE BLANKET'S DEATH BEFORE THE DOT FLIP. Had the notation
;; flipped first, every partially-migrated head would have become a silent wrong ANSWER instead
;; of a loud error. Measured on clean main before the deletion:
;;   (:wat::core::Option.Some {:value 7})   check=0   =>   #wat.core/Option.None {}
;; and after it: refused at resolve. The ordering argument, discharged.
;;
;; MOVED 2026-09-09 (arc 255 Stone 4) from `wat-scripts/scratch-pad/keyword-accessor-vs-enum-map-
;; ctor.wat` to `tests/resolve/`, out from under the loader gate's scan root
;; (`tests/lint/wat_scripts_fixes_load.rs`). This file's two `:wat::core::Option.Some` lines
;; type-check ONLY because the `:wat::*` blanket lets that head reach the keyword-accessor
;; fall-through described above; a `:usr::` head would be refused by resolve first. Asserted (not
;; merely exhibited) by `tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head.rs`, which
;; pins TODAY's behaviour (`--check` exit 0, all five printlns succeed at `run`) as a RATCHET
;; aimed at the blanket: when the blanket's own stone lands, the two `Option.Some` lines are
;; refused at resolve instead of reaching the accessor, this row goes red at exactly the right
;; moment, and that stone owns updating it.
;; `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`
(:wat::core::defn :user::main [] -> :wat::core::nil
  ;; the ctor — K resolves, the map is the payload
  (:wat::kernel::println (:wat::core::Option.Some {:value 7}))
  ;; the accessor — K does not resolve, so the map is the RECEIVER and K is the key
  (:wat::kernel::println (:wat::core::Option.Some {:value 7}))
  ;; ... and it finds the key when the key is actually there
  (:wat::kernel::println (:wat::core::Option.Some {:wat::core::Option.Some 42}))
  ;; a bare keyword needs no blanket at all — the accessor is not about `:wat::*`
  (:wat::kernel::println (:anything-at-all {:value 7}))
  ;; and the accessor is TOTAL: a hit is Some, a miss is None, neither is an error.
  ;; (Give the same unknown head a NON-map receiver and it is refused loudly —
  ;;  `(:wat::core::Nope 7)` raises UnknownFunction. The map receiver is the door.)
  (:wat::kernel::println (:value {:value 7})))

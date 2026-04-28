;; :wat::holon::Hologram — convenience getters atop the substrate
;; primitive.
;;
;; The substrate ships:
;;   :wat::holon::Hologram/new   d                                 -> Hologram
;;   :wat::holon::Hologram/put   store pos key val                 -> ()
;;   :wat::holon::Hologram/get   store pos probe filter            -> Option<HolonAST>
;;   :wat::holon::Hologram/len   store                             -> i64
;;   :wat::holon::Hologram/dim   store                             -> i64
;;
;; The user-supplied filter takes a cosine score and returns whether
;; the highest-cosine candidate is "close enough." Two opinionated
;; defaults from `wat/holon/Filter.wat` cover the common cases:
;; `filter-coincident d` (strict — same point on the algebra grid)
;; and `filter-present d` (looser — signal detected above noise).
;;
;; This file ships the named-shape conveniences that compose the
;; defaults around `Hologram/get`. The store knows its `d` (via
;; `Hologram/dim`), so neither caller needs to pass d explicitly.
;;
;; Usage:
;;
;;   ;; strict — return only when probe+stored match at the same point
;;   (:wat::holon::Hologram/coincident-get store pos probe)
;;
;;   ;; looser — return when there's any presence above the noise floor
;;   (:wat::holon::Hologram/present-get store pos probe)
;;
;; Callers wanting a custom threshold compose `Hologram/get` with
;; their own filter directly. These two are the named shapes — the
;; ones the substrate carries an opinion about.

;; ─── get — substrate-side find-best + user-supplied filter ──────
;;
;; The substrate primitive Hologram/find-best returns the raw cosine
;; readout: the highest-cosine (key, val, cosine) triple in the
;; spread cells, or None if both cells are empty. This wat-stdlib
;; wrapper applies the user's filter to the cosine; if the filter
;; accepts, return Some(val); else None.
;;
;; Why wat-stdlib not Rust: the substrate's #[wat_dispatch] macro
;; doesn't pass SymbolTable to method bodies, so a Rust-side method
;; can't invoke a user-supplied wat lambda (apply_function needs
;; sym). Composing at the wat layer sidesteps that and makes the
;; same primitive composable for both Hologram and HologramLRU
;; (the latter's get is structurally identical, with added LRU
;; bookkeeping).
(:wat::core::define
  (:wat::holon::Hologram/get
    (store :wat::holon::Hologram)
    (pos :f64)
    (probe :wat::holon::HolonAST)
    (filter :fn(f64)->bool)
    -> :Option<wat::holon::HolonAST>)
  (:wat::core::match
    (:wat::holon::Hologram/find-best store pos probe)
    -> :Option<wat::holon::HolonAST>
    ((Some triple)
      (:wat::core::let*
        (((val :wat::holon::HolonAST) (:wat::core::second triple))
         ((cos :f64) (:wat::core::third triple)))
        (:wat::core::if (filter cos) -> :Option<wat::holon::HolonAST>
          (Some val)
          :None)))
    (:None :None)))

;; ─── coincident-get — strict variant ─────────────────────────────
(:wat::core::define
  (:wat::holon::Hologram/coincident-get
    (store :wat::holon::Hologram)
    (pos :f64)
    (probe :wat::holon::HolonAST)
    -> :Option<wat::holon::HolonAST>)
  (:wat::holon::Hologram/get store pos probe
    (:wat::holon::filter-coincident (:wat::holon::Hologram/dim store))))

;; ─── present-get — looser variant ────────────────────────────────
(:wat::core::define
  (:wat::holon::Hologram/present-get
    (store :wat::holon::Hologram)
    (pos :f64)
    (probe :wat::holon::HolonAST)
    -> :Option<wat::holon::HolonAST>)
  (:wat::holon::Hologram/get store pos probe
    (:wat::holon::filter-present (:wat::holon::Hologram/dim store))))

;; wat/rete/factbag.wat — one owner for the rete fact base.
;;
;; Session.facts is a FactBag (declared in wat/rete.wat so Session can name the type).
;; Every multiplicity policy lives here, named, instead of being reinvented at each
;; writer. Loads AFTER wat/rete.wat (FactBag, Session).
;;
;; Doors, all under `:wat::rete::factbag::` — the whitelist a future rung-3 seal will name:
;;   empty              → FactBag
;;   add                multiplicity +1 — what insert means
;;   add-if-absent      set-add — what merge-facts means, named
;;   remove-every-equal today's retract, named honestly. Strike 2 deletes it.
;;   retain             predicate filter — sub-multiset by construction
;;   count-of / size    the multiplicity acc::count already observes
;;   items              the read view — the ONE place that unwraps FactBag/items
;;   of                 Session → FactBag — the ONE place that reads Session/facts
;;
;; ⛔ THIS FILE IS A BUILD GATE, NOT THE TYPE SYSTEM. A record's :restricted-to is
;; parsed, stored, and never enforced (`NOTE-a-records-restricted-to-is-stored-and-never-enforced.md`).
;; `tests/lint/no_raw_factbag_access.rs` is the seal today. When rung 3 arrives, that
;; gate is deleted — the deletion is the proof.

;; of — Session → FactBag. The ONE call of Session/facts in wat/.
(:wat::core::defn :wat::rete::factbag::of
  [session <- :wat::rete::Session]
  -> :wat::rete::FactBag
  (:wat::rete::Session/facts session))

;; items — the ONE unwrap. Every other reader goes through a semantic door or this.
(:wat::core::defn :wat::rete::factbag::items
  [b <- :wat::rete::FactBag]
  -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::rete::FactBag/items b))

(:wat::core::defn :wat::rete::factbag::empty [] -> :wat::rete::FactBag
  (:wat::rete::FactBag :items (:wat::core::PersistentVector)))

(:wat::core::defn :wat::rete::factbag::size
  [b <- :wat::rete::FactBag]
  -> :wat::core::i64
  (:wat::core::length (:wat::rete::factbag::items b)))

(:wat::core::defn :wat::rete::factbag::add
  [b <- :wat::rete::FactBag
   f <- :wat::core::Record]
  -> :wat::rete::FactBag
  (:wat::rete::FactBag :items
    (:wat::core::PersistentVector/conj (:wat::rete::factbag::items b) f)))

(:wat::core::defn :wat::rete::factbag::add-if-absent
  [b <- :wat::rete::FactBag
   f <- :wat::core::Record]
  -> :wat::rete::FactBag
  (:wat::core::if (:wat::core::PersistentVector/contains? (:wat::rete::factbag::items b) f)
    b
    (:wat::rete::factbag::add b f)))

;; remove-every-equal — today's retract. Byte-identical fold: keep f iff f ≠ fact.
;; Strike 2 deletes this door.
(:wat::core::defn :wat::rete::factbag::remove-every-equal
  [b    <- :wat::rete::FactBag
   fact <- :wat::core::Record]
  -> :wat::rete::FactBag
  (:wat::core::let [old-facts (:wat::rete::factbag::items b)
                    new-facts (:wat::core::foldl
                                 (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                                                  f   <- :wat::core::Record]
                                   -> (:wat::core::PersistentVector :- [:wat::core::Record])
                                   (:wat::core::if (:wat::core::not (:wat::core::= f fact))
                                     (:wat::core::PersistentVector/conj acc f)
                                     acc))
                                 (:wat::core::PersistentVector)
                                 old-facts)]
    (:wat::rete::FactBag :items new-facts)))

;; retain — predicate filter. Sub-multiset by construction: walks items in order,
;; keeps matches, never dedups. fire.wat:316-322's length-as-set-test depends on this.
(:wat::core::defn :wat::rete::factbag::retain
  [b    <- :wat::rete::FactBag
   pred <- [:wat::core::Record :-> :wat::core::bool]]
  -> :wat::rete::FactBag
  (:wat::rete::FactBag :items
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                       f   <- :wat::core::Record]
        -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::if (pred f)
          (:wat::core::PersistentVector/conj acc f)
          acc))
      (:wat::core::PersistentVector)
      (:wat::rete::factbag::items b))))

(:wat::core::defn :wat::rete::factbag::count-of
  [b    <- :wat::rete::FactBag
   fact <- :wat::core::Record]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [n <- :wat::core::i64
                     f <- :wat::core::Record]
      -> :wat::core::i64
      (:wat::core::if (:wat::core::= f fact)
        (:wat::core::i64::+ n 1)
        n))
    0
    (:wat::rete::factbag::items b)))

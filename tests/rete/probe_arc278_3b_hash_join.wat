;; tests/rete/probe_arc278_3b_hash_join.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the Temperature and WindSpeed records for hash-join tests.

(:wat::core::defrecord :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])

;; P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
;; regenerates on re-fire. Join-correctness coverage relocated to:
;;   src/rete/kernel.rs #[cfg(test)]::hash_join_produces_one_token_on_same_loc
;;   src/rete/kernel.rs #[cfg(test)]::hash_join_drops_on_mismatched_loc
;;   src/rete/kernel.rs #[cfg(test)]::hash_join_no_cross_loc_leakage
;; These entries stay only so the (permanently #[ignore]d) sibling probes have somewhere to point.
;; THE HEART: a two-condition rule joining on ?loc. Temperature(Oslo,15) is always inserted; the
;; WindSpeed location varies (Oslo = match, Bergen = no-join) across the two scenario groups below.

(:wat::core::defn :user::htoks-length-oslo [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location "Oslo"))
     sess2 (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules sess2)
     network (:wat::rete::Session/network fired)
     bmem  (:wat::rete::Session/beta-memory fired)
     hjid  (:wat::core::Option/expect -> :wat::core::i64
              (:wat::core::get
                (:wat::core::filter
                  (:wat::core::fn [k <- :wat::core::i64] -> :wat::core::bool
                    (:wat::core::= (:wat::rete::node-kind-label
                                     (:wat::core::Option/expect -> :wat::core::Record (:wat::core::PersistentMap/get network k) "n"))
                                   "HashJoinNode"))
                  (:wat::core::PersistentMap/keys network))
                0) "hjid")
     htoks (:wat::core::match (:wat::core::PersistentMap/get bmem hjid) -> :wat::core::PersistentVector
              ((:wat::core::Some pv) pv)
              (:wat::core::None (:wat::core::PersistentVector)))]
    (:wat::core::length htoks)))

(:wat::core::defn :user::oslo-t-binding [] -> :wat::core::Option<wat::core::i64>
  (:wat::core::let
    [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location "Oslo"))
     sess2 (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules sess2)
     network (:wat::rete::Session/network fired)
     bmem  (:wat::rete::Session/beta-memory fired)
     hjid  (:wat::core::Option/expect -> :wat::core::i64
              (:wat::core::get
                (:wat::core::filter
                  (:wat::core::fn [k <- :wat::core::i64] -> :wat::core::bool
                    (:wat::core::= (:wat::rete::node-kind-label
                                     (:wat::core::Option/expect -> :wat::core::Record (:wat::core::PersistentMap/get network k) "n"))
                                   "HashJoinNode"))
                  (:wat::core::PersistentMap/keys network))
                0) "hjid")
     htoks (:wat::core::match (:wat::core::PersistentMap/get bmem hjid) -> :wat::core::PersistentVector
              ((:wat::core::Some pv) pv)
              (:wat::core::None (:wat::core::PersistentVector)))
     tok (:wat::core::Option/expect -> :wat::rete::Token (:wat::core::get htoks 0) "tok")
     b   (:wat::rete::Token/bindings tok)]
    (:wat::core::PersistentMap/get b "?t")))

(:wat::core::defn :user::oslo-w-binding [] -> :wat::core::Option<wat::core::i64>
  (:wat::core::let
    [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location "Oslo"))
     sess2 (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules sess2)
     network (:wat::rete::Session/network fired)
     bmem  (:wat::rete::Session/beta-memory fired)
     hjid  (:wat::core::Option/expect -> :wat::core::i64
              (:wat::core::get
                (:wat::core::filter
                  (:wat::core::fn [k <- :wat::core::i64] -> :wat::core::bool
                    (:wat::core::= (:wat::rete::node-kind-label
                                     (:wat::core::Option/expect -> :wat::core::Record (:wat::core::PersistentMap/get network k) "n"))
                                   "HashJoinNode"))
                  (:wat::core::PersistentMap/keys network))
                0) "hjid")
     htoks (:wat::core::match (:wat::core::PersistentMap/get bmem hjid) -> :wat::core::PersistentVector
              ((:wat::core::Some pv) pv)
              (:wat::core::None (:wat::core::PersistentVector)))
     tok (:wat::core::Option/expect -> :wat::rete::Token (:wat::core::get htoks 0) "tok")
     b   (:wat::rete::Token/bindings tok)]
    (:wat::core::PersistentMap/get b "?w")))

(:wat::core::defn :user::oslo-loc-binding [] -> :wat::core::Option<wat::core::String>
  (:wat::core::let
    [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location "Oslo"))
     sess2 (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules sess2)
     network (:wat::rete::Session/network fired)
     bmem  (:wat::rete::Session/beta-memory fired)
     hjid  (:wat::core::Option/expect -> :wat::core::i64
              (:wat::core::get
                (:wat::core::filter
                  (:wat::core::fn [k <- :wat::core::i64] -> :wat::core::bool
                    (:wat::core::= (:wat::rete::node-kind-label
                                     (:wat::core::Option/expect -> :wat::core::Record (:wat::core::PersistentMap/get network k) "n"))
                                   "HashJoinNode"))
                  (:wat::core::PersistentMap/keys network))
                0) "hjid")
     htoks (:wat::core::match (:wat::core::PersistentMap/get bmem hjid) -> :wat::core::PersistentVector
              ((:wat::core::Some pv) pv)
              (:wat::core::None (:wat::core::PersistentVector)))
     tok (:wat::core::Option/expect -> :wat::rete::Token (:wat::core::get htoks 0) "tok")
     b   (:wat::rete::Token/bindings tok)]
    (:wat::core::PersistentMap/get b "?loc")))

(:wat::core::defn :user::htoks-length-bergen [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location "Oslo"))
     sess2 (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location "Bergen"))
     fired (:wat::rete::fire-rules sess2)
     network (:wat::rete::Session/network fired)
     bmem  (:wat::rete::Session/beta-memory fired)
     hjid  (:wat::core::Option/expect -> :wat::core::i64
              (:wat::core::get
                (:wat::core::filter
                  (:wat::core::fn [k <- :wat::core::i64] -> :wat::core::bool
                    (:wat::core::= (:wat::rete::node-kind-label
                                     (:wat::core::Option/expect -> :wat::core::Record (:wat::core::PersistentMap/get network k) "n"))
                                   "HashJoinNode"))
                  (:wat::core::PersistentMap/keys network))
                0) "hjid")
     htoks (:wat::core::match (:wat::core::PersistentMap/get bmem hjid) -> :wat::core::PersistentVector
              ((:wat::core::Some pv) pv)
              (:wat::core::None (:wat::core::PersistentVector)))]
    (:wat::core::length htoks)))

;; HAZARD #1 — cross-product leakage. 2 Temps × 2 Winds across 2 locations must yield EXACTLY the 2
;; same-loc joins (Oslo×Oslo, Bergen×Bergen), NOT 4 (a naive cross ignoring ?loc) and NOT 0 (a bad
;; compatibility check).
(:wat::core::defn :user::htoks-length-2x2 [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))
     s0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     s1 (:wat::rete::insert s0 (:user::Temperature :celsius 15 :location "Oslo"))
     s2 (:wat::rete::insert s1 (:user::Temperature :celsius 10 :location "Bergen"))
     s3 (:wat::rete::insert s2 (:user::WindSpeed :kph 45 :location "Oslo"))
     s4 (:wat::rete::insert s3 (:user::WindSpeed :kph 50 :location "Bergen"))
     fired (:wat::rete::fire-rules s4)
     network (:wat::rete::Session/network fired)
     bmem  (:wat::rete::Session/beta-memory fired)
     hjid  (:wat::core::Option/expect -> :wat::core::i64
              (:wat::core::get
                (:wat::core::filter
                  (:wat::core::fn [k <- :wat::core::i64] -> :wat::core::bool
                    (:wat::core::= (:wat::rete::node-kind-label
                                     (:wat::core::Option/expect -> :wat::core::Record (:wat::core::PersistentMap/get network k) "n"))
                                   "HashJoinNode"))
                  (:wat::core::PersistentMap/keys network)) 0) "hjid")
     htoks (:wat::core::match (:wat::core::PersistentMap/get bmem hjid) -> :wat::core::PersistentVector
              ((:wat::core::Some pv) pv)
              (:wat::core::None (:wat::core::PersistentVector)))]
    (:wat::core::length htoks)))

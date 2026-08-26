;; BRIEF-native-where-vsa-ops — holon verbs inside a defrule `where`, oracle vs native.
;; Four-row Catalog (STOP-2: length==1 on identity is not enough — assert the NAME).
;; Degenerate cosine takes the CALLER's `:undefined`, not a constant.
;; coincident?/presence? must fire, not `compiled apply cannot dispatch`.

(:wat::core::defrecord :vsa::Catalog     [name <- :wat::core::String  obs <- :wat::holon::HolonAST])
(:wat::core::defrecord :vsa::Observation [obs  <- :wat::holon::HolonAST])
(:wat::core::defrecord :vsa::Guess       [name <- :wat::core::String])
(:wat::core::defrecord :vsa::Pair        [a <- :wat::holon::HolonAST  b <- :wat::holon::HolonAST])
(:wat::core::defrecord :vsa::Hit         [tag <- :wat::core::String])

(:wat::core::defn :vsa::table-of
  [f <- :wat::core::Fn(wat::core::bool)->wat::core::bool]
  -> :wat::holon::HolonAST
  (:wat::holon::to-holon
    (:wat::core::Vector :wat::core::bool (f true) (f false))))

(:wat::core::defn :vsa::id-fn [] -> :wat::core::Fn(wat::core::bool)->wat::core::bool
  (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool b))
(:wat::core::defn :vsa::not-fn [] -> :wat::core::Fn(wat::core::bool)->wat::core::bool
  (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool (:wat::core::if b false true)))
(:wat::core::defn :vsa::const-true-fn [] -> :wat::core::Fn(wat::core::bool)->wat::core::bool
  (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool true))
(:wat::core::defn :vsa::const-false-fn [] -> :wat::core::Fn(wat::core::bool)->wat::core::bool
  (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool false))

(:wat::rete::defrule :vsa::classify-cosine
  :when
  [(:vsa::Catalog (?name <- :name) (?cobs <- :obs))
   (:vsa::Observation (?obs <- :obs))
   (:wat::rete::where
     (:wat::rete::f64::>
       (:wat::rete::holon::cosine ?obs ?cobs :undefined 0.0)
       0.9))]
  :then
  [(:vsa::Guess :name ?name)])

(:wat::rete::defrule :vsa::classify-coincident
  :when
  [(:vsa::Catalog (?name <- :name) (?cobs <- :obs))
   (:vsa::Observation (?obs <- :obs))
   (:wat::rete::where (:wat::rete::holon::coincident? ?obs ?cobs))]
  :then
  [(:vsa::Guess :name ?name)])

(:wat::rete::defrule :vsa::classify-presence
  :when
  [(:vsa::Catalog (?name <- :name) (?cobs <- :obs))
   (:vsa::Observation (?obs <- :obs))
   (:wat::rete::where (:wat::rete::holon::presence? ?obs ?cobs))]
  :then
  [(:vsa::Guess :name ?name)])

(:wat::rete::defrule :vsa::deg-neg1
  :when
  [(:vsa::Pair (?a <- :a) (?b <- :b))
   (:wat::rete::where
     (:wat::rete::f64::=
       (:wat::rete::holon::cosine ?a ?b :undefined -1.0)
       -1.0))]
  :then
  [(:vsa::Hit :tag "neg1")])

(:wat::rete::defrule :vsa::deg-seven
  :when
  [(:vsa::Pair (?a <- :a) (?b <- :b))
   (:wat::rete::where
     (:wat::rete::f64::=
       (:wat::rete::holon::cosine ?a ?b :undefined 7.0)
       7.0))]
  :then
  [(:vsa::Hit :tag "seven")])

(:wat::rete::defquery :vsa::q-Guess
  :params []
  :when [(:vsa::Guess (?name <- :name))])

(:wat::rete::defquery :vsa::q-Hit
  :params []
  :when [(:vsa::Hit (?tag <- :tag))])

(:wat::core::defn :vsa::catalog [] -> (:wat::core::PersistentVector :- [:vsa::Catalog])
  (:wat::core::PersistentVector
    (:vsa::Catalog :name "identity"    :obs (:vsa::table-of (:vsa::id-fn)))
    (:vsa::Catalog :name "not"         :obs (:vsa::table-of (:vsa::not-fn)))
    (:vsa::Catalog :name "const-true"  :obs (:vsa::table-of (:vsa::const-true-fn)))
    (:vsa::Catalog :name "const-false" :obs (:vsa::table-of (:vsa::const-false-fn)))))

(:wat::core::defn :vsa::guess-name
  [fire    <- :wat::core::Fn(wat::rete::Session)->wat::rete::Session
   rule    <- :wat::rete::Rule
   mystery <- :wat::core::Fn(wat::core::bool)->wat::core::bool]
  -> :wat::core::String
  (:wat::core::let
    [s0   (:wat::rete::compile-all
            (:wat::core::PersistentVector rule)
            (:wat::core::PersistentVector (:vsa::q-Guess)))
     s1   (:wat::rete::insert-all s0 (:vsa::catalog))
     s2   (:wat::rete::insert s1 (:vsa::Observation :obs (:vsa::table-of mystery)))
     fired (fire s2)
     hits  (:wat::rete::query fired (:vsa::q-Guess))
     n     (:wat::core::length hits)]
    (:wat::core::if (:wat::i64::= n 1)
      (:wat::core::Option/expect
        (:wat::core::PersistentMap/get (:wat::core::first hits) "?name")
        "q-Guess: ?name")
      (:wat::core::String/concat "count=" (:wat::i64::to-string n)))))

(:wat::core::defn :vsa::deg-count
  [fire <- :wat::core::Fn(wat::rete::Session)->wat::rete::Session
   rule <- :wat::rete::Rule]
  -> :wat::core::i64
  (:wat::core::let
    [h     (:wat::holon::to-holon "some-atom")
     other (:wat::holon::to-holon "an-entirely-different-atom")
     zero  (:wat::holon::Blend h h 1.0 -1.0)
     s0    (:wat::rete::compile-all
             (:wat::core::PersistentVector rule)
             (:wat::core::PersistentVector (:vsa::q-Hit)))
     s1    (:wat::rete::insert s0 (:vsa::Pair :a zero :b other))
     fired (fire s1)]
    (:wat::core::length (:wat::rete::query fired (:vsa::q-Hit)))))

(:wat::core::defn :vsa::deg-counts
  [fire <- :wat::core::Fn(wat::rete::Session)->wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:vsa::deg-count fire (:vsa::deg-neg1))
    (:vsa::deg-count fire (:vsa::deg-seven))))

;; ── cosine classify (the j2 fixture) ─────────────────────────────────────────
(:wat::core::defn :user::oracle-id [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules$oracle (:vsa::classify-cosine) (:vsa::id-fn)))
(:wat::core::defn :user::native-id [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules (:vsa::classify-cosine) (:vsa::id-fn)))
(:wat::core::defn :user::oracle-not [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules$oracle (:vsa::classify-cosine) (:vsa::not-fn)))
(:wat::core::defn :user::native-not [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules (:vsa::classify-cosine) (:vsa::not-fn)))
(:wat::core::defn :user::oracle-const-true [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules$oracle (:vsa::classify-cosine) (:vsa::const-true-fn)))
(:wat::core::defn :user::native-const-true [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules (:vsa::classify-cosine) (:vsa::const-true-fn)))
(:wat::core::defn :user::oracle-const-false [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules$oracle (:vsa::classify-cosine) (:vsa::const-false-fn)))
(:wat::core::defn :user::native-const-false [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules (:vsa::classify-cosine) (:vsa::const-false-fn)))

;; ── coincident? / presence? ──────────────────────────────────────────────────
(:wat::core::defn :user::oracle-coincident-id [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules$oracle (:vsa::classify-coincident) (:vsa::id-fn)))
(:wat::core::defn :user::native-coincident-id [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules (:vsa::classify-coincident) (:vsa::id-fn)))
(:wat::core::defn :user::oracle-coincident-not [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules$oracle (:vsa::classify-coincident) (:vsa::not-fn)))
(:wat::core::defn :user::native-coincident-not [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules (:vsa::classify-coincident) (:vsa::not-fn)))
(:wat::core::defn :user::oracle-presence-id [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules$oracle (:vsa::classify-presence) (:vsa::id-fn)))
(:wat::core::defn :user::native-presence-id [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules (:vsa::classify-presence) (:vsa::id-fn)))
(:wat::core::defn :user::oracle-presence-not [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules$oracle (:vsa::classify-presence) (:vsa::not-fn)))
(:wat::core::defn :user::native-presence-not [] -> :wat::core::String
  (:vsa::guess-name :wat::rete::fire-rules (:vsa::classify-presence) (:vsa::not-fn)))

;; presence? is a noise-floor detector, not a 0.9 classifier — four-row catalog
;; can hit more than one name. Self vs orthogonal pins it is not silent-false.
(:wat::core::defn :vsa::presence-pair
  [fire     <- :wat::core::Fn(wat::rete::Session)->wat::rete::Session
   cat-name <- :wat::core::String
   cat-fn   <- :wat::core::Fn(wat::core::bool)->wat::core::bool
   mystery  <- :wat::core::Fn(wat::core::bool)->wat::core::bool]
  -> :wat::core::String
  (:wat::core::let
    [s0    (:wat::rete::compile-all
             (:wat::core::PersistentVector (:vsa::classify-presence))
             (:wat::core::PersistentVector (:vsa::q-Guess)))
     s1    (:wat::rete::insert s0 (:vsa::Catalog :name cat-name :obs (:vsa::table-of cat-fn)))
     s2    (:wat::rete::insert s1 (:vsa::Observation :obs (:vsa::table-of mystery)))
     fired (fire s2)
     hits  (:wat::rete::query fired (:vsa::q-Guess))
     n     (:wat::core::length hits)]
    (:wat::core::if (:wat::i64::= n 1)
      (:wat::core::Option/expect
        (:wat::core::PersistentMap/get (:wat::core::first hits) "?name")
        "q-Guess: ?name")
      (:wat::core::String/concat "count=" (:wat::i64::to-string n)))))

(:wat::core::defn :user::oracle-presence-self [] -> :wat::core::String
  (:vsa::presence-pair :wat::rete::fire-rules$oracle "identity" (:vsa::id-fn) (:vsa::id-fn)))
(:wat::core::defn :user::native-presence-self [] -> :wat::core::String
  (:vsa::presence-pair :wat::rete::fire-rules "identity" (:vsa::id-fn) (:vsa::id-fn)))
(:wat::core::defn :user::oracle-presence-orthogonal [] -> :wat::core::String
  (:vsa::presence-pair :wat::rete::fire-rules$oracle "not" (:vsa::not-fn) (:vsa::id-fn)))
(:wat::core::defn :user::native-presence-orthogonal [] -> :wat::core::String
  (:vsa::presence-pair :wat::rete::fire-rules "not" (:vsa::not-fn) (:vsa::id-fn)))

;; ── degenerate cosine inside a rule ──────────────────────────────────────────
(:wat::core::defn :user::oracle-deg [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:vsa::deg-counts :wat::rete::fire-rules$oracle))
(:wat::core::defn :user::native-deg [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:vsa::deg-counts :wat::rete::fire-rules))

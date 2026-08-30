;; grid/neg-consumer.wat — AXIS: a POSITIVE consumer downstream of a NEGATION gate.
;;
;; ★ THIS IS THE FIRST THREE-WAY AXIS. Every other axis compares NATIVE against Clara and
;; never runs the oracle (measured 2026-08-13: 27 `fire-rules` calls across the grid, zero
;; `fire-rules-spec`; the five mentions of the oracle in grid files are comments saying
;; "NOT the wat oracle"). This one fires BOTH impls on the SAME staged session and emits
;; both derived sets, so the runner can render three verdicts instead of one:
;;
;;   oracle vs clara  ≠  ⇒ the SPEC is wrong
;;   native vs clara  ≠  ⇒ the fast path is wrong
;;   oracle vs native ≠  ⇒ a PORT bug (what the internal differential already catches)
;;
;; The old two-way could not distinguish "spec bug" from "port bug", and — the reason this
;; file exists — could not see a flaw the oracle and its faithful Rust port SHARE. That is
;; not hypothetical: it is how task #94 survived weeks of work on a green grid.
;;
;; ─── WHY THIS SHAPE, AND WHY NO EXISTING AXIS COVERS IT ──────────────────────
;; Measured, this session, across every negation-bearing axis:
;;   strat-neg    chains strata by NEGATING the previous one (S_n :- …, not S_{n-1}), so
;;                EVERY rule in the chain carries a negation and every rule gets bumped.
;;   negation     a single `not Bad`, one level, and NOTHING consumes its output.
;;   deep-cascade `grep -c 'rete::not'` => 0. A pure positive chain, all in stratum 0.
;;
;; So the grid tests two PURE regimes — deep positive chaining with no negation, and deep
;; negation chaining with negation at every level — and never the INTERLEAVING. A rule that
;; consumes POSITIVELY a fact produced by a negation-bearing rule is the one shape where
;; positive dependency must propagate the stratum, and no axis crosses that seam.
;;
;; ─── THE WORKLOAD ────────────────────────────────────────────────────────────
;;   Item(k)          k in [0, N)                     input
;;   Bad(k)           every EVEN k                    input
;;   Tag(k)           every k                          input — the "ruling table" partner
;;   Ok(k)    :- Item(k), NOT Bad(k)                  the GATE  (stratum >= 1)
;;   Final(k) :- Ok(k), Tag(k)                        the POSITIVE CONSUMER  <- the subject
;;
;; :derived is the sorted Final keys. Correct answer: exactly the ODD k. Clara derives them.
;;
;; ★ THIS AXIS FOUND AND THEN CLOSED task #94. As built it was RED: the stratifier ordered by
;; negation dependency ONLY, so `final` — which negates nothing — was assigned a stratum BELOW
;; the one producing Ok, fired before Ok existed, and never re-fired. BOTH wat impls returned
;; EMPTY, which is why `oracle == native` passed and the internal differential was blind.
;; Fixed in ff581b6f (positive dependencies now propagate). It must now read :accuracy :match
;; on ALL THREE columns; any MISMATCH is that regression returning.

(:wat::core::defrecord :nc::Item  [k <- :wat::core::i64])
(:wat::core::defrecord :nc::Bad   [k <- :wat::core::i64])
(:wat::core::defrecord :nc::Tag   [k <- :wat::core::i64])
(:wat::core::defrecord :nc::Ok    [k <- :wat::core::i64])
(:wat::core::defrecord :nc::Final [k <- :wat::core::i64])

;; :derived / :native-ns keep the existing runner contract byte-for-byte; the two oracle
;; fields are ADDITIVE, so every other axis and the 2-way path are untouched.
(:wat::core::defrecord :grid::Result
  [axis           <- :wat::core::String
   size           <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived        <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :nc::q-Final
  :params []
  :when [(?fact <- :nc::Final)])


(:wat::core::defn :nc::build-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    ;; THE GATE — negates a base fact, so the stratifier lifts it correctly.
    (:wat::rete::Rule :name "ok"
      :lhs (:wat::core::PersistentVector
        (:wat::core::quasiquote (:nc::Item (?k <- :k)))
        (:wat::core::quasiquote (:wat::rete::not (:nc::Bad (?k <- :k)))))
      :rhs (:wat::core::PersistentVector
        (:wat::core::quasiquote (:nc::Ok ?k))))
    ;; THE SUBJECT — consumes the gate's output POSITIVELY and negates nothing.
    (:wat::rete::Rule :name "final"
      :lhs (:wat::core::PersistentVector
        (:wat::core::quasiquote (:nc::Ok  (?k <- :k)))
        (:wat::core::quasiquote (:nc::Tag (?k <- :k))))
      :rhs (:wat::core::PersistentVector
        (:wat::core::quasiquote (:nc::Final ?k))))))

(:wat::core::defn :nc::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::let [a2 (:wat::vector::conj acc (:nc::Item i))
                          a3 (:wat::vector::conj a2 (:nc::Tag i))]
          (:wat::core::if (:wat::core::= i (:wat::i64::* (:wat::i64::/ i 2) 2))
            (:wat::vector::conj a3 (:nc::Bad i))
            a3)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

(:wat::core::defn :nc::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

(:wat::core::defn :nc::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [codes (:wat::core::into (:wat::core::Vector :- [:wat::core::i64])
                            (:wat::core::map
                              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:nc::Final/k f)))
                              (:wat::rete::query fired (:nc::q-Final))))]
    (:nc::vec->pvec (:wat::core::sort codes))))

(:wat::core::defn :nc::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
                    rules   (:nc::build-rules)
                    staged  (:nc::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:nc::q-Final))) items)
                    ;; NATIVE — the production fast path. Timed alone.
                    n0      (:wat::time::now)
                    fired   (:wat::rete::fire-rules staged)
                    n1      (:wat::time::now)
                    ;; ORACLE — the wat spec, fired on the SAME staged session. Value semantics
                    ;; make the two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::rete::fire-rules$oracle staged)
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "neg-consumer"
                     :size (:wat::core::PersistentVector items)
                     :derived (:nc::derived-vector fired)
                     :native-ns (:nc::ns-between n0 n1)
                     :oracle-derived (:nc::derived-vector ofired)
                     :oracle-ns (:nc::ns-between o0 o1)))))

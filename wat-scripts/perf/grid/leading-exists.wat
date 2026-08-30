;; wat-scripts/perf/grid/leading-exists.wat — GRID AXIS: a LEADING :exists observed
;; through a QUERY, across a MULTI-ROUND fixpoint.
;;
;; WHY THIS AXIS EXISTS. On 2026-08-24 a full vigilia found that a leading
;; (parentless) `:not`/`:exists` emitted one token PER FIXPOINT ROUND into the
;; cumulative beta, so a query over such a rule returned N rows where 1 is correct
;; — N being the round count, exactly (chain 2 -> 2 rows, 3 -> 3, 4 -> 4, 6 -> 6).
;; It survived 5016 tests and the whole grid.
;;
;; IT WAS NOT A DIFFERENTIAL FAILURE. Both references were RIGHT:
;;   - the wat `$oracle` returned the correct count. It is immune BY CONSTRUCTION —
;;     `fire-once$oracle` rebuilds alpha and beta from an EMPTY PersistentMap every
;;     fire, so it has no cumulative memory for a re-emission to compound into.
;;   - Clara returned the correct count too (1 activation for a leading :exists,
;;     with an unrelated cascade running).
;; So the machinery would have caught this instantly. THE CORPUS HAD NO CASE THAT
;; POINTED AT IT. This axis is that case.
;;
;; THE THREE THINGS THAT MUST ALL HOLD, or the defect hides again:
;;   1. LEADING. The `:exists` must have NO parent condition. A mid-chain exists is
;;      fed by its parent's delta and is already round-scoped — it never showed the
;;      defect. (`probe_arc278_7exists_native_differential` is named for the
;;      "no multiplicity" contract but puts `:exists` SECOND, so it could not reach
;;      the configuration it claims to guard, and passed while the defect was live.)
;;   2. OBSERVED THROUGH A QUERY, not through derived facts. `production_delta`
;;      dedups derived facts by value, which masks token multiplicity completely.
;;   3. MULTI-ROUND. The inert S1..S6 chain below exists ONLY to make the fixpoint
;;      iterate. With one round, "once per fire" and "once per round" are the same
;;      number and the defect is invisible. The chain touches nothing Wind-related
;;      on purpose: the row count must not depend on it, and that independence IS
;;      the property under test.
;;
;; SCOPE — WHY :exists AND NOT :not, stated so nobody reads this as an oversight.
;; The defect hit BOTH leading arms and one fix closed both (they share `filter_pass`
;; and one fire-scoped guard). Clara CAN express the `:not` arm, but only unbound —
;; `[:not [Ghost (= ?k k)]]` is rejected outright ("Unbound variables: #{?k}"), and
;; `[:not [Ghost]]` is accepted and returns 1 row. So a leading-`:not` axis could only
;; ever witness a COUNT, where this axis witnesses the whole distinct SET. Both arms are
;; gated on every floor by `tests/rete/probe_arc278_leading_filter_multiplicity`, which
;; runs far more often than the grid; a second axis would add a JVM start per rung to buy
;; a strictly weaker witness. One axis, the richer witness, and the `:not` arm named here.
;;
;; Shape:
;;   Wind(loc)  — each loc in [0, items) seeded TWICE, so the distinct-inner-binding
;;                rule is exercised (two Winds at one loc => ONE {?loc} token, the
;;                Clara `test-simple-exists` semantics; verified against Clara 0.24.0
;;                directly: 5 Winds over 3 distinct locs => 3 rows, {:?loc "A"}).
;;   S1(1)      — one seed for the cascade; r2..r6 carry it to S6 over 6 rounds.
;;   q-exists   — LEADING :exists over Wind, binding ?loc.
;; => :derived is the sorted DISTINCT locs: exactly [0 .. items). Under the defect it
;;    was that vector repeated once per round, so the byte-for-byte compare fails
;;    loudly on length before it ever fails on content.
;;
;; Usage (stdin = an i64 vector [items]; stdout = one #grid/Result EDN line):
;;   echo '[200]' | cargo wat ./wat-scripts/perf/grid/leading-exists.wat
;;   => #grid/Result {:axis "leading-exists" :size [200] :derived [0 1 2 ...] :native-ns N}

(:wat::core::defrecord :lx::Wind [loc <- :wat::core::i64])
(:wat::core::defrecord :lx::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :lx::S2 [k <- :wat::core::i64])
(:wat::core::defrecord :lx::S3 [k <- :wat::core::i64])
(:wat::core::defrecord :lx::S4 [k <- :wat::core::i64])
(:wat::core::defrecord :lx::S5 [k <- :wat::core::i64])
(:wat::core::defrecord :lx::S6 [k <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

;; THE WITNESS: a LEADING :exists, no parent condition, binding ?loc outward.
(:wat::rete::defquery :lx::q-exists
  :params []
  :when [(:wat::rete::exists (:lx::Wind (?loc <- :loc)))])

;; The inert cascade — five rules carrying S1 to S6, forcing six fixpoint rounds.
;; Nothing here mentions Wind; that is the point.
(:wat::core::defn :lx::build-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::rete::Rule :name "r2"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S1 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S2 ?k))))
    (:wat::rete::Rule :name "r3"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S2 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S3 ?k))))
    (:wat::rete::Rule :name "r4"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S3 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S4 ?k))))
    (:wat::rete::Rule :name "r5"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S4 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S5 ?k))))
    (:wat::rete::Rule :name "r6"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S5 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lx::S6 ?k))))))

;; Seed: Wind(i) TWICE for each i in [0, items) — the duplicate is what makes the
;; distinct-inner-binding rule load-bearing — plus one S1 to start the cascade.
(:wat::core::defn :lx::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::PersistentVector/conj
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                        -> (:wat::core::PersistentVector :- [:wat::core::Record])
          (:wat::core::PersistentVector/conj
            (:wat::core::PersistentVector/conj acc (:lx::Wind i))
            (:lx::Wind i)))
        (:wat::core::PersistentVector)
        (:wat::core::range 0 items))
      (:lx::S1 1))))

(:wat::core::defn :lx::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; THE ACCURACY WITNESS. Sorted distinct ?loc from the leading-:exists query.
;; Under the defect this vector was `rounds` times too long — length fails first.
(:wat::core::defn :lx::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [locs (:wat::core::into (:wat::core::Vector :wat::core::i64)
                           (:wat::core::map
                             (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
                               (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?loc") "query: ?loc"))
                             (:wat::rete::query fired (:lx::q-exists))))]
    (:lx::vec->pvec (:wat::core::sort locs))))

(:wat::core::defn :lx::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
                    rules   (:lx::build-rules)
                    staged  (:lx::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:lx::q-exists))) items)
                    n0      (:wat::time::now)
                    fired   (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    n1      (:wat::time::now)
                    derived (:lx::derived-vector fired)
                    nat-ns  (:lx::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "leading-exists" :size (:wat::core::PersistentVector items) :derived derived :native-ns nat-ns :oracle-derived (:lx::derived-vector ofired) :oracle-ns (:lx::ns-between o0 o1)))))

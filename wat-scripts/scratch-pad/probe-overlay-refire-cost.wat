;; probe-overlay-refire-cost.wat — DOES FIRING OVER AN ALREADY-FIRED SESSION REDO THE BASE'S WORK?
;;
;; THE QUESTION (2026-08-01). The builder's session model:
;;
;;     session = make_session(rules)
;;     session.insert(persistent_facts)          ;; a committed BASE, fired once
;;     with_session(session) do |s|              ;; an OVERLAY: temp facts, fire, query, discard
;;       s.insert(temp); s.fire_rules; s.query(...)
;;     end
;;
;; NOTE-overlay-read-path (2026-06-20) establishes the base is protected BY CONSTRUCTION — value
;; semantics over persistent collections, so the overlay is structural sharing and the discard is
;; free. That half is settled. What is NOT settled, and what decides whether the pattern is real or
;; a trap, is the COST: when the block fires, does the engine re-derive the whole base, or only the
;; delta the block added?
;;
;;   O(delta)      -> a base can be large and blocks are cheap; pre-fire the base and share it.
;;   O(everything) -> every block pays for the whole base, and the ergonomics would be selling
;;                    something the engine does not deliver.
;;
;; DO NOT ANSWER THIS BY READING THE KERNEL. Twice on 2026-08-01 a component was blamed from reading
;; code and disproved by measurement (seeding: ~13ms not seconds; vec->pvec: deleting it moved the
;; wall by nothing). This probe measures it.
;;
;; ── THE DISCRIMINATOR, and why it is a LADDER and not one number ──────────────────────────────
;;
;; A single timing cannot answer this: any constant could be explained away. The signal is how the
;; overlay's cost behaves AS THE BASE GROWS, with the delta held FIXED:
;;
;;   overlay time FLAT as N grows   ->  incremental. The engine sees only the delta.
;;   overlay time TRACKS N          ->  full redo. Each block re-derives the base.
;;
;; Three timings per rung, so the two hypotheses are separated rather than inferred:
;;   :cold-ns    fire an UNFIRED base of N facts            — the full cost, the yardstick
;;   :noop-ns    fire an ALREADY-FIRED base, nothing added  — is a re-fire free at all?
;;   :ovl-ns     fire an already-fired base + M new facts   — the with-block's real cost
;;
;; :noop-ns is the sharpest of the three. If a re-fire with NOTHING new still costs ~:cold-ns, the
;; engine redoes its work unconditionally and no overlay design can be cheap.
;;
;; ── NON-VACUITY ───────────────────────────────────────────────────────────────────────────────
;;
;; A probe that times an overlay deriving NOTHING measures nothing. The temp facts are chosen to
;; MATCH, and the run asserts the derived count actually rose by exactly M — so a silently-empty
;; overlay fails loudly instead of reporting a flattering zero.
;;
;; Ladder grows UPWARD from small (R24: these harnesses are interpreted and can be super-linear; a
;; rung that returns fast says nothing about the next one). N caps at 8000 — one non-join rule over
;; 8000 facts is far under the sizes that have hurt this box before.
;;
;; Usage:  ./target/release/wat wat-scripts/scratch-pad/probe-overlay-refire-cost.wat

(:wat::core::defrecord :ovl::Req [k <- :wat::core::i64])
(:wat::core::defrecord :ovl::Hit [k <- :wat::core::i64])

(:wat::rete::defquery :ovl::q-Hit
  :params []
  :when [(?fact <- :ovl::Hit)])


;; The rule is deliberately MINIMAL — one alpha condition, one production, no join. Cost is then
;; ~proportional to the facts the engine actually processes, so a "redo" shows up as time tracking N
;; instead of being buried under join work. Hit(k) :- Req(?k) AND k mod 10 == 3.
(:wat::core::defn :ovl::rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::let [conds   (:wat::core::quasiquote (:ovl::Req (?k <- :k)))
                      where-c (:wat::core::quasiquote
                                (:wat::rete::where
                                  (:wat::core::= 3
                                    (:wat::core::i64::- ?k
                                      (:wat::core::i64::* (:wat::core::i64::/ ?k 10) 10)))))
                      ins     (:wat::core::quasiquote (:ovl::Hit ?k))]
      (:wat::rete::Rule :name "mod10"
        :lhs (:wat::core::PersistentVector conds where-c)
        :rhs (:wat::core::PersistentVector ins)))))

;; stage session lo hi — insert Req(i) for i in [lo, hi) in ONE rebuild.
(:wat::core::defn :ovl::stage
  [session <- :wat::rete::Session  lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj acc (:ovl::Req :k i)))
      (:wat::core::PersistentVector)
      (:wat::core::range lo hi))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :ovl::derived-count [fired <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::Vector/length
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:ovl::Hit/k f)))
        (:wat::rete::query fired (:ovl::q-Hit))))))

(:wat::core::defn :ovl::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; TEMP-BASE — the overlay's facts start here, far past any base rung, so they cannot collide with
;; base keys. All 10 are ≡ 3 (mod 10), so every one of them MUST derive a Hit.
(:wat::core::defn :ovl::temp-lo [] -> :wat::core::i64 1000003)
(:wat::core::defn :ovl::temp-n  [] -> :wat::core::i64 10)

;; rung n — the three timings at one base size, plus the non-vacuity assertion.
(:wat::core::defn :ovl::rung [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [staged    (:ovl::stage (:wat::rete::compile-all (:ovl::rules) (:wat::core::PersistentVector (:ovl::q-Hit))) 0 n)

     ;; (1) COLD — fire an unfired base of n facts. The yardstick.
     c0        (:wat::time::now)
     fired     (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     c1        (:wat::time::now)

     ;; (2) NO-OP — fire the ALREADY-FIRED session again, nothing added. If this is not ~free, the
     ;;     engine redoes its work unconditionally and no overlay design can be cheap.
     p0        (:wat::time::now)
     refired   (:wat::core::match (:wat::rete::fire-rules fired) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     p1        (:wat::time::now)

     ;; (3) OVERLAY — the with-block's real shape: already-fired base + a FIXED small delta.
     scratch   (:ovl::stage fired (:ovl::temp-lo)
                 (:wat::core::i64::+ (:ovl::temp-lo)
                   (:wat::core::i64::* (:ovl::temp-n) 10)))
     o0        (:wat::time::now)
     overlaid  (:wat::core::match (:wat::rete::fire-rules scratch) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     o1        (:wat::time::now)

     base-d    (:ovl::derived-count fired)
     ovl-d     (:ovl::derived-count overlaid)

     ;; NON-VACUITY: the 10 temp keys are all ≡3 mod 10, so the overlay MUST derive exactly 10 more
     ;; than the base. If it does not, the overlay derived nothing (or the wrong thing) and every
     ;; timing above is measuring air — fail loudly rather than report a flattering number.
     _guard    (:wat::core::if
                 (:wat::core::= ovl-d (:wat::core::i64::+ base-d (:ovl::temp-n)))
                 nil
                 (:wat::kernel::assertion-failed!
                   (:wat::core::String/concat
                     (:wat::core::String/concat "overlay derived " (:wat::core::i64::to-string ovl-d))
                     (:wat::core::String/concat " but base was " (:wat::core::i64::to-string base-d)))
                   :wat::core::None :wat::core::None))]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat "n=" (:wat::core::i64::to-string n))
        (:wat::core::String/concat " cold-ns=" (:wat::core::i64::to-string (:ovl::ns-between c0 c1))))
      (:wat::core::String/concat
        (:wat::core::String/concat " noop-ns=" (:wat::core::i64::to-string (:ovl::ns-between p0 p1)))
        (:wat::core::String/concat
          (:wat::core::String/concat " ovl-ns=" (:wat::core::i64::to-string (:ovl::ns-between o0 o1)))
          (:wat::core::String/concat " base-derived=" (:wat::core::i64::to-string base-d)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  n <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:ovl::rung n)))
    nil
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::PersistentVector 1000 2000 4000 8000))))

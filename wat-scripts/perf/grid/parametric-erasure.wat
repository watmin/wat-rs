;; wat-scripts/perf/grid/parametric-erasure.wat — GRID AXIS: ONE ERASED CLASS, MIXED PACKABILITY,
;; AT SCALE. The shape the grid corpus did not have.
;;
;; ⛔ WHY THIS AXIS EXISTS. The other grid axes derive their facts from records whose every field
;; is a bare `i64`, so every instance of every class packs. Measured 2026-09-03 across the whole
;; grid: **0 of 185 `defrecord` forms declare a parametric record** (`:- [T]`). D7 — arc 278's
;; silent fact-drop, a NATIVE-vs-ORACLE divergence, exactly the pairing this axis reports — needs
;; a single runtime CLASS whose instances DIFFER in packability, and there is no non-generic route
;; to that (no `Any` supertype `i64` and `String` share). So the port pairing could run green on
;; the entire corpus and say nothing about D7. This axis carries the shape.
;;
;; THE MECHANISM, restated from tests/rete/probe_arc278_d7_parametric_erasure_differential.wat:
;; `pack_i64_row` (src/rete/session.rs) tests RUNTIME values; `build_alpha_index` files every alpha
;; node under ONE erased `pat.type_head`. `(:pe::Box :- [T] [k <- i64  v <- :T])` erases `T`, so
;; `Box[i64]`, `Box[String]` and `Box[Tag]` are ONE class `pe::Box`. Both seed writers —
;; `alpha_activate_fact`'s push and the occupancy batch's whole-entry replace — land on the SAME
;; `aid`; before class-uniform batching the replace discarded the push and `d_alpha` was left
;; indexing DIFFERENT elements. The `Pair` rule below CONSUMES `Box`'s alpha delta as slot indices
;; through a join, which is what turns the aliasing into a WRONG BINDING rather than merely a
;; missing fact — so a right-sized wrong answer is exactly what this mechanism produces, and
;; `:derived` is the full key set, never a count.
;;
;; Shape (three rules, one session):
;;   Box(k, v)           — items facts, ONE class, v cycling i64 / String / Tag by (k mod 3), so
;;                         the class holds both packable and unpackable instances in BOTH
;;                         interleavings at any items >= 3.
;;   Plain(k)            — items facts, a NON-parametric uniformly-packable class in the same
;;                         session. It must KEEP the occupancy batch while its neighbour loses it:
;;                         a cure that narrowed batching to nothing would satisfy every equality
;;                         here and silently delete the fast path, so `Plain` is a live control.
;;   Hit(k)      :- Box(k, v)               — the erased class's own alpha consumer.
;;   PlainHit(k) :- Plain(k)                — the control's.
;;   Pair(k)     :- Box(k, v) AND Plain(k)  — ★ the JOIN. Box's alpha delta reaches this as slot
;;                                            indices, giving `pe::Box` a SECOND leaf aid and
;;                                            making a mis-indexed slot a wrong `?k`.
;;
;; Every key derives all three, so :derived is exactly 3*items facts — non-empty for any items>=1
;; and non-vacuous by construction (see the non-vacuity assertion in the gate that drives this).
;;
;; :derived is the FULL SORTED derived set, each fact canonicalized as one i64
;; (tag * 1,000,000 + k; Hit=0, PlainHit=1, Pair=2). NOT A COUNT.
;;
;; NO `gen-parametric-erasure.sh` TWIN, DELIBERATELY. `run-all.sh:81` treats a `.wat` without a
;; `gen-` twin as "not a perf axis" and skips it; this axis is a CORRECTNESS axis for the port
;; pairing, not a rung on the perf ladder, and the ladder's sizes are a published artifact that
;; must not drift.
;;
;; ⛔ THE CLARA TWIN IS A **STATIC** `parametric-erasure.clj`, AND THIS HEADER USED TO DENY IT.
;; It said: *"Clara has no parametric records either, so there is no `.clj` twin to author."*
;; STRUCK 2026-09-03. Clara referees RULE SEMANTICS, not wat's type system: the erasure is what wat
;; does to the DECLARATION, and what reaches the network is a bag of ordinary facts of ONE class
;; whose `v` fields hold different runtime types — which dynamically-typed Clojure expresses as its
;; NATIVE case. Leaving the one axis that carries D7's shape unrefereed by the reference engine is
;; a hole in the differential corpus exactly where the known bug lives. The twin reproduces the
;; DERIVED SET (not the declaration) and is STATIC precisely so it stays off the perf ladder —
;; see its own header, and `wat-scripts/perf/grid/check-grid-three-way.sh`, which drives it.
;;
;; Usage (stdin = an i64 vector [items]; stdout = one #grid/Result EDN line):
;;   echo '[200]' | cargo wat ./wat-scripts/perf/grid/parametric-erasure.wat
;;   => #grid/Result {:axis "parametric-erasure" :size [200] :derived [...] :native-ns N ...}

;; ★ THE PARAMETRIC RECORD — the constructor of the erasure seam. Copied in form from
;; tests/rete/probe_arc278_d7_parametric_erasure_differential.wat:34.
(:wat::core::defrecord :pe::Box :- [T] [k <- :wat::core::i64  v <- :T])

;; A RECORD-valued filler: a second, independent way for one `Box` instance to fail
;; `pack_i64_row` (Aggregate, not `Value::i64`) — so the axis is not pinned to `String`.
(:wat::core::defrecord :pe::Tag [n <- :wat::core::i64])

;; The NON-parametric, uniformly-packable neighbour. A live control, not decoration.
(:wat::core::defrecord :pe::Plain [k <- :wat::core::i64])

(:wat::core::defrecord :pe::Hit      [k <- :wat::core::i64])
(:wat::core::defrecord :pe::PlainHit [k <- :wat::core::i64])
(:wat::core::defrecord :pe::Pair     [k <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defrule :pe::r-box
  :when  [(:pe::Box (?k <- :k) (?v <- :v))]
  :then  [(:pe::Hit ?k)])

(:wat::rete::defrule :pe::r-plain
  :when  [(:pe::Plain (?k <- :k))]
  :then  [(:pe::PlainHit ?k)])

;; ★ The JOIN arm — Box's alpha delta consumed as SLOT INDICES into `wm.alpha[aid]`.
(:wat::rete::defrule :pe::r-pair
  :when  [(:pe::Box (?k <- :k) (?v <- :v))
          (:pe::Plain (?k <- :k))]
  :then  [(:pe::Pair ?k)])

(:wat::rete::defquery :pe::q-hit   :params [] :when [(?fact <- :pe::Hit)])
(:wat::rete::defquery :pe::q-plain :params [] :when [(?fact <- :pe::PlainHit)])
(:wat::rete::defquery :pe::q-pair  :params [] :when [(?fact <- :pe::Pair)])

;; encode tag k — canonical single-i64 witness (Hit=0, PlainHit=1, Pair=2). `items` is far below
;; 1,000,000 at every size this axis is run at, so the encoding is injective here.
(:wat::core::defn :pe::encode [tag <- :wat::core::i64  k <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+ (:wat::core::i64::* tag 1000000) k))

;; i64-mod a b — non-negative modulo via truncating division (min-finding.wat uses the same
;; idiom; there is no native i64::mod). a >= 0 and b > 0 at every call here.
(:wat::core::defn :pe::i64-mod [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::- a (:wat::core::i64::* (:wat::core::i64::/ a b) b)))

;; A `PersistentVector`'s element type is INVARIANT and inferred from its first element, so each
;; `Box` INSTANTIATION must be upcast to `Record` before they can share one bag — that upcast is
;; all `:pe::as-record` does.
(:wat::core::defn :pe::as-record [r <- :wat::core::Record] -> :wat::core::Record r)

;; box-for i — ONE class, THREE erasures, cycling by (i mod 3):
;;   0 -> Box[i64]     packable
;;   1 -> Box[String]  NOT packable
;;   2 -> Box[Tag]     NOT packable, and a DIFFERENT erasure from the String one
;; The cycle puts a packable instance before an erased one AND after it, so neither interleaving
;; is privileged — the occupancy batch runs after the fact loop and order must not matter.
(:wat::core::defn :pe::box-for [i <- :wat::core::i64] -> :wat::core::Record
  (:wat::core::cond
    ((:wat::core::= (:pe::i64-mod i 3) 0) (:pe::as-record (:pe::Box :k i :v i)))
    ((:wat::core::= (:pe::i64-mod i 3) 1) (:pe::as-record (:pe::Box :k i :v (:wat::core::i64::to-string i))))
    (:else                                (:pe::as-record (:pe::Box :k i :v (:pe::Tag :n i))))))

;; seed session items — stage Box(i, <erasure>) and Plain(i) for i in [0, items), in ONE batch
;; `insert-all` (the verb a user should write, so the axis writes it too).
(:wat::core::defn :pe::seed
  [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj
          (:wat::core::PersistentVector/conj acc (:pe::box-for i))
          (:pe::as-record (:pe::Plain :k i))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :pe::hit-codes [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::Vector :wat::core::i64)
    (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:pe::encode 0 (:pe::Hit/k f))))
      (:wat::rete::query fired (:pe::q-hit)))))

(:wat::core::defn :pe::plain-codes [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::Vector :wat::core::i64)
    (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:pe::encode 1 (:pe::PlainHit/k f))))
      (:wat::rete::query fired (:pe::q-plain)))))

(:wat::core::defn :pe::pair-codes [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::Vector :wat::core::i64)
    (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:pe::encode 2 (:pe::Pair/k f))))
      (:wat::rete::query fired (:pe::q-pair)))))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]).
(:wat::core::defn :pe::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector fired — every derived fact (Hit, PlainHit, Pair), canonically encoded and
;; sorted ascending. THE accuracy witness: the full set, not a count.
(:wat::core::defn :pe::derived-vector [fired <- :wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [h   (:pe::hit-codes fired)
                    hp  (:wat::core::into h (:pe::plain-codes fired))
                    all (:wat::core::into hp (:pe::pair-codes fired))]
    (:pe::vec->pvec (:wat::core::sort all))))

;; ns-between t0 t1 — nanoseconds between two Instants (cf. asym-join.wat).
(:wat::core::defn :pe::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
                    staged  (:pe::seed (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector (:pe::r-box) (:pe::r-plain) (:pe::r-pair)) (:wat::core::PersistentVector (:pe::q-hit) (:pe::q-plain) (:pe::q-pair))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))) items)
                    ;; time the NATIVE production verb only (compile + seed are un-timed setup)
                    n0      (:wat::time::now)
                    fired   (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    n1      (:wat::time::now)
                    derived (:pe::derived-vector fired)
                    nat-ns  (:pe::ns-between n0 n1)
                    ;; ORACLE — fired on the SAME staged session. Value semantics make the
                    ;; two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "parametric-erasure" :size (:wat::core::PersistentVector items) :derived derived :native-ns nat-ns :oracle-derived (:pe::derived-vector ofired) :oracle-ns (:pe::ns-between o0 o1)))))

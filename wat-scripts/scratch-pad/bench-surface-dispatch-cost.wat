;; COST PROBE — what does ONE surface dispatch cost? Measured 2026-08-17, two runs:
;;
;;   NONVACUITY  ra=rb=rc=rd=600000     all four arms computed the SAME answer
;;   order A     disp=376ms  direct=219ms
;;   order B     direct=211ms  disp=368ms      <- ordering reversed, same picture
;;   repeat      disp=383/345  direct=216/190
;;
;;   direct      ~209ms / 200k = ~1.05 us/op
;;   dispatched  ~368ms / 200k = ~1.84 us/op
;;   delta       ~795 ns per dispatch, 1.76x
;;
;; ⚠ 795ns is an UPPER BOUND, not the dispatch cost. The dispatched arm has one EXTRA CALL
;; LAYER (Shaped/val -> the extend-type impl) that the direct arm does not. True dispatch cost
;; is lower by one wat fn call — which this bench measures at ~1us, so possibly much lower.
;;
;; ★ WHAT IT MEANS FOR chain-D's Seqable: NOTHING BAD. The design is
;;   (seq [self] :- (wat.type/Seq [T])) — ONE call per COLLECTION, then iterate the result.
;; So the cost is ~795ns once per join/map/filter CALL, not per element. join's 26 live sites
;; are macro-time over 2-5 elements. Negligible.
;;
;; ⛔ IT WOULD MATTER for a per-element first/rest ISeq walk: 795ns x N. At the grid's
;; fanout N=40,000 that is ~32ms; at N=1M, ~0.8s. If anyone later proposes per-element
;; dispatch, THIS is the number to argue with.
;;
;; ★★ READ THIS BEFORE USING THE NUMBER AGAINST A SURFACE DESIGN. Builder, 2026-08-17:
;;   "wat will be byte code compiled.... we are finishing the surface... the surface will be our
;;    expression language for optimized code it produces... interpretted wat has a death sentence
;;    ... we are building towards amazing perf"
;;
;; So ~795ns is a measurement OF THE INTERPRETER, and the interpreter is condemned. Note the
;; DIRECT arm costs ~1.05us for what is a length() call — the baseline here IS interpreter
;; overhead, not dispatch. This number is a fact about today's execution model, NOT a constraint
;; on the surface design.
;;
;; ⛔ DO NOT cite this bench to argue against surfaces, Seqable, or extend-type. The surface IS
;; the expression language the compiler will consume, and ONE polymorphic verb is strictly easier
;; to compile than SEVEN hand-rolled `-stream` twins. Seqable makes the compiler's job smaller.
;;
;; What the number IS good for: bounding a PER-ELEMENT dispatch proposal on today's runtime, and
;; as a before-figure when the compiler lands.
;;
;; Shape discipline (feedback_a_benchmarks_shape_manufactures_its_result): fixed n, BOTH
;; block orderings run to catch ordering artifacts, and a non-vacuity control proving both
;; arms compute the same value. No recalibration inside the run.

;; COST PROBE — what does ONE surface dispatch cost vs a direct call?
(:wat::core::defsurface :bench::Shaped :nature :wat::core::Struct
  :features [(val [self <- :bench::Shaped] -> :wat::core::i64)])

(:wat::core::extend-type :wat::core::Vector :bench::Shaped
  (val [self] -> :wat::core::i64 (:wat::core::length self)))

(:wat::core::defn :bench::direct [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::length v))

(:wat::core::defn :bench::dispatched [s <- :bench::Shaped] -> :wat::core::i64
  (:bench::Shaped/val s))

(:wat::core::defn :bench::loop-direct [n <- :wat::core::i64 v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64 _i <- :wat::core::i64] -> :wat::core::i64
                       (:wat::core::i64::+ acc (:bench::direct v)))
                     0 (:wat::core::range 0 n)))

(:wat::core::defn :bench::loop-disp [n <- :wat::core::i64 v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64 _i <- :wat::core::i64] -> :wat::core::i64
                       (:wat::core::i64::+ acc (:bench::dispatched v)))
                     0 (:wat::core::range 0 n)))

(:wat::core::defn :bench::ns [t0 <- :wat::time::Instant t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n  200000
     v  (:wat::core::Vector :wat::core::i64 1 2 3)
     ;; ── ORDER A: dispatched first, then direct ──
     a0 (:wat::time::now) ra (:bench::loop-disp n v)   a1 (:wat::time::now)
     b0 (:wat::time::now) rb (:bench::loop-direct n v) b1 (:wat::time::now)
     ;; ── ORDER B: direct first, then dispatched (block-ordering control) ──
     c0 (:wat::time::now) rc (:bench::loop-direct n v) c1 (:wat::time::now)
     d0 (:wat::time::now) rd (:bench::loop-disp n v)   d1 (:wat::time::now)]
    (:wat::kernel::println
      (:wat::core::string::interpolate
        "n={n} NONVACUITY ra={ra} rb={rb} rc={rc} rd={rd} | A: disp={ad}ms direct={bd}ms | B: direct={cd}ms disp={dd}ms"
        :n n :ra ra :rb rb :rc rc :rd rd
        :ad (:wat::core::i64::/ (:bench::ns a0 a1) 1000000)
        :bd (:wat::core::i64::/ (:bench::ns b0 b1) 1000000)
        :cd (:wat::core::i64::/ (:bench::ns c0 c1) 1000000)
        :dd (:wat::core::i64::/ (:bench::ns d0 d1) 1000000)))))

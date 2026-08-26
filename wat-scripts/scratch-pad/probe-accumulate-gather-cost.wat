;; probe-accumulate-gather-cost.wat — IS THE ACCUMULATE PASS'S GATHER QUADRATIC IN THE GROUP COUNT?
;;
;; THE QUESTION. The Clara grid re-measure (2026-07-30) inverted three axes: `accum` is a DECISIVE
;; LOSS (Clara ~19x) and `min-finding` a loss (~6x), both accumulator-family, and `negation` is a
;; coin flip — all three recorded as wins. Reading the kernel (NOT asserting) gives a candidate
;; mechanism, grounded to file:line:
;;
;;   src/rete/kernel.rs:1939-1952 — the accumulate-pass:
;;     let from_elements = wm.alpha[from_alpha_id].clone();       // the FULL cumulative alpha
;;     for tok in new_tokens {                                    // every new parent token
;;         let gathered = from_elements.iter().filter(|el| token_element_compatible(..)).collect();
;;
;; That is O(|new_tokens| x |from_elements|) per round, with NO index. The same shape is in the
;; filter-pass for NegationNode / ExistsNode (`filter_elements` snapshot + a per-token `.any()`),
;; so one defect class covers all three losing axes.
;;
;; THE ASYMMETRY THAT MAKES IT SUSPICIOUS. The JOIN nodes ARE keyed — `left_idx`/`right_idx` are
;; `HashMap<Vec<Value>, Vec<Token>>` with a cached `join_keys` and `key_of` (kernel.rs:1542-1544,
;; :1203; P6 delivered them). The Accumulate/Negation/Exists gathers never got the same treatment.
;; This is R24's `merge_facts` shape again — a linear scan where a hash lookup belongs.
;;
;; ★ THE CONTROL (the load-bearing half — this probe exists FOR it, and it can KILL the diagnosis).
;; A plain size ladder cannot distinguish "the gather is quadratic" from "there are simply more
;; facts." So the decisive sweep holds the TOTAL FACT COUNT CONSTANT and varies only the group
;; count. At N = G*W fixed:
;;
;;     gather cost  ~ G tokens x (N-G) elements  ~ G*N   -> GROWS LINEARLY IN G
;;     keyed cost   ~ O(N) regardless of G                -> FLAT IN G
;;     seeding      ~ O(N)                                -> FLAT IN G
;;     joins (keyed) ~ O(G)                               -> mild, linear, small constant
;;
;; So: run [50 160] [100 80] [200 40] [400 20] — every one is N=8000 facts. If `fire-ns` climbs
;; roughly 8x across that 8x rise in G, the per-token scan is CONFIRMED as the mechanism. If
;; `fire-ns` stays FLAT, the diagnosis is WRONG — the cost is elsewhere and no one should touch
;; `kernel.rs` on my say-so. This probe is drawn so that it can come back and say I was wrong.
;;
;; The scale sweep ([50 20] [100 20] [200 20] [400 20], W fixed) is the secondary reading: it
;; should show super-linear growth (~4x per doubling of G if O(G^2 * W)), but it CANNOT by itself
;; distinguish the mechanism — read the CONTROL for that.
;;
;; WHY PHASE-SPLIT AT ALL. The grid times `fire-rules` and nothing else, and on the node-share axis
;; that was 1.4% of the process — a verdict about the fastest fiftieth of the work. Before trusting
;; ANY fire number on this axis, we need its share. So all five phases are timed here and the axis
;; itself is left untouched.
;;
;; NON-VACUITY. `derived-count` MUST equal `expected-count` (= 5*G: every group emits CountF, SumF,
;; MinF, MaxF, ExistsF, since W>=1 always). A gather that short-circuited would look cheap AND drop
;; its count, so a fast number with a WRONG count means the run is void, not fast. Both fields are
;; emitted rather than asserted, so that a genuine token-dropping finding stays visible instead of
;; being fail-closed into a probe error.
;;
;; FAITHFUL TO THE AXIS: the records, the five rules, `val`, `enc`, `seed`, and the derive chain are
;; copied from wat-scripts/perf/grid/accum.wat byte-for-byte modulo the namespace (:acp:: vs :acc::,
;; since the loader gate loads both files and the names would collide). If the axis drifts, this
;; probe stops describing it.
;;
;; ⛔ SAFETY — READ BEFORE RUNNING. A timeout is NOT a resource bound. A workstation died at an
;; unvalidated size on this grid. Run EVERY invocation under the memory guard and climb the ladder
;; UPWARD, smallest first:
;;   systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0 -- \
;;     timeout 120 ./target/release/wat wat-scripts/scratch-pad/probe-accumulate-gather-cost.wat
;; No forks, no services, single-threaded.
;;
;; stdin : [groups readings]
;; stdout: one #acp/Split EDN line
;;   echo '[50 20]' | ./target/release/wat wat-scripts/scratch-pad/probe-accumulate-gather-cost.wat

(:wat::core::defrecord :acp::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :acp::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :acp::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :acp::SumF    [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :acp::MinF    [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :acp::MaxF    [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :acp::ExistsF [g <- :wat::core::i64])

;; Split — the per-phase census. `derived-count` vs `expected-count` is the non-vacuity witness.
(:wat::core::defrecord :acp::Split
  [groups         <- :wat::core::i64
   readings       <- :wat::core::i64
   facts          <- :wat::core::i64   ;; G*(W+1) — the constant held across the CONTROL sweep
   build-ns       <- :wat::core::i64
   compile-ns     <- :wat::core::i64
   seed-ns        <- :wat::core::i64
   fire-ns        <- :wat::core::i64
   derive-ns      <- :wat::core::i64
   derived-count  <- :wat::core::i64
   expected-count <- :wat::core::i64])

;; ─── copied from grid/accum.wat (namespace changed only) ─────────────────────

(:wat::rete::defrule :acp::count-rule
  :when
  [(:acp::Group (?g <- :g))
   (?n <- (:wat::rete::acc::count) :from (:acp::Reading (?g <- :g)))]
  :then
  [(:acp::CountF ?g ?n)])

(:wat::rete::defrule :acp::sum-rule
  :when
  [(:acp::Group (?g <- :g))
   (?n <- (:wat::rete::acc::sum ?v) :from (:acp::Reading (?g <- :g) (?v <- :v)))]
  :then
  [(:acp::SumF ?g ?n)])

(:wat::rete::defrule :acp::min-rule
  :when
  [(:acp::Group (?g <- :g))
   (?n <- (:wat::rete::acc::min ?v) :from (:acp::Reading (?g <- :g) (?v <- :v)))]
  :then
  [(:acp::MinF ?g ?n)])

(:wat::rete::defrule :acp::max-rule
  :when
  [(:acp::Group (?g <- :g))
   (?n <- (:wat::rete::acc::max ?v) :from (:acp::Reading (?g <- :g) (?v <- :v)))]
  :then
  [(:acp::MaxF ?g ?n)])

(:wat::rete::defrule :acp::exists-rule
  :when
  [(:acp::Group (?g <- :g))
   (:wat::rete::exists (:acp::Reading (?g <- :g)))]
  :then
  [(:acp::ExistsF ?g)])

(:wat::rete::defquery :acp::q-CountF
  :params []
  :when [(?fact <- :acp::CountF)])


(:wat::rete::defquery :acp::q-SumF
  :params []
  :when [(?fact <- :acp::SumF)])


(:wat::rete::defquery :acp::q-MinF
  :params []
  :when [(?fact <- :acp::MinF)])


(:wat::rete::defquery :acp::q-MaxF
  :params []
  :when [(?fact <- :acp::MaxF)])


(:wat::rete::defquery :acp::q-ExistsF
  :params []
  :when [(?fact <- :acp::ExistsF)])


(:wat::core::defn :acp::val [g <- :wat::core::i64  j <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [x (:wat::i64::+ (:wat::i64::* g 31) (:wat::i64::* j 17))]
    (:wat::i64::- x (:wat::i64::* (:wat::i64::/ x 1000) 1000))))

(:wat::core::defn :acp::enc [kind <- :wat::core::i64  g <- :wat::core::i64  val <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::i64::+
    (:wat::i64::+ (:wat::i64::* kind 1000000000000000) (:wat::i64::* g 1000000000))
    val))

(:wat::core::defn :acp::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])  x <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:wat::core::i64])
      (:wat::core::PersistentVector/conj acc x))
    (:wat::core::PersistentVector)
    v))

(:wat::core::defn :acp::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  W <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert s (:acp::Reading :g g :v (:acp::val g j))))
    session
    (:wat::core::range 0 W)))

(:wat::core::defn :acp::seed [session <- :wat::rete::Session  G <- :wat::core::i64  W <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session
      (:acp::seed-readings (:wat::rete::insert s (:acp::Group g)) g W))
    session
    (:wat::core::range 0 G)))

(:wat::core::defn :acp::codes [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let
    [c0 (:wat::core::into (:wat::core::Vector :wat::core::i64)
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:acp::enc 0 (:acp::CountF/g f) (:acp::CountF/n f))))
            (:wat::rete::query fired (:acp::q-CountF))))
     c1 (:wat::core::into c0
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:acp::enc 1 (:acp::SumF/g f) (:acp::SumF/n f))))
            (:wat::rete::query fired (:acp::q-SumF))))
     c2 (:wat::core::into c1
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:acp::enc 2 (:acp::MinF/g f) (:acp::MinF/n f))))
            (:wat::rete::query fired (:acp::q-MinF))))
     c3 (:wat::core::into c2
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:acp::enc 3 (:acp::MaxF/g f) (:acp::MaxF/n f))))
            (:wat::rete::query fired (:acp::q-MaxF))))
     c4 (:wat::core::into c3
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:acp::enc 4 (:acp::ExistsF/g f) 0)))
            (:wat::rete::query fired (:acp::q-ExistsF))))]
    c4))

(:wat::core::defn :acp::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:acp::vec->pvec (:wat::core::sort (:acp::codes fired))))

(:wat::core::defn :acp::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; ─── main — five instants, four phases, nothing else between them ────────────
;;
;; The `let` is strict and sequential, so each binding completes before the next instant is taken;
;; no phase can leak into a neighbour's window. Process startup and the final println are
;; deliberately unmeasured — the question is the split BETWEEN phases and how `fire-ns` moves
;; under the CONTROL sweep.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln )
                              ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                              (:wat::kernel::ReadlnOutcome::Eof
                                (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                              (:wat::kernel::ReadlnOutcome::Stopped
                                (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    groups  (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [groups readings]")
                    reads   (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [groups readings]")

                    t0      (:wat::time::now)
                    rules   (:wat::rete::collect-rules :acp)
                    t1      (:wat::time::now)
                    session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:acp::q-CountF) (:acp::q-SumF) (:acp::q-MinF) (:acp::q-MaxF) (:acp::q-ExistsF)))
                    t2      (:wat::time::now)
                    staged  (:acp::seed session groups reads)
                    t3      (:wat::time::now)
                    fired   (:wat::rete::fire-rules staged)
                    t4      (:wat::time::now)
                    derived (:acp::derived-vector fired)
                    t5      (:wat::time::now)]
    (:wat::kernel::println
      (:acp::Split
        :groups         groups
        :readings       reads
        :facts          (:wat::i64::* groups (:wat::i64::+ reads 1))
        :build-ns       (:acp::ns-between t0 t1)
        :compile-ns     (:acp::ns-between t1 t2)
        :seed-ns        (:acp::ns-between t2 t3)
        :fire-ns        (:acp::ns-between t3 t4)
        :derive-ns      (:acp::ns-between t4 t5)
        :derived-count  (:wat::core::length derived)
        :expected-count (:wat::i64::* groups 5)))))

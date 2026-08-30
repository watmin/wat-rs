;; Fixture BESIDE probe_arc278_fallback_generic_ret.rs.
;;
;; THE CONTRACT: a fallback-carrying op takes its `:undefined` value only at ITS OWN
;; undefined point. Whether a non-finite f64 IS that point is decided by the ROW's
;; DECLARED `ret`, never by sniffing the runtime value's type. `runtime.rs`'s
;; canonical copy says why: "a value-sniff would silently change behaviour for any
;; future row that happens to return a float for a non-arithmetic reason."
;;
;; THE ROW ALREADY EXISTED. `PersistentVector/first` is Fallback-class with
;; `ret: Var("T")` — six such generic rows (`get`/`first` over the three sequence
;; types) sit in the same table as the f64 arithmetic family.
;;
;; MEASURED 2026-08-24, on this exact program:
;;   native, sniffing the value   -> 1   (fallback -1.0 taken; the rule FIRED)
;;   the $oracle                  -> 0
;;   native, guarding on ret      -> 0   (the element +Inf returned)
;; So the fast path disagreed with the engine's own definition of correct, and no
;; fixture in the corpus could show it: `where-*` covers the f64 arithmetic family,
;; where sniff and guard happen to agree.
;;
;; `first` returning -1.0 where the element is +Inf is not a rounding difference. It
;; is the WRONG ELEMENT, silently, from a total op whose whole purpose is predictability.

(:wat::core::defrecord :fgr::Item [k <- :wat::core::i64  vs <- (:wat::core::PersistentVector :- [:wat::core::f64])])
(:wat::core::defrecord :fgr::Hit  [k <- :wat::core::i64])

;; Fires iff first(vs, :undefined -1.0) < 0. The head is +Inf, so it must NOT fire:
;; +Inf is the element, not an undefined point, because `first` never promised a float.
(:wat::rete::defrule :fgr::r
  :when [(:fgr::Item (?k <- :k) (?vs <- :vs))
         (:wat::rete::where
           (:wat::rete::core::f64::< (:wat::rete::core::PersistentVector/first ?vs :undefined -1.0) 0.0))]
  :then [(:fgr::Hit :k ?k)])

(:wat::rete::defquery :fgr::q :params [] :when [(?fact <- :fgr::Hit)])

(:wat::core::defn :fgr::staged [] -> :wat::rete::Session
  (:wat::core::let [inf (:wat::core::f64::/ 1.0 0.0)]
    (:wat::core::match (:wat::rete::insert-all
      (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :fgr)
                               (:wat::core::PersistentVector (:fgr::q))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
      (:wat::core::PersistentVector (:fgr::Item :k 1 :vs (:wat::core::PersistentVector inf)))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))))

;; [native-hits, oracle-hits] — both must be 0, and they must agree.
(:wat::core::defn :user::native-and-oracle [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::mapv
    (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
    (:wat::core::PersistentVector
      (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules (:fgr::staged)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:fgr::q)))
      (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules$oracle (:fgr::staged)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:fgr::q))))))

;; Arc 278 — an accumulate result is SUPERSEDED, not extended, when its source grows.
;;
;; Three shapes of the SAME rule set, differing only in how many times the accumulate's
;; count changes during the fixpoint. Each answers [tally-rows, sum-of-n] under native,
;; then the same under $oracle. Clara 0.24.0 is the authority (see the .rs header).
;;
;;   shape   count goes   Clara says
;;   ─────   ──────────   ──────────────────────────────
;;   empty   0            one Tally, n=0   → [1 0]
;;   one     0→1          one Tally, n=1   → [1 1]
;;   two     0→1→2        one Tally, n=2   → [1 2]
;;
;; `sum-of-n` is the discriminator that a bare row count cannot be: the pre-fix oracle
;; answered [3 3] on `two` (n=0 + n=1 + n=2), and an over-correction into "never emit"
;; would answer [0 0] on `empty` while looking perfect on `one` and `two`.

;; ─── shape EMPTY: nothing ever derives Out, so the count is 0 forever ────────────────
(:wat::core::defrecord :oas0::Seed  [y <- :wat::core::i64])
(:wat::core::defrecord :oas0::Out   [y <- :wat::core::i64])
(:wat::core::defrecord :oas0::Tally [n <- :wat::core::i64])

(:wat::rete::defrule :oas0::tally
  :when [(:oas0::Seed (?s :- :y))
         (?n :- (:wat::rete::acc::count) :from (:oas0::Out))]
  :then [(:oas0::Tally :n ?n)])

(:wat::rete::defquery :oas0::q :params [] :when [(?f :- :oas0::Tally)])

(:wat::core::defn :oas0::readback [s <- :wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [rows (:wat::rete::query s (:oas0::q))]
    (:wat::core::PersistentVector
      (:wat::core::length rows)
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64  r <- :wat::core::PersistentMap] -> :wat::core::i64
          (:wat::i64::+ acc
            (:oas0::Tally/n
              (:wat::core::Option/expect (:wat::map::get r "?f") "?f"))))
        0
        rows))))

(:wat::core::defn :oas0::staged [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
      (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :oas0)
          (:wat::core::PersistentVector (:oas0::q)))
        [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
        [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f}
          (:wat::kernel::assertion-failed! :message "oas0: the rule set may not terminate")])
      (:oas0::Seed :y 1))
    [:wat::rete::InsertOutcome.Inserted {:session __st} __st]
    [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c}
      (:wat::kernel::assertion-failed! :message "oas0: session memory ceiling exceeded while staging")]))

;; ─── shape ONE: Seed → Out, so the count goes 0 → 1 ──────────────────────────────────
(:wat::core::defrecord :oas1::Seed  [y <- :wat::core::i64])
(:wat::core::defrecord :oas1::Out   [y <- :wat::core::i64])
(:wat::core::defrecord :oas1::Tally [n <- :wat::core::i64])

(:wat::rete::defrule :oas1::a
  :when [(:oas1::Seed (?y :- :y))]
  :then [(:oas1::Out :y ?y)])

(:wat::rete::defrule :oas1::tally
  :when [(:oas1::Seed (?s :- :y))
         (?n :- (:wat::rete::acc::count) :from (:oas1::Out))]
  :then [(:oas1::Tally :n ?n)])

(:wat::rete::defquery :oas1::q :params [] :when [(?f :- :oas1::Tally)])

(:wat::core::defn :oas1::readback [s <- :wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [rows (:wat::rete::query s (:oas1::q))]
    (:wat::core::PersistentVector
      (:wat::core::length rows)
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64  r <- :wat::core::PersistentMap] -> :wat::core::i64
          (:wat::i64::+ acc
            (:oas1::Tally/n
              (:wat::core::Option/expect (:wat::map::get r "?f") "?f"))))
        0
        rows))))

(:wat::core::defn :oas1::staged [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
      (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :oas1)
          (:wat::core::PersistentVector (:oas1::q)))
        [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
        [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f}
          (:wat::kernel::assertion-failed! :message "oas1: the rule set may not terminate")])
      (:oas1::Seed :y 1))
    [:wat::rete::InsertOutcome.Inserted {:session __st} __st]
    [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c}
      (:wat::kernel::assertion-failed! :message "oas1: session memory ceiling exceeded while staging")]))

;; ─── shape TWO: Seed → Out(1) → D(2) → Out(2), so the count goes 0 → 1 → 2 ───────────
(:wat::core::defrecord :oas2::Seed  [y <- :wat::core::i64])
(:wat::core::defrecord :oas2::Out   [y <- :wat::core::i64])
(:wat::core::defrecord :oas2::D     [y <- :wat::core::i64])
(:wat::core::defrecord :oas2::Tally [n <- :wat::core::i64])

(:wat::rete::defrule :oas2::a
  :when [(:oas2::Seed (?y :- :y))]
  :then [(:oas2::Out :y ?y)])

(:wat::rete::defrule :oas2::b
  :when [(:oas2::Out (?y :- :y))
         (:wat::rete::where (:wat::rete::i64::= ?y 1))]
  :then [(:oas2::D :y 2)])

(:wat::rete::defrule :oas2::c
  :when [(:oas2::D (?y :- :y))]
  :then [(:oas2::Out :y ?y)])

(:wat::rete::defrule :oas2::tally
  :when [(:oas2::Seed (?s :- :y))
         (?n :- (:wat::rete::acc::count) :from (:oas2::Out))]
  :then [(:oas2::Tally :n ?n)])

(:wat::rete::defquery :oas2::q :params [] :when [(?f :- :oas2::Tally)])

(:wat::core::defn :oas2::readback [s <- :wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [rows (:wat::rete::query s (:oas2::q))]
    (:wat::core::PersistentVector
      (:wat::core::length rows)
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64  r <- :wat::core::PersistentMap] -> :wat::core::i64
          (:wat::i64::+ acc
            (:oas2::Tally/n
              (:wat::core::Option/expect (:wat::map::get r "?f") "?f"))))
        0
        rows))))

(:wat::core::defn :oas2::staged [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
      (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :oas2)
          (:wat::core::PersistentVector (:oas2::q)))
        [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
        [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f}
          (:wat::kernel::assertion-failed! :message "oas2: the rule set may not terminate")])
      (:oas2::Seed :y 1))
    [:wat::rete::InsertOutcome.Inserted {:session __st} __st]
    [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c}
      (:wat::kernel::assertion-failed! :message "oas2: session memory ceiling exceeded while staging")]))

;; ─── the witness ─────────────────────────────────────────────────────────────────────
;; [empty-rows empty-sum one-rows one-sum two-rows two-sum] under NATIVE,
;; then the same six under $ORACLE. Expect [1 0 1 1 1 2] twice.
(:wat::core::defn :user::native-and-oracle [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::mapv
    (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
    (:wat::vector::concat
      (:wat::vector::concat
        (:wat::vector::concat
          (:oas0::readback (:wat::core::match (:wat::rete::fire-rules (:oas0::staged))
            [:wat::rete::FireOutcome.Fired {:value __f} __f]
            [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r}
              (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")]
            [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
              (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")]))
          (:oas1::readback (:wat::core::match (:wat::rete::fire-rules (:oas1::staged))
            [:wat::rete::FireOutcome.Fired {:value __f} __f]
            [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r}
              (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")]
            [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
              (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
        (:oas2::readback (:wat::core::match (:wat::rete::fire-rules (:oas2::staged))
          [:wat::rete::FireOutcome.Fired {:value __f} __f]
          [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r}
            (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")]
          [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
            (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
      (:wat::vector::concat
        (:wat::vector::concat
          (:oas0::readback (:wat::core::match (:wat::rete::fire-rules$oracle (:oas0::staged))
            [:wat::rete::FireOutcome.Fired {:value __f} __f]
            [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r}
              (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")]
            [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
              (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")]))
          (:oas1::readback (:wat::core::match (:wat::rete::fire-rules$oracle (:oas1::staged))
            [:wat::rete::FireOutcome.Fired {:value __f} __f]
            [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r}
              (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")]
            [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
              (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
        (:oas2::readback (:wat::core::match (:wat::rete::fire-rules$oracle (:oas2::staged))
          [:wat::rete::FireOutcome.Fired {:value __f} __f]
          [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r}
            (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")]
          [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
            (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")]))))))

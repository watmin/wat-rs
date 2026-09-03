;; D7 negative controls — the packability angles that do NOT collide, and why.
;;
;; ANGLE 2 (field count vs `I64_ROW_CAP` = 8): field COUNT is a property of the
;; CLASS, so it cannot vary between facts of one class. A 9-field class is
;; UNIFORMLY unpackable, `class_ids[class]` stays empty, `ids.is_empty()` skips
;; the batch (pass/alpha.rs:119), and writer 2 never runs — so writer 1's pushes
;; survive. A 0-field class is the same story via `fields.is_empty()`.
;;
;; The only way angle 2 could collide is if one `aid` were reachable under two
;; classes of different width — and it is not: `build_alpha_index` (arm.rs:333)
;; files each alpha node id under exactly ONE `pat.type_head`.
;;
;; EXPECTED: wide=3 (all three 9-field facts derive), narrow=3.

(:wat::core::defrecord :d7w::Wide
  [a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64
   d <- :wat::core::i64  e <- :wat::core::i64  f <- :wat::core::i64
   g <- :wat::core::i64  h <- :wat::core::i64  i <- :wat::core::i64])
(:wat::core::defrecord :d7w::Narrow [k <- :wat::core::i64])
(:wat::core::defrecord :d7w::WideHit   [k <- :wat::core::i64])
(:wat::core::defrecord :d7w::NarrowHit [k <- :wat::core::i64])

(:wat::rete::defrule :d7w::rw
  :when [(:d7w::Wide (?a <- :a) (?b <- :b) (?c <- :c) (?d <- :d) (?e <- :e)
                     (?f <- :f) (?g <- :g) (?h <- :h) (?i <- :i))]
  :then [(:d7w::WideHit ?a)])

(:wat::rete::defrule :d7w::rn
  :when [(:d7w::Narrow (?k <- :k))]
  :then [(:d7w::NarrowHit ?k)])

(:wat::rete::defquery :d7w::qw :params [] :when [(?fact <- :d7w::WideHit)])
(:wat::rete::defquery :d7w::qn :params [] :when [(?fact <- :d7w::NarrowHit)])

(:wat::core::defn :d7w::as-record [r <- :wat::core::Record] -> :wat::core::Record r)

(:wat::core::defn :d7w::wide [k <- :wat::core::i64] -> :wat::core::Record
  (:d7w::as-record (:d7w::Wide :a k :b 1 :c 2 :d 3 :e 4 :f 5 :g 6 :h 7 :i 8)))

(:wat::core::defn :d7w::facts [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector
    (:d7w::wide 0) (:d7w::wide 1) (:d7w::wide 2)
    (:d7w::as-record (:d7w::Narrow :k 0))
    (:d7w::as-record (:d7w::Narrow :k 1))
    (:d7w::as-record (:d7w::Narrow :k 2))))

(:wat::core::defn :d7w::count
  [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::i64
  (:wat::core::Vector/length
    (:wat::core::into (:wat::core::Vector :wat::core::PersistentMap)
      (:wat::rete::query s q))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all
           (:wat::core::PersistentVector (:d7w::rw) (:d7w::rn))
           (:wat::core::PersistentVector (:d7w::qw) (:d7w::qn)))
           ((:wat::rete::CompileOutcome::Compiled __s) __s)
           ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
             (:wat::kernel::assertion-failed! "compile" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert-all s0 (:d7w::facts))
           ((:wat::rete::InsertOutcome::Inserted __s) __s)
           ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c)
             (:wat::kernel::assertion-failed! "insert" :wat::core::None :wat::core::None)))
     fired (:wat::core::match (:wat::rete::fire-rules s1)
           ((:wat::rete::FireOutcome::Fired __f) __f)
           ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
             (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None))
           ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
             (:wat::kernel::assertion-failed! "cap" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println
      (:wat::core::String/concat
        (:wat::core::String/concat "wide=" (:wat::core::i64::to-string (:d7w::count fired (:d7w::qw))))
        (:wat::core::String/concat " narrow=" (:wat::core::i64::to-string (:d7w::count fired (:d7w::qn))))))))

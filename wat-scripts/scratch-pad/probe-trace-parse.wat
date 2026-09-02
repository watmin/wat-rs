;; Parse seq|t0|t1|t2|t3|t4 and bucket one interval. Proves edn::read of i64::to-string
;; and the histogram line shape before the circuit pays 8000 of them.
(:wat::core::defrecord :probe::Hist
  [c0 <- :wat::core::i64
   c1 <- :wat::core::i64
   c2 <- :wat::core::i64
   c3 <- :wat::core::i64
   c4 <- :wat::core::i64
   c5 <- :wat::core::i64
   mx <- :wat::core::i64])

(:wat::core::defn :probe::parse-i64 [s <- :wat::core::String] -> :wat::core::i64
  (:wat::edn::read s))

(:wat::core::defn :probe::hist-add
  [h <- :probe::Hist  dt-ms <- :wat::core::i64]
  -> :probe::Hist
  (:wat::core::let
    [dt (:wat::core::if (:wat::i64::< dt-ms 0) 0 dt-ms)
     mx (:wat::core::if (:wat::i64::> dt (:probe::Hist/mx h)) dt (:probe::Hist/mx h))]
    (:wat::core::if (:wat::i64::< dt 1)
      (:probe::Hist :c0 (:wat::i64::+ (:probe::Hist/c0 h) 1) :c1 (:probe::Hist/c1 h) :c2 (:probe::Hist/c2 h) :c3 (:probe::Hist/c3 h) :c4 (:probe::Hist/c4 h) :c5 (:probe::Hist/c5 h) :mx mx)
      (:wat::core::if (:wat::i64::< dt 10)
        (:probe::Hist :c0 (:probe::Hist/c0 h) :c1 (:wat::i64::+ (:probe::Hist/c1 h) 1) :c2 (:probe::Hist/c2 h) :c3 (:probe::Hist/c3 h) :c4 (:probe::Hist/c4 h) :c5 (:probe::Hist/c5 h) :mx mx)
        (:wat::core::if (:wat::i64::< dt 50)
          (:probe::Hist :c0 (:probe::Hist/c0 h) :c1 (:probe::Hist/c1 h) :c2 (:wat::i64::+ (:probe::Hist/c2 h) 1) :c3 (:probe::Hist/c3 h) :c4 (:probe::Hist/c4 h) :c5 (:probe::Hist/c5 h) :mx mx)
          (:wat::core::if (:wat::i64::< dt 250)
            (:probe::Hist :c0 (:probe::Hist/c0 h) :c1 (:probe::Hist/c1 h) :c2 (:probe::Hist/c2 h) :c3 (:wat::i64::+ (:probe::Hist/c3 h) 1) :c4 (:probe::Hist/c4 h) :c5 (:probe::Hist/c5 h) :mx mx)
            (:wat::core::if (:wat::i64::< dt 1000)
              (:probe::Hist :c0 (:probe::Hist/c0 h) :c1 (:probe::Hist/c1 h) :c2 (:probe::Hist/c2 h) :c3 (:probe::Hist/c3 h) :c4 (:wat::i64::+ (:probe::Hist/c4 h) 1) :c5 (:probe::Hist/c5 h) :mx mx)
              (:probe::Hist :c0 (:probe::Hist/c0 h) :c1 (:probe::Hist/c1 h) :c2 (:probe::Hist/c2 h) :c3 (:probe::Hist/c3 h) :c4 (:probe::Hist/c4 h) :c5 (:wat::i64::+ (:probe::Hist/c5 h) 1) :mx mx))))))))

(:wat::core::defn :probe::hist-line [name <- :wat::core::String  h <- :probe::Hist] -> :wat::core::String
  (:wat::core::format
    "{name} <1ms={c0} 1-10={c1} 10-50={c2} 50-250={c3} 250-1000={c4} >1000={c5} max={mx}ms"
    :name name
    :c0 (:probe::Hist/c0 h) :c1 (:probe::Hist/c1 h) :c2 (:probe::Hist/c2 h)
    :c3 (:probe::Hist/c3 h) :c4 (:probe::Hist/c4 h) :c5 (:probe::Hist/c5 h)
    :mx (:probe::Hist/mx h)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [t0 1000000000
     t1 1001000000
     t2 1002000000
     t3 1003000000
     t4 1253000000
     body (:wat::core::format "{seq}|{t0}|{t1}|{t2}|{t3}|{t4}"
            :seq 7 :t0 t0 :t1 t1 :t2 t2 :t3 t3 :t4 t4)
     parts (:wat::string::split body "|")
     n (:wat::core::count parts)
     seq0 (:wat::core::nth parts 0)
     u0 (:probe::parse-i64 (:wat::core::nth parts 1))
     u1 (:probe::parse-i64 (:wat::core::nth parts 2))
     u4 (:probe::parse-i64 (:wat::core::nth parts 5))
     empty (:probe::Hist :c0 0 :c1 0 :c2 0 :c3 0 :c4 0 :c5 0 :mx 0)
     h1 (:probe::hist-add empty (:wat::i64::/ (:wat::i64::- u1 u0) 1000000))
     h2 (:probe::hist-add h1 1)
     h3 (:probe::hist-add h2 300)
     h4 (:probe::hist-add h3 2000)]
    (:wat::kernel::println
      (:wat::core::format "n={n};seq={seq};body={body}"
        :n n :seq seq0 :body body))
    (:wat::kernel::println (:probe::hist-line "t3->t4" h4))))

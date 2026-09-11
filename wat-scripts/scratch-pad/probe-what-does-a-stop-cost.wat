;; probe-what-does-a-stop-cost.wat — why is one worker/stop ~100x one Queue/stats?
;;
;; MEASURED, 1196720c3: at negligible payload a `<svc>/stop` on a process costs 250-520 ms,
;; while a `Queue/stats` round-trip on the same kind of process peer costs ~2.8 ms (calibrated
;; via `arm` at 202763705). Both are one `send` + one `recv`. wat/service.wat:2928-2968 shows
;; the generated stop has NO timeout, NO poll and NO reap in it. Nothing explains the gap.
;;
;; THIS PROBE ISOLATES IT. The state here is ONE i64, so payload is provably negligible --
;; whatever it measures is not serialization of a big outbox.
;;
;; Two controls, both free:
;;   THREAD vs PROCESS -- identical but for the locus token. If thread-stop is fast and
;;     process-stop is slow, the cost is the process boundary, not the service machinery.
;;   START vs STOP     -- spawn is already known to be ~500 ms/process (a cold-boot cost the
;;     builder has ruled out of scope). If stop matches start, teardown is the mirror of boot.
;;
;; It asserts nothing; the disk decides. Shape copied from wat-tests/service-stop-resp.wat.

(:wat::core::defsurface :probe::Ctr :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Ctr::BumpRequest [n <- :wat::core::i64])
   (:wat::core::defenum :probe::Ctr::BumpResponse :wat::enum::Pure
     :Ok               [value <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(bump [self <- :probe::Ctr  req <- :probe::Ctr::BumpRequest] -> :probe::Ctr::BumpResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::ctr
  :satisfies :probe::Ctr
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(bump [s ctx req]
     (:wat::core::let [c (:wat::i64::+
                           (:probe::ctr::Record/count (:probe::ctr::State/durable s))
                           (:probe::Ctr::BumpRequest/n req))]
       (:wat::service::Outcome::Continue
         (:probe::ctr::State :durable (:probe::ctr::Record :count c))
         (:wat::core::Some (:probe::Ctr::Reply::Bump (:probe::Ctr::BumpResponse::Ok c)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Ctr::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::ctr::Op])]))))]
  :stop (:wat::core::fn [s <- :probe::ctr::State] -> :wat::core::i64
          (:probe::ctr::Record/count (:probe::ctr::State/durable s))))

(:wat::core::defn :probe::ms [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::- b a) 1000000))

;; One spawn+stop on a THREAD, returning (start-ms stop-ms).
(:wat::core::defn :probe::once-thread [] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     h  (:probe::ctr/start :locus (:wat::spawn::thread) :record (:probe::ctr::Record :count 0))
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     _f (:wat::service::stop-faced (:probe::ctr/stop h))
     t2 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::core::Tuple (:probe::ms t0 t1) (:probe::ms t1 t2))))

;; IDENTICAL except the locus token.
(:wat::core::defn :probe::once-process [] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     h  (:probe::ctr/start :locus (:wat::spawn::process) :record (:probe::ctr::Record :count 0))
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     _f (:wat::service::stop-faced (:probe::ctr/stop h))
     t2 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::core::Tuple (:probe::ms t0 t1) (:probe::ms t1 t2))))

(:wat::core::defn :probe::line [tag <- :wat::core::String  p <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::format "{t} start={s}ms stop={p}ms"
      :t tag :s (:wat::core::first p) :p (:wat::core::second p))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::core::run! (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::nil
                           (:probe::line "thread " (:probe::once-thread)))
         (:wat::core::range 0 5))
     _ (:wat::core::run! (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::nil
                           (:probe::line "process" (:probe::once-process)))
         (:wat::core::range 0 5))]
    nil))

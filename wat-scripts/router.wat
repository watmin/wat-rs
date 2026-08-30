;; wat-scripts/router.wat — pipeline stage 1.
;;
;; Reads `:demo::Event {:n :i64}` frames from stdin. For each event
;; with n > 0 (a "hit"), emits a `:demo::Hit {:n :i64}` frame to
;; stdout. Drops n <= 0 ("miss") events.
;;
;; Usage:
;;   cat wat-scripts/events.edn \
;;     | cargo wat ./wat-scripts/router.wat \
;;     | cargo wat ./wat-scripts/aggregator.wat \
;;     | cargo wat ./wat-scripts/sink.wat

(:wat::core::defstruct :demo::Event
  [n <- :wat::core::i64])

(:wat::core::defstruct :demo::Hit
  [n <- :wat::core::i64])


(:wat::core::defn :demo::router::loop [] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::readln)
    (:wat::kernel::ReadlnOutcome::Eof nil)
    (:wat::kernel::ReadlnOutcome::Stopped nil)
    ((:wat::kernel::ReadlnOutcome::Datum event)
      (:wat::core::let [n (:demo::Event/n event)]
        (:wat::core::if (:wat::i64::> n 0)
          (:wat::kernel::println (:demo::Hit :n n))
          nil)
        (:demo::router::loop)))))


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:demo::router::loop))

;; examples/interrogate/wat/main.wat — arc 093 worked example.
;;
;; A self-contained pry/gdb-style interrogation script. The `:user::main`
;; entry runs three phases:
;;
;;   1) Write — auto-deleting TempFile-backed telemetry sink
;;      (arc 093 slice 1e infrastructure); 6 sample Event::Log rows
;;      whose `data` column carries `:demo::Trade` struct values
;;      lifted via :wat::core::struct->form.
;;
;;   2) Read — `:wat::sqlite::open-readonly` on the same path; the
;;      writer driver has already joined so the file is fully flushed.
;;
;;   3) Interrogate — two queries demonstrate the full circuit:
;;
;;      Q1 (warmup): stream all logs, count them.
;;
;;      Q2: filter via :wat::form::matches? (arc 098) on the
;;          lifted Value::Struct (via Event::Log/data-value, slice 3) —
;;          return trades whose side is "buy" AND qty > 10.
;;
;; The whole flow is the arc 093 DESIGN's pry/gdb framing made
;; concrete: SQL narrows by time (we use no constraint here for
;; brevity); Stream<Event::Log> drives lock-step bounded(1)
;; channels through filter/for-each; matches? does the Clara
;; predicate over the lifted struct value.

;; ─── Domain — the synthetic struct logged in the data column ────

(:wat::core::struct :demo::Trade
  (side  :String)   ; "buy" or "sell"
  (qty   :i64)
  (price :f64))

;; ─── Phase 1 — write fixture ────────────────────────────────────

;; Build one Event::Log carrying a :demo::Trade as its data.
;; struct->form lifts the runtime Value::Struct to the WatAST
;; that struct construction expands to; from-watast lowers that
;; to a HolonAST; Tagged/new wraps for the round-trip-safe data
;; column. arc 091 slice 8's machinery; arc 093 slice 3 reverses
;; it via Event::Log/data-value<:demo::Trade>.
(:wat::core::define
  (:demo::trade-event
    (time-ns :i64)
    (side    :String)
    (qty     :i64)
    (price   :f64)
    -> :wat::telemetry::Event)
  (:wat::core::let*
    (((trade :demo::Trade) (:demo::Trade/new side qty price))
     ((form  :wat::WatAST) (:wat::core::struct->form trade))
     ((data  :wat::holon::HolonAST) (:wat::holon::from-watast form))
     ((tagged :wat::edn::Tagged) (:wat::edn::Tagged/new data))
     ((notag-ns :wat::edn::NoTag)
      (:wat::edn::NoTag/new (:wat::holon::leaf :demo::trades)))
     ((notag-cal :wat::edn::NoTag)
      (:wat::edn::NoTag/new (:wat::holon::leaf :demo::interrogate)))
     ((notag-lvl :wat::edn::NoTag)
      (:wat::edn::NoTag/new (:wat::holon::leaf :info)))
     ((tags :wat::telemetry::Tags)
      (:wat::core::HashMap
        :(wat::holon::HolonAST,wat::holon::HolonAST))))
    (:wat::telemetry::Event::Log
      time-ns notag-ns notag-cal notag-lvl
      "interrogate-demo" tags tagged)))


(:wat::core::define
  (:demo::write-fixture
    (path :String)
    -> :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
  (:wat::core::let*
    (((spawn :wat::telemetry::Spawn<wat::telemetry::Event>)
      (:wat::telemetry::Sqlite/auto-spawn
        :wat::telemetry::Event
        path 1
        (:wat::telemetry::null-metrics-cadence)
        :wat::telemetry::Sqlite/null-pre-install))
     ((pool :wat::telemetry::HandlePool<wat::telemetry::Event>)
      (:wat::core::first spawn))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::second spawn))
     ;; Six sample trades — 4 buys + 2 sells, varied qtys.
     ((handle :wat::telemetry::Handle<wat::telemetry::Event>)
      (:wat::kernel::HandlePool::pop pool))
     ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
     ((req-tx :wat::telemetry::ReqTx<wat::telemetry::Event>)
      (:wat::core::first handle))
     ((ack-rx :wat::telemetry::AckRx)
      (:wat::core::second handle))
     ((entries :wat::core::Vector<wat::telemetry::Event>)
      (:wat::core::Vector :wat::telemetry::Event
        (:demo::trade-event 1000 "buy"  5  100.0)
        (:demo::trade-event 2000 "sell" 12 102.5)
        (:demo::trade-event 3000 "buy"  15 99.0)   ; ← Q2 hit
        (:demo::trade-event 4000 "buy"  3  101.0)
        (:demo::trade-event 5000 "buy"  20 98.5)   ; ← Q2 hit
        (:demo::trade-event 6000 "sell" 8  103.0)))
     ((_log :wat::core::unit)
      (:wat::telemetry::batch-log req-tx ack-rx entries)))
    driver))


;; ─── Phase 2/3 — read + interrogate ─────────────────────────────

;; Q2's predicate. Buy AND qty > 10. The matcher destructures the
;; struct's :side and :qty fields into ?side / ?qty and ANDs the
;; constraints.
(:wat::core::define
  (:demo::big-buy?
    (event :wat::telemetry::Event)
    -> :wat::core::bool)
  (:wat::core::match
    (:wat::telemetry::Event::Log/data-value event)
    -> :wat::core::bool
    ((:wat::core::Some trade)
      (:wat::form::matches? trade
        (:demo::Trade
          (= ?side :side)
          (= ?qty :qty)
          (= ?side "buy")
          (> ?qty 10))))
    (:wat::core::None false)))


;; ─── Entry ──────────────────────────────────────────────────────

(:wat::core::define
  (:user::main
    (_stdin  :wat::io::IOReader)
    (stdout  :wat::io::IOWriter)
    (_stderr :wat::io::IOWriter)
    -> :wat::core::unit)
  (:wat::core::let*
    (;; Auto-deleting fixture path.
     ((tf :wat::io::TempFile) (:wat::io::TempFile/new))
     ((path :wat::core::String) (:wat::io::TempFile/path tf))

     ;; Phase 1.
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:demo::write-fixture path))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver))
     ((_p1 :wat::core::unit)
      (:wat::io::IOWriter/println stdout
        "── Q1: warmup — count all logged trades ──"))

     ;; Q1 — count every Event::Log row, no narrowing.
     ((handle :wat::sqlite::ReadHandle)
      (:wat::sqlite::open-readonly path))
     ((no-constraints :wat::core::Vector<wat::telemetry::TimeConstraint>)
      (:wat::core::Vector :wat::telemetry::TimeConstraint))
     ((all-events :wat::core::Vector<wat::telemetry::Event>)
      (:wat::stream::collect
        (:wat::telemetry::sqlite/stream-logs handle no-constraints)))
     ((q1-count :wat::core::i64) (:wat::core::length all-events))
     ((_p2 :wat::core::unit)
      (:wat::io::IOWriter/println stdout
        (:wat::core::string::concat
          "  total logs: " (:wat::core::i64::to-string q1-count))))

     ;; Q2 — Clara filter for buys with qty > 10.
     ((_p3 :wat::core::unit)
      (:wat::io::IOWriter/println stdout ""))
     ((_p4 :wat::core::unit)
      (:wat::io::IOWriter/println stdout
        "── Q2: matches? — buy AND qty > 10 ──"))
     ((handle2 :wat::sqlite::ReadHandle)
      (:wat::sqlite::open-readonly path))
     ((no-constraints2 :wat::core::Vector<wat::telemetry::TimeConstraint>)
      (:wat::core::Vector :wat::telemetry::TimeConstraint))
     ((big-buys :wat::core::Vector<wat::telemetry::Event>)
      (:wat::stream::collect
        (:wat::stream::filter
          (:wat::telemetry::sqlite/stream-logs handle2 no-constraints2)
          :demo::big-buy?)))
     ((q2-count :wat::core::i64) (:wat::core::length big-buys))
     ((_p5 :wat::core::unit)
      (:wat::io::IOWriter/println stdout
        (:wat::core::string::concat
          "  hits: " (:wat::core::i64::to-string q2-count)))))
    (:wat::io::IOWriter/println stdout "── done ──")))

;; Co-located fixture for probe_arc278_log_captures_call_line.rs — arc 278 §4, the `log` widget gate.
;; (DESIGN-telemetry-caller-and-capacity.md §4)
;;
;; `:wat::telemetry::log` (the client call-site widget) bakes `:emitted-from (:wat::kernel::macro-call-site)`
;; at the `(log …)` line, then issues the span's `log` op (which stamps the span's correlation `uuid`).
;; This proves the per-log-line capture END-TO-END: two `(log …)` calls on ADJACENT lines, written through
;; a span to a MemStore-backed journal, then queried back — their `emitted-from` lines differ by EXACTLY 1.
;; (Strike 1 proved `macro-call-site` itself; this proves the WIDGET wires it into the stored Log. The two
;; log forms keep their real source spans through `with-span`'s `~body` splice — `restamp_unknown_spans` is
;; a no-op, "every span is real", so the nested log lines ARE the real source lines.)
;;
;; `sk = time-sk(time-ns)` (journal.wat:52) ⇒ two distinct times ⇒ two rows; `query-logs` returns them
;; ascending by sk (time) ⇒ `logs[0]` = the earlier (first) log = the line-N call, `logs[1]` = the line-N+1
;; call. The `count == 2` guard fails LOUD on the near-impossible same-nanos collision (→ diff `-1`) rather
;; than passing silently. `diff` returns: 1 = GREEN, -1 = count != 2, -2 = query-logs not Success.
;;
;; RED at HEAD: `:wat::telemetry::log` does not exist → unknown callee → startup fails.
;; GREEN after: the widget captures each `(log …)` call's own line → adjacent calls differ by exactly 1.

(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

(:wat::core::defn :user::log-line-diff [] -> :wat::core::i64
  (:wat::core::let
    [msh     (:wat::query::mem-store/start :locus (:wat::spawn::thread)
               :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr   (:wat::query::mem-store::Handle/addr msh)
     jh      (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
               :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr   (:wat::telemetry::journal::Handle/addr jh)
     tags    (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     ;; two (log …) on ADJACENT lines — the whole point; keep them consecutive (a line between → diff 2 → RED).
     _ws     (:wat::telemetry::with-span span jaddr "probe-ns" tags
               (:wat::core::do
                 (:wat::core::match (:wat::telemetry::log span :wat::telemetry::Level::Info (:probe::Note :text "a")) ((:wat::kernel::RecvOutcome::Message _resp) nil) ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
                 (:wat::telemetry::log span :wat::telemetry::Level::Info (:probe::Note :text "b"))))
     jclient (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     resp    (:wat::telemetry::Journal/query-logs jclient
               (:wat::telemetry::Journal::QueryLogsRequest
                 :namespace "probe-ns" :time-lo 0 :time-hi 9223372036854775807 :limit 20 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::telemetry::Journal::QueryLogsResponse::Success logs _cursor)
        (:wat::core::if (:wat::core::= (:wat::core::count logs) 2)
          (:wat::core::let
            ;; Arc 109 — Frame/line is a concrete (non-Option) i64, read directly.
            [ln1 (:wat::kernel::Frame/line (:wat::telemetry::Log/emitted-from (:wat::core::first logs)))
             ln2 (:wat::kernel::Frame/line (:wat::telemetry::Log/emitted-from (:wat::core::second logs)))]
            (:wat::i64::- ln2 ln1))
          -1))
      (_ -2))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

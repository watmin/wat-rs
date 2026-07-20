;; scratchpad/design-io-budgets-ux.wat — SELF-PROMPT-INJECTION (R17): the user-forms that
;; DESIGN-service-io-budgets.md IMPLIES, materialized so we judge the FORMS (not the abstract names).
;; NOT compilable — write-logs-batched / page-all / the budget annotations are the PROPOSED tooling.
;; A UX sketch to riff on. Grounded against the real arena idiom (tests/services/probe_arc278_sift_rules_arena.wat).

;; ════════════════════════════════════════════════════════════════════════════════════════════
;; (1) SERVICE AUTHOR — declare a per-op budget on the surface :features (the discoverable contract)
;;     The budget rides on the OP, beside its req/resp types — "declare it like messages and features."
;; ════════════════════════════════════════════════════════════════════════════════════════════
(:wat::core::defsurface :wat::telemetry::Journal
  :nature :wat::kernel::Peer'
  :messages [ #_"…WriteLogsRequest / WriteLogsResponse / QueryLogsRequest / QueryLogsResponse…" ]
  :features
  [(write-logs [self <- :wat::telemetry::Journal  req <- :wat::telemetry::Journal::WriteLogsRequest]
                 -> :wat::telemetry::Journal::WriteLogsResponse
                 :max-request-bytes 10485760)                 ;; 10 MiB — the bulk write (the only op that needs it)
   (query-logs [self <- :wat::telemetry::Journal  req <- :wat::telemetry::Journal::QueryLogsRequest]
                 -> :wat::telemetry::Journal::QueryLogsResponse
                 :max-page-bytes    524288)                    ;; 512 KiB per response page (paged output)
   (stats      [self <- :wat::telemetry::Journal]
                 -> :wat::telemetry::Journal::StatsResponse)]) ;; no annotation → default 512 KiB

;; ── discovery (mostly the tooling reads this; a user CAN too) — CRUX-1, one candidate shape: ──
;;   (:wat::telemetry::Journal::write-logs/max-request-bytes)   ;; => 10485760  (synthesized constant)

;; ════════════════════════════════════════════════════════════════════════════════════════════
;; (2) WRITER — the write loop (symmetric with read: consume a Stream / push with backpressure).
;;     Caller never sees a batch boundary; the mandatory :max-request-bytes budget is invisible.
;; ════════════════════════════════════════════════════════════════════════════════════════════
;;   TODAY (the shadowdancer's shortcut, REJECTED):
;;     _wr1 (:wat::telemetry::Journal/write-logs journal (…WriteLogsRequest logs-0-400))
;;     _wr2 (:wat::telemetry::Journal/write-logs journal (…WriteLogsRequest logs-400-800))
;;
;;   (a) consume a stream (materialized OR lazy), batch to fit → WriteResult:
(:wat::telemetry::write-logs-stream journal log-stream)
;;   (b) have-it-in-hand convenience  ==  (write-logs-stream journal (stream-from items)):
(:wat::telemetry::write-logs-batched journal all-800-logs)
;;   (c) buffered producer sink — flush on time-OR-size (Kafka linger.ms+batch.size); push single logs:
(:wat::telemetry::with-log-sink journal :max-bytes 1048576 :max-latency-ms 1000
  (:wat::core::fn [sink] -> :wat::query::WriteResult
    (:wat::telemetry::push sink log-a)     ;; backpressured enqueue-ack: room → ack now; busy → block, then enqueue
    (:wat::telemetry::push sink log-b)))    ;; scope exit (RAII) → flush remainder, reap → WriteResult
;;     sink = a defservice actor (buffer in :ephemeral, serve loop select's {bounded input, flush-timer});
;;     flush at EVERY exit (:hibernate/:stop/close). NOT fire-and-forget: push handshakes; durable outcome surfaced.
;;
;;   the durable outcome — a named WriteResult, case-matched (no silent loss):
(:wat::core::match (:wat::telemetry::write-logs-stream journal log-stream) -> :wat::core::nil
  ((:wat::query::WriteResult::Done written)               (:user::ok written))
  ((:wat::query::WriteResult::ItemTooLarge item b c)      (:user::fix-item item))           ;; 400 — one un-chunkable item
  ((:wat::query::WriteResult::Failed written cause rest)  (:user::retry rest)))             ;; wrote `written`, retry `rest`

;; ════════════════════════════════════════════════════════════════════════════════════════════
;; (3) READER — `<op>-stream` returns a LAZY :wat::stream::Stream<Value> (the builder's Enumerator idiom).
;;     The cursor-paging is the ENGINE inside the stream; the consumer uses NORMAL lazy ops and NEVER
;;     sees a page or a cursor. The composite Cursor is threaded internally only.
;; ════════════════════════════════════════════════════════════════════════════════════════════
;;   Ruby:  loop { resp = client.query(params.merge(next_token:)); resp.items.each { yielder << _ };
;;                 break if next_token.nil? };  enum.each { |item| ... }
;;   wat — the synthesized per-op stream. The element is an ENUM (:Item + the op's NAMED failures), so a
;;   mid-stream failure is IN-BAND + case-matched — never a raise, never a silent stream::empty.
;;     SiftRulesItem = :Item[deduction <- Value] | :RequestTooLarge[bytes cap] | :ImpureRules[cause] | :Fatal[cause]
(:arena::my-sift/sift-rules-stream svc
  (:arena::my-sift::SiftRulesRequest :namespace "arena" :time-lo 0 :time-hi 100000000 :limit 100 :cursor :wat::core::None))
;;   => :wat::stream::Stream<:arena::my-sift::SiftRulesItem>. Consume by matching each element:
(:wat::core::foldl
  (:wat::core::fn [acc <- :i64 si <- :arena::my-sift::SiftRulesItem] -> :i64
    (:wat::core::match si -> :i64
      ((:arena::my-sift::SiftRulesItem::Item d)              (:wat::core::+ acc 1))         ;; a datum
      ((:arena::my-sift::SiftRulesItem::RequestTooLarge b c) (:user::stop acc))             ;; 400 — graceful, no fault
      ((:arena::my-sift::SiftRulesItem::ImpureRules cause)   (:user::report acc cause))     ;; 400
      ((:arena::my-sift::SiftRulesItem::Fatal cause)         (:user::escalate acc cause)))) ;; 500
  0 (:arena::my-sift/sift-rules-stream svc base-req))
;;   happy-path sugar (opt-in, YAGNI until wanted) — unwrap-or-raise gives a plain Stream<Value> that raises on failure:
(:wat::core::into (:wat::core::Vector :Value)
  (:wat::core::map :wat::query::unwrap-or-raise (:arena::my-sift/sift-rules-stream svc base-req)))
(:wat::core::take 10 (:arena::my-sift/sift-rules-stream svc base-req))   ;; early stop, constant-mem (still matches items)
;;
;;   the generator, sketched (Enumerator.new in wat — stream::lazy + stream::cons, fetch-a-page-when-drained):
;;     (defn page-stream [svc base-req state buffered] -> Stream<SiftRulesItem>
;;       (:wat::stream::lazy
;;         (:wat::core::if (:wat::core::not (:wat::core::empty? buffered))
;;           (:wat::stream::cons (:SiftRulesItem::Item (:wat::core::first buffered))
;;                               (page-stream svc base-req state (:wat::core::rest buffered)))
;;           (:wat::core::match state -> Stream<SiftRulesItem>      ;; Start | More(cursor) | Done — fetch-first, then check
;;             (:Done (:wat::stream::empty))
;;             (_ (:wat::core::match (:arena::my-sift/sift-rules svc (base-req-with-cursor base-req state)) -> Stream<SiftRulesItem>
;;                  ((SiftRulesResponse::Deductions items cursor)
;;                    (page-stream svc base-req (cursor->state cursor) items))
;;                  ;; a page failure → ONE terminal in-band element, then the stream ends. NOT a raise, NOT silent empty:
;;                  ((SiftRulesResponse::RequestTooLarge b c) (:wat::stream::cons (:SiftRulesItem::RequestTooLarge b c) (:wat::stream::empty)))
;;                  ((SiftRulesResponse::Fatal cause)         (:wat::stream::cons (:SiftRulesItem::Fatal cause) (:wat::stream::empty)))))))))

;; ════════════════════════════════════════════════════════════════════════════════════════════
;; (4) THE RESPONSE ENUM — a NAMED variant per FAILURE KIND. Exhaustive match forces the caller to
;;     handle each; no overloaded `cause` bucket to guess at (conformare — completeness by structure).
;;     400-class (caller-fixable, connection LIVES) is distinct from 500-class (server fault).
;; ════════════════════════════════════════════════════════════════════════════════════════════
;;   (:wat::core::defenum :arena::my-sift::SiftRulesResponse :wat::enum::Pure
;;     :Deductions       [items <- :PV<Value>  cursor <- (:Option :Cursor)]  ;; 200 — paged
;;     :RequestTooLarge  [bytes <- :i64  cap <- :i64]                        ;; 400 — fragment your request
;;     :ImpureRules      [cause <- :wat::kernel::Failure]                    ;; 400 — your rules aren't pure (safety)
;;     :UnknownMessageType [type-name <- :wat::core::String]                 ;; 400 — a Log class isn't in :defs
;;     :Fatal            [cause <- :wat::kernel::Failure])                   ;; 500 — server-side, not your fault
(:wat::core::match (:arena::my-sift/sift-rules svc req) -> :wat::core::nil
  ((:arena::my-sift::SiftRulesResponse::Deductions items cursor) (:user::collect items cursor))
  ((:arena::my-sift::SiftRulesResponse::RequestTooLarge bytes cap) (:user::refragment bytes cap)) ;; 400 — retry smaller
  ((:arena::my-sift::SiftRulesResponse::ImpureRules cause)         (:user::report cause))          ;; 400 — fix rules
  ((:arena::my-sift::SiftRulesResponse::UnknownMessageType t)      (:user::report-unknown t))      ;; 400 — fix :defs
  ((:arena::my-sift::SiftRulesResponse::Fatal cause)              (:user::escalate cause)))        ;; 500
;;   the checker FORCES all five arms — the caller cannot miss a failure kind (no guessing which 400).
;;   connection LIVES on every 400; only the caller's own logic decides retry-vs-abort.

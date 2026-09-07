;; probe-the-handler-is-spliced-once.wat — count the op body in the EMITTED form.
;;
;; `clamp the request, not the handler` claims ~outcome-match now appears once.
;; Reading the macro source shows the outer (= page-field "") is expand-time, so
;; only one branch is emitted — but that is a reading, not a measurement.
;; Macroexpand a defservice whose op declares :max-page and COUNT the marker.
;;
;; ⛔ THIS PROBE IS BLIND — RECORDED SO NOBODY REBUILDS IT.
;; macroexpand of a QUOTED defservice does not produce the clamp-bearing path:
;; the expansion contains zero MAX-PAGE / TakeRequest/limit nodes, and is
;; byte-identical (44502) with and without :max-page. It reports 2 handler
;; copies either way — which is the PRE-EXISTING two-dispatch-path baseline
;; (see probe-what-defservice-emits.wat), not evidence about the clamp.
;;
;; Row 1 is verifiable by READING the macro source (the outer (= page-field "")
;; is expand-time, so one branch is emitted; ~outcome-match appears once inside
;; the quasiquote) and by the TIMING recovering. Not by this.

(:wat::core::defsurface :hs::Bag :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :hs::Bag::TakeRequest
     [limit  <- :wat::core::i64
      cursor <- (:wat::core::Option :- [:wat::core::String])])
   (:wat::core::defenum :hs::Bag::TakeResponse :wat::enum::Pure
     :Ok [items  <- (:wat::core::Vector :- [:wat::core::String])
          cursor <- (:wat::core::Option :- [:wat::core::String])]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(take [self <- :hs::Bag  req <- :hs::Bag::TakeRequest]
     -> :hs::Bag::TakeResponse :max-request-bytes 524288 :max-page [items 64])])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::write-forms
    (:wat::core::macroexpand (:wat::core::quote
      (:wat::service::defservice :hs::bag
        :satisfies :hs::Bag
        :durable   [n <- :wat::core::i64]
        :ephemeral []
        :init (:wat::core::fn [record <- :hs::bag::Record] -> :hs::bag::State
                (:hs::bag::State :durable record))
        :impls
        [(take [s ctx req] MARKER-THE-TAKE-HANDLER)]))))))

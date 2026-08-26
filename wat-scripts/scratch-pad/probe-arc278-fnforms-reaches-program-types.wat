;; probe-arc278-fnforms-reaches-program-types.wat — a MEASUREMENT, and it must be RUN.
;;
;; THE QUESTION. `defservice` ships its child a HAND-ENUMERATED manifest (`<fqdn>::service-forms` =
;; the satisfied surfaces' forms + the macro's own generated defs). The substrate also ships a
;; transitive-closure extractor (`src/closure_extract.rs`, exposed as `:wat::kernel::fn-forms`) that
;; `wat/bracket.wat` uses for the identical "what does a forked child need" problem, and that
;; `wat/service.wat` never calls (zero hits).
;;
;; Already PROVEN by run (probe-arc278-nullary-enum-process-repro.wat): a PROGRAM-LEVEL `defenum`
;; named in `:durable` and matched in an op body does NOT cross a process fork — the child dies at
;; startup because it has the type's NAME but not its VARIANTS.
;;
;; THIS PROBE MEASURES whether the closure extractor would have carried it. It prints the FORM COUNT
;; from each source and whether the program-level enum's name appears in the rendered forms:
;;
;;   (:probe::ffx::service-forms)                  — what defservice actually ships today
;;   (:wat::kernel::fn-forms :probe::ffx::serve …) — the closure from the service's own serve fn
;;
;; ⚠ A REFUSAL IS A LEGITIMATE OUTCOME, not a failed measurement. closure_extract refuses captured
;; non-portable VALUES (Sender/Receiver/HandlePool/ChildHandle/IOReader/IOWriter). `serve` takes its
;; Listener and peers as PARAMETERS rather than captures, so the refusal is not expected to fire —
;; but that is a prediction, and this probe exists because a prediction is not a result. If it
;; raises, report the raise verbatim; that is the finding.

(:wat::core::defsurface :probe::FFX :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::FFX::PingRequest [])
   (:wat::core::defenum :probe::FFX::PingResponse :wat::enum::Pure
     :Ok               [ok <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::FFX  req <- :probe::FFX::PingRequest] -> :probe::FFX::PingResponse :max-request-bytes 524288)])

;; ★ THE SUBJECT — declared at PROGRAM level, NOT inside the surface's `:messages`. This is the
;; declaration that does not cross the fork today. If the closure extractor reaches it, its name
;; appears in the fn-forms rendering below and not in service-forms.
(:wat::core::defenum :probe::FFXTag :wat::enum::Pure
  :Alpha []
  :Beta  [])

(:wat::service::defservice :probe::ffx
  :satisfies :probe::FFX
  :durable   [tag <- :probe::FFXTag]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::ffx::Record] -> :probe::ffx::State
          (:probe::ffx::State :durable record))
  :impls
  [(ping [s ctx req]
     (:wat::core::let
       [t  (:probe::ffx::Record/tag (:probe::ffx::State/durable s))
        ok (:wat::core::match t
             ((:probe::FFXTag::Alpha) true)
             ((:probe::FFXTag::Beta)  false))]
       (:wat::service::Outcome::Reply s (:probe::FFX::PingResponse::Ok ok))))])

;; ── render a (Vector :- [WatAST]) to one string so we can ask whether a name appears in it ───────────
(:wat::core::defn :user::render-forms
  [forms <- (:wat::core::Vector :- [:wat::WatAST])  i <- :wat::core::i64  acc <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::i64::>= i (:wat::core::length forms))
    acc
    (:user::render-forms forms (:wat::i64::+ i 1)
      (:wat::string::concat acc
        (:wat::core::ast->source (:wat::core::nth forms i))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [manifest  (:probe::ffx::service-forms)
     closure   (:wat::kernel::fn-forms :probe::ffx::serve :user::shipped-serve)
     man-src   (:user::render-forms manifest 0 "")
     clo-src   (:user::render-forms closure  0 "")
     _counts   (:wat::kernel::println
                 (:wat::string::concat "COUNTS manifest="
                   (:wat::string::concat (:wat::i64::to-string (:wat::core::length manifest))
                     (:wat::string::concat " closure="
                       (:wat::i64::to-string (:wat::core::length closure))))))
     _m        (:wat::kernel::println (:wat::string::concat "MANIFEST_SRC " man-src))]
    ;; the shell greps these two lines for the needle — a substring test wat has no verb for
    ;; (`str-in?` is (Vector :- [String]) membership, as the checker said when this probe first tried it)
    (:wat::kernel::println (:wat::string::concat "CLOSURE_SRC " clo-src))))

;; probe-durable-forms-vector.wat — DISCONFIRMING PROBE (arc 170, the REPL stone).
;;
;; ★ REPLACES the durable half of `probe-repl-durable-forms.wat`, which #74 consumed.
;; That file carried TWO subjects: this one (its original, from arc 170) and a later
;; annotation making its deliberately-non-conforming `EvalResponse` the acceptance case
;; for "the RequestTooLarge ctor is READ, never guessed". The builder's 2026-08-05 ruling
;; made the second subject FALSE — a response type's name is now LAW — so that file
;; inverted into `tests/services/probe_arc278_repl_durable_forms_response_law.wat.bad`,
;; a deliberately-illegal declaration with a Rust test asserting it is REFUSED. Correct
;; for the annotation; but the ORIGINAL question went with it, and it was never the
;; annotation's to take. This file restores it, with a lawful name pair.
;;
;; THE ONE QUESTION: may a `defservice` declare its `:durable` state as a vector of
;; FORMS — `(:wat::core::Vector :- [:wat::WatAST])` — or does the portability wall (293.W,
;; "no portable aggregate declares a non-portable field") reject it?
;;
;; Why it decides a design: the REPL's durable state is the user's accumulated definition
;; set. The seam (278 24w) recorded that as `:durable [defs <- (Vector :- [WatAST])]`. But
;; `src/edn_shim.rs` refuses an `Edn::Symbol` on the general value DECODE path ("wat has
;; no symbol value type"), and a form is full of bare symbols — so a WatAST field may
;; ENCODE faithfully and still be un-decodable on the far side. If the wall fires here,
;; `:durable` carries `::`-SOURCE TEXT (String) and the forms are rebuilt with
;; `read-string` — the same carry `DESIGN-sift-server-side-filter.md` was forced onto for
;; `Sieve::Predicate`, for this same reason.
;;
;; The `:ephemeral` slot is deliberately EMPTY so the probe isolates the durable question
;; alone (a non-empty `:ephemeral` would also demand `:init`, per service.wat).
;;
;; GREEN (this file loads) → forms ARE legal durable state; the seam's shape stands.
;; RED  → read the located diagnostic; it names the wall, and `:durable` takes String.
;;
;; It lives under `wat-scripts/scratch-pad/` on purpose: the
;; `every_wat_scripts_file_loads_on_the_current_runtime` gate parses and type-checks it on
;; every build, so the answer is re-proven continuously and cannot rot into a graveyard
;; that reads like live code. That gate is the equipment that caught R64.

(:wat::core::defsurface :probe::DurableForms :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::DurableForms::EvalSrcRequest [src <- :wat::core::String])
   (:wat::core::defenum :probe::DurableForms::EvalSrcResponse :wat::enum::Pure
     :Ok               [out <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path     <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got      <- :wat::core::String])]
  :features
  [(eval-src [self <- :probe::DurableForms  req <- :probe::DurableForms::EvalSrcRequest]
     -> :probe::DurableForms::EvalSrcResponse :max-request-bytes 524288)])

;; ★ THE SUBJECT — `:durable` holding a vector of FORMS.
(:wat::service::defservice :probe::durable-forms-svc
  :satisfies :probe::DurableForms
  :durable   [defs <- (:wat::core::Vector :- [:wat::WatAST])]
  :ephemeral []
  :impls
  [(eval-src [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::DurableForms::EvalSrcResponse::Ok
         (:probe::DurableForms::EvalSrcRequest/src req))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "probe-durable-forms-vector"))

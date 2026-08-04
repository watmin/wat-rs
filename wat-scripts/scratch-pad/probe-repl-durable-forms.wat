;; probe-repl-durable-forms.wat — DISCONFIRMING PROBE (arc 170, the REPL stone).
;;
;; THE ONE QUESTION: may a `defservice` declare its `:durable` state as a vector of
;; FORMS — `:wat::core::Vector<wat::WatAST>` — or does the portability wall (293.W,
;; "no portable aggregate declares a non-portable field") reject it?
;;
;; Why it decides the design: the REPL's durable state is the user's accumulated
;; definition set. The seam (278 24w) recorded that as `:durable [defs <- Vector<WatAST>]`.
;; But `src/edn_shim.rs:1577` refuses an `Edn::Symbol` on the general value DECODE path
;; ("wat has no symbol value type"), and a form is full of bare symbols — so a WatAST
;; field may encode faithfully and still be un-decodable on the far side. If the wall
;; fires here, `:durable` carries `::`-SOURCE TEXT (String) and the forms are rebuilt with
;; `read-string` — the same carry `DESIGN-sift-server-side-filter.md` was forced onto for
;; `Sieve::Predicate`, for this same reason.
;;
;; The `:ephemeral` slot is deliberately EMPTY here so the probe isolates the durable
;; question alone (a non-empty `:ephemeral` would also demand `:init`, per service.wat:364).
;;
;; RUN: target/release/wat --check wat-scripts/scratch-pad/probe-repl-durable-forms.wat
;;   GREEN → forms are legal durable state; the seam's shape stands.
;;   RED   → read the located diagnostic; it names the wall, and `:durable` takes String.

;; arc 278 BRIEF-client-validates-locally — this file's `EvalResponse` (for op `eval-src`)
;; is now the ACCEPTANCE CASE for "the RequestTooLarge ctor is read from the op's DECLARED
;; return type, never guessed by `<OpPascal>Response` concatenation": it deliberately does
;; NOT follow that naming convention (arbitrary-but-legal — nothing requires a response
;; type's NAME to echo its op's), so a still-guessing call site fails here first.
(:wat::core::defsurface :probe::Repl :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Repl::EvalRequest [src <- :wat::core::String])
   (:wat::core::defenum :probe::Repl::EvalResponse :wat::enum::Pure
     :Ok               [out <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(eval-src [self <- :probe::Repl  req <- :probe::Repl::EvalRequest]
     -> :probe::Repl::EvalResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::repl-svc
  :satisfies :probe::Repl
  :durable   [defs <- :wat::core::Vector<wat::WatAST>]
  :ephemeral []
  :impls
  [(eval-src [s req]
     (:wat::service::Outcome::Reply s
       (:probe::Repl::EvalResponse::Ok (:probe::Repl::EvalRequest/src req))))])

;; Arc 179: `()` retired as a value; the original no-op body was `()`. A bare
;; `nil` body trips the pre-existing UselessMain wall (src/freeze.rs:1433),
;; which `()` had been silently dodging by not being a `WatAST::NilLit` node.
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "probe-repl-durable-forms"))

;; PROBE — what happens to `(wat.core/+ 2 2)`?
;;
;; THE QUESTION (the builder's). `wat.core/+` is a NAMESPACED EDN SYMBOL, not a keyword —
;; no leading colon. `resolve::normalize` exists specifically to rewrite `wat.core/+` into
;; `:wat::core::+`, so the Clojure-idiom spelling is legitimate wat source.
;;
;; WHY IT MATTERS RIGHT NOW. `edn_to_value_caps` (`edn::render::edn_to_value_caps`'s `Edn::Symbol` arm) refuses EVERY `Edn::Symbol`
;; — "wat has no symbol value type" — and that refusal is what kills any WatAST crossing a
;; process-locus service boundary (task #92). The open question is the BLAST RADIUS:
;;
;;   NARROW  — only field binders (`c`, `<-`) carry symbols, so it bites declaration forms.
;;   CENTRAL — a CALL HEAD in the Clojure idiom is a symbol too, so it bites ordinary code.
;;
;; This file measures which. It asks three separate things and prints each, because they can
;; disagree and the disagreement is the finding:
;;
;;   1. does a symbol-headed form EVALUATE? (is it live source at all?)
;;   2. what does it LOOK like as EDN once quoted? (symbol or normalized keyword?)
;;   3. does it survive `edn::validate` against `:wat::WatAST`? (the wire's typed door)
;;
;; ⚠ This probe does NOT cross a process boundary — that is `probe-arc278-watast-on-the-wire-
;; decomposed.wat`'s job and it is red there by measurement. This one isolates the VALUE, not
;; the transport, so a failure here cannot be blamed on a locus.
;;
;; ★ MEASURED 2026-08-12 — verbatim:
;;
;;   "1. (wat.core/+ 2 2) evaluates to 4"
;;   "2. quoted, as EDN:"
;;   (wat.core/+ 2 2)
;;   "3. edn::validate against :wat::WatAST:"
;;   "   VALID"
;;
;; VERDICT — THE BLAST RADIUS IS CENTRAL, NOT NARROW.
;;   (1) It EVALUATES. `normalize` rewrites the symbol head to `:wat::core::+`; this is live,
;;       legitimate wat source, not an exotic spelling.
;;   (2) The SYMBOL SURVIVES IN THE QUOTED AST — `quote` is `Boundary::AllData`, so the head is
;;       NOT normalized in data position and the payload genuinely carries `Symbol("wat.core/+")`.
;;   (3) The typed door ACCEPTS it — the identity arm (b472fe3e) is doing its job.
;;
;; And yet this form still cannot cross a PROCESS boundary, because `edn_to_value_caps`
;; (`edn::render::edn_to_value_caps`'s `Edn::Symbol` arm) refuses every `Edn::Symbol` upstream of the typed walk (task #92).
;;
;; The correction this file exists to record: the defect was first characterised as biting
;; DECLARATION forms, because the symbols noticed were field binders (`c`, `<-`). This form has
;; no binder, no arrow, and is not a declaration — it is two integers and a call head — and it is
;; still refused. ANY Clojure-idiom call head is a symbol, so essentially NO non-trivial wat form
;; crosses a process wire. Narrow was the wrong word.
;;
;; ★ AND THE EDGE: 299 ruled wat "a clojure dialect, not a clojure impl". `(wat.core/+ 2 2)` is
;; the CLOJURE SPELLING — the one a Clojure programmer writes by reflex. It is exactly the form
;; the wire refuses.

(:wat::core::defn :probe::eval-symbol-head [] -> :wat::core::i64
  (wat.core/+ 2 2))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; (1) does it evaluate? if normalize rewrites the head, this is 4.
    (:wat::kernel::println
      (:wat::string::concat "1. (wat.core/+ 2 2) evaluates to "
        (:wat::i64::to-string (:probe::eval-symbol-head))))

    ;; (2) what is it as data? THE load-bearing line — a symbol head or a normalized keyword?
    (:wat::kernel::println "2. quoted, as EDN:")
    (:wat::kernel::println (:wat::core::quote (wat.core/+ 2 2)))

    ;; (3) does the wire's typed door accept it as a WatAST?
    (:wat::kernel::println "3. edn::validate against :wat::WatAST:")
    (:wat::core::match
      (:wat::edn::validate (:wat::core::quote (wat.core/+ 2 2)) :wat::WatAST)
      (:wat::edn::Validation::Valid (:wat::kernel::println "   VALID"))
      ((:wat::edn::Validation::Invalid path expected got)
        (:wat::core::do
          (:wat::kernel::println
            (:wat::string::concat "   INVALID expected=" expected " got=" got))
          (:wat::kernel::println path))))))

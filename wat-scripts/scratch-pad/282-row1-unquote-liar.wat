;; wat-scripts/scratch-pad/282-row1-unquote-liar.wat — arc 282 STONE acceptance ROW 1, THE
;; CONTROL. Builds the deliberate liar the brief describes: a rule that matches the `~`
;; (unquote) node by its Span only (never checking Written), then CLAIMS its old-text is the
;; canonical 20-character name ":wat::core::unquote" — when the span it matched actually
;; covers exactly ONE character, `~`. Before arc 282 this edit would have silently spliced,
;; replacing `~` (and 19 characters of whatever followed it) with the new text. After arc 282,
;; fix-text-apply must REFUSE — naming the offset, the claim, and what is really there.
;;
;; `~b` desugars (wat/fix.wat:621's own comment) to a List `(:wat::core::unquote b)` whose
;; HEAD is a reader-synthesized keyword node: ast-name is the canonical
;; ":wat::core::unquote" (20 chars) but its OWN span covers only the literal `~` (1 char) —
;; verified below before the liar is built, so the control is grounded in a measured fact,
;; not an assumption.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src       "(:a ~b)"
     lines     (:wat::string::split src "\n")
     tree      (:wat::core::match (:wat::core::read-string src)
                 ((:wat::core::ReadOutcome::Forms __forms) __forms)
                 ((:wat::core::ReadOutcome::Malformed __cause)
                   (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms     (:wat::core::ast->children tree)
     a-form    (:wat::core::first forms)
     ach       (:wat::core::ast->children a-form)
     unq-list  (:wat::core::nth ach 1)
     uch       (:wat::core::ast->children unq-list)
     ;; the reader-synthesized head: matched by SPAN, never by Written.
     uhead     (:wat::core::first uch)
     span      (:wat::core::ast-span uhead)
     end-span  (:wat::core::ast-end-span uhead)
     span-len  (:wat::fix::fix-text-span-len span end-span lines)
     off       (:wat::fix::fix-text-offset-of span lines)]
    (:wat::core::do
      ;; ground the control: the span this liar matches is 1 char (the `~`), not 20.
      (:wat::kernel::println
        (:wat::core::format "GROUND TRUTH: uhead ast-name={n} (len {nl}) but its OWN span covers {sl} char(s) — off={o}"
          :n (:wat::core::ast-name uhead)
          :nl (:wat::core::i64::to-string (:wat::string::length (:wat::core::ast-name uhead)))
          :sl (:wat::core::i64::to-string span-len)
          :o (:wat::core::i64::to-string off)))
      ;; THE LIAR: claims old-text is the canonical 20-char name — the SAME claim the OLD
      ;; (pre-282) apply would have silently trusted via a span-derived old-len of 1 (it would
      ;; have overwritten `~` plus whatever followed, believing it was replacing 20 chars of
      ;; ":wat::core::unquote"). fix-text-apply must now refuse instead of splicing.
      (:wat::kernel::println "ROW 1 — attempting fix-text-apply with a liar's claim…")
      (:wat::fix::fix-text-apply src
        (:wat::core::Vector :wat::fix::Edit
          (:wat::core::Tuple off ":wat::core::unquote" "REPLACED")))
      (:wat::kernel::println "ROW 1 FAILED TO RAISE — the stone did not hold.")
      nil)))

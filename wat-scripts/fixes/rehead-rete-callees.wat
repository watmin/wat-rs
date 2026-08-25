;; wat-scripts/fixes/rehead-rete-callees.wat — arc 278 #88, THE RETE-DEFN MIGRATION.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; Re-heads a NAMED SET of declarations:
;;   (:wat::core::defn :usr::big? …)  ->  (:wat::rete::core::defn :usr::big? …)
;;
;; ⚠ WHY A NAME LIST AND NOT A PREFIX RENAME. Every target spells the SAME head,
;; `:wat::core::defn`; only the bound name says whether this declaration is a rete callee.
;; `rename-keyword-prefix`/`-exact` key on the token being renamed, so either would re-head the
;; ENTIRE corpus. `:wat::fix::rehead-rete-defn` keys on child[1] and edits child[0] — the
;; predicate is a sibling's value, the edit lands on the head.
;;
;; ⚠ THE LIST IS THE CHECKER'S, NOT A GREP'S. Each name below was named by a live refusal —
;; "':X' is not a rete primitive; a where admits only :wat::rete:: ops" — in a `scripts/floor.sh`
;; run with the #88 membrane armed (R52 QVOD LEX ACCENDIT / R65 SCVTVM IDEM INDEX: the fire is
;; the worklist). A grep over this corpus has produced a wrong count repeatedly in this arc.
;;
;; ⚠ EXPECT A WATERFALL. A file stops at its FIRST refusal, and law A is transitive: re-heading a
;; helper makes ITS undeclared callees scream in the next round. Re-run the floor after applying
;; and add whatever the checker newly names. Do not pre-populate from a guess.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["tests/rete/a.wat" "wat-scripts/perf/grid/b.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/rehead-rete-callees.wat
;;
;; Idempotent by construction: a migrated form's head is no longer `:wat::core::defn`, so it
;; cannot match twice. Dry-run on a /tmp copy and `diff` before touching the corpus.
;;
;; NOTE — this codemod walks the FORM TREE, so it cannot see a name built or parsed inside a
;; STRING literal, nor inline wat embedded in a Rust test string. Both are hand-check surfaces
;; (the 2026-07-24 class-4 lesson); the floor is what surfaces the second.

(:wat::core::defn :user::targets [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::Vector :wat::core::String
    ":cg::make-rate"
    ":fix::head-keyword-str?"
    ":fix::type-shaped-keyword-str?"
    ":test::big?"
    ":tf::compute-scalar"
    ":tf::first-rate"
    ":ur::sum-of-squares"
    ":w::sum-of-squares"
    ":wc::heavy?"
    ":wmv::combo?"
    ":wmv::pent?"
    ":wnst::c1"
    ":wnst::c2"
    ":wnst::c3"
    ":wnst::c4"
    ":wnst::c5"
    ":wnst::c6"
    ":wnst::c7"
    ":wnst::c8"
    ":wnst::c9"
    ":wnst::c10"
    ":wnst::f"
    ":wnst::g"
    ":wnst::h"
    ":wnst::hub"
    ":wnst::is-good"
    ":wnst::score"
    ":wnst::twoarg"
    ":wnst::wrap"
    ":wr::is-risky?"
    ":wr::note-positive?"
    ":wr::pos?"
    ":wr::rep-pos?"
    ":wsb::edge?"
    ":wsc::bump"
    ":wsh::big?"
    ":wst::feline?"))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)
                      src  (:wat::io::read-file path)
                      out  (:wat::fix::rehead-rete-defn (:user::targets) src)]
      (:wat::core::do
        (:wat::io::write-file path out)
        (:wat::kernel::println
          (:wat::string::concat
            (:wat::core::if (:wat::core::= src out) "[unchanged] " "[reheaded]  ") path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

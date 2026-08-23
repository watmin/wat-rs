;; census-defclause-arm-overlap.wat — STRIKE 1's gate instrument for stone 118.B2c.
;;
;; ⛔ WHAT IT IS FOR: B2c strike 1 arms a wall refusing OVERLAPPING clause arms. A wall is armed at
;; ZERO OFFENDERS (task #41's pattern) — so the corpus must be shown clean FIRST. STOP-1 of the stone
;; says: if this finds offenders, do NOT arm the wall over them and do NOT fix them silently; each is
;; a live ambiguity whose disposition is the builder's.
;;
;; ★ WHAT IT EMITS, AND WHAT IT DELIBERATELY DOES NOT DECIDE.
;; It walks the real form tree (`read-string` -> `ast->children`, RECURSIVELY, because a `defclause`
;; can sit inside a `do`) and emits one structured row per ARM:
;;
;;     <path> || CLAUSE <name> | GUARD <yes|no> | TYPES <t1> ~ <t2> ~ ...
;;
;; The PATH is part of the key: two files may declare the same clause name, and conflating them
;; would manufacture overlaps that do not exist.
;;
;; The pair-overlap decision is made OVER THESE ROWS, not here. That split is deliberate: the part
;; grep cannot do is finding the arm boundaries and the per-parameter declared types, and that is
;; exactly what the reader does. Deciding whether two clean token-lists overlap is set logic over
;; already-parsed data, not text-scraping of source.
;; `[[feedback_three_boundary_errors_need_a_reader_not_a_fourth_pattern]]`
;;
;; ⚠ WHAT THE INSTRUMENT CAN SEE (state this before quoting its count —
;; `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`):
;;   - It sees SOURCE. A `defclause` emitted by a macro appears as its unexpanded template, so a
;;     macro-generated arm list is NOT resolved here. Those rows are visible as such and must be
;;     macroexpanded before anyone calls the corpus clean.
;;   - It reports `:guard` presence but does not interpret guards. A guard NARROWS an arm's domain
;;     (`ClauseFailureReason::GuardFalse` is a real dispatch outcome), so two same-typed arms with
;;     different guards are NOT necessarily an ambiguity. Guarded rows are for the builder to rule.
;;   - Parameter types are taken as the token FOLLOWING each `<-`, which is the binder form's own
;;     structure — variadic `&`-params included, no positional arithmetic.
;;
;; Usage:
;;   printf '["wat/seq.wat"]\n' | ./target/release/wat wat-scripts/scratch-pad/census-defclause-arm-overlap.wat

(:wat::core::defn :census::head-of [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::if (:wat::core::empty? ch) "" (:wat::core::ast->source (:wat::core::first ch)))))

;; The declared types of a binder vector: every token that FOLLOWS a `<-`.
(:wat::core::defn :census::types-after-arrows
  [toks <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::String
  (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq toks))
    ((:wat::stream::NextOutcome::Item t rest)
      (:wat::core::if (:wat::core::= t "<-")
        (:wat::core::match (:wat::stream::next rest)
          ((:wat::stream::NextOutcome::Item ty more)
            (:wat::core::string::concat
              (:wat::core::string::concat ty " ~ ")
              (:census::types-after-arrows (:wat::core::into [] more))))
          (:wat::stream::NextOutcome::Exhausted "<MISSING-TYPE>"))
        (:census::types-after-arrows (:wat::core::into [] rest))))
    (:wat::stream::NextOutcome::Exhausted "")))

;; Does this arm carry a `:guard`? (top-level tokens of the arm form)
(:wat::core::defn :census::has-guard? [arm <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::reduce
    (:wat::core::fn [acc <- :wat::core::bool c <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::or acc (:wat::core::= (:wat::core::ast->source c) ":guard")))
    false
    (:wat::core::ast->children arm)))

(:wat::core::defn :census::report-arm
  [path <- :wat::core::String name <- :wat::core::String arm <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::let
    [ch     (:wat::core::into [] (:wat::core::ast->children arm))
     binder (:wat::core::if (:wat::core::empty? ch)
              (:wat::core::Vector :wat::core::String)
              (:wat::core::into []
                (:wat::core::map (:wat::core::fn [t <- :wat::WatAST] -> :wat::core::String
                                   (:wat::core::ast->source t))
                  (:wat::core::ast->children (:wat::core::first ch)))))
     guard  (:wat::core::if (:census::has-guard? arm) "yes" "no")]
    (:wat::kernel::println
      (:wat::core::string::concat
        (:wat::core::string::concat
          (:wat::core::string::concat
            (:wat::core::string::concat (:wat::core::string::concat path " || CLAUSE ") name)
            " | GUARD ") guard)
        (:wat::core::string::concat " | TYPES " (:census::types-after-arrows binder))))))

(:wat::core::defn :census::walk [path <- :wat::core::String form <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::if (:wat::core::= (:census::head-of form) ":wat::core::defclause")
      (:wat::core::let
        [ch   (:wat::core::into [] (:wat::core::ast->children form))
         name (:wat::core::if (:wat::core::< (:wat::core::length ch) 2) "<anon>"
                (:wat::core::ast->source (:wat::core::nth ch 1)))
         ;; ⚠ AN ARM IS NOT "EVERY CHILD PAST THE NAME". A `defclause` may carry a SHARED return
         ;; type on its head line — `(defclause :p05::pick -> :i64 (arm) (arm))` — and a naive
         ;; `drop 2` then counts `->` and `:i64` as arms. That produced two bogus EMPTY-typed rows
         ;; and a phantom overlap on the first run of this census; the absurd row (`[] vs []`) is
         ;; what exposed it. An ARM is a child that HAS children AND whose first child is a BINDER
         ;; VECTOR — detected by that child's source carrying a `[`. `->` and a bare type keyword
         ;; have no children at all, so both fall out.
         arms (:wat::core::into []
                (:wat::core::filter
                  (:wat::core::fn [c <- :wat::WatAST] -> :wat::core::bool
                    (:wat::core::let [cc (:wat::core::into [] (:wat::core::ast->children c))]
                      (:wat::core::if (:wat::core::empty? cc)
                        false
                        (:wat::core::string::contains?
                          (:wat::core::ast->source (:wat::core::first cc)) "["))))
                  (:wat::core::into [] (:wat::core::drop ch 2))))]
        ;; No arm INDEX is emitted: the rows come out in declaration order, which IS the arm
        ;; order, and that order is the only thing first-match-wins dispatch depends on.
        (:wat::core::run!
          (:wat::core::fn [a <- :wat::WatAST] -> :wat::core::nil (:census::report-arm path name a))
          arms))
      nil)
    (:wat::core::run!
      (:wat::core::fn [c <- :wat::WatAST] -> :wat::core::nil (:census::walk path c))
      (:wat::core::into [] (:wat::core::ast->children form)))))

(:wat::core::defn :census::file [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::run!
    (:wat::core::fn [f <- :wat::WatAST] -> :wat::core::nil (:census::walk path f))
    (:wat::core::into []
      (:wat::core::ast->children
        (:wat::core::match (:wat::core::read-string (:wat::io::read-file path))
          ((:wat::core::ReadOutcome::Forms __forms) __forms)
          ((:wat::core::ReadOutcome::Malformed __cause)
            (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause)
              :wat::core::None :wat::core::None)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::run!
    (:wat::core::fn [p <- :wat::core::String] -> :wat::core::nil (:census::file p))
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

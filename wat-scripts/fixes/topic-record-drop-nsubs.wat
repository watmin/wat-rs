;; wat-scripts/fixes/topic-record-drop-nsubs.wat — excursus-001: the topic forgets its
;; subscriber count.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; `nsubs` on `:demo::topic::Record` is the last fossil of the expansion `e0c552bf0` removed
;; from `publish`: a `:durable` field supplied at every construction site and read by nothing,
;; duplicating a fact the WORKER owns correctly as `(:wat::core::count subs)`. This codemod
;; deletes the field and every kwarg that supplies it.
;;
;; ⛔ FORM, not token. `nsubs` is ALSO the worker's `let` binding
;; (`[nsubs (:wat::core::count subs)]`, sns-fanout.wat:425) and its references, and it is a
;; `let` binding / function parameter in several scratch-pad probes. A rule keyed on the
;; SYMBOL NAME alone breaks the worker. Both rules below key on PARENTAGE:
;;
;;   form 1  kwarg pair `:nsubs <value>`      keyword `:nsubs` whose enclosing list's head
;;                                            (index 0) is exactly `:demo::topic::Record`
;;                                            -> delete [end(prev sibling) .. end(value)]
;;                                            (so the separating whitespace goes with it)
;;   form 2  binder triple `nsubs <- :T`      SYMBOL `nsubs` inside the VECTOR that directly
;;                                            follows `:durable` in the list headed by
;;                                            `:wat::service::defservice` whose index-1 is
;;                                            `:demo::topic`
;;                                            -> delete [start(nsubs) .. start(next binder)]
;;   form 3  accessor `…::Record/nsubs`       SUBSUMED — it is the *value* of a form-1 pair
;;                                            (sns-fanout.wat:220), so deleting the pair takes
;;                                            it. The finder still reports it, so a survivor
;;                                            would be visible rather than silent.
;;
;; ⚠ Every line number in this header is PRE-migration (the corpus this fix was recorded
;; against, HEAD f9d1684ea). The `:durable` adjacency in form 2 is enforced by the APPLIER; the
;; finder rule is one join wider — see its own note for why.
;;
;; The two TOKEN-census rules (`token-nsubs-symbol`, `token-nsubs-keyword`) report EVERY leaf
;; named `nsubs` / `:nsubs`. They are the HYPOTHESIS, not the rewrite population: compare them
;; against the form rules to prove the form filter works, exactly as `alarm-after-to-delay.wat`
;; compares `token-after` against `alarm-ctor-after`.
;;
;; Deletion is SPAN-based (`fix-text-span-text` as old-text), which is the sanctioned door for
;; "delete a whole matched region": the subject of the edit genuinely IS the span, and the value
;; node may be a LIST (the `:220` accessor), so `fix-text-deletion-edit`'s ast-name length does
;; not reach it. Ranges are chosen to carry the separating whitespace, so no double space and no
;; blank line is left behind.
;;
;; Idempotent: after the rewrite there is no `:nsubs` keyword under a `:demo::topic::Record`
;; head and no `nsubs` symbol in the `:durable` vector, so a second run emits zero edits.
;;
;; NOT rewritten: comments (the tool walks the form tree) — prose mentioning `nsubs` is a
;; separate manual pass; the worker's `let` binding and its references; any `nsubs` bound in a
;; probe's own `let` or parameter list.
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>  -> :user::grep  (prints every Match, unapplied)
;;   `wat` <this file>         -> :user::main  (rewrites files in place)
;;
;; Usage — finder (the census):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/topic-record-drop-nsubs.wat
;;
;; Usage — dry-run:
;;   cp <file> /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/topic-record-drop-nsubs.wat
;;   diff <file> /tmp/pilot.wat
;;
;; Usage — apply (list EVERY path the finder named):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/topic-record-drop-nsubs.wat

;; ── finder ───────────────────────────────────────────────────────────────────

;; TOKEN census — every SYMBOL leaf named `nsubs`. Hypothesis, not the rewrite population:
;; the worker's binding at :425 and its references are in here and MUST survive.
(:wat::rete::defrule :tn::token-nsubs-symbol
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind) (:wat::rete::string::= ?k "symbol"))
         (:wat::grep::Named  (?id <- :id) (?n <- :name) (:wat::rete::string::= ?n "nsubs"))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "token-nsubs-symbol"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

;; TOKEN census — every KEYWORD leaf named `:nsubs`, whatever its parent.
(:wat::rete::defrule :tn::token-nsubs-keyword
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind) (:wat::rete::string::= ?k "keyword"))
         (:wat::grep::Named  (?id <- :id) (?n <- :name) (:wat::rete::string::= ?n ":nsubs"))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "token-nsubs-keyword"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

;; FORM 1 — a `:nsubs` keyword whose enclosing list is headed by `:demo::topic::Record`.
;; THIS is the rewrite population for the kwarg pairs.
(:wat::rete::defrule :tn::record-kwarg-nsubs
  :when [(:wat::grep::Node   (?id <- :id) (?p <- :parent) (?k <- :kind) (:wat::rete::string::= ?k "keyword"))
         (:wat::grep::Named  (?id <- :id) (?n <- :name) (:wat::rete::string::= ?n ":nsubs"))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::grep::Node   (?hid <- :id) (?p <- :parent) (?hi <- :index) (:wat::rete::i64::= ?hi 0))
         (:wat::grep::Named  (?hid <- :id) (?hn <- :name) (:wat::rete::string::= ?hn ":demo::topic::Record"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "record-kwarg-nsubs"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

;; FORM 3 — the accessor. Unique full name, so no parentage needed. Subsumed by form 1
;; (it is that pair's value); reported so a survivor cannot be silent.
(:wat::rete::defrule :tn::record-accessor-nsubs
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind) (:wat::rete::string::= ?k "keyword"))
         (:wat::grep::Named  (?id <- :id) (?n <- :name) (:wat::rete::string::= ?n ":demo::topic::Record/nsubs"))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "record-accessor-nsubs"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

;; FORM 2 — the `:durable` field binder. SYMBOL `nsubs` inside a VECTOR whose enclosing list is
;; headed by `:wat::service::defservice` with index-1 `:demo::topic`. The defservice parentage is
;; what keeps this off `:wat::core::let`'s vector at :425 — that is the whole rule.
;;
;; ⚠ This is deliberately the `alarm-after-to-delay.wat:78-97` shape EXACTLY — four `Node`
;; patterns, two of them sharing `?rec`/`?svc`. An earlier draft added a fifth (`:durable` at
;; index(vec)−1, via `(:wat::rete::i64::+ ?di 1 :undefined 0)`); it MATCHED correctly on
;; `sns-fanout.wat` but the four-way self-join on one parent is O(children⁴) and the process was
;; SIGKILLed (exit 137, OOM) on `wat-scripts/fanout/circuit.wat`. The `:durable` adjacency
;; therefore lives in the APPLIER (`:tn::defservice-edits`), which is stricter than this rule:
;; the finder may report a `nsubs` symbol in this defservice's `:ephemeral`/`:peers` vector, the
;; applier will not touch one. There is no such symbol in the corpus; the asymmetry is stated so
;; a future one is not silently rewritten.
(:wat::rete::defrule :tn::durable-binder-nsubs
  :when [(:wat::grep::Node   (?id <- :id) (?vec <- :parent) (?k <- :kind) (:wat::rete::string::= ?k "symbol"))
         (:wat::grep::Named  (?id <- :id) (?n <- :name) (:wat::rete::string::= ?n "nsubs"))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::grep::Node   (?vec <- :id) (?svc <- :parent) (?vk <- :kind) (:wat::rete::string::= ?vk "vector"))
         (:wat::grep::Node   (?hid <- :id) (?svc <- :parent) (?hi <- :index) (:wat::rete::i64::= ?hi 0))
         (:wat::grep::Named  (?hid <- :id) (?hn <- :name) (:wat::rete::string::= ?hn ":wat::service::defservice"))
         (:wat::grep::Node   (?sid <- :id) (?svc <- :parent) (?si <- :index) (:wat::rete::i64::= ?si 1))
         (:wat::grep::Named  (?sid <- :id) (?sn <- :name) (:wat::rete::string::= ?sn ":demo::topic"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "durable-binder-nsubs"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :tn))

;; ── applier ──────────────────────────────────────────────────────────────────

(:wat::core::defn :tn::empty-edits []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :tn::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :tn::sym-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "symbol")
    (:wat::core::ast-name n) ""))

;; cut — one deletion edit for the source region [from .. to). old-text is the VERBATIM slice:
;; the subject of this edit genuinely IS the span (see fix-text-span-text's own note), and the
;; deleted region spans whitespace and, at :220, a whole LIST node — neither has a name to
;; claim instead.
(:wat::core::defn :tn::cut
  [from <- :wat::core::i64  to <- :wat::core::i64  src <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple from (:wat::string::subs src from to) "")))

;; FORM 1 applier — for each `:nsubs` kwarg in a `:demo::topic::Record` construction, delete
;; from the END of the PRECEDING sibling through the END of the VALUE node. Taking the leading
;; whitespace with the pair is what keeps `(:demo::topic::Record :nsubs 3 :inbox-addr …)` from
;; becoming a double space and the multi-line ctor from keeping a blank, indented line.
;; `:nsubs` is a kwarg so it is never index 0; the guard says so rather than assuming it.
(:wat::core::defn :tn::record-kwarg-edits
  [ch    <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     i   <- :wat::core::i64]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::if
        (:wat::core::if (:wat::core::= (:tn::kw-name (:wat::core::nth ch i)) ":nsubs")
          (:wat::core::if (:wat::core::>= i 1)
            (:wat::core::< (:wat::core::+ i 1) (:wat::core::length ch))
            false)
          false)
        (:wat::core::concat acc
          (:tn::cut
            (:wat::fix::node-end-offset (:wat::core::nth ch (:wat::core::- i 1)) lines)
            (:wat::fix::node-end-offset (:wat::core::nth ch (:wat::core::+ i 1)) lines)
            src))
        acc))
    (:tn::empty-edits)
    (:wat::core::range 0 (:wat::core::length ch))))

;; FORM 2 applier — inside the `:durable` vector, delete the binder TRIPLE
;; `nsubs <- :wat::core::i64`. Range: from the symbol's start to the START of the NEXT binder's
;; symbol when there is one (so the newline + indent goes too), else to the END of the type.
;; The `<-` at i+1 is asserted, not assumed: a two-token `nsubs` in this vector is not a binder
;; and is left alone.
(:wat::core::defn :tn::durable-vec-edits
  [vch   <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     i   <- :wat::core::i64]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::if
        (:wat::core::if (:wat::core::= (:tn::sym-name (:wat::core::nth vch i)) "nsubs")
          (:wat::core::if (:wat::core::< (:wat::core::+ i 2) (:wat::core::length vch))
            (:wat::core::= (:tn::sym-name (:wat::core::nth vch (:wat::core::+ i 1))) "<-")
            false)
          false)
        (:wat::core::concat acc
          (:tn::cut
            (:wat::fix::node-start-offset (:wat::core::nth vch i) lines)
            (:wat::core::if (:wat::core::< (:wat::core::+ i 3) (:wat::core::length vch))
              (:wat::fix::node-start-offset (:wat::core::nth vch (:wat::core::+ i 3)) lines)
              (:wat::fix::node-end-offset (:wat::core::nth vch (:wat::core::+ i 2)) lines))
            src))
        acc))
    (:tn::empty-edits)
    (:wat::core::range 0 (:wat::core::length vch))))

;; the `:demo::topic` defservice form, by head AND service name — never `:wat::core::let`.
(:wat::core::defn :tn::topic-defservice?
  [ch <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
    false
    (:wat::core::if (:wat::core::= (:tn::kw-name (:wat::core::first ch)) ":wat::service::defservice")
      (:wat::core::= (:tn::kw-name (:wat::core::nth ch 1)) ":demo::topic")
      false)))

;; the vector that DIRECTLY follows `:durable` in the defservice form — not `:ephemeral`'s,
;; not `:peers`'s.
(:wat::core::defn :tn::defservice-edits
  [ch    <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     i   <- :wat::core::i64]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::if
        (:wat::core::if (:wat::core::= (:tn::kw-name (:wat::core::nth ch i)) ":durable")
          (:wat::core::< (:wat::core::+ i 1) (:wat::core::length ch))
          false)
        (:wat::core::let [v (:wat::core::nth ch (:wat::core::+ i 1))]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind v) "vector")
            (:wat::core::concat acc (:tn::durable-vec-edits (:wat::core::ast->children v) lines src))
            acc))
        acc))
    (:tn::empty-edits)
    (:wat::core::range 0 (:wat::core::length ch))))

(:wat::core::defn :tn::node-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [ch   (:wat::core::ast->children node)
                      here (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
                             (:wat::core::if (:wat::core::empty? ch)
                               (:tn::empty-edits)
                               (:wat::core::if (:wat::core::= (:tn::kw-name (:wat::core::first ch)) ":demo::topic::Record")
                                 (:tn::record-kwarg-edits ch lines src)
                                 (:wat::core::if (:tn::topic-defservice? ch)
                                   (:tn::defservice-edits ch lines src)
                                   (:tn::empty-edits))))
                             (:tn::empty-edits))]
      (:wat::core::concat here (:tn::seq-edits ch lines src)))
    (:tn::empty-edits)))

(:wat::core::defn :tn::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     it  <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:tn::node-edits it lines src)))
    (:tn::empty-edits)
    items))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     eds   (:tn::seq-edits forms lines src)
     ;; descending by offset — every splice is then behind the offsets not yet applied.
     sorted (:wat::core::sort
              (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                               b <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                -> :wat::core::bool
                (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
              eds)]
    (:wat::fix::fix-text-apply src sorted)))

(:wat::core::defn :tn::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[topic-record-drop-nsubs] " path))
        (:tn::apply-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:tn::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

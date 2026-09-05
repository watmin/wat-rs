;; 277-width-as-a-fold.wat — WIDTH IS A FACT, NOT A RULE. The repair for the stratification blocker.
;;
;; `[[277-width-fixpoint-probe]]` established that this rete REFUSES to derive width:
;;   "stratify: negation cycle detected — rule set is not stratifiable"
;; because a parent's width is an AGGREGATE over the very relation the rule derives, and stratified
;; evaluation forbids recursion through an aggregate. Isolation: the identical rule aggregating over
;; `Node` instead of over its own `Width` output runs fine. The blocker is precise, not my syntax.
;;
;; THE REPAIR: compute width the way `wat/grep.wat` already computes Node/Named/Span — by WALKING
;; THE TREE in ordinary wat, where a post-order fold has no stratification problem at all. Rules then
;; CONSUME width like they consume a span. The DESIGN's acceptance ("a new style rule is a new file
;; and nothing else") is untouched: width joins the fact base, it does not become rule logic.
;;
;; THE CONTROL, and it is the whole point of this file: for a form ALREADY on one line the DERIVED
;; width must EQUAL its actual span width. Any disagreement is printed. Silence is the proof.

(:wat::core::defn :wf::width [node <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::let [kids (:wat::core::ast->children node)]
    (:wat::core::if (:wat::core::empty? kids)
      ;; leaf — its own source text, verbatim
      (:wat::string::length (:wat::core::ast->source node))
      ;; interior — 2 delimiters + Σ children + (n-1) separators  ==  Σ + n + 1
      (:wat::core::+
        (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::i64 k <- :wat::WatAST] -> :wat::core::i64
            (:wat::core::+ acc (:wf::width k)))
          0 kids)
        (:wat::core::+ (:wat::core::length kids) 1)))))

;; walk every node; report ONLY a disagreement on a single-line form.
(:wat::core::defn :wf::check [node <- :wat::WatAST path <- :wat::core::String] -> :wat::core::i64
  (:wat::core::let
    ;; `wat/grep.wat` promises extent-of is the ONLY site that unwraps a span. Honour it.
    [x        (:wat::grep::extent-of node)
     line     (:wat::grep::Extent/line     x)
     end-line (:wat::grep::Extent/end-line x)
     col      (:wat::grep::Extent/col      x)
     end-col  (:wat::grep::Extent/end-col  x)
     kids     (:wat::core::ast->children node)
     mine     (:wat::core::if (:wat::core::= line end-line)
                (:wat::core::let [derived (:wf::width node)
                                  actual  (:wat::core::- end-col col)]
                  (:wat::core::if (:wat::core::= derived actual)
                    1000000
                    (:wat::core::do
                      (:wat::kernel::println (:wat::string::interpolate
                        "MISMATCH {p}:{l} derived={d} actual={a}"
                        :p path :l (:wat::i64::to-string line)
                        :d (:wat::i64::to-string derived) :a (:wat::i64::to-string actual)))
                      1000001)))
                0)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64 k <- :wat::WatAST] -> :wat::core::i64
        (:wat::core::+ acc (:wf::check k path)))
      mine kids)))

(:wat::core::defn :wf::run [path <- :wat::core::String] -> :wat::core::i64
  (:wat::core::match (:wat::core::read-string (:wat::io::read-file path))
    ((:wat::core::ReadOutcome::Forms forms)
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64 f <- :wat::WatAST] -> :wat::core::i64
          (:wat::core::+ acc (:wf::check f path)))
        0 (:wat::core::ast->children forms)))
    ((:wat::core::ReadOutcome::Malformed c)
      (:wat::kernel::assertion-failed! (:wat::core::Error/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :wf::report [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let [n       (:wf::run path)
                    checked (:wat::i64::quot n 1000000)
                    bad     (:wat::i64::rem  n 1000000)]
    (:wat::kernel::println (:wat::string::interpolate
      "{p}  single-line forms CHECKED={c}  MISMATCH={b}"
      :p path :c (:wat::i64::to-string checked) :b (:wat::i64::to-string bad)))))

;; ⛔ THE CONTROL MUST BE SEEN TO FIRE. A bare "0 mismatches" is indistinguishable from
;; "0 forms were examined" — the vacuous green. CHECKED is printed beside it, always.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wf::report "wat/io.wat")
    (:wf::report "wat/grep.wat")
    (:wf::report "wat/fix.wat")
    (:wf::report "wat/core.wat")
    (:wf::report "wat/service.wat")
    (:wf::report "wat/rete.wat")))

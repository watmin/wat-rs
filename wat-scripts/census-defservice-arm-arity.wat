;; wat-scripts/census-defservice-arm-arity.wat — FORM-AWARE census of defservice op-arm arities.
;;
;; WHY THIS EXISTS: the arity migration needs to know how many op arms exist and of what shape.
;; Regex answered that question THREE different ways in five minutes (52, 179, 44) because an arm
;; is a STRUCTURE, not a line: the first arm in every `:impls` vector begins `[(name …` rather than
;; ` (name …`, binder names vary, and `(make [self x])` — an extend-type method — looks identical to
;; an op arm on a line. [[feedback_validate_a_search_pattern_before_trusting_its_count]].
;;
;; This walks the parse tree instead: for each top-level `defservice`, find the `:impls` child,
;; take its children as the arms, and read each arm's PARAM VECTOR length. There is nothing to
;; positive-control because there is no pattern — the shape is read, not matched.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["a.wat" "b.wat"]\n' | cargo wat ./wat-scripts/census-defservice-arm-arity.wat

;; ── is this top-level form a defservice? ────────────────────────────────────────────────
(:wat::core::defn :user::defservice-form?
  [form <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::if (:wat::core::empty? ch)
      false
      (:wat::core::= (:wat::core::ast->source (:wat::core::first ch))
                     ":wat::service::defservice"))))

;; ── the children FOLLOWING the child whose source is `kw` (i.e. that section's value) ───
(:wat::core::defn :user::index-after-keyword
  [ch <- :wat::core::Vector<wat::WatAST>  kw <- :wat::core::String  i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length ch))
    -1
    (:wat::core::if (:wat::core::= (:wat::core::ast->source (:wat::core::nth ch i)) kw)
      (:wat::core::i64::+ i 1)
      (:user::index-after-keyword ch kw (:wat::core::i64::+ i 1)))))

;; ── one arm → its param-vector arity (0 if the arm has no param vector) ────────────────
(:wat::core::defn :user::arm-arity
  [arm <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::let [ch (:wat::core::ast->children arm)]
    (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
      0
      (:wat::core::length (:wat::core::ast->children (:wat::core::nth ch 1))))))

;; ── one arm → is it INTERNAL (leading `-` on the op name)? ─────────────────────────────
(:wat::core::defn :user::arm-internal?
  [arm <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [ch (:wat::core::ast->children arm)]
    (:wat::core::if (:wat::core::empty? ch)
      false
      (:wat::core::string::starts-with?
        (:wat::core::ast->source (:wat::core::first ch)) "-"))))

;; ── one defservice form → its arms ─────────────────────────────────────────────────────
(:wat::core::defn :user::arms-of
  [form <- :wat::WatAST] -> :wat::core::Vector<wat::WatAST>
  (:wat::core::let [ch  (:wat::core::ast->children form)
                    idx (:user::index-after-keyword ch ":impls" 0)]
    (:wat::core::if (:wat::core::i64::< idx 0)
      (:wat::core::Vector :wat::WatAST)
      (:wat::core::if (:wat::core::i64::>= idx (:wat::core::length ch))
        (:wat::core::Vector :wat::WatAST)
        (:wat::core::ast->children (:wat::core::nth ch idx))))))

;; ── report one arm as a line: "<file> <op> <arity> <internal?>" ────────────────────────
(:wat::core::defn :user::report-arms
  [arms <- :wat::core::Vector<wat::WatAST>  path <- :wat::core::String  i <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length arms))
    nil
    (:wat::core::let
      [arm  (:wat::core::nth arms i)
       ch   (:wat::core::ast->children arm)
       name (:wat::core::if (:wat::core::empty? ch) "?"
              (:wat::core::ast->source (:wat::core::first ch)))]
      (:wat::core::do
        (:wat::kernel::println
          (:wat::core::string::concat path
            (:wat::core::string::concat " "
              (:wat::core::string::concat name
                (:wat::core::string::concat " arity="
                  (:wat::core::string::concat
                    (:wat::core::i64::to-string (:user::arm-arity arm))
                    (:wat::core::if (:user::arm-internal? arm) " INTERNAL" " public")))))))
        (:user::report-arms arms path (:wat::core::i64::+ i 1))))))

(:wat::core::defn :user::census-forms
  [forms <- :wat::core::Vector<wat::WatAST>  path <- :wat::core::String  i <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length forms))
    nil
    (:wat::core::do
      (:wat::core::if (:user::defservice-form? (:wat::core::nth forms i))
        (:user::report-arms (:user::arms-of (:wat::core::nth forms i)) path 0)
        nil)
      (:user::census-forms forms path (:wat::core::i64::+ i 1)))))

(:wat::core::defn :user::census-each
  [paths <- :wat::core::Vector<wat::core::String>  i <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length paths))
    nil
    (:wat::core::let
      [path (:wat::core::nth paths i)
       tree (:wat::core::match (:wat::core::read-string (:wat::io::read-file path))
              ((:wat::core::ReadOutcome::Forms __forms) __forms)
              ((:wat::core::ReadOutcome::Malformed __cause)
                (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))]
      (:wat::core::do
        (:user::census-forms (:wat::core::ast->children tree) path 0)
        (:user::census-each paths (:wat::core::i64::+ i 1))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::census-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None))) 0))

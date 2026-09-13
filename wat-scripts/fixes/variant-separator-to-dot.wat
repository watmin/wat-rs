;; wat-scripts/fixes/variant-separator-to-dot.wat — arc 255 ③b-ii phase ②a
;; SCOPE: corpus
;; (DESIGN-the-flip-asks-843-times.md).
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; BRIEF: docs/arc/2026/06/255-builtin-registry/BRIEF-phase-two-the-codemod-and-its-dry-run.md
;; PRIOR ART: wat-scripts/fixes/bare-variant-to-qualified.wat (arc 296 N) — the exact INVERSE
;; migration (qualifies a bare `::`-spelled variant leaf up to its type's FQDN), and the proof
;; that `:wat::fix::rename-keyword-exact` is the right primitive here too: a variant-boundary
;; flip is a WHOLE-TOKEN keyword rename, not a prefix/boundary walk, so no rete rule set is
;; needed — a straight fold over exact-token renames, exactly that file's shape.
;;
;; Flips the `::` separator between a type and its variant to `.` iff the door
;; (program ∪ stdlib) says `:P` is an enum declaring `V`. A `::` keyword whose
;; `:P` is a declared enum and whose `V` is not one of its variants is REPORTED.
;; Keep `rename-keyword-exact` (defenum declaration slots are already safe in it)
;; and a substring prefilter.
;;
;; ⛔ DEFENUM DECLARATION SLOTS. `rename-keyword-exact` already treats a `defenum` variant-name
;; slot as a DECLARATION, never a use site (`wat/fix.wat`'s
;; `rename-exact-edits-defenum-variants`, built for exactly this hazard after 296 N RELAND 8 — a
;; whole-file keyword rename that once corrupted `Option`'s unit variant by rewriting its OWN
;; declared name as if it were a call site). That is this migration's STOP-1 risk, and it is
;; already closed in the shared primitive — not re-solved or re-guarded here.
;;
;; ── SHAPE ────────────────────────────────────────────────────────────────────────────────
;;   for each input path:
;;     tree := read-string (text := read-file path)
;;     fmap := :wat::fix::enum-fields on this program's forms + parent paths of every `::`
;;             keyword (stdlib via is-type?/type-of). Each `(:wat::core::forms …)` literal
;;             is its own program: ask the door on ITS children and collect pairs from
;;             that subtree only; the parent's map applies outside. Never merge.
;;     pairs := for each `::` keyword, if the current door has it as a variant
;;              (old, parent+"."+leaf); if `:P` is a known enum and `V` is not among
;;              its variants, print UNRESOLVED, no pair.
;;     hits := pairs whose OLD token appears in text     (:wat::string::contains? — a plain
;;             substring PREFILTER, not the rewrite's correctness boundary: rename-keyword-exact
;;             only ever touches a keyword AST leaf, so a false-positive "hit" — old text sitting
;;             inside a comment or string literal — costs one wasted parse, never a bad edit)
;;     if hits is empty -> skip: no read-file result is mutated, no write-file call, no printed
;;                          line — the file is left byte-identical and untouched
;;     else              -> fold rename-keyword-exact over hits, then write-file
;; The prefilter is load-bearing for RUNTIME (rename-keyword-exact calls read-string
;; internally; most files match a handful of pairs, not every `::` keyword).
;;
;; ── IDEMPOTENCE ──────────────────────────────────────────────────────────────────────────
;; After the rewrite the old token (`Type::Variant`) is gone from the file, replaced by
;; `Type.Variant`. A second run still asks the door (the enums have not changed) but
;; collect-keywords on a dotted leaf does not find that token in `fields` (keys are the
;; `::` form), and `known-enum?` on the `::`-split parent of a dotted name is not the
;; enum — so no pair is generated. Same no-op argument as bare-variant-to-qualified.wat.
;; Proved, not assumed: run twice on the /tmp copy and the second diff must be empty.
;;
;; ── NO CASCADE ───────────────────────────────────────────────────────────────────────────
;; Order-independence (the fold in apply-renames may hit pairs in any order) requires that no
;; pair's NEW token ever equal another pair's OLD token. Every NEW is parent+"."+leaf; every
;; OLD is parent+"::"+leaf. A NEW therefore cannot equal any OLD: the variant boundary
;; differs. Holds per file from the door, without a census pair-list intersection.
;;
;; Dry-run on a /tmp copy + diff, THEN the orchestrator lands it alongside the separator flip
;; (identifier.rs's compose_variant/decompose_variant + the 28 display strings — one commit,
;; because the tree cannot build between a dot-spelled corpus and a `::`-reading decomposer):
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/variant-separator-to-dot.wat

(:wat::core::defn :user::parent-path [vpath <- :wat::core::String] -> :wat::core::String
  (:wat::fix::parent-path vpath))

(:wat::core::defn :user::leaf-of [vpath <- :wat::core::String] -> :wat::core::String
  (:wat::fix::leaf-of vpath))

(:wat::core::defn :user::to-dot [nm <- :wat::core::String] -> :wat::core::String
  (:wat::string::concat (:user::parent-path nm) (:wat::string::concat "." (:user::leaf-of nm))))

(:wat::core::defn :user::conj-unique
  [acc <- (:wat::core::Vector :- [:wat::core::String])
   s   <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::vec::contains? acc s) acc (:wat::core::conj acc s)))

(:wat::core::defn :user::collect-keywords
  [acc  <- (:wat::core::Vector :- [:wat::core::String])
   node <- :wat::WatAST]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::forms")
    acc
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:wat::core::let [nm (:wat::core::ast-name node)]
        (:wat::core::if (:wat::core::> (:wat::core::length (:wat::string::split nm "::")) 1)
          (:wat::core::if (:wat::core::> (:wat::core::length (:wat::string::split nm "/")) 1)
            acc
            (:user::conj-unique acc nm))
          acc))
      (:wat::core::if (:wat::fix::structural? node)
        (:wat::core::foldl :user::collect-keywords acc (:wat::core::ast->children node))
        acc))))

(:wat::core::defn :user::parents-of
  [kws <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) kw <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:user::conj-unique acc (:user::parent-path kw)))
    (:wat::core::Vector :- [:wat::core::String])
    kws))

(:wat::core::defn :user::pairs-from-kws
  [kws  <- (:wat::core::Vector :- [:wat::core::String])
   fmap <- :wat::fix::EnumFields
   path <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn
      [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
       kw  <- :wat::core::String]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
      (:wat::core::match (:wat::fix::enum-fields-get fmap kw)
        [:wat::core::Option.Some {:value _}
          (:wat::core::conj acc (:wat::core::Tuple kw (:user::to-dot kw)))]
        [:wat::core::Option.None {}
          (:wat::core::do
            (:wat::core::if (:wat::fix::known-enum? fmap (:user::parent-path kw))
              (:wat::kernel::println
                (:wat::string::concat
                  "[variant-separator] UNRESOLVED "
                  (:wat::string::concat kw (:wat::string::concat " " path))))
              nil)
            acc)]))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
    kws))

;; hits-for — the prefilter: every pair whose OLD token is a substring of `text`.
(:wat::core::defn :user::hits-for
  [pairs <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
   text  <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? pairs)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
    (:wat::core::let [p  (:wat::core::first pairs)
                      tl (:wat::core::rest pairs)]
      (:wat::core::if (:wat::string::contains? text (:wat::core::first p))
        (:wat::core::concat
          (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])] p)
          (:user::hits-for tl text))
        (:user::hits-for tl text)))))

;; apply-renames — a fold over the (old,new) hits, never a nested staircase (24t's lesson,
;; namespace-bare-top-level-names.wat's own phrase for the same shape).
(:wat::core::defn :user::apply-renames
  [text <- :wat::core::String
   hits <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])]
  -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String p <- (:wat::core::Tuple :- [:wat::core::String :wat::core::String])] -> :wat::core::String
      (:wat::fix::rename-keyword-exact (:wat::core::first p) (:wat::core::second p) acc))
    text
    hits))

(:wat::core::defn :user::pairs-for-forms
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   path  <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::let
    [kws (:wat::core::foldl :user::collect-keywords
           (:wat::core::Vector :- [:wat::core::String]) forms)
     fmap (:wat::fix::enum-fields "variant-separator" path forms (:user::parents-of kws))]
    (:user::pairs-from-kws kws fmap path)))

(:wat::core::defn :user::nested-pairs
  [acc  <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
   node <- :wat::WatAST
   path <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::forms")
    (:wat::core::foldl
      (:wat::core::fn
        [a <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
         n <- :wat::WatAST]
        -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
        (:user::nested-pairs a n path))
      (:wat::core::concat acc (:user::pairs-for-forms (:wat::fix::forms-children node) path))
      (:wat::core::ast->children node))
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl
        (:wat::core::fn
          [a <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
           n <- :wat::WatAST]
          -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
          (:user::nested-pairs a n path))
        acc
        (:wat::core::ast->children node))
      acc)))

(:wat::core::defn :user::convert-one
  [path <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let
    [src  (:wat::io::read-file path)
     tree (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome.Forms {:forms f} f]
             [:wat::core::ReadOutcome.Malformed {:cause c}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))])
     top (:user::pairs-for-forms (:wat::core::ast->children tree) path)
     pairs (:user::nested-pairs top tree path)
     hits  (:user::hits-for pairs src)]
    (:wat::core::if (:wat::core::empty? hits)
      nil
      (:wat::core::do
        (:wat::io::write-file path (:user::apply-renames src hits))
        (:wat::kernel::println (:wat::string::concat "[variant-separator-to-dot] " path))))))

(:wat::core::defn :user::convert-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:user::convert-one (:wat::core::first paths))
      (:user::convert-each (:wat::core::rest paths)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
             [:wat::kernel::ReadlnOutcome.Eof {}
               (:wat::kernel::assertion-failed! :message "readln: end of input")]
             [:wat::kernel::ReadlnOutcome.Stopped {}
               (:wat::kernel::assertion-failed! :message "readln: stop requested")])]
    (:user::convert-each paths)))

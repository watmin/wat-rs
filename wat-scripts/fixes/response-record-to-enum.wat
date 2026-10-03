;; wat-scripts/fixes/response-record-to-enum.wat — arc 278 #16 Stone 16.1 (ruling A) migration codemod.
;; SCOPE: corpus
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; Ruling A: every serviceable op-Response is an outcome ENUM carrying `RequestTooLarge{bytes,cap}`;
;; records-as-Responses are retired for services. This migrates, per file:
;;
;;   (a) the op-Response DECL:  (:wat::core::defrecord :T [FIELDS])
;;        ->  (:wat::core::defenum :T :wat::enum::Pure :Ok [FIELDS]
;;              :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])
;;   (b) every CONSTRUCTION:    (:T :field v)          ->  (:T::Ok v)
;;   (c) every FIELD-ACCESS:    (:T/field EXPR)        ->  a match:
;;        (:wat::core::match EXPR -> <field-type>
;;          ((:T::Ok field) field)
;;          ((:T::RequestTooLarge bytes cap)
;;            (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None)))
;;
;; WHICH records are op-Responses is DISCOVERED, not hardcoded: a record `:T` is an op-Response iff
;; its name also appears as a `-> :T` return type in the same file (the surface feature return). This
;; gate is exact — a REQUEST record, a `Peer'` surface handle, and the feature-call `:Surface/op` all
;; have names NOT in the response set, so they (and their ctors/accessors) are left byte-untouched.
;; Requests keep record semantics; only serviceable Responses flip. Single-field Responses only (all
;; op-Responses in this corpus are single-field); a multi-field Response record is left for a hand-fix.
;;
;; Comment/format faithful (span edits via fix-text-apply). Idempotent (re-run = 0 edits: a defenum is
;; no longer a defrecord, an `::Ok` ctor head is no longer the bare `:T`, a `match` is no longer `:T/f`).
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat-scripts/probes/arc-278/s2s-parent-echo.wat" ...]\n' \
;;     | cargo wat ./wat-scripts/fixes/response-record-to-enum.wat

;; ── small helpers ────────────────────────────────────────────────────────────
(:wat::core::defn :user::strip-params [name <- wat.type/String] -> wat.type/String
  (:wat::core::first (:wat::string::split name "<")))

(:wat::core::defn :user::start-off [n <- wat.type/AST  lines <- (wat.type/Vector :- [wat.type/String])]
  -> wat.type/i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- wat.type/AST  lines <- (wat.type/Vector :- [wat.type/String])]
  -> wat.type/i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- wat.type/AST] -> wat.type/String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; ── DISCOVERY: return-type name set (walk for a `->` symbol followed by a keyword) ──
(:wat::core::defn :user::returns-in-children
  [ch <- (wat.type/Vector :- [wat.type/AST])  acc <- (wat.type/HashSet :- [wat.type/String])]
  -> (wat.type/HashSet :- [wat.type/String])
  (:wat::core::foldl
    (:wat::core::fn [s <- (wat.type/HashSet :- [wat.type/String])  i <- wat.type/i64]
      -> (wat.type/HashSet :- [wat.type/String])
      (:wat::core::let
        [cur  (:wat::core::Option/expect (:wat::core::get ch i) "returns cur")
         nxt  (:wat::core::get ch (:wat::core::+ i 1))]
        (:wat::core::match nxt 
          [:wat::core::Option.None {} s]
          [:wat::core::Option.Some {:value nn}
            (:wat::core::if
              (:wat::core::if (:wat::core::= (:wat::core::ast-kind cur) "symbol")
                (:wat::core::if (:wat::core::= (:wat::core::ast-name cur) "->")
                  (:wat::core::= (:wat::core::ast-kind nn) "keyword") false) false)
              (:wat::core::conj s (:user::strip-params (:wat::core::ast-name nn)))
              s)])))
    acc
    (:wat::core::range 0 (:wat::core::length ch))))

(:wat::core::defn :user::collect-returns
  [node <- wat.type/AST  acc <- (wat.type/HashSet :- [wat.type/String])]
  -> (wat.type/HashSet :- [wat.type/String])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::foldl
        (:wat::core::fn [a <- (wat.type/HashSet :- [wat.type/String]) n <- wat.type/AST]
          -> (wat.type/HashSet :- [wat.type/String])
          (:user::collect-returns n a))
        (:user::returns-in-children ch acc) ch))
    acc))

;; ── DISCOVERY: defrecord map  name -> (field-name, field-type)  (single-field only) ──
(:wat::core::defn :user::add-record
  [m <- (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
   node <- wat.type/AST]
  -> (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        m
        (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::defrecord")
          (:wat::core::let
            [tyname (:user::strip-params (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch 1) "rec ty")))
             fv     (:wat::core::Option/expect (:wat::core::get ch 2) "rec fv")]
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind fv) "vector")
              (:wat::core::let [fch (:wat::core::ast->children fv)]
                (:wat::core::if (:wat::core::= (:wat::core::length fch) 3)
                  (:wat::core::assoc m tyname
                    (wat.type/Tuple :- [wat.type/String wat.type/String]
                      (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get fch 0) "fn"))
                      (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get fch 2) "ft"))))
                  m))
              m))
          m)))
    m))

(:wat::core::defn :user::collect-records
  [node <- wat.type/AST
   m <- (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])]
  -> (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::foldl
      (:wat::core::fn [mm <- (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
                       c <- wat.type/AST]
        -> (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
        (:user::collect-records c mm))
      (:user::add-record m node)
      (:wat::core::ast->children node))
    (:user::add-record m node)))

;; respmap = records whose name is also a return type. Membership gates every rewrite below.
(:wat::core::defn :user::resp-map
  [forms <- (wat.type/Vector :- [wat.type/AST])]
  -> (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
  (:wat::core::let
    [recs (:wat::core::foldl
            (:wat::core::fn [m <- (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])]) f <- wat.type/AST]
              -> (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
              (:user::collect-records f m))
            (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
            forms)
     rets (:wat::core::foldl
            (:wat::core::fn [a <- (wat.type/HashSet :- [wat.type/String]) n <- wat.type/AST]
              -> (wat.type/HashSet :- [wat.type/String])
              (:user::collect-returns n a))
            (wat.type/HashSet :- [wat.type/String]) forms)]
    (:wat::core::foldl
      (:wat::core::fn [m <- (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])]) k <- wat.type/String]
        -> (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
        (:wat::core::if (:wat::core::contains? rets k)
          (:wat::core::assoc m k (:wat::core::Option/expect (:wat::core::get recs k) "resp"))
          m))
      (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
      (:wat::core::keys recs))))

;; ── EDITS ──────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::defrecord-edits
  [ch <- (wat.type/Vector :- [wat.type/AST])  lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let
    [head (:wat::core::Option/expect (:wat::core::get ch 0) "dr head")
     ty   (:wat::core::Option/expect (:wat::core::get ch 1) "dr ty")
     fv   (:wat::core::Option/expect (:wat::core::get ch 2) "dr fv")
     h0   (:user::start-off head lines)]
    ;; old-text = (ast-name head) — the rule's own belief; NEVER span text (a rename).
    (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
      (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] h0 (:wat::core::ast-name head) ":wat::core::defenum")
      (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] (:user::end-off ty lines) "" " :wat::enum::Pure :Ok")
      (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] (:user::end-off fv lines) ""
        " :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]"))))

;; op-Response construction is single-field: kwargs `(:T :field value)` (>=3 children) OR positional
;; `(:T value)` (2 children). BOTH become `(:T::Ok value)`; only the kwargs form has a `:field ` to
;; delete. (Positional single-arg construction is the corner the tests/ arc-170 negatives exposed.)
(:wat::core::defn :user::ctor-edits
  [ch <- (wat.type/Vector :- [wat.type/AST])  src <- wat.type/String  lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let
    [head (:wat::core::Option/expect (:wat::core::get ch 0) "ct head")]
    (:wat::core::if (:wat::core::>= (:wat::core::length ch) 3)
      (:wat::core::let
        [fkw (:wat::core::Option/expect (:wat::core::get ch 1) "ct fkw")
         val (:wat::core::Option/expect (:wat::core::get ch 2) "ct val")
         fs  (:user::start-off fkw lines)
         ;; old-text = fix-text-span-text from fkw's start to val's start — sanctioned:
         ;; this deletes the GAP between two independently-located node boundaries (the
         ;; field keyword plus its trailing whitespace), not a rename; there is no
         ;; separate name-based claim about that whitespace to diverge from it.
         gap-text (:wat::fix::fix-text-span-text (:wat::core::ast-span fkw) (:wat::core::ast-span val) lines src)]
        (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
          (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] (:user::end-off head lines) "" "::Ok")
          (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] fs gap-text "")))
      (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
        (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] (:user::end-off head lines) "" "::Ok")))))

(:wat::core::defn :user::field-edits
  [ch <- (wat.type/Vector :- [wat.type/AST])  prefix <- wat.type/String  field <- wat.type/String
   ftype <- wat.type/String  lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let
    [head (:wat::core::Option/expect (:wat::core::get ch 0) "fa head")
     expr (:wat::core::Option/expect (:wat::core::get ch 1) "fa expr")
     h0   (:user::start-off head lines)
     arms (:wat::string::concat " -> " ftype
            (:wat::string::concat "\n  ((" prefix
              (:wat::string::concat "::Ok " field
                (:wat::string::concat ") " field
                  (:wat::string::concat ")\n  ((" prefix
                    (:wat::string::concat "::RequestTooLarge bytes cap)\n    (:wat::kernel::assertion-failed! \"unexpected RequestTooLarge\" :wat::core::None :wat::core::None))"
                      ""))))))]
    ;; old-text = (ast-name head) — the rule's own belief; NEVER span text (a rename).
    (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
      (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] h0 (:wat::core::ast-name head) ":wat::core::match")
      (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] (:user::end-off expr lines) "" arms))))

;; walk one node → its edits + descendants'.
(:wat::core::defn :user::node-edits
  [node <- wat.type/AST
   rm <- (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
   src <- wat.type/String
   lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
        (:wat::core::let
          [hname (:user::kw-name (:wat::core::first ch))
           tyname (:wat::core::if (:wat::core::>= (:wat::core::length ch) 2)
                    (:user::strip-params (:user::kw-name (:wat::core::Option/expect (:wat::core::get ch 1) "h1"))) "")
           prefix (:wat::core::first (:wat::string::split hname "/"))
           this
           (:wat::core::if
             (:wat::core::if (:wat::core::= hname ":wat::core::defrecord")
               (:wat::core::contains? rm tyname) false)
             (:user::defrecord-edits ch lines)
             (:wat::core::if (:wat::core::contains? rm hname)
               (:user::ctor-edits ch src lines)
               (:wat::core::if
                 (:wat::core::if (:wat::string::contains? hname "/")
                   (:wat::core::contains? rm prefix) false)
                 (:wat::core::let [ft (:wat::core::Option/expect (:wat::core::get rm prefix) "ft")]
                   (:user::field-edits ch prefix (:wat::core::first ft) (:wat::core::second ft) lines))
                 (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]))))]
          (:wat::core::concat this (:user::seq-edits ch rm src lines)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) rm src lines)
      (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]))))

(:wat::core::defn :user::seq-edits
  [items <- (wat.type/Vector :- [wat.type/AST])
   rm <- (wat.type/HashMap :- [wat.type/String (wat.type/Tuple :- [wat.type/String wat.type/String])])
   src <- wat.type/String
   lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]) it <- wat.type/AST]
      -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
      (:wat::core::concat acc (:user::node-edits it rm src lines)))
    (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
    items))

;; ── per-file migrate ─────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- wat.type/String] -> wat.type/String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))]))
     rm    (:user::resp-map forms)
     eds   (:user::seq-edits forms rm src lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each [paths <- (wat.type/Vector :- [wat.type/String])] -> wat.type/nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[response->enum] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))

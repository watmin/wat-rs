;; wat/fmt.wat — layout engine: Break + a DUMB emitter. Rules assert Breaks; this file
;; holds no style opinion. Arc 277 STONE indent-is-structural.
;;
;; A node with a Break starts a new line. `:block` indents one level (2) from its
;; form's indent; `:align` sits one past the container's emitted opening delimiter.
;; A line comment PINS A NEWLINE after itself. Spans locate comments; they never
;; decide an indent.

;; kind is a String, not a keyword: rete RHS may insert a string literal but
;; refuses a keyword literal (`RhsUnresolvableOperand`). `"block"` | `"align"`.
(:wat::core::defrecord :wat::fmt::Break
  [id   <- :wat::core::i64
   kind <- :wat::core::String])

(:wat::core::defrecord :wat::fmt::Comment
  [text     <- :wat::core::String
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

;; A rule claims exactly the node it dispatched on and positions that node's
;; immediate children. The fallback (R11) fires where the parent is unclaimed.
;; The wall: applying a Break for X requires X's parent to be owned — Claim
;; (a specific rule) or Fallback (R11). R11 cannot assert Claim: that is
;; `not Claim -> Claim` and it races the per-child Breaks.
(:wat::core::defrecord :wat::fmt::Claim
  [form <- :wat::core::i64])

(:wat::core::defrecord :wat::fmt::Fallback
  [node <- :wat::core::i64])

;; Vertical separation, a different axis from Break.kind. A node may carry both.
(:wat::core::defrecord :wat::fmt::BlankBefore
  [id <- :wat::core::i64])

(:wat::core::defrecord :wat::fmt::Acc
  [out      <- :wat::core::String
   next-id  <- :wat::core::i64
   comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])
   col      <- :wat::core::i64])

(:wat::rete::defquery :wat::fmt::q-break
  :params []
  :when [(?b <- :wat::fmt::Break)])

(:wat::rete::defquery :wat::fmt::q-claim
  :params []
  :when [(?c <- :wat::fmt::Claim)])

(:wat::rete::defquery :wat::fmt::q-fallback
  :params []
  :when [(?f <- :wat::fmt::Fallback)])

(:wat::rete::defquery :wat::fmt::q-blank
  :params []
  :when [(?bl <- :wat::fmt::BlankBefore)])

(:wat::core::defn :wat::fmt::spaces [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::i64::<= n 0)
    ""
    (:wat::string::concat " " (:wat::fmt::spaces (:wat::i64::- n 1)))))

(:wat::core::defn :wat::fmt::ensure-nl [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::empty? s)
    s
    (:wat::core::if (:wat::string::ends-with? s "\n")
      s
      (:wat::string::concat s "\n"))))

;; Column of the next write, given the previous column and a suffix just appended.
;; Derived from EMITTED text, never from a source span.
(:wat::core::defn :wat::fmt::col-after
  [col <- :wat::core::i64  s <- :wat::core::String]
  -> :wat::core::i64
  (:wat::core::if (:wat::string::empty? s)
    col
    (:wat::core::if (:wat::string::ends-with? s "\n")
      0
      (:wat::core::if (:wat::string::contains? s "\n")
        (:wat::core::let [lines (:wat::string::split s "\n")
                          n     (:wat::core::length lines)]
          (:wat::string::length (:wat::core::nth lines (:wat::i64::- n 1))))
        (:wat::i64::+ col (:wat::string::length s))))))

(:wat::core::defn :wat::fmt::write
  [acc <- :wat::fmt::Acc  s <- :wat::core::String]
  -> :wat::fmt::Acc
  (:wat::fmt::Acc
    :out      (:wat::string::concat (:wat::fmt::Acc/out acc) s)
    :next-id  (:wat::fmt::Acc/next-id acc)
    :comments (:wat::fmt::Acc/comments acc)
    :col      (:wat::fmt::col-after (:wat::fmt::Acc/col acc) s)))

(:wat::core::defn :wat::fmt::write-nl [acc <- :wat::fmt::Acc] -> :wat::fmt::Acc
  (:wat::core::let [s  (:wat::fmt::Acc/out acc)
                    s2 (:wat::fmt::ensure-nl s)]
    (:wat::core::if (:wat::core::= s s2)
      acc
      (:wat::fmt::Acc
        :out      s2
        :next-id  (:wat::fmt::Acc/next-id acc)
        :comments (:wat::fmt::Acc/comments acc)
        :col      0))))

(:wat::core::defn :wat::fmt::comment-before?
  [c <- :wat::fmt::Comment  line <- :wat::core::i64  col <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::< (:wat::fmt::Comment/line c) line)
    true
    (:wat::core::if (:wat::i64::= (:wat::fmt::Comment/line c) line)
      (:wat::i64::<= (:wat::fmt::Comment/col c) col)
      false)))

(:wat::core::defn :wat::fmt::flush-comments
  [acc <- :wat::fmt::Acc  line <- :wat::core::i64  col <- :wat::core::i64  indent <- :wat::core::i64]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::core::empty? (:wat::fmt::Acc/comments acc))
    acc
    (:wat::core::let [c (:wat::core::first (:wat::fmt::Acc/comments acc))]
      (:wat::core::if (:wat::fmt::comment-before? c line col)
        (:wat::fmt::flush-comments
          (:wat::core::let [written (:wat::fmt::write
                                      (:wat::fmt::write
                                        (:wat::fmt::write-nl acc)
                                        (:wat::fmt::spaces indent))
                                      (:wat::string::concat (:wat::fmt::Comment/text c) "\n"))]
            (:wat::fmt::Acc
              :out      (:wat::fmt::Acc/out written)
              :next-id  (:wat::fmt::Acc/next-id written)
              :comments (:wat::core::rest (:wat::fmt::Acc/comments acc))
              :col      (:wat::fmt::Acc/col written)))
          line col indent)
        acc))))

(:wat::core::defn :wat::fmt::open-of [kind <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= kind "list") "("
    (:wat::core::if (:wat::core::= kind "vector") "["
      (:wat::core::if (:wat::core::= kind "set") "#{"
        "{"))))

(:wat::core::defn :wat::fmt::close-of [kind <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= kind "list") ")"
    (:wat::core::if (:wat::core::= kind "vector") "]"
      "}")))

(:wat::core::defn :wat::fmt::pad-break
  [acc      <- :wat::fmt::Acc
   bk       <- :wat::core::String
   indent   <- :wat::core::i64
   open-col <- :wat::core::i64]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::core::or (:wat::core::= bk "block")
                                 (:wat::core::= bk "align"))
    (:wat::core::let [n (:wat::core::if (:wat::core::= bk "block")
                        (:wat::i64::+ indent 2)
                        (:wat::i64::+ open-col 1))]
      (:wat::fmt::write (:wat::fmt::write-nl acc) (:wat::fmt::spaces n)))
    (:wat::kernel::assertion-failed!
      "fmt: Break.kind must be block or align"
      :wat::core::None
      :wat::core::None)))

(:wat::core::defn :wat::fmt::claimed?
  [claims    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   parent-id <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::= parent-id 0)
    true
    (:wat::core::match (:wat::core::get claims parent-id)
      ((:wat::core::Some _) true)
      (:wat::core::None false))))

(:wat::core::defn :wat::fmt::apply-blank
  [acc    <- :wat::fmt::Acc
   id     <- :wat::core::i64
   blanks <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])]
  -> :wat::fmt::Acc
  (:wat::core::match (:wat::core::get blanks id)
    ((:wat::core::Some _)
      (:wat::fmt::write (:wat::fmt::write-nl acc) "\n"))
    (:wat::core::None acc)))

(:wat::core::defn :wat::fmt::apply-break
  [acc       <- :wat::fmt::Acc
   bk        <- :wat::core::String
   indent    <- :wat::core::i64
   open-col  <- :wat::core::i64
   id        <- :wat::core::i64
   parent-id <- :wat::core::i64
   claims    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::fmt::claimed? claims parent-id)
    (:wat::fmt::pad-break acc bk indent open-col)
    (:wat::kernel::assertion-failed!
      (:wat::string::interpolate
        "fmt: rule positioned a grandchild — node {n}'s parent is unclaimed"
        :n (:wat::i64::to-string id))
      :wat::core::None
      :wat::core::None)))

;; Child 1 is the symbol/keyword `:-` and child 2 is a vector. Shared by
;; type DECLARATIONS (arity 3, atomic) and CONSTRUCTORS (arity > 3, glue then explode).
(:wat::core::defn :wat::fmt::colon-args?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind node) "list"))
    false
    (:wat::core::let [kids (:wat::core::ast->children node)]
      (:wat::core::if (:wat::i64::< (:wat::core::length kids) 3)
        false
        (:wat::core::let [c1 (:wat::core::nth kids 1)
                          c2 (:wat::core::nth kids 2)
                          k1 (:wat::core::ast-kind c1)]
          (:wat::core::if (:wat::core::not (:wat::core::or (:wat::core::= k1 "symbol")
                                                          (:wat::core::= k1 "keyword")))
            false
            (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-name c1) ":-"))
              false
              (:wat::core::= (:wat::core::ast-kind c2) "vector"))))))))

(:wat::core::defn :wat::fmt::type-application?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::fmt::colon-args? node))
    false
    (:wat::i64::= (:wat::core::length (:wat::core::ast->children node)) 3)))

(:wat::core::defn :wat::fmt::type-constructor?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::fmt::colon-args? node))
    false
    (:wat::i64::> (:wat::core::length (:wat::core::ast->children node)) 3)))

(:wat::core::defn :wat::fmt::subtree-size
  [node <- :wat::WatAST]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::not (:wat::grep::structural? node))
    1
    (:wat::core::foldl
      (:wat::core::fn [n <- :wat::core::i64  child <- :wat::WatAST] -> :wat::core::i64
        (:wat::i64::+ n (:wat::fmt::subtree-size child)))
      1
      (:wat::core::ast->children node))))

(:wat::core::defn :wat::fmt::count-colon-args
  [node <- :wat::WatAST]
  -> :wat::core::i64
  (:wat::core::let [here (:wat::core::if (:wat::fmt::colon-args? node) 1 0)]
    (:wat::core::if (:wat::core::not (:wat::grep::structural? node))
      here
      (:wat::core::foldl
        (:wat::core::fn [n <- :wat::core::i64  child <- :wat::WatAST] -> :wat::core::i64
          (:wat::i64::+ n (:wat::fmt::count-colon-args child)))
        here
        (:wat::core::ast->children node)))))

(:wat::core::defn :wat::fmt::count-type-apps
  [node <- :wat::WatAST]
  -> :wat::core::i64
  (:wat::core::let [here (:wat::core::if (:wat::fmt::type-application? node) 1 0)]
    (:wat::core::if (:wat::core::not (:wat::grep::structural? node))
      here
      (:wat::core::foldl
        (:wat::core::fn [n <- :wat::core::i64  child <- :wat::WatAST] -> :wat::core::i64
          (:wat::i64::+ n (:wat::fmt::count-type-apps child)))
        here
        (:wat::core::ast->children node)))))

(:wat::core::defn :wat::fmt::emit-kids
  [acc       <- :wat::fmt::Acc
   kids      <- (:wat::core::Vector :- [:wat::WatAST])
   i         <- :wat::core::i64
   ctor?     <- :wat::core::bool
   breaks    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
   claims    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   blanks    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   indent    <- :wat::core::i64
   open-col  <- :wat::core::i64
   parent-id <- :wat::core::i64]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::i64::>= i (:wat::core::length kids))
    acc
    (:wat::core::let
      [child (:wat::core::nth kids i)
       o     (:wat::fmt::Acc/out acc)
       first-kid? (:wat::core::or
                    (:wat::string::ends-with? o "(")
                    (:wat::core::or
                      (:wat::string::ends-with? o "[")
                      (:wat::core::or
                        (:wat::string::ends-with? o "{")
                        (:wat::string::ends-with? o "\n"))))
       acc2 (:wat::fmt::emit-node acc child breaks claims blanks indent open-col first-kid? parent-id
               (:wat::core::if ctor? (:wat::i64::= i 2) false))]
      (:wat::fmt::emit-kids acc2 kids (:wat::i64::+ i 1) ctor? breaks claims blanks indent open-col parent-id))))

(:wat::core::defn :wat::fmt::emit-node
  [acc        <- :wat::fmt::Acc
   node       <- :wat::WatAST
   breaks     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
   claims     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   blanks     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   indent     <- :wat::core::i64
   open-col   <- :wat::core::i64
   first?     <- :wat::core::bool
   parent-id  <- :wat::core::i64
   force-leaf <- :wat::core::bool]
  -> :wat::fmt::Acc
  (:wat::core::let
    [id        (:wat::fmt::Acc/next-id acc)
     node-kind (:wat::core::ast-kind node)
     x         (:wat::grep::extent-of node)
     src-line  (:wat::grep::Extent/line x)
     src-col   (:wat::grep::Extent/col x)
     br        (:wat::core::get breaks id)
     acc-b     (:wat::fmt::Acc
                 :out      (:wat::fmt::Acc/out acc)
                 :next-id  (:wat::i64::+ id 1)
                 :comments (:wat::fmt::Acc/comments acc)
                 :col      (:wat::fmt::Acc/col acc))
     acc-bl    (:wat::fmt::apply-blank acc-b id blanks)
     acc-pad   (:wat::core::match br
                 ((:wat::core::Some bk)
                   (:wat::fmt::apply-break acc-bl bk indent open-col id parent-id claims))
                 (:wat::core::None
                   (:wat::core::if first?
                     acc-bl
                     (:wat::core::if (:wat::string::empty? (:wat::fmt::Acc/out acc-bl))
                       acc-bl
                       (:wat::fmt::write acc-bl " ")))))
     this-indent (:wat::fmt::Acc/col acc-pad)
     acc1        (:wat::fmt::flush-comments acc-pad src-line src-col this-indent)]
    (:wat::core::if (:wat::core::or (:wat::fmt::type-application? node) force-leaf)
      (:wat::core::let
        [acc2 (:wat::fmt::write acc1 (:wat::core::ast->source node))
         acc3 (:wat::fmt::Acc
                :out      (:wat::fmt::Acc/out acc2)
                :next-id  (:wat::i64::+ id (:wat::fmt::subtree-size node))
                :comments (:wat::fmt::Acc/comments acc2)
                :col      (:wat::fmt::Acc/col acc2))]
        (:wat::fmt::flush-comments acc3
          (:wat::grep::Extent/end-line x)
          (:wat::grep::Extent/end-col x)
          this-indent))
    (:wat::core::if (:wat::grep::structural? node)
      (:wat::core::let
        [this-open (:wat::fmt::Acc/col acc1)
         acc2      (:wat::fmt::write acc1 (:wat::fmt::open-of node-kind))
         kids      (:wat::core::ast->children node)
         acc3      (:wat::fmt::emit-kids acc2 kids 0
                     (:wat::fmt::type-constructor? node)
                     breaks claims blanks this-indent this-open id)
         acc4 (:wat::fmt::write acc3 (:wat::fmt::close-of node-kind))]
        (:wat::fmt::flush-comments acc4
          (:wat::grep::Extent/end-line x)
          (:wat::grep::Extent/end-col x)
          this-indent))
      (:wat::core::let
        [acc2 (:wat::fmt::write acc1 (:wat::core::ast->source node))]
        (:wat::fmt::flush-comments acc2
          (:wat::grep::Extent/end-line x)
          (:wat::grep::Extent/end-col x)
          this-indent))))))

(:wat::core::defn :wat::fmt::emit
  [forms    <- :wat::WatAST
   comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])
   breaks   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
   claims   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   blanks   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])]
  -> :wat::core::String
  (:wat::core::let
    [top  (:wat::core::ast->children forms)
     acc0 (:wat::fmt::Acc :out "" :next-id 1 :comments comments :col 0)
     acc1 (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::fmt::Acc  form <- :wat::WatAST] -> :wat::fmt::Acc
              (:wat::core::let [acc-nl (:wat::fmt::write-nl acc)]
                (:wat::fmt::emit-node acc-nl form breaks claims blanks 0 0 true 0 false)))
            acc0
            top)
     acc2 (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::fmt::Acc  c <- :wat::fmt::Comment] -> :wat::fmt::Acc
              (:wat::fmt::write
                (:wat::fmt::write-nl acc)
                (:wat::string::concat (:wat::fmt::Comment/text c) "\n")))
            acc1
            (:wat::fmt::Acc/comments acc1))]
    (:wat::fmt::Acc/out acc2)))

(:wat::core::defn :wat::fmt::breaks-map
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
      (:wat::core::let [b (:wat::core::Option/expect
                            (:wat::map::get binding "?b")
                            "fmt::breaks-map: no ?b")
                        id (:wat::fmt::Break/id b)
                        k  (:wat::fmt::Break/kind b)]
        (:wat::core::match (:wat::core::get m id)
          (:wat::core::None
            (:wat::hashmap::assoc m id k))
          ((:wat::core::Some prev)
            (:wat::core::if (:wat::core::= prev k)
              (:wat::hashmap::assoc m id k)
              (:wat::kernel::assertion-failed!
                (:wat::string::interpolate
                  "fmt: conflicting Breaks for node {n} — {a} vs {b}"
                  :n (:wat::i64::to-string id)
                  :a prev
                  :b k)
                :wat::core::None
                :wat::core::None))))))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
    (:wat::rete::query session (:wat::fmt::q-break))))

(:wat::core::defn :wat::fmt::claims-set
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::core::let [c (:wat::core::Option/expect
                            (:wat::map::get binding "?c")
                            "fmt::claims-set: no ?c")]
        (:wat::hashmap::assoc m (:wat::fmt::Claim/form c) true)))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
    (:wat::rete::query session (:wat::fmt::q-claim))))

(:wat::core::defn :wat::fmt::owned-set
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::core::let [f (:wat::core::Option/expect
                            (:wat::map::get binding "?f")
                            "fmt::owned-set: no ?f")]
        (:wat::hashmap::assoc m (:wat::fmt::Fallback/node f) true)))
    (:wat::fmt::claims-set session)
    (:wat::rete::query session (:wat::fmt::q-fallback))))

(:wat::core::defn :wat::fmt::blanks-set
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::core::let [bl (:wat::core::Option/expect
                             (:wat::map::get binding "?bl")
                             "fmt::blanks-set: no ?bl")]
        (:wat::hashmap::assoc m (:wat::fmt::BlankBefore/id bl) true)))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
    (:wat::rete::query session (:wat::fmt::q-blank))))

(:wat::core::defn :wat::fmt::format-source
  [path  <- :wat::core::String
   src   <- :wat::core::String
   rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> :wat::core::String
  (:wat::core::match (:wat::core::read-string-with-comments src)
    ((:wat::core::ReadWithCommentsOutcome::Forms forms comments)
      (:wat::core::let
        [facts   (:wat::grep::facts-of path src)
         records (:wat::grep::facts-as-records facts)
         queries (:wat::core::PersistentVector :- [:wat::rete::Query]
                   (:wat::fmt::q-break)
                   (:wat::fmt::q-claim)
                   (:wat::fmt::q-fallback)
                   (:wat::fmt::q-blank))]
        (:wat::rete::with-overlay rules queries
          (:wat::core::fn [overlay <- :wat::rete::Overlay]
            -> :wat::core::String
            (:wat::core::let [fired (overlay records)]
              (:wat::fmt::emit forms comments
                (:wat::fmt::breaks-map fired)
                (:wat::fmt::owned-set fired)
                (:wat::fmt::blanks-set fired)))))))
    ((:wat::core::ReadWithCommentsOutcome::Malformed cause)
      (:wat::kernel::assertion-failed! (:wat::core::Error/message cause) :wat::core::None :wat::core::None))))

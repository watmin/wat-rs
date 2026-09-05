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

;; A specific layout rule asserts Claim on the form it owns. ClaimedUnder is the
;; transitive closure: the claimed node AND every descendant. R11 (the default)
;; fires only where no ancestor is claimed — a rule owns a form's whole extent.
(:wat::core::defrecord :wat::fmt::Claim
  [form <- :wat::core::i64])

(:wat::core::defrecord :wat::fmt::ClaimedUnder
  [node <- :wat::core::i64])

;; Derived. Always collected (namespace :fmt, stdlib). Recursive over Node.parent,
;; not over an aggregate of its own output — stratifiable.
(:wat::rete::defrule :fmt::claimed-under-root
  :when [(:wat::fmt::Claim (?f <- :form))]
  :then [(:wat::fmt::ClaimedUnder :node ?f)])

(:wat::rete::defrule :fmt::claimed-under-child
  :when [(:wat::fmt::ClaimedUnder (?p <- :node))
         (:wat::grep::Node (?n <- :id) (?p <- :parent))]
  :then [(:wat::fmt::ClaimedUnder :node ?n)])

(:wat::core::defrecord :wat::fmt::Acc
  [out      <- :wat::core::String
   next-id  <- :wat::core::i64
   comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])
   col      <- :wat::core::i64])

(:wat::rete::defquery :wat::fmt::q-break
  :params []
  :when [(?b <- :wat::fmt::Break)])

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

(:wat::core::defn :wat::fmt::emit-node
  [acc      <- :wat::fmt::Acc
   node     <- :wat::WatAST
   breaks   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
   indent   <- :wat::core::i64
   open-col <- :wat::core::i64
   first?   <- :wat::core::bool]
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
     acc-pad   (:wat::core::match br
                 ((:wat::core::Some bk)
                   (:wat::fmt::pad-break acc-b bk indent open-col))
                 (:wat::core::None
                   (:wat::core::if first?
                     acc-b
                     (:wat::core::if (:wat::string::empty? (:wat::fmt::Acc/out acc-b))
                       acc-b
                       (:wat::fmt::write acc-b " ")))))
     this-indent (:wat::fmt::Acc/col acc-pad)
     acc1        (:wat::fmt::flush-comments acc-pad src-line src-col this-indent)]
    (:wat::core::if (:wat::grep::structural? node)
      (:wat::core::let
        [this-open (:wat::fmt::Acc/col acc1)
         acc2      (:wat::fmt::write acc1 (:wat::fmt::open-of node-kind))
         kids      (:wat::core::ast->children node)
         acc3      (:wat::core::foldl
                     (:wat::core::fn [ca <- :wat::fmt::Acc  child <- :wat::WatAST] -> :wat::fmt::Acc
                       (:wat::core::let [o (:wat::fmt::Acc/out ca)
                                         first-kid? (:wat::core::or
                                                      (:wat::string::ends-with? o "(")
                                                      (:wat::core::or
                                                        (:wat::string::ends-with? o "[")
                                                        (:wat::core::or
                                                          (:wat::string::ends-with? o "{")
                                                          (:wat::string::ends-with? o "\n"))))]
                         (:wat::fmt::emit-node ca child breaks this-indent this-open first-kid?)))
                     acc2
                     kids)
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
          this-indent)))))

(:wat::core::defn :wat::fmt::emit
  [forms    <- :wat::WatAST
   comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])
   breaks   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])]
  -> :wat::core::String
  (:wat::core::let
    [top  (:wat::core::ast->children forms)
     acc0 (:wat::fmt::Acc :out "" :next-id 1 :comments comments :col 0)
     acc1 (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::fmt::Acc  form <- :wat::WatAST] -> :wat::fmt::Acc
              (:wat::core::let [acc-nl (:wat::fmt::write-nl acc)]
                (:wat::fmt::emit-node acc-nl form breaks 0 0 true)))
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
                            "fmt::breaks-map: no ?b")]
        (:wat::hashmap::assoc m (:wat::fmt::Break/id b) (:wat::fmt::Break/kind b))))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
    (:wat::rete::query session (:wat::fmt::q-break))))

(:wat::core::defn :wat::fmt::format-source
  [path  <- :wat::core::String
   src   <- :wat::core::String
   rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> :wat::core::String
  (:wat::core::match (:wat::core::read-string-with-comments src)
    ((:wat::core::ReadWithCommentsOutcome::Forms forms comments)
      (:wat::core::let
        [facts    (:wat::grep::facts-of path src)
         records  (:wat::grep::facts-as-records facts)
         queries  (:wat::core::PersistentVector :- [:wat::rete::Query] (:wat::fmt::q-break))
         breaks   (:wat::rete::with-overlay rules queries
                    (:wat::core::fn [overlay <- :wat::rete::Overlay]
                      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
                      (:wat::fmt::breaks-map (overlay records))))]
        (:wat::fmt::emit forms comments breaks)))
    ((:wat::core::ReadWithCommentsOutcome::Malformed cause)
      (:wat::kernel::assertion-failed! (:wat::core::Error/message cause) :wat::core::None :wat::core::None))))

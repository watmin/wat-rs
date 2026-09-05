;; wat/fmt.wat — layout engine: Break + a DUMB emitter. Rules assert Breaks; this file
;; holds no style opinion. Arc 277 STONE the-first-layout-rules.
;;
;; A node with a Break starts a new line at that indent; otherwise it follows a single space.
;; A line comment PINS A NEWLINE after itself.

(:wat::core::defrecord :wat::fmt::Break
  [id     <- :wat::core::i64
   indent <- :wat::core::i64])

(:wat::core::defrecord :wat::fmt::Comment
  [text     <- :wat::core::String
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

(:wat::core::defrecord :wat::fmt::Parsed
  [forms    <- :wat::WatAST
   comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])])

(:wat::core::defrecord :wat::fmt::Acc
  [out      <- :wat::core::String
   next-id  <- :wat::core::i64
   comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])])

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
          (:wat::fmt::Acc
            :out (:wat::string::concat
                   (:wat::fmt::ensure-nl (:wat::fmt::Acc/out acc))
                   (:wat::fmt::spaces indent)
                   (:wat::fmt::Comment/text c)
                   "\n")
            :next-id (:wat::fmt::Acc/next-id acc)
            :comments (:wat::core::rest (:wat::fmt::Acc/comments acc)))
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

(:wat::core::defn :wat::fmt::emit-node
  [acc     <- :wat::fmt::Acc
   node    <- :wat::WatAST
   breaks  <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   indent  <- :wat::core::i64
   first?  <- :wat::core::bool]
  -> :wat::fmt::Acc
  (:wat::core::let
    [id     (:wat::fmt::Acc/next-id acc)
     kind   (:wat::core::ast-kind node)
     x      (:wat::grep::extent-of node)
     line   (:wat::grep::Extent/line x)
     col    (:wat::grep::Extent/col x)
     br     (:wat::core::get breaks id)
     out0   (:wat::fmt::Acc/out acc)
     out1   (:wat::core::match br
              ((:wat::core::Some ind)
                (:wat::string::concat (:wat::fmt::ensure-nl out0) (:wat::fmt::spaces ind)))
              (:wat::core::None
                (:wat::core::if first?
                  out0
                  (:wat::core::if (:wat::string::empty? out0)
                    out0
                    (:wat::string::concat out0 " ")))))
     acc1   (:wat::fmt::flush-comments
              (:wat::fmt::Acc :out out1 :next-id (:wat::i64::+ id 1) :comments (:wat::fmt::Acc/comments acc))
              line col indent)]
    (:wat::core::if (:wat::grep::structural? node)
      (:wat::core::let
        [acc2 (:wat::fmt::Acc
                :out (:wat::string::concat (:wat::fmt::Acc/out acc1) (:wat::fmt::open-of kind))
                :next-id (:wat::fmt::Acc/next-id acc1)
                :comments (:wat::fmt::Acc/comments acc1))
         kids (:wat::core::ast->children node)
         acc3 (:wat::core::foldl
                (:wat::core::fn [ca <- :wat::fmt::Acc  child <- :wat::WatAST] -> :wat::fmt::Acc
                  ;; is-first is "output currently ends with opener or newline" — first kid
                  ;; of this container. Detect by whether out ends with ( [ { or #{ or \n.
                  (:wat::core::let [o (:wat::fmt::Acc/out ca)
                                    first-kid? (:wat::core::or
                                                 (:wat::string::ends-with? o "(")
                                                 (:wat::core::or
                                                   (:wat::string::ends-with? o "[")
                                                   (:wat::core::or
                                                     (:wat::string::ends-with? o "{")
                                                     (:wat::string::ends-with? o "\n"))))]
                    (:wat::fmt::emit-node ca child breaks indent first-kid?)))
                acc2
                kids)
         end-line (:wat::grep::Extent/end-line x)
         end-col  (:wat::grep::Extent/end-col x)
         acc4 (:wat::fmt::Acc
                :out (:wat::string::concat (:wat::fmt::Acc/out acc3) (:wat::fmt::close-of kind))
                :next-id (:wat::fmt::Acc/next-id acc3)
                :comments (:wat::fmt::Acc/comments acc3))]
        (:wat::fmt::flush-comments acc4 end-line end-col indent))
      (:wat::core::let
        [acc2 (:wat::fmt::Acc
                :out (:wat::string::concat (:wat::fmt::Acc/out acc1) (:wat::core::ast->source node))
                :next-id (:wat::fmt::Acc/next-id acc1)
                :comments (:wat::fmt::Acc/comments acc1))
         end-line (:wat::grep::Extent/end-line x)
         end-col  (:wat::grep::Extent/end-col x)]
        (:wat::fmt::flush-comments acc2 end-line end-col indent)))))

(:wat::core::defn :wat::fmt::emit
  [forms    <- :wat::WatAST
   comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])
   breaks   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])]
  -> :wat::core::String
  (:wat::core::let
    [top (:wat::core::ast->children forms)
     acc0 (:wat::fmt::Acc :out "" :next-id 1 :comments comments)
     acc1 (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::fmt::Acc  form <- :wat::WatAST] -> :wat::fmt::Acc
              (:wat::core::let [acc-nl (:wat::fmt::Acc
                                         :out (:wat::core::if (:wat::string::empty? (:wat::fmt::Acc/out acc))
                                                (:wat::fmt::Acc/out acc)
                                                (:wat::fmt::ensure-nl (:wat::fmt::Acc/out acc)))
                                         :next-id (:wat::fmt::Acc/next-id acc)
                                         :comments (:wat::fmt::Acc/comments acc))]
                (:wat::fmt::emit-node acc-nl form breaks 0 true)))
            acc0
            top)
     acc2 (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::fmt::Acc  c <- :wat::fmt::Comment] -> :wat::fmt::Acc
              (:wat::fmt::Acc
                :out (:wat::string::concat
                       (:wat::fmt::ensure-nl (:wat::fmt::Acc/out acc))
                       (:wat::fmt::Comment/text c)
                       "\n")
                :next-id (:wat::fmt::Acc/next-id acc)
                :comments (:wat::core::PersistentVector :- [:wat::fmt::Comment])))
            acc1
            (:wat::fmt::Acc/comments acc1))]
    (:wat::fmt::Acc/out acc2)))

(:wat::core::defn :wat::fmt::breaks-map
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
      (:wat::core::let [b (:wat::core::Option/expect
                            (:wat::map::get binding "?b")
                            "fmt::breaks-map: no ?b")]
        (:wat::hashmap::assoc m (:wat::fmt::Break/id b) (:wat::fmt::Break/indent b))))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
    (:wat::rete::query session (:wat::fmt::q-break))))

(:wat::core::defn :wat::fmt::format-source
  [path  <- :wat::core::String
   src   <- :wat::core::String
   rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> :wat::core::String
  (:wat::core::match (:wat::core::read-string-with-comments src)
    ((:wat::core::Ok parsed)
      (:wat::core::let
        [forms    (:wat::fmt::Parsed/forms parsed)
         comments (:wat::fmt::Parsed/comments parsed)
         facts    (:wat::grep::facts-of path src)
         records  (:wat::grep::facts-as-records facts)
         queries  (:wat::core::PersistentVector :- [:wat::rete::Query] (:wat::fmt::q-break))
         breaks   (:wat::rete::with-overlay rules queries
                    (:wat::core::fn [overlay <- :wat::rete::Overlay]
                      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
                      (:wat::fmt::breaks-map (overlay records))))]
        (:wat::fmt::emit forms comments breaks)))
    ((:wat::core::Err cause)
      (:wat::kernel::assertion-failed! (:wat::core::Error/message cause) :wat::core::None :wat::core::None))))

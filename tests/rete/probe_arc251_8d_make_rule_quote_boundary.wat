;; tests/rete/probe_arc251_8d_make_rule_quote_boundary.wat — co-located fixture for the
;; sibling .rs (slurped by `call_beside_value`). See that file for the whole argument.
;;
;; Every rule below is a HAND-WRITTEN `:wat::rete::make-rule` call, never a `defrule`,
;; because the shape under test is exactly what a faithful-Clojure `defrule` TEMPLATE
;; emits once `wat/rete/syntax.wat` is converted:
;;
;;     `(:wat::core::defn ~name [] :- :wat::rete::Rule
;;        (wat.rete/make-rule ~name-str
;;          (wat.core/quote ~when-vec)      ← a SYMBOL-headed quote
;;          (wat.core/quote ~then-vec)))
;;
;; ⭐ No conversion is needed to drive it: the shape is writeable by hand TODAY, which is
;; why this probe is a two-binary probe on an UNCONVERTED tree.
;;
;; Eight entry points, four namespaces of rules per spelling pair. Each returns the number
;; of derived `CountF` facts after one fire.

(:wat::core::defrecord :q1::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :q1::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :q1::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::rete::defquery :q1::q :params [] :when [(:q1::CountF)])
(:wat::core::defn :q1::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "q1::r"
    (:wat::core::quote [(:q1::Group (?g :- :g))
                        (?n :- (:wat::rete::acc::count) :from (:q1::Reading (?g :- :g)))])
    (:wat::core::quote [(:q1::CountF ?g ?n)])))
(:wat::core::defn :user::kw-quote [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :q1) (:wat::core::PersistentVector (:q1::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:q1::Group 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q1::Reading :g 1 :v 5)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:q1::q)))))

(:wat::core::defrecord :q2::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :q2::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :q2::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::rete::defquery :q2::q :params [] :when [(:q2::CountF)])
(:wat::core::defn :q2::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "q2::r"
    (wat.core/quote [(:q2::Group (?g :- :g))
                        (?n :- (:wat::rete::acc::count) :from (:q2::Reading (?g :- :g)))])
    (wat.core/quote [(:q2::CountF ?g ?n)])))
(:wat::core::defn :user::sym-quote [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :q2) (:wat::core::PersistentVector (:q2::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:q2::Group 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q2::Reading :g 1 :v 5)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:q2::q)))))

(:wat::core::defrecord :q3::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :q3::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :q3::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::rete::defquery :q3::q :params [] :when [(:q3::CountF)])
(:wat::core::defn :q3::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "q3::r"
    (wat.core/quote [(:q3::Group (?g :- :g))
                        (?n :- (:wat::rete::acc::count) :from (:q3::Reading (?g :- :g)))])
    (:wat::core::quote [(:q3::CountF ?g ?n)])))
(:wat::core::defn :user::sym-when-kw-then [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :q3) (:wat::core::PersistentVector (:q3::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:q3::Group 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q3::Reading :g 1 :v 5)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:q3::q)))))

(:wat::core::defrecord :q4::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :q4::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :q4::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::rete::defquery :q4::q :params [] :when [(:q4::CountF)])
(:wat::core::defn :q4::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "q4::r"
    (:wat::core::quote [(:q4::Group (?g :- :g))
                        (?n :- (:wat::rete::acc::count) :from (:q4::Reading (?g :- :g)))])
    (wat.core/quote [(:q4::CountF ?g ?n)])))
(:wat::core::defn :user::kw-when-sym-then [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :q4) (:wat::core::PersistentVector (:q4::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:q4::Group 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q4::Reading :g 1 :v 5)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:q4::q)))))

(:wat::core::defrecord :q5::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :q5::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :q5::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::rete::defquery :q5::q :params [] :when [(:q5::CountF)])
(:wat::core::defn :q5::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "q5::r"
    (wat.core/quote [(:q5::Group (?g :- :g))
                        (?n :- (:wat::rete::acc::count) :from (:q5::Reading (?g :- :g)))
                        (:wat::rete::where (wat.rete.i64/> ?n 1))])
    (wat.core/quote [(:q5::CountF ?g ?n)])))
(:wat::core::defn :user::sym-quote-where-passes [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :q5) (:wat::core::PersistentVector (:q5::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:q5::Group 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q5::Reading :g 1 :v 5)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q5::Reading :g 1 :v 6)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:q5::q)))))

(:wat::core::defrecord :q6::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :q6::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :q6::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::rete::defquery :q6::q :params [] :when [(:q6::CountF)])
(:wat::core::defn :q6::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "q6::r"
    (:wat::core::quote [(:q6::Group (?g :- :g))
                        (?n :- (:wat::rete::acc::count) :from (:q6::Reading (?g :- :g)))
                        (:wat::rete::where (wat.rete.i64/> ?n 1))])
    (:wat::core::quote [(:q6::CountF ?g ?n)])))
(:wat::core::defn :user::kw-quote-where-passes [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :q6) (:wat::core::PersistentVector (:q6::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:q6::Group 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q6::Reading :g 1 :v 5)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q6::Reading :g 1 :v 6)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:q6::q)))))

(:wat::core::defrecord :q7::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :q7::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :q7::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::rete::defquery :q7::q :params [] :when [(:q7::CountF)])
(:wat::core::defn :q7::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "q7::r"
    (wat.core/quote [(:q7::Group (?g :- :g))
                        (?n :- (:wat::rete::acc::count) :from (:q7::Reading (?g :- :g)))
                        (:wat::rete::where (wat.rete.i64/> ?n 5))])
    (wat.core/quote [(:q7::CountF ?g ?n)])))
(:wat::core::defn :user::sym-quote-where-filters [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :q7) (:wat::core::PersistentVector (:q7::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:q7::Group 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q7::Reading :g 1 :v 5)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q7::Reading :g 1 :v 6)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:q7::q)))))

(:wat::core::defrecord :q8::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :q8::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :q8::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])
(:wat::rete::defquery :q8::q :params [] :when [(:q8::CountF)])
(:wat::core::defn :q8::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "q8::r"
    (:wat::core::quote [(:q8::Group (?g :- :g))
                        (?n :- (:wat::rete::acc::count) :from (:q8::Reading (?g :- :g)))
                        (:wat::rete::where (wat.rete.i64/> ?n 5))])
    (:wat::core::quote [(:q8::CountF ?g ?n)])))
(:wat::core::defn :user::kw-quote-where-filters [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :q8) (:wat::core::PersistentVector (:q8::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:q8::Group 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q8::Reading :g 1 :v 5)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       session (:wat::core::match (:wat::rete::insert session (:q8::Reading :g 1 :v 6)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:q8::q)))))

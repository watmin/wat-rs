;; tests/rete/probe_arc251_8d_then_item_identity.wat — co-located fixture for the sibling .rs
;; (slurped by `call_beside_value`). See that file for the whole argument.
;;
;; Every rule is a HAND-WRITTEN `:wat::rete::make-rule` with a NESTED constructor inside its
;; `:then` item's operand — exactly the shape `wat/Record.wat:207` emits once converted:
;;
;;     `(wat.core/kwargs-construct ~_kc-type ~@call-args)
;;
;; ⭐ The shape is writeable by hand on an UNCONVERTED tree, so this is a two-binary probe with
;; no 22-minute conversion in it.
;;
;; Each entry returns the number of derived `Out` facts after one fire, or RAISES — the `.rs`
;; asserts both directions.

;; ── t1 / :user::kw-ctor ─────────────────────────────────────────────────────────────────
(:wat::core::defrecord :t1::In    [g <- :wat::core::i64])
(:wat::core::defrecord :t1::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :t1::Out   [g <- :wat::core::i64  inner <- :t1::Inner])
(:wat::rete::defquery :t1::q :params [] :when [(:t1::Out)])
(:wat::core::defn :t1::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "t1::r"
    (:wat::core::quote [(:t1::In (?g :- :g))])
    (:wat::core::quote [(:t1::Out :g ?g :inner (:wat::core::kwargs-construct :t1::Inner :x 1))])))
(:wat::core::defn :user::kw-ctor [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :t1) (:wat::core::PersistentVector (:t1::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:t1::In 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:t1::q)))))

;; ── t2 / :user::sym-ctor ─────────────────────────────────────────────────────────────────
(:wat::core::defrecord :t2::In    [g <- :wat::core::i64])
(:wat::core::defrecord :t2::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :t2::Out   [g <- :wat::core::i64  inner <- :t2::Inner])
(:wat::rete::defquery :t2::q :params [] :when [(:t2::Out)])
(:wat::core::defn :t2::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "t2::r"
    (:wat::core::quote [(:t2::In (?g :- :g))])
    (:wat::core::quote [(:t2::Out :g ?g :inner (wat.core/kwargs-construct :t2::Inner :x 1))])))
(:wat::core::defn :user::sym-ctor [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :t2) (:wat::core::PersistentVector (:t2::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:t2::In 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:t2::q)))))

;; ── t3 / :user::sym-type-ctor ─────────────────────────────────────────────────────────────────
(:wat::core::defrecord :t3::In    [g <- :wat::core::i64])
(:wat::core::defrecord :t3::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :t3::Out   [g <- :wat::core::i64  inner <- :t3::Inner])
(:wat::rete::defquery :t3::q :params [] :when [(:t3::Out)])
(:wat::core::defn :t3::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "t3::r"
    (:wat::core::quote [(:t3::In (?g :- :g))])
    (:wat::core::quote [(:t3::Out :g ?g :inner (:wat::core::kwargs-construct t3/Inner :x 1))])))
(:wat::core::defn :user::sym-type-ctor [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :t3) (:wat::core::PersistentVector (:t3::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:t3::In 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:t3::q)))))

;; ── t4 / :user::sym-head-and-type ─────────────────────────────────────────────────────────────────
(:wat::core::defrecord :t4::In    [g <- :wat::core::i64])
(:wat::core::defrecord :t4::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :t4::Out   [g <- :wat::core::i64  inner <- :t4::Inner])
(:wat::rete::defquery :t4::q :params [] :when [(:t4::Out)])
(:wat::core::defn :t4::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "t4::r"
    (:wat::core::quote [(:t4::In (?g :- :g))])
    (:wat::core::quote [(:t4::Out :g ?g :inner (wat.core/kwargs-construct t4/Inner :x 1))])))
(:wat::core::defn :user::sym-head-and-type [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :t4) (:wat::core::PersistentVector (:t4::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:t4::In 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:t4::q)))))

;; ── t5 / :user::kw-computation ─────────────────────────────────────────────────────────────────
(:wat::core::defrecord :t5::In    [g <- :wat::core::i64])
(:wat::core::defrecord :t5::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :t5::Out   [g <- :wat::core::i64  inner <- :t5::Inner])
(:wat::rete::defquery :t5::q :params [] :when [(:t5::Out)])
(:wat::core::defn :t5::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "t5::r"
    (:wat::core::quote [(:t5::In (?g :- :g))])
    (:wat::core::quote [(:t5::Out :g ?g :inner (:t5::Inner :x (:wat::core::if (:wat::core::not false) 1 2)))])))
(:wat::core::defn :user::kw-computation [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :t5) (:wat::core::PersistentVector (:t5::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:t5::In 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:t5::q)))))

;; ── t6 / :user::sym-computation ─────────────────────────────────────────────────────────────────
(:wat::core::defrecord :t6::In    [g <- :wat::core::i64])
(:wat::core::defrecord :t6::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :t6::Out   [g <- :wat::core::i64  inner <- :t6::Inner])
(:wat::rete::defquery :t6::q :params [] :when [(:t6::Out)])
(:wat::core::defn :t6::r [] -> :wat::rete::Rule
  (:wat::rete::make-rule "t6::r"
    (:wat::core::quote [(:t6::In (?g :- :g))])
    (:wat::core::quote [(:t6::Out :g ?g :inner (:t6::Inner :x (:wat::core::if (wat.core/not false) 1 2)))])))
(:wat::core::defn :user::sym-computation [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [session (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :t6) (:wat::core::PersistentVector (:t6::q))) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:t6::In 1)) [:wat::rete::InsertOutcome.Inserted {:session __a} __a] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "insert: ceiling")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __f} __f] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: round cap")])]
      (:wat::rete::query fired (:t6::q)))))


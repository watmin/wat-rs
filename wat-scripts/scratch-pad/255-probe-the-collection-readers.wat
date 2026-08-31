;; Scratch probe — arc 255 Stone "the collection readers get homes".
;;
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-the-collection-readers.md
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-collection-readers.md
;;
;; Homes `:wat::core::assoc`, `conj`, `drop`, `take` — thin `#[wat_intrinsic]` delegates over
;; their existing named fns (`eval_assoc`/`eval_conj` in `src/runtime.rs`,
;; `eval_vec_take`/`eval_vec_drop` in `src/collection/transform.rs`; no body moves, STOP-4).
;; `find-last-index`/`seqable->stream` are W7 (run caller code) and stay out of scope.
;;
;; `@Totality` is measured PER VERB, not copied across the four
;; (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`):
;;   - `assoc`/`conj` reach a container-capability gate (`MapContainer::can_assoc()` /
;;     `StreamContainer::has_append()`) that admits more than one receiver kind, and WITHIN
;;     that admitted domain each still raises on a value it cannot place (assoc: an unknown
;;     Record field, or a HashMap/PersistentMap key that fails `value_is_key_hashable`; conj:
;;     a HashSet element that fails `value_is_set_hashable`) — `Partial`.
;;   - `drop`/`take` touch no such gate: their one receiver check (`value_as_stream`) DEFINES
;;     the domain, and past it they only construct a lazy `Stream` thunk — arithmetic on `n`,
;;     no failure path — `Total`.
;;
;; The debt-ledger prediction (`FROZEN_CHECKER_DEBT_LEDGER`, `src/intrinsic/mod.rs`) is a
;; SEPARATE, orthogonal axis from `@Totality` — whether `check.rs` carries an `env.register()`
;; TypeScheme for the verb. It is MIXED: `assoc`/`conj` carry one (the `contains?`/`get`/
;; `conj`/`assoc` fingerprint block, `src/check.rs`) so they need NO ledger row; `drop`/`take`
;; have only a custom `infer_drop`/`infer_take` arm and no registered scheme, so they DO —
;; `FROZEN_CHECKER_DEBT_LEDGER` 62 -> 64, rows for `drop`/`take` only.
;;
;; ★ That same registered-TypeScheme split is independently visible RIGHT HERE, at check time:
;; a bare `:wat::core::assoc`/`:wat::core::conj` in a position typed `:wat::core::keyword`
;; fails to check — the checker resolves the bare FQDN against its registered TypeScheme first
;; and infers a FUNCTION type, not a keyword literal — while `:wat::core::drop`/
;; `:wat::core::take`, carrying no such scheme, parse as plain keywords there without
;; complaint. That is why section 2 below calls `:wat::runtime::metadata-of` on each verb
;; DIRECTLY (its own hand-written inference arm accepts the bare FQDN in the callee's own
;; argument position) rather than through a `:user::` wrapper fn — passing `:wat::core::assoc`
;; through an ordinary `:wat::core::keyword`-typed parameter does NOT check on this binary.
;;
;;   section 1 — behaviour unchanged: `assoc` on its own domain (HashMap — Vector/Stream are
;;               outside `can_assoc()`'s domain, so not exercised here), `conj` on a Vector,
;;               `take`/`drop` on BOTH a Vector and a lazy (genuinely infinite) Stream —
;;               proving `take`/`drop` stay lazy (an infinite source, `take 3` terminates and
;;               `take 2` composed with `drop 3` still terminates: neither forces past what a
;;               downstream `take` needs).
;;   section 2 — `metadata-of :totality` for all four — the mixed Purity/Totality split above,
;;               `metadata-of` doubling as the "is it registered" probe (`lookup_entry`'s own
;;               question): `Some hm` once rebuilt, `:None` against this PRE-EXISTING binary.
;;
;; ⚠ Run against the PRE-EXISTING `target/release/wat` (predates this stone's Rust changes, per
;; the rider's brief) — expect section 1 to behave exactly as before (the dispatch arms this
;; stone deletes are still literal match arms in that binary) and section 2's `metadata-of` to
;; answer `:None` for all four (`registry().lookup_entry` finds nothing pre-rebuild; only a
;; rebuilt binary registers them and can answer `Some hm`). See the rider's report for what this
;; binary actually printed.

(:wat::core::defn :wat-tests::255::collection-readers::nat
  [i <- :wat::core::i64] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::stream::lazy
    (:wat::stream::cons i (:wat-tests::255::collection-readers::nat (:wat::core::+ i 1)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── section 1 — behaviour unchanged ──")
    (:wat::kernel::println
      (:wat::string::concat "assoc on HashMap             => "
        (:wat::edn::write (:wat::hashmap::get
          (:wat::core::assoc (:wat::core::HashMap :- [:wat::core::String :wat::core::i64]) "a" 1) "a"))))
    (:wat::kernel::println
      (:wat::string::concat "conj on Vector                => "
        (:wat::edn::write (:wat::core::conj (:wat::core::Vector :- [:wat::core::i64] 1 2) 3))))
    (:wat::kernel::println
      (:wat::string::concat "take 2 on Vector [1 2 3]      => "
        (:wat::edn::write (:wat::core::stream->vec (:wat::core::Vector :- [:wat::core::i64])
          (:wat::core::take (:wat::core::Vector :- [:wat::core::i64] 1 2 3) 2)))))
    (:wat::kernel::println
      (:wat::string::concat "drop 1 on Vector [1 2 3]      => "
        (:wat::edn::write (:wat::core::stream->vec (:wat::core::Vector :- [:wat::core::i64])
          (:wat::core::drop (:wat::core::Vector :- [:wat::core::i64] 1 2 3) 1)))))
    (:wat::kernel::println
      (:wat::string::concat "take 3 on an INFINITE Stream  => "
        (:wat::edn::write (:wat::core::stream->vec (:wat::core::Vector :- [:wat::core::i64])
          (:wat::core::take (:wat-tests::255::collection-readers::nat 0) 3)))))
    (:wat::kernel::println
      (:wat::string::concat "take 2 (drop 3 INFINITE Stream) => "
        (:wat::edn::write (:wat::core::stream->vec (:wat::core::Vector :- [:wat::core::i64])
          (:wat::core::take (:wat::core::drop (:wat-tests::255::collection-readers::nat 0) 3) 2)))))
    (:wat::kernel::println "── section 2 — metadata-of :totality (assoc/conj Partial vs. drop/take Total) ──")
    (:wat::kernel::println
      (:wat::string::concat "assoc :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::assoc)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "conj  :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::conj)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "drop  :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::drop)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    (:wat::kernel::println
      (:wat::string::concat "take  :totality => "
        (:wat::core::match (:wat::runtime::metadata-of :wat::core::take)
          ((:wat::core::Some hm)
           (:wat::core::match (:wat::hashmap::get hm :totality)
             ((:wat::core::Some t) (:wat::edn::write t))
             (:None "registered, but no :totality key (unexpected)")))
          (:None "None (not registered in this binary)"))))
    nil))

;; wat-scripts/fixes/one-name-per-operation.wat — stone 255.86.
;; SCOPE: corpus
;;
;; Exact-token rename. The pairs live in one-name-per-operation.edn
;; (committed data). This program reads that table and applies
;; rename-keyword-exact once per pair. A keyword leaf moves only when its
;; whole name equals the old string. Comments and string literals stay.
;; Idempotent: no replacement is also an old name.
;;
;; printf '[...paths...]\n' | ./target/release/wat ./wat-scripts/fixes/one-name-per-operation.wat
;;
;; :wat::hashmap::length -> :wat::core::length
;; :wat::hashmap::empty? -> :wat::core::empty?
;; :wat::hashmap::contains-key? -> :wat::core::contains?
;; :wat::hashmap::get -> :wat::core::get
;; :wat::hashmap::assoc -> :wat::core::assoc
;; :wat::hashmap::dissoc -> :wat::core::dissoc
;; :wat::hashmap::keys -> :wat::core::keys
;; :wat::hashmap::values -> :wat::core::values
;; :wat::map::length -> :wat::core::length
;; :wat::map::empty? -> :wat::core::empty?
;; :wat::map::contains-key? -> :wat::core::contains?
;; :wat::map::get -> :wat::core::get
;; :wat::map::assoc -> :wat::core::assoc
;; :wat::map::dissoc -> :wat::core::dissoc
;; :wat::map::keys -> :wat::core::keys
;; :wat::map::values -> :wat::core::values
;; :wat::vec::length -> :wat::core::length
;; :wat::vec::empty? -> :wat::core::empty?
;; :wat::vec::contains? -> :wat::core::contains?
;; :wat::vec::get -> :wat::core::get
;; :wat::vec::conj -> :wat::core::conj
;; :wat::vec::concat -> :wat::core::concat
;; :wat::vec::extend -> :wat::core::into
;; :wat::vector::length -> :wat::core::length
;; :wat::vector::empty? -> :wat::core::empty?
;; :wat::vector::contains? -> :wat::core::contains?
;; :wat::vector::get -> :wat::core::get
;; :wat::vector::conj -> :wat::core::conj
;; :wat::vector::concat -> :wat::core::into
;; :wat::hashset::length -> :wat::core::length
;; :wat::hashset::empty? -> :wat::core::empty?
;; :wat::hashset::contains? -> :wat::core::contains?
;; :wat::hashset::conj -> :wat::core::conj
;; :wat::linkedlist::length -> :wat::core::length
;; :wat::linkedlist::empty? -> :wat::core::empty?
;; :wat::linkedlist::contains? -> :wat::core::contains?
;; :wat::linkedlist::get -> :wat::core::get
;; :wat::linkedlist::conj -> :wat::core::conj
;; :wat::core::Bytes/to-hex -> :wat::bytes::to-hex
;; :wat::core::Bytes/from-hex -> :wat::bytes::from-hex
;; :wat::core::Bytes::to-hex -> :wat::bytes::to-hex
;; :wat::core::Bytes::from-hex -> :wat::bytes::from-hex
;; :wat::core::Record/field-at -> :wat::record::field-at
;; :wat::core::Record/same-data? -> :wat::record::same-data?
;; :wat::core::Record/assoc -> :wat::core::assoc
;; :wat-tests::cache-svc/dial -> :wat-tests::cache-svc::dial
;; :wat-tests::cache-svc/get-label -> :wat-tests::cache-svc::get-label
;; :wat-tests::cache-svc/put-label -> :wat-tests::cache-svc::put-label
;; :wat-tests::cache-svc/result-label -> :wat-tests::cache-svc::result-label
;; :wat-tests::cache-svc/run -> :wat-tests::cache-svc::run
;; :wat-tests::hologram-svc/assert-put-ok -> :wat-tests::hologram-svc::assert-put-ok
;; :wat-tests::hologram-svc/dial -> :wat-tests::hologram-svc::dial
;; :wat-tests::hologram-svc/get-results -> :wat-tests::hologram-svc::get-results
;; :wat-tests::hologram-svc/run -> :wat-tests::hologram-svc::run
;; :wat-tests::pcache/dial -> :wat-tests::pcache::dial
;; :wat-tests::pcache/label -> :wat-tests::pcache::label
;; :wat-tests::pcache/run -> :wat-tests::pcache::run
;; :wat-tests::mal/dial -> :wat-tests::mal::dial
;; :wat-tests::mal/run -> :wat-tests::mal::run
;; :wat-tests::mal/try -> :wat-tests::mal::try
;; :wat-tests::barebox/run -> :wat-tests::barebox::run
;; :t::svc/add -> :t::svc::add
;; :t::svc/plain -> :t::svc::plain
;; :t::worker/start -> :t::worker::start
;; :myapp::Formattable/format -> :myapp::Formattable::format
;; :wat-tests::holon::Reject/bundle-or-fail -> :wat-tests::holon::Reject::bundle-or-fail
;; :wat-tests::holon::Reject/project-bundle-or-fail -> :wat-tests::holon::Reject::project-bundle-or-fail
;; Neither parent is a registered type, so `/` respells to `::`.

(wat.core/defn user/rename-all
  [src :- wat.type/String
   pairs :- (wat.type/Vector :- [(wat.type/Vector :- [wat.type/String])])]
  :- wat.type/String
  (wat.core/if (wat.core/empty? pairs)
    src
    (wat.core/let
      [pair (wat.core/first pairs)
       old  (wat.core.Option/expect (wat.core/get pair 0) "pair 0")
       new  (wat.core.Option/expect (wat.core/get pair 1) "pair 1")
       next (wat.fix/rename-keyword-exact old new src)]
      (user/rename-all next (wat.core/rest pairs)))))

(wat.core/defn user/rewrite-each [paths :- (wat.type/Vector :- [wat.type/String])] :- wat.type/nil
  (wat.core/if (wat.core/empty? paths)
    nil
    (wat.core/let [path (wat.core/first paths)]
      (wat.core/do
        (wat.io/write-file path
          (user/rename-all
            (wat.io/read-file path)
            (wat.edn/read (wat.io/read-file "wat-scripts/fixes/one-name-per-operation.edn"))))
        (wat.kernel/println (wat.string/concat "[one-name] " path))
        (user/rewrite-each (wat.core/rest paths))))))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [paths (wat.core/match (wat.kernel/readln)
             [wat.kernel/ReadlnOutcome.Datum {:v __datum} __datum]
             [wat.kernel/ReadlnOutcome.Eof {}
               (wat.kernel/assertion-failed! :message "readln: end of input")]
             [wat.kernel/ReadlnOutcome.Stopped {}
               (wat.kernel/assertion-failed! :message "readln: stop requested")])]
    (user/rewrite-each paths)))

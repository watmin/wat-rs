;; Co-located fixture for wat_io.rs — slurped via startup_beside(file!()).
;; One named compute fn per test case.

;; ─── IOReader / read-line ─────────────────────────────────────────────────────

(:wat::core::defn :my::compute-read-line [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "hello\nworld\n")]
    (:wat::io::IOReader/read-line r)))

(:wat::core::defn :my::compute-read-line-crlf [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "hello\r\n")]
    (:wat::io::IOReader/read-line r)))

(:wat::core::defn :my::compute-read-line-eof [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "only-line\n")
     _ (:wat::io::IOReader/read-line r)]
    (:wat::io::IOReader/read-line r)))

;; ─── IOReader / read (bytes) ─────────────────────────────────────────────────

(:wat::core::defn :my::compute-read-bytes [] -> (:wat::core::Option :- [(:wat::core::Vector :- [:wat::core::u8])])
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "hello")]
    (:wat::io::IOReader/read r 3)))

(:wat::core::defn :my::compute-read-bytes-eof [] -> (:wat::core::Option :- [(:wat::core::Vector :- [:wat::core::u8])])
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "hi")
     _ (:wat::io::IOReader/read r 100)]
    (:wat::io::IOReader/read r 100)))

;; ─── IOReader / read-all ─────────────────────────────────────────────────────

(:wat::core::defn :my::compute-read-all [] -> (:wat::core::Vector :- [:wat::core::u8])
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "hello")]
    (:wat::io::IOReader/read-all r)))

;; ─── IOReader / rewind ───────────────────────────────────────────────────────

(:wat::core::defn :my::compute-rewind [] -> (:wat::core::Vector :- [:wat::core::u8])
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "again")
     _ (:wat::io::IOReader/read-all r)
     _ (:wat::io::IOReader/rewind r)]
    (:wat::io::IOReader/read-all r)))

;; ─── IOWriter / writeln + to-string ─────────────────────────────────────────

(:wat::core::defn :my::compute-writeln-to-string [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [w (:wat::io::IOWriter/new)
     _ (:wat::io::IOWriter/writeln w "first")
     _ (:wat::io::IOWriter/writeln w "second")]
    (:wat::io::IOWriter/to-string w)))

(:wat::core::defn :my::compute-writeln-count [] -> :wat::core::i64
  (:wat::core::let
    [w (:wat::io::IOWriter/new)]
    (:wat::io::IOWriter/writeln w "hello")))

;; ─── IOWriter / write (bytes) ────────────────────────────────────────────────

(:wat::core::defn :my::compute-write-bytes [] -> :wat::core::i64
  (:wat::core::let
    [w (:wat::io::IOWriter/new)
     bytes (:wat::core::Vector :wat::core::u8
             (:wat::core::u8 72)
             (:wat::core::u8 105)
             (:wat::core::u8 33))]
    (:wat::io::IOWriter/write w bytes)))

(:wat::core::defn :my::compute-write-all-to-bytes [] -> (:wat::core::Vector :- [:wat::core::u8])
  (:wat::core::let
    [w (:wat::io::IOWriter/new)
     bytes (:wat::core::Vector :wat::core::u8
             (:wat::core::u8 65)
             (:wat::core::u8 66)
             (:wat::core::u8 67))
     _ (:wat::io::IOWriter/write-all w bytes)]
    (:wat::io::IOWriter/to-bytes w)))

;; ─── IOWriter / write-string ─────────────────────────────────────────────────

(:wat::core::defn :my::compute-write-string-no-newline [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [w (:wat::io::IOWriter/new)
     _ (:wat::io::IOWriter/write-string w "hello ")
     _ (:wat::io::IOWriter/write-string w "world")]
    (:wat::io::IOWriter/to-string w)))

(:wat::core::defn :my::compute-write-string-byte-count [] -> :wat::core::i64
  (:wat::core::let
    [w (:wat::io::IOWriter/new)]
    (:wat::io::IOWriter/write-string w "héllo")))

;; ─── IOWriter / flush ────────────────────────────────────────────────────────

(:wat::core::defn :my::compute-flush [] -> :wat::core::nil
  (:wat::core::let
    [w (:wat::io::IOWriter/new)]
    (:wat::io::IOWriter/flush w)))

;; ─── Full round-trip reader → writer ─────────────────────────────────────────

(:wat::core::defn :my::compute-copy-lines [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "alpha\nbeta\n")
     w (:wat::io::IOWriter/new)
     _ (:wat::core::match (:wat::io::IOReader/read-line r) 
         ((:wat::core::Some line) (:wat::io::IOWriter/writeln w line))
         (:wat::core::None -1))
     _ (:wat::core::match (:wat::io::IOReader/read-line r) 
         ((:wat::core::Some line) (:wat::io::IOWriter/writeln w line))
         (:wat::core::None -1))]
    (:wat::io::IOWriter/to-string w)))

;; ─── Empty cases ─────────────────────────────────────────────────────────────

(:wat::core::defn :my::compute-fresh-writer-empty [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [w (:wat::io::IOWriter/new)]
    (:wat::io::IOWriter/to-string w)))

(:wat::core::defn :my::compute-empty-reader-read-line [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "")]
    (:wat::io::IOReader/read-line r)))


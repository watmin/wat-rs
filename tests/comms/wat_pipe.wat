;; tests/comms/wat_pipe.wat — co-located fixture for the pipe probe group,
;; slurped via startup_beside(file!()). Each test function has a unique name.
;; No placeholder main — startup_beside loads defns only.

;; pipe_returns_writer_reader_tuple
(:wat::core::defn :my::pipe-returns-writer-reader-tuple [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::pipe)
     _w   (:wat::core::first pair)
     _r   (:wat::core::second pair)]
    42))

;; pipe_writeln_then_read_line_round_trips
(:wat::core::defn :my::pipe-writeln-round-trips [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [pair (:wat::kernel::pipe)
     w    (:wat::core::first pair)
     r    (:wat::core::second pair)
     _    (:wat::io::IOWriter/writeln w "hello")]
    (:wat::io::IOReader/read-line r)))

;; pipe_multiple_writelns_read_line_by_line
(:wat::core::defn :my::pipe-multiple-writelns [] -> :wat::core::String
  (:wat::core::let
    [pair (:wat::kernel::pipe)
     w    (:wat::core::first pair)
     r    (:wat::core::second pair)
     _    (:wat::io::IOWriter/writeln w "first")
     _    (:wat::io::IOWriter/writeln w "second")
     a    (:wat::io::IOReader/read-line r)
     b    (:wat::io::IOReader/read-line r)]
    (:wat::core::match a 
      ((:wat::core::Some sa)
       (:wat::core::match b 
         ((:wat::core::Some sb) (:wat::string::join "," (:wat::core::Vector :wat::core::String sa sb)))
         (:wat::core::None     "second-missing")))
      (:wat::core::None "first-missing"))))

;; pipe_write_string_then_read_exact_bytes
(:wat::core::defn :my::pipe-write-string-exact-bytes [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::pipe)
     w    (:wat::core::first pair)
     r    (:wat::core::second pair)
     n    (:wat::io::IOWriter/write-string w "hello")
     got  (:wat::io::IOReader/read r 5)]
    (:wat::core::match got 
      ((:wat::core::Some bytes) n)
      (:wat::core::None        -1))))

;; pipe_preserves_utf8_lines
(:wat::core::defn :my::pipe-preserves-utf8 [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [pair (:wat::kernel::pipe)
     w    (:wat::core::first pair)
     r    (:wat::core::second pair)
     _    (:wat::io::IOWriter/writeln w "héllo")]
    (:wat::io::IOReader/read-line r)))

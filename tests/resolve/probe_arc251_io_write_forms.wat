(:wat::core::defn :user::compute-c01 [] -> :wat::core::nil
  (:wat::io::write-file "/tmp/wat-iowriteforms-c01.txt" "hello-write-file"))
(:wat::core::defn :user::compute-c02 [] -> :wat::core::i64
  (:wat::io::with-open-file "/tmp/wat-iowriteforms-c02.txt"
    (:wat::core::fn [w <- :wat::io::IOWriter] -> :wat::core::i64
      (:wat::io::IOWriter/write-string w "hello-with-open"))))
(:wat::core::defn :user::compute-c03 [] -> :wat::core::i64
  (:wat::io::with-open-file "/tmp/wat-iowriteforms-c03.txt"
    (:wat::core::fn [w <- :wat::io::IOWriter] -> :wat::core::i64
      (:wat::core::do (:wat::io::IOWriter/write-string w "x") 99))))

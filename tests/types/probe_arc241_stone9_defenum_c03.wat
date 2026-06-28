;; Contract 03: interleaved unit and tagged variants.
(:wat::core::defenum :app::Event
  :Tick
  :Move [x <- :wat::core::i64
         y <- :wat::core::i64]
  :Reset
  :Resize [width <- :wat::core::i64])

# BRIEF — the map-intern counter is laned per thread

`next_intern` was one process-global `AtomicU64` bumped on every
mint, and every one-entry `PMap` mints — 40k per fire. 512
concurrent retes serialise on that cache line. Lane the id:
high bits name the thread, low bits count within it. Uniqueness
preserved. Ids stay outside `Eq`/`Hash`. Never mint 0.
Do not bundle the allocator decision into this.

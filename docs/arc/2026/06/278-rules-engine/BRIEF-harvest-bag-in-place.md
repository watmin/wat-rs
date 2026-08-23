# BRIEF — class-scan harvest writes in place, no intermediate bag

`harvest_class_scan` returned its own `Vec` and every caller
`extend`ed from it. Write into the caller's vec instead.
`PMap` is 56 B — the bag was 2.24 MB per fire at 40k.
Same maps, same order. 7strat 3/3. Do not `PMap::Array1`.
Do not Session-Vec. Do not skip the walk.

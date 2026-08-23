# BRIEF — the stratified membership set is carried, not rebuilt

`merge_facts` rebuilds `present` from the whole closure every
stratum. Carry it across the loop instead. Same dedup, same
`push_back` order, same closure Value. 7strat 3/3.
Do not change dedup to identity. Do not Session-Vec.

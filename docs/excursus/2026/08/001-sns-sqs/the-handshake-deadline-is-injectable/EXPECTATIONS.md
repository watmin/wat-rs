# EXPECTATIONS — the handshake deadline is injectable

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **Exactly ONE place holds `30000`** | grep the corpus and `src/`: one definition, zero second copies. Two homes for one value is the drift `ab419aaa3` removed. |
| 2 | ⭑⭑ **The knob moves the arm, DRIVEN** | `WAT_STARTUP_HANDSHAKE_DEADLINE_MS=200` + `probe-a-silent-child-cannot-hang-launch.wat` → the `TimedOut` arm fires in **~0.2 s, not 30 s**, on **both** tiers. Quote both wall-clocks. |
| 3 | ⭑⭑ **Unset behaves exactly as today** | with no env var, the same probe still fires at ~30 s on both tiers. This is the regression gate. |
| 4 | ⭑ **The CHILD's deadline moves too** | process tier: show the child end (`child-main`'s wait) honouring the injected value, not just the parent's `launch`. Env inheritance is the assumption — **verify it**. |
| 5 | ⛔ **`recv_by_deadline` is NOT the knob's home** | `owner-recv-loop`'s 10 000 ms is unchanged with the env var set. Demonstrate it, don't assert it. |
| 6 | **Invalid and zero are decided and stated** | garbage → 30000 silently; say what `=0` does (refuse or clamp) and why — `zero-is-not-a-wait/` is precedent. |
| 7 | **Read once** | not per call; say how you verified (one read, `OnceLock` or equivalent). |
| 8 | ⭑⭑ **Floor** | Summary line verbatim, `.floor/` path, `ARM.txt` or not. `wat/test.wat`'s two sites are on the path of every spawned test program. |
| 9 | clippy + `--no-run` | `--workspace --all-targets`. |
| 10 | blast radius | `git status --short`; the 5 call sites are in **three** files, one of them a quasiquoted body. |

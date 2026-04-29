//! Arc 093 worked-example binary. The wat program at
//! `wat/main.wat` writes a fresh sqlite-backed telemetry .db
//! (auto-deleting via `:wat::io::TempFile`), reopens read-only,
//! streams Event::Log rows through the substrate's Stream<T>
//! circuit, filters via `:wat::form::matches?` (arc 098), and
//! prints the hits. End-to-end pry/gdb-shaped interrogation UX
//! the arc 093 DESIGN's worked examples were designed around.

wat::main! {
    deps: [wat_telemetry, wat_sqlite, wat_telemetry_sqlite],
}

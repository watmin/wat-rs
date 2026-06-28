//! Arc 209 Stone C0b — UDS abstract-namespace spike: PROVE the process-tier
//! connection mechanism actually works on this box.
//!
//! The claim (DESIGN-STONE-C0b): a service listens on an abstract-namespace AF_UNIX
//! socket (in-memory, NO filesystem entry), a client connects, the service accepts a
//! per-connection endpoint, and a request/response round-trips — the *same* syscall
//! sequence as AF_INET (remote), which is why "process correct guarantees remote."
//!
//! This is the substrate-boundary proof behind that claim: before we build
//! `listener'`/`accept'`/`connect'` as wat verbs, prove the OS mechanism is real.
//! It uses std's own abstract-namespace UDS support (`from_abstract_name` /
//! `bind_addr` / `connect_addr`, stable since 1.70) — the same surface the wat
//! verbs will wrap.
//!
//! Scope: this proves the SOCKET plumbing (the parties here are two threads in one
//! process — the listen/accept/connect/round-trip syscalls are identical whether the
//! parties are threads, processes, or hosts). The real process-boundary integration
//! (`deftest-hermetic'`) and the io_uring-driven non-blocking accept (reactor reuse,
//! for `select'`-over-`Listener'`) are the next layers; here we nail the foundation.
//!
//! It mimics the service shape in miniature: the "service" owns a protected scalar
//! (10), the "client" sends `increment 5`, the service threads the state and replies
//! `15` over the connection. The full deadlock-free loop the legacy proofs were
//! `:ignore`d for — here, on an abstract UDS, for the first time.
//!
//! Run: cargo test --release -p wat --test nursery probe_arc209_c0b_uds_abstract_spike -- --test-threads=1

use std::io::{Read, Write};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};

#[test]
fn uds_abstract_listen_accept_connect_round_trips_a_protected_scalar() {
    // The rendezvous: an abstract-namespace address — in-memory, no filesystem entry.
    let addr = SocketAddr::from_abstract_name(b"wat.arc209.c0b.spike")
        .expect("abstract-namespace UDS address");
    let listener = UnixListener::bind_addr(&addr).expect("bind abstract UDS (no fs entry)");

    // ── The service: owns a protected scalar; accepts one client; handles one op ──
    let service = std::thread::spawn(move || {
        let (mut conn, _who) = listener.accept().expect("accept the connecting client");
        // State-as-self: the protected scalar lives only in this loop.
        let mut state: i64 = 10;
        let mut buf = [0u8; 64];
        let n = conn.read(&mut buf).expect("service reads the request");
        let req = std::str::from_utf8(&buf[..n]).expect("utf8 request");
        // The trivial counter mechanism: "increment N".
        let n_arg: i64 = req
            .strip_prefix("increment ")
            .and_then(|s| s.trim().parse().ok())
            .expect("request is `increment <i64>`");
        state += n_arg; // the mutex: the loop owns the mutation, serialized
        conn.write_all(state.to_string().as_bytes())
            .expect("service replies the new state");
    });

    // ── The client: connects to the named place, sends a request, reads the reply ──
    let mut client = UnixStream::connect_addr(&addr).expect("connect to the abstract UDS");
    client
        .write_all(b"increment 5")
        .expect("client sends the request");
    let mut buf = [0u8; 64];
    let n = client.read(&mut buf).expect("client reads the reply");
    let reply = std::str::from_utf8(&buf[..n]).expect("utf8 reply");

    assert_eq!(
        reply, "15",
        "the protected scalar (10) incremented by 5 to 15, through an abstract-namespace \
         UDS connection — listen/accept/connect proven on this box"
    );

    service.join().expect("service thread joins clean");
}

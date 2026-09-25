//! Stone 255.40 — a crash-channel io failure after output EOF is `Lost`
//! from both classifiers. `classify_peer_error` calls `classify_peer_death`
//! for that read, so the two cannot disagree.

use std::fs::File;
use std::os::fd::{FromRawFd, OwnedFd};

use wat::comms::{RecvError, process::sender_receiver_from_split_fds};
use wat::kernel::spawn::{PeerDeath, classify_peer_death, classify_peer_error};

fn lost(death: PeerDeath) -> String {
    match death {
        PeerDeath::Lost(reason) => reason,
        PeerDeath::Closed => panic!("a crash-channel io failure was reported as a clean Closed"),
        PeerDeath::Shutdown => panic!("a crash-channel io failure was reported as Shutdown"),
    }
}

#[test]
fn a_crash_channel_io_failure_after_output_eof_is_lost_on_both_paths() {
    let dir = File::open(".").expect("open .");
    let read_fd: OwnedFd = dir.into();
    let mut fds = [0i32; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0, "pipe");
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let _discard_read = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let (_tx, err_rx) =
        sender_receiver_from_split_fds::<String>(read_fd, write_fd).expect("crash receiver");

    let from_error = lost(classify_peer_error(&RecvError::Disconnected, &err_rx));
    let from_death = lost(classify_peer_death(Err(RecvError::Failed(
        "io_uring read failed".to_string(),
    ))));
    assert_eq!(from_error, from_death);
    assert_eq!(from_error, "io_uring read failed");
}

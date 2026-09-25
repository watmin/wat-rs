//! Stone 255.40 — an io_uring read that fails is `RecvError::Failed`.
//! A bad frame is `Malformed`. The source here is the read itself: the fd
//! is a directory, so the read cannot return bytes or a clean EOF.

use std::fs::File;
use std::os::fd::{FromRawFd, OwnedFd};

use wat::comms::{RecvError, process::sender_receiver_from_split_fds};

#[test]
fn an_io_uring_read_of_a_directory_is_failed() {
    let dir = File::open(".").expect("open .");
    let read_fd: OwnedFd = dir.into();
    let mut fds = [0i32; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0, "pipe");
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let _discard_read = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let (_tx, rx) =
        sender_receiver_from_split_fds::<String>(read_fd, write_fd).expect("receiver");
    assert_eq!(
        rx.recv(),
        Err(RecvError::Failed("io_uring read failed".to_string()))
    );
}

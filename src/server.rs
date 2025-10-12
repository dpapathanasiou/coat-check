use nix::errno::Errno;
use nix::sys::socket::{
    AddressFamily, Backlog, MsgFlags, SockFlag, SockProtocol, SockType, SockaddrIn, accept, bind,
    listen, recv, send, socket,
};
use std::os::fd::{AsRawFd, RawFd};

pub fn start(port: u16) -> Result<(), Errno> {
    // Create the server socket
    let fd = socket(
        AddressFamily::Inet,
        SockType::Stream,
        SockFlag::empty(),
        SockProtocol::Tcp,
    )?;
    let sockfd = fd.as_raw_fd();

    // Bind the socket to the specified port (on localhost)
    let addr = SockaddrIn::new(127, 0, 0, 1, port);
    bind(fd.as_raw_fd(), &addr)?;

    // Listen for incoming connections
    listen(&fd, Backlog::MAXALLOWABLE)?;
    println!("Server listening on {port}");

    // Accept and handle incoming connections
    handle(sockfd);

    Ok(())
}

pub fn handle(sockfd: RawFd) {
    /*
     * simple echo handler, inspired by https://github.com/woile/beej-rs/blob/main/src/examples/listener.rs
     * via https://woile.dev/posts/network-programming-in-rust/
     */

    let client = accept(sockfd).expect("Failed to accept client");
    let mut buf = [0u8; 1024];
    loop {
        recv(client, &mut buf, MsgFlags::empty()).expect("Failed to read from client");
        send(client, &mut buf, MsgFlags::empty()).expect("Failed to send to client");
    }
}

use crate::file_syscalls::{read_key, write_key_val};
use nix::errno::Errno;
use nix::sys::socket::{
    AddressFamily, Backlog, MsgFlags, SockFlag, SockProtocol, SockType, SockaddrIn, accept, bind,
    listen, recv, send, socket,
};
use std::os::fd::{AsRawFd, RawFd};

#[derive(Debug)]
pub struct Server {
    pub port: u16,
    pub filepath: String,
}

impl Server {
    pub fn start(&self) -> Result<(), Errno> {
        // Create the server socket
        let fd = socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            SockProtocol::Tcp,
        )?;
        let sockfd = fd.as_raw_fd();

        // Bind the socket to the specified port (on localhost)
        let addr = SockaddrIn::new(127, 0, 0, 1, self.port);
        bind(fd.as_raw_fd(), &addr)?;

        // Listen for incoming connections
        listen(&fd, Backlog::MAXALLOWABLE)?;
        println!("Server listening on {}", self.port);

        // Accept and handle incoming connections
        self.handle(sockfd);

        Ok(())
    }

    pub fn handle(&self, sockfd: RawFd) {
        let client = accept(sockfd).expect("Failed to accept client");

        let mut buf = [0u8; 1024];
        let read_err_msg = String::from("Failed to read from client");
        let write_err_msg = String::from("Failed to send to client");
        let usage = String::from("Usage:\r\n<get> <key> | <set> <key> <value>");

        let mut nbytes = recv(client, &mut buf, MsgFlags::empty()).expect(&read_err_msg);
        while nbytes > 0 {
            let input_size = buf
                .clone()
                .iter()
                .take_while(|c| **c != b'\n' && **c != b'\r')
                .count();
            if input_size > 0 {
                // Split the byte array on spaces
                let raw_input = buf.clone();
                let parts: Vec<&[u8]> = raw_input[..input_size].split(|&b| b == b' ').collect();

                // Reset the buffer for writing back to the client
                buf.fill(0);
                let mut replied: bool = false;

                let cmd_size = parts.len();
                if cmd_size > 1 {
                    let cmd = std::str::from_utf8(parts[0]).unwrap();
                    if cmd == "get" && cmd_size == 2 {
                        match read_key(self.filepath.clone(), str::from_utf8(parts[1]).unwrap()) {
                            Ok(bytes) => match bytes {
                                Some(result) => {
                                    let r = result.len();
                                    buf[0..r].copy_from_slice(result.as_slice());
                                    buf[r..r + 2].copy_from_slice(b"\r\n")
                                }
                                None => {
                                    let no_match = String::from("*** no match found");
                                    let r = no_match.len();
                                    buf[0..r].copy_from_slice(no_match.as_bytes());
                                    buf[r..r + 2].copy_from_slice(b"\r\n")
                                }
                            },
                            Err(e) => {
                                let result = format!("*** error: {:?}", e.desc());
                                let r = result.len();
                                buf[0..r].copy_from_slice(result.as_bytes());
                                buf[r..r + 2].copy_from_slice(b"\r\n")
                            }
                        };
                        send(client, &mut buf, MsgFlags::empty()).expect(&write_err_msg);
                        replied = true;
                    } else if cmd == "set" && cmd_size > 2 {
                        let key = str::from_utf8(parts[1]).unwrap();
                        // val as the remaining input, after the key
                        let val_start = key.len() + 5; // 5 = "set" and two spaces
                        match write_key_val(
                            self.filepath.clone(),
                            key,
                            &raw_input[val_start..input_size],
                        ) {
                            Ok(bytes) => {
                                let result = format!("*** success: wrote {bytes} bytes");
                                let r = result.len();
                                buf[0..r].copy_from_slice(result.as_bytes());
                                buf[r..r + 2].copy_from_slice(b"\r\n")
                            }
                            Err(e) => {
                                let result = format!("*** error: {:?}", e.desc());
                                let r = result.len();
                                buf[0..r].copy_from_slice(result.as_bytes());
                                buf[r..r + 2].copy_from_slice(b"\r\n")
                            }
                        }
                        send(client, &mut buf, MsgFlags::empty()).expect(&write_err_msg);
                        replied = true;
                    }
                };
                if !replied {
                    let result = format!("*** invalid command\r\n{usage}");
                    let r = result.len();
                    buf[0..r].copy_from_slice(result.as_bytes());
                    buf[r..r + 2].copy_from_slice(b"\r\n");
                    send(client, &mut buf, MsgFlags::empty()).expect(&write_err_msg);
                }
            }
            nbytes = recv(client, &mut buf, MsgFlags::empty()).expect(&read_err_msg);
        }
    }
}

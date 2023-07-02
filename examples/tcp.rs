use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use win_async::{Event, Events, Poller, SocketState};

fn main() {
    std::thread::spawn(|| listen().unwrap());

    let mut stream = TcpStream::connect("127.0.0.1:8001").unwrap();
    stream.write(b"Hello server!").unwrap();
    let mut buf = [0u8; 20];
    stream.read(&mut buf).unwrap();
    println!("Received: {}", std::str::from_utf8(&buf).unwrap());
    drop(stream);
    let time = std::time::Duration::new(2, 0);
    std::thread::sleep(time);
}

fn listen() -> std::io::Result<()> {
    const SERVER: mio::Token = mio::Token(0);
    let mut unique_token = mio::Token(SERVER.0 + 1);

    let mut states = std::collections::HashMap::new();
    let mut events = Events::with_capacity(128);

    let mut server = TcpListener::bind("127.0.0.1:8001")?;
    server.set_nonblocking(true)?;
    println!("Starting server...");

    let mut poll = Poller::new()?;
    let mut state = SocketState::new();
    poll.register(&mut server, &mut state, SERVER, mio::Interest::READABLE)?;

    loop {
        events.clear();
        poll.poll(&mut events, None)?;

        for event in events.events.iter() {
            match event.token() {
                SERVER => loop {
                    let (mut connection, address) = match server.accept() {
                        Ok((connection, address)) => (connection, address),
                        Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                        Err(e) => return Err(e),
                    };
                    println!("Accepting new connection from {}...", address);
                    let token = mio::Token(unique_token.0);
                    unique_token.0 += 1;
                    let mut state = SocketState::new();
                    poll.register(
                        &mut connection,
                        &mut state,
                        token,
                        mio::Interest::READABLE.add(mio::Interest::WRITABLE),
                    )?;
                    states.insert(token, (connection, state));
                },
                token => {
                    let done = if let Some((connection, state)) = states.get_mut(&token) {
                        handle_connection_event(&poll, state, connection, event)?
                    } else {
                        false
                    };
                    if done {
                        if let Some((_, mut state)) = states.remove(&token) {
                            poll.deregister(&mut state)?;
                        }
                    }
                }
            }
        }
    }
}

fn handle_connection_event(
    poll: &Poller,
    state: &mut SocketState,
    connection: &mut TcpStream,
    event: &Event,
) -> std::io::Result<bool> {
    if event.is_writable() {
        let data = b"Hello client!";

        match connection.write(data) {
            Ok(n) if n < data.len() => return Err(ErrorKind::WriteZero.into()),
            Ok(_) => poll.reregister(state, event.token(), mio::Interest::READABLE)?,
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                poll.reregister(state, state.token, state.interest)?;
            }
            Err(ref err) if err.kind() == ErrorKind::Interrupted => {
                return handle_connection_event(poll, state, connection, event)
            }
            Err(err) => return Err(err),
        }
    }

    if event.is_readable() {
        let mut connection_closed = false;
        let mut received_data = vec![0; 4096];
        let mut bytes_read = 0;
        loop {
            match connection.read(&mut received_data[bytes_read..]) {
                Ok(0) => {
                    connection_closed = true;
                    break;
                }
                Ok(n) => {
                    bytes_read += n;
                    if bytes_read == received_data.len() {
                        received_data.resize(received_data.len() + 1024, 0);
                    }
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                    poll.reregister(state, state.token, state.interest)?;
                    break;
                }
                Err(ref err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(err) => return Err(err),
            }
        }

        if bytes_read != 0 {
            let received_data = &received_data[..bytes_read];
            if let Ok(str_buf) = std::str::from_utf8(received_data) {
                println!("Received data: {}", str_buf.trim_end());
            } else {
                println!("Received (none UTF-8) data: {:?}", received_data);
            }
        }

        if connection_closed {
            println!("Connection closed");
            return Ok(true);
        }
    }

    Ok(false)
}

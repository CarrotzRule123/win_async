use std::net::{TcpListener, TcpStream};
use win_async::{Events, Poller};

fn main() {
    std::thread::spawn(listener);

    let _ = TcpStream::connect("127.0.0.1:8001").unwrap();
    let time = std::time::Duration::new(2, 0);
    std::thread::sleep(time);
}

fn listener() -> std::io::Result<()> {
    const SERVER: mio::Token = mio::Token(0);

    let mut events = Events::with_capacity(128);

    let mut server = TcpListener::bind("127.0.0.1:8001")?;
    server.set_nonblocking(true)?;
    println!("Starting server...");

    let poller = Poller::new()?;
    poller.register(&mut server, SERVER, mio::Interest::READABLE)?;

    loop {
        events.clear();
        poller.poll(&mut events, None)?;

        for ev in events.events.iter() {
            match ev.token() {
                SERVER => {
                    println!("Accepting new connection...");
                    server.accept()?;
                }
                _ => unreachable!(),
            }
        }
    }
}

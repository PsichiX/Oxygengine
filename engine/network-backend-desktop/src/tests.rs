#![cfg(test)]

use super::*;
use byteorder::{BigEndian, WriteBytesExt};
use std::{
    io::Cursor,
    thread::{sleep, spawn},
    time::Duration,
};
use ws::connect;

#[test]
fn test_general() {
    let msg = MessageId::new(42, 1);

    println!("Create server");
    let mut server = DesktopServer::open("127.0.0.1:12345").unwrap();
    println!("Wait for server");
    while server.state() != ServerState::Open {
        server.process();
        sleep(Duration::from_millis(10));
    }

    println!("Create client");
    let client = spawn(move || {
        connect("ws://127.0.0.1:12345", |out| {
            println!("Send message");
            let mut stream = Cursor::new(Vec::<u8>::with_capacity(8));
            drop(stream.write_u32::<BigEndian>(msg.id()));
            drop(stream.write_u32::<BigEndian>(msg.version()));
            let data = stream.into_inner();
            out.send(data).unwrap();

            move |msg| {
                println!("Client got message {:?}", msg);
                out.close(CloseCode::Normal)
            }
        })
        .unwrap()
    });
    println!("Wait for client");
    while server.clients().is_empty() {
        server.process();
        sleep(Duration::from_millis(10));
    }

    println!("Wait for message");
    loop {
        server.process();
        if let Some((_, m, _)) = server.read() {
            println!("Resend message");
            assert_eq!(m, msg);
            server.send_all(m, &[]);
            break;
        }
    }

    println!("Wait for client disconnect");
    while !server.clients().is_empty() {
        server.process();
        sleep(Duration::from_millis(10));
    }

    let server = server.close();
    println!("Wait for server close");
    while server.state() != ServerState::Closed {
        sleep(Duration::from_millis(10));
    }

    println!("Wait for client close");
    client.join().unwrap();
}

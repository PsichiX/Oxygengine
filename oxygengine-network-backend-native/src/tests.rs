#![cfg(test)]

use super::prelude::*;
use crate::network::{
    client::{Client, ClientState, MessageID},
    server::{Server, ServerState},
};
use std::{thread::sleep, time::Duration};

#[test]
fn test_general() {
    let msg = MessageID::new(42, 1);

    println!("Create server");
    let mut server = NativeServer::open("127.0.0.1:12345").unwrap();
    println!("Wait for server");
    while server.state() != ServerState::Open {
        server.process();
        sleep(Duration::from_millis(10));
    }

    println!("Create client");
    let mut client = NativeClient::open("127.0.0.1:12345").unwrap();
    println!("Wait for client");
    while client.state() != ClientState::Open {
        sleep(Duration::from_millis(10));
    }
    while server.clients().is_empty() {
        server.process();
        sleep(Duration::from_millis(10));
    }

    println!("Send message to server");
    client.send(msg, &[]).unwrap();

    println!("Wait for message from client");
    loop {
        server.process();
        if let Some((_, m, _)) = server.read() {
            println!("Resend message to client");
            assert_eq!(m, msg);
            server.send_all(m, &[]);
            break;
        }
    }

    println!("Wait for message from server");
    loop {
        if let Some((m, _)) = client.read() {
            assert_eq!(m, msg);
            client = client.close();
            break;
        }
    }

    println!("Wait for client close");
    while client.state() != ClientState::Closed {
        sleep(Duration::from_millis(10));
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
}

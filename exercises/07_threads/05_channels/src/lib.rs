use crate::data::TicketDraft;
use crate::store::TicketStore;
use std::sync::mpsc::{Receiver, Sender};

pub mod data;
pub mod store;

pub enum Command {
    Insert(TicketDraft),
}

// Start the system by spawning the server thread.
// It returns a `Sender` instance which can then be used
// by one or more clients to interact with the server.
pub fn launch() -> Sender<Command> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || server(receiver));
    sender
}

// TODO: The server task should **never** stop.
//  Enter a loop: wait for a command to show up in
//  the channel, then execute it, then start waiting
//  for the next command.
pub fn server(receiver: Receiver<Command>) {
    let mut ticket_store = TicketStore::new();
    loop {
        let result = receiver.recv();
        match result {
            Ok(Command::Insert(ticket_drift)) => {
                ticket_store.add_ticket(ticket_drift);
                ()
            }
            Err(error) => println!("recv error"),
        }
    }
}

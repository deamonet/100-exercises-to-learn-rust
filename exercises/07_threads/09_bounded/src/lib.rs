// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc::{Receiver, RecvError, SendError, SyncSender};

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>,
    capacity: usize,
}

#[derive(Debug)]
pub enum Error {
    SendError(SendError<Command>),
    RecvError(RecvError),
}

impl TicketStoreClient {
    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, Error> {
        let (response_sender, response_receiver) = std::sync::mpsc::sync_channel(self.capacity);
        let command = Command::Insert {
            draft,
            response_channel: response_sender,
        };
        let result = self.sender.send(command);
        match result {
            Ok(()) => match response_receiver.recv() {
                Ok(id) => Ok(id),
                Err(error) => Err(Error::RecvError(error)),
            },
            Err(error) => Err(Error::SendError(error)),
        }
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, Error> {
        let (response_sender, response_receiver) = std::sync::mpsc::sync_channel(self.capacity);
        let command = Command::Get {
            id,
            response_channel: response_sender,
        };
        let result = self.sender.send(command);
        match result {
            Ok(()) => match response_receiver.recv() {
                Ok(ticket) => Ok(ticket),
                Err(_error) => Ok(Option::None),
            },
            Err(error) => Err(Error::SendError(error)),
        }
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = std::sync::mpsc::sync_channel(capacity);
    std::thread::spawn(move || server(receiver));
    TicketStoreClient { sender, capacity }
}

enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: SyncSender<TicketId>,
    },
    Get {
        id: TicketId,
        response_channel: SyncSender<Option<Ticket>>,
    },
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
                response_channel.send(id).expect("");
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
                response_channel.send(ticket.cloned()).expect("")
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}

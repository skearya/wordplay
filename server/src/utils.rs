use crate::game::{messages::ServerMessage, room::Client};
use axum::extract::ws::Message;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub trait ClientUtils {
    fn connected(&self) -> impl Iterator<Item = (&Uuid, &Client)>;
    fn send_each(&self, f: impl Fn(&Uuid, &Client) -> ServerMessage);
    fn broadcast(&self, message: ServerMessage);
}

impl ClientUtils for HashMap<Uuid, Client> {
    fn connected(&self) -> impl Iterator<Item = (&Uuid, &Client)> {
        self.iter().filter(|client| client.1.socket.is_some())
    }

    fn send_each(&self, f: impl Fn(&Uuid, &Client) -> ServerMessage) {
        for (uuid, client) in self.connected() {
            client.send(f(uuid, client));
        }
    }

    fn broadcast(&self, message: ServerMessage) {
        let serialized: Message = message.into();

        for (_uuid, client) in self.connected() {
            client.tx.send(serialized.clone()).ok();
        }
    }
}

pub trait UnixTime {
    fn to_unix_timestamp(&self) -> u32;
}

impl UnixTime for SystemTime {
    fn to_unix_timestamp(&self) -> u32 {
        self.duration_since(UNIX_EPOCH)
            .expect("we have time traveled to before 1970-01-01 00:00:00 UTC")
            .as_secs() as u32
    }
}

// https://docs.rs/itertools/0.9.0/src/itertools/lib.rs.html#2061
pub trait Sorted: Iterator {
    fn sorted_by_vec<F>(self, cmp: F) -> Vec<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
    {
        let mut v: Vec<_> = self.collect();
        v.sort_by(cmp);
        v
    }
}

impl<I: Iterator> Sorted for I {}

pub fn filter_string(input: &mut String) {
    input.make_ascii_lowercase();
    input.retain(|c| c.is_alphabetic());
}

pub fn random_string(len: usize) -> String {
    Alphanumeric.sample_string(&mut thread_rng(), len)
}

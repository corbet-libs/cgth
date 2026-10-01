//! Thin composition of current volatile presence, candidate planning and quotas.
#![forbid(unsafe_code)]

use cmmr::{Clock, Revision, Store};
use cswb::{Admission, Current, Live, Session, Switchboard, Tokens};

pub use clkp::{IndexKey, Plan, Projection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    QuotaUnavailable,
    QuotaExceeded,
    Presence(cswb::Error),
    RoomsUnavailable,
}

/// Composition holds child handles only, never its own roster or matching state.
pub struct Gather<S, C, R, Q> {
    presence: Switchboard<S, C, R>,
    throttle: cthl::Throttle<Q>,
}

impl<S: Store, C: Clock, R: Tokens, Q: cthl::Store> Gather<S, C, R, Q> {
    pub fn new(presence: Switchboard<S, C, R>, throttle: cthl::Throttle<Q>) -> Self {
        Self { presence, throttle }
    }

    /// Cheap configured quotas precede the caller's expensive verification.
    /// This does not authenticate or admit a member.
    pub async fn preflight(&self, key: &[u8], action: &str) -> Result<(), Error> {
        match self.throttle.check(key, action).await {
            Ok(cthl::Decision::Allowed) => Ok(()),
            Ok(cthl::Decision::Denied { .. }) => Err(Error::QuotaExceeded),
            Err(_) => Err(Error::QuotaUnavailable),
        }
    }

    pub async fn enter(&self, admission: Admission) -> Result<(Session, Live), Error> {
        self.presence.enter(admission).await.map_err(Error::Presence)
    }

    pub async fn read(&self, session: &Session, current: &Current) -> Result<Live, Error> {
        self.presence.read(session, current).await.map_err(Error::Presence)
    }

    pub async fn replace(&self, session: &Session, current: &Current, expected: Revision, admission: Admission) -> Result<Live, Error> {
        self.presence.replace(session, current, expected, admission).await.map_err(Error::Presence)
    }

    pub async fn heartbeat(&self, session: &Session, current: &Current, expected: Revision) -> Result<Live, Error> {
        self.presence.heartbeat(session, current, expected).await.map_err(Error::Presence)
    }

    pub async fn depart(&self, session: &Session) -> Result<usize, Error> {
        self.presence.depart(session).await.map_err(Error::Presence)
    }

    pub async fn expire(&self) -> Result<usize, Error> {
        self.presence.expire().await.map_err(Error::Presence)
    }
}

/// Assembly's anonymous permits, ordering and restart recovery remain an owner
/// integration boundary. No local roster, counter or substitute room protocol.
pub enum RoomAuthority {}

pub fn require_rooms() -> Result<RoomAuthority, Error> {
    Err(Error::RoomsUnavailable)
}

#[cfg(test)]
mod tests;

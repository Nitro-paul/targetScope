//! Redis bus: requests and responses, the request loop, events (doc/shared_libraries.md#redis-bus).

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::messages::{ErrorBody, Request, ServiceError};

/// Connection of one program to Redis.
pub struct Bus {
    // TODO: redis::aio::ConnectionManager, program name, pending-response map (request_id -> oneshot)
}

/// How an event stream is read (doc/interfaces.md#who-listens-to-which-events).
#[derive(Debug, Clone)]
pub enum ReadMode {
    /// XREADGROUP with a group named after the listener: no event is lost.
    ConsumerGroup,
    /// XREAD BLOCK starting after the given entry id.
    After(String),
}

#[derive(Debug, thiserror::Error)]
pub enum BusError {
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("timeout")]
    Timeout,
    #[error("remote error: {0:?}")]
    Remote(ErrorBody),
    #[error("decode: {0}")]
    Decode(String),
}

#[async_trait]
pub trait RequestHandler<Req>: Send + Sync {
    async fn handle(&self, request_id: Uuid, req: Req) -> Result<serde_json::Value, ServiceError>;
}

impl Bus {
    /// Connects and starts a background task that reads `<program>:responses`
    /// and completes waiting requests by request_id.
    pub async fn connect(url: &str, program: &str) -> Result<Self, BusError> {
        let _ = (url, program);
        todo!()
    }

    /// Sends a request with a new request_id and waits for the response (`Request::TIMEOUT`).
    pub async fn request<R: Request>(&self, req: R) -> Result<R::Response, BusError> {
        self.request_with_id(Uuid::new_v4(), req).await
    }

    /// Same, but with a given request_id, for resending.
    pub async fn request_with_id<R: Request>(
        &self,
        id: Uuid,
        req: R,
    ) -> Result<R::Response, BusError> {
        let _ = (id, req);
        todo!()
    }

    /// Sends an event to `<program>:events`.
    pub async fn publish<E: Serialize + Send>(&self, event: E) -> Result<(), BusError> {
        let _ = event;
        todo!()
    }

    /// Reads `<source>:events` with a consumer group or as a plain reader.
    pub fn events<E: DeserializeOwned>(&self, source: &str, mode: ReadMode) -> EventReader<E> {
        let _ = (source, mode);
        todo!()
    }

    /// Request loop: reads pending, then new requests; calls the handler; sends the response;
    /// acknowledges. Answers `ping` itself. Parsing errors are answered with `invalid_message`.
    pub async fn serve<Req, H>(&self, handler: H) -> Result<(), BusError>
    where
        Req: DeserializeOwned + Send,
        H: RequestHandler<Req>,
    {
        let _ = handler;
        todo!()
    }
}

/// Reader of one event stream.
pub struct EventReader<E> {
    _marker: std::marker::PhantomData<E>,
    // TODO: stream name, mode, last id
}

impl<E: DeserializeOwned> EventReader<E> {
    /// Waits for the next event; returns its entry id, event_id and the event.
    pub async fn next(&mut self) -> Result<(String, Uuid, E), BusError> {
        todo!()
    }

    /// Acknowledges an entry (consumer group mode only).
    pub async fn ack(&mut self, entry_id: &str) -> Result<(), BusError> {
        let _ = entry_id;
        todo!()
    }
}

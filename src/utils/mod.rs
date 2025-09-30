use futures::Stream;
use rmcp::model::{ClientJsonRpcMessage, ServerJsonRpcMessage};
use rmcp::transport::common::server_side_http::ServerSseMessage;
use rmcp::transport::streamable_http_server::{SessionId, SessionManager};
use std::future::Future;
use std::sync::Arc;

pub mod shutdown;
mod swagger_util;
mod util;

use crate::services::SessionService;
pub use shutdown::*;
pub use swagger_util::*;
pub use util::*;

pub struct MonitoredSessionManager<SM> {
    inner: SM,
    session_service: Arc<SessionService>,
}

impl<SM> MonitoredSessionManager<SM> {
    pub fn new(inner: SM, session_service: Arc<SessionService>) -> Self {
        Self {
            inner,
            session_service,
        }
    }
}

#[async_trait::async_trait]
impl<SM> SessionManager for MonitoredSessionManager<SM>
where
    SM: SessionManager,
{
    type Error = SM::Error;
    type Transport = SM::Transport;

    fn create_session(
        &self,
    ) -> impl Future<Output = Result<(SessionId, Self::Transport), Self::Error>> + Send {
        let future = self.inner.create_session();
        async {
            match future.await {
                Ok((session_id, transport)) => {
                    self.session_service.pre_save_cache(session_id.clone());
                    Ok((session_id, transport))
                }
                Err(e) => Err(e),
            }
        }
    }

    fn initialize_session(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> impl Future<Output = Result<ServerJsonRpcMessage, Self::Error>> + Send {
        self.inner.initialize_session(id, message)
    }

    fn has_session(
        &self,
        id: &SessionId,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send {
        self.inner.has_session(id)
    }

    fn close_session(
        &self,
        id: &SessionId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async {
            self.session_service.destroy_session(id).await;
            self.inner.close_session(id).await
        }
    }

    fn create_stream(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> impl Future<
        Output = Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error>,
    > + Send {
        self.inner.create_stream(id, message)
    }

    fn accept_message(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.inner.accept_message(id, message)
    }

    fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> impl Future<
        Output = Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error>,
    > + Send {
        self.inner.create_standalone_stream(id)
    }

    fn resume(
        &self,
        id: &SessionId,
        last_event_id: String,
    ) -> impl Future<
        Output = Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error>,
    > + Send {
        self.inner.resume(id, last_event_id)
    }
}

//! The RFC 959 Abort (`ABOR`) command
//
// This command tells the server to abort the previous FTP
// service command and any associated transfer of data. The
// abort command may require "special action", as discussed in
// the Section on FTP Commands, to force recognition by the
// server.  No action is to be taken if the previous command
// has been completed (including data transfer).  The control
// connection is not to be closed by the server, but the data
// connection must be closed.

use crate::auth::UserDetail;
use crate::{
    server::controlchan::{
        error::ControlChanError,
        handler::{CommandContext, CommandHandler},
        Reply, ReplyCode,
    },
    storage::{Metadata, StorageBackend},
};
use async_trait::async_trait;
use futures::prelude::*;
use log::warn;

#[derive(Debug)]
pub struct Abor;

#[async_trait]
impl<S, U> CommandHandler<S, U> for Abor
where
    S: StorageBackend<U> + 'static,
    S::File: tokio::io::AsyncRead + Send,
    S::Metadata: Metadata,
    U: UserDetail + 'static,
{
    #[tracing_attributes::instrument]
    async fn handle(&self, args: CommandContext<S, U>) -> Result<Reply, ControlChanError> {
        let mut session = args.session.lock().await;
        match session.data_abort_tx.take() {
            Some(mut tx) => {
                tokio::spawn(async move {
                    if let Err(err) = tx.send(()).await {
                        warn!("abort failed: {}", err);
                    }
                });
                Ok(Reply::new(ReplyCode::ClosingDataConnection, "Closed data channel"))
            }
            None => Ok(Reply::new(ReplyCode::ClosingDataConnection, "Data channel already closed")),
        }
    }
}

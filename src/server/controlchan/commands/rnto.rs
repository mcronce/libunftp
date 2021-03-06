//! The RFC 959 Rename To (`RNTO`) command

use crate::storage::{Metadata, StorageBackend};
use crate::{
    auth::UserDetail,
    server::controlchan::{
        error::ControlChanError,
        handler::{CommandContext, CommandHandler},
        Reply, ReplyCode,
    },
};
use async_trait::async_trait;
use log::warn;
use std::{path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct Rnto {
    path: PathBuf,
}

impl Rnto {
    pub fn new(path: PathBuf) -> Self {
        Rnto { path }
    }
}

#[async_trait]
impl<S, U> CommandHandler<S, U> for Rnto
where
    U: UserDetail + 'static,
    S: StorageBackend<U> + 'static,
    S::File: tokio::io::AsyncRead + Send,
    S::Metadata: Metadata,
{
    #[tracing_attributes::instrument]
    async fn handle(&self, args: CommandContext<S, U>) -> Result<Reply, ControlChanError> {
        let mut session = args.session.lock().await;
        let storage = Arc::clone(&session.storage);
        let reply = match session.rename_from.take() {
            Some(from) => {
                let to = session.cwd.join(self.path.clone());
                match storage.rename(&session.user, from, to).await {
                    Ok(_) => Reply::new(ReplyCode::FileActionOkay, "Renamed"),
                    Err(err) => {
                        warn!("Error renaming: {:?}", err);
                        Reply::new(ReplyCode::FileError, "Storage error while renaming")
                    }
                }
            }
            None => Reply::new(ReplyCode::TransientFileError, "Please tell me what file you want to rename first"),
        };
        Ok(reply)
    }
}

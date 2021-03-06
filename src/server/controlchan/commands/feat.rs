//! The RFC 2389 Feature (`FEAT`) command

use crate::{
    auth::UserDetail,
    server::controlchan::{
        error::ControlChanError,
        handler::{CommandContext, CommandHandler},
        Reply, ReplyCode,
    },
    storage::{Metadata, StorageBackend, FEATURE_RESTART},
};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Feat;

#[async_trait]
impl<S, U> CommandHandler<S, U> for Feat
where
    U: UserDetail + 'static,
    S: StorageBackend<U> + 'static,
    S::File: tokio::io::AsyncRead + Send,
    S::Metadata: Metadata,
{
    #[tracing_attributes::instrument]
    async fn handle(&self, args: CommandContext<S, U>) -> Result<Reply, ControlChanError> {
        let mut feat_text = vec![" SIZE", " MDTM", "UTF8"];
        // Add the features. According to the spec each feature line must be
        // indented by a space.
        if args.tls_configured {
            feat_text.push(" AUTH TLS");
            feat_text.push(" PBSZ");
            feat_text.push(" PROT");
        }
        if args.storage_features & FEATURE_RESTART > 0 {
            feat_text.push(" REST STREAM");
        }

        // Show them in alphabetical order.
        feat_text.sort();
        feat_text.insert(0, "Extensions supported:");
        feat_text.push("END");

        let reply = Reply::new_multiline(ReplyCode::SystemStatus, feat_text);
        Ok(reply)
    }
}

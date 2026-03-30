use super::{LspClient, VirtualChange, VirtualDocument};
use crate::{Result, TsgoError};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, Uri,
    notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
};
use parking_lot::RwLock;
use std::sync::Arc;
use tsgo_rs_core::fast::{CompactString, FastMap, SmallVec, compact_format};

/// In-memory virtual document overlay synchronized with an [`LspClient`].
#[derive(Clone)]
pub struct LspOverlay {
    client: LspClient,
    documents: Arc<RwLock<FastMap<CompactString, VirtualDocument>>>,
}

impl LspOverlay {
    /// Creates an empty overlay bound to a client.
    pub fn new(client: LspClient) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(FastMap::default())),
        }
    }

    /// Returns the client used to send LSP notifications.
    pub fn client(&self) -> &LspClient {
        &self.client
    }

    /// Returns a virtual document by URI if it is currently open.
    pub fn document(&self, uri: &Uri) -> Option<VirtualDocument> {
        self.documents.read().get(uri.as_str()).cloned()
    }

    /// Returns a snapshot of all open virtual documents.
    pub fn documents(&self) -> SmallVec<[VirtualDocument; 8]> {
        self.documents.read().values().cloned().collect()
    }

    /// Opens a virtual document and emits `textDocument/didOpen`.
    pub fn open(&self, document: VirtualDocument) -> Result<VirtualDocument> {
        let key = document.key();
        if self.documents.read().contains_key(key.as_str()) {
            return Err(TsgoError::Protocol(compact_format(format_args!(
                "virtual document is already open: {key}"
            ))));
        }
        self.client
            .notify::<DidOpenTextDocument>(DidOpenTextDocumentParams {
                text_document: document.text_document_item(),
            })?;
        self.documents.write().insert(key, document.clone());
        Ok(document)
    }

    /// Replaces the full contents of a virtual document.
    pub fn replace(&self, uri: &Uri, text: impl Into<CompactString>) -> Result<VirtualDocument> {
        self.change(uri, [VirtualChange::replace(text)])
    }

    /// Applies one or more incremental changes to a virtual document.
    pub fn change<I>(&self, uri: &Uri, changes: I) -> Result<VirtualDocument>
    where
        I: IntoIterator<Item = VirtualChange>,
    {
        let changes = changes
            .into_iter()
            .collect::<SmallVec<[VirtualChange; 4]>>();
        let mut documents = self.documents.write();
        let document = documents.get_mut(uri.as_str()).ok_or_else(|| {
            TsgoError::Protocol(compact_format(format_args!(
                "unknown virtual document: {}",
                uri.as_str()
            )))
        })?;
        let events = document.apply_changes(&changes)?;
        if events.is_empty() {
            return Ok(document.clone());
        }
        self.client
            .notify::<DidChangeTextDocument>(DidChangeTextDocumentParams {
                text_document: document.versioned_identifier(),
                content_changes: events.into_iter().collect(),
            })?;
        Ok(document.clone())
    }

    /// Closes a virtual document and emits `textDocument/didClose`.
    pub fn close(&self, uri: &Uri) -> Result<Option<VirtualDocument>> {
        let removed = self.documents.write().remove(uri.as_str());
        if let Some(document) = &removed {
            self.client
                .notify::<DidCloseTextDocument>(DidCloseTextDocumentParams {
                    text_document: document.identifier(),
                })?;
        }
        Ok(removed)
    }
}

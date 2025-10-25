// src/lsp_layer.rs

use crossbeam::channel::Sender;
use lsp_server::{Message, Notification};
use lsp_types::{
    LogMessageParams, MessageType,
    notification::{LogMessage, Notification as _},
};
use tracing::{Event, Level, Subscriber, field::Visit};
use tracing_subscriber::{Layer, layer::Context};

/// A `tracing_subscriber::Layer` that sends formatted log messages
/// over a channel to be forwarded to the LSP client.
pub struct LspLayer {
    sender: Sender<Message>,
}

impl LspLayer {
    pub fn new(sender: Sender<Message>) -> Self {
        Self { sender }
    }
}

// This is the main implementation that hooks into the `tracing` ecosystem.
impl<S> Layer<S> for LspLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    /// This method is called for every `tracing` event.
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Use a visitor to extract the formatted message from the event.
        let mut visitor = LspMessageVisitor::new();
        event.record(&mut visitor);

        // Send the log message through the channel.
        let _ = self.sender.send(Message::Notification(Notification::new(
            LogMessage::METHOD.to_owned(),
            LogMessageParams {
                typ: match *event.metadata().level() {
                    Level::ERROR => MessageType::ERROR,
                    Level::WARN => MessageType::WARNING,
                    Level::INFO => MessageType::INFO,
                    Level::DEBUG | Level::TRACE => MessageType::LOG,
                },
                message: format!("{} {}", visitor.message, visitor.fields),
            },
        )));
    }
}

/// A `tracing::field::Visit` implementation to extract the formatted message
/// from an event's fields.
struct LspMessageVisitor {
    message: String,
    fields: String,
}

impl LspMessageVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: String::new(),
        }
    }
}

impl Visit for LspMessageVisitor {
    // We are only interested in the `message` field.
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            self.fields
                .push_str(&format!("{:?} = {:?}, ", field.name(), value));
        }
    }
}

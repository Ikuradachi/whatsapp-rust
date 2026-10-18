use wacore_binary::Jid;

/// Opt-in policy that decides whether a message we failed to decrypt gets an
/// outbound retry receipt, or is acked and dropped without one.
///
/// By default every decrypt failure asks the sender to re-encrypt, capped at
/// `MAX_DECRYPT_RETRIES` per message id, matching WhatsApp Web. That cap is per
/// message, not per sender: a cohort of senders whose sessions with us are
/// broken makes every one of their messages fail, so in a large, busy group the
/// retry receipts we emit scale with the group's message volume. A bot account
/// emitting retry receipts at that rate has been locked by the server.
///
/// A `DecryptFailureAdmission` policy is the seam for bot-scale deployments
/// that want to bound that outbound volume (e.g. a token bucket per sender),
/// without the SDK itself diverging from WA Web. Leaving it unset keeps the
/// default behavior; the check is a single [`std::sync::OnceLock::get`].
///
/// When the policy returns `false`, the retry receipt is skipped but the
/// stanza is still transport-acked, so the server stops redelivering it. The
/// `UndecryptableMessage` event is dispatched either way. A dropped message is
/// not requested again, so a policy should refill over time to keep genuine
/// recovery possible. A refused first attempt still sends the one-shot PDO
/// request to our primary phone, as the default path does.
///
/// [`admit`](DecryptFailureAdmission::admit) runs inline on the receive path:
/// it must be a fast, local decision with no I/O or blocking.
///
/// Scope: consulted only on the automatic decrypt-failure path, for messages
/// from other accounts, after the per-message cap: redeliveries already past
/// `MAX_DECRYPT_RETRIES` never reach the policy. Our own devices' messages
/// (`is_from_me`) and explicit retry requests are never consulted either.
pub trait DecryptFailureAdmission: wacore::sync_marker::MaybeSendSync {
    /// Return `true` to send the retry receipt (default behavior), `false` to
    /// ack the stanza without one.
    ///
    /// `chat` is the conversation the message arrived in (the group, or the
    /// peer for a DM) and `sender` the sending device.
    fn admit(&self, chat: &Jid, sender: &Jid) -> bool;
}

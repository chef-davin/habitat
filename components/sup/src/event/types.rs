//! All the individual event types that can be sent out by the
//! Supervisor.

use super::EventCore;
use crate::manager::service::{HealthCheckResult as DomainHealthCheckResult,
                              Service,
                              UpdateStrategy as DomainUpdateStrategy};
use prost::Message;

include!(concat!(env!("OUT_DIR"), "/chef.habitat.supervisor.event.rs"));

// Note: `HealthCheck` here is the protobuf-generated type for the
// event we're sending out; `DomainHealthCheckResult` is the one we use
// elsewhere in the Supervisor.
impl Into<HealthCheck> for DomainHealthCheckResult {
    fn into(self) -> HealthCheck {
        match self {
            DomainHealthCheckResult::Ok => HealthCheck::Ok,
            DomainHealthCheckResult::Warning => HealthCheck::Warning,
            DomainHealthCheckResult::Critical => HealthCheck::Critical,
            DomainHealthCheckResult::Unknown => HealthCheck::Unknown,
        }
    }
}

impl Service {
    /// Create a protobuf metadata struct for Service-related event
    /// messages.
    // NOTE This was originally `pub(super)`, but it's `pub` for now
    // because it seems better to be able to generate this when we
    // kick off the health checking future for a service, rather than
    // cloning the entire service for eventing.
    pub fn to_service_metadata(&self) -> ServiceMetadata {
        ServiceMetadata { package_ident: self.pkg.ident.to_string(),
                          spec_ident:    self.spec_ident.to_string(),
                          service_group: self.service_group.to_string(),
                          update_config: self.update_config(), }
    }

    /// `UpdateConfig` is a (currently protobuf-only) type that
    /// encapsulates a channel and update strategy. Importantly, the
    /// existing `UpdateStrategy::None` variant is essentially
    /// converted to `Option::None`, whereas the other variants are
    /// coupled with the channel from which the Supervisor pulls
    /// updates.
    fn update_config(&self) -> Option<UpdateConfig> {
        let strategy = match self.update_strategy {
            DomainUpdateStrategy::None => {
                return None;
            }
            DomainUpdateStrategy::AtOnce => UpdateStrategy::AtOnce,
            DomainUpdateStrategy::Rolling => UpdateStrategy::Rolling,
        };

        Some(UpdateConfig { strategy: strategy.into(),
                            channel:  self.channel.to_string(), })
    }
}

impl EventCore {
    /// Create a protobuf metadata struct for all event messages.
    pub(super) fn to_event_metadata(&self) -> EventMetadata {
        EventMetadata { supervisor_id: self.supervisor_id.clone(),
                        ip_address:    self.ip_address.to_string(),
                        application:   self.application.clone(),
                        environment:   self.environment.clone(),
                        timestamp:     None,
                        meta:          self.meta.clone().into(), }
    }
}

pub trait EventMessage: Message + Sized {
    /// All messages will have some top-level metadata about the
    /// Supervisor they come from. This function allows us to set it
    /// generically when we send the message out.
    fn event_metadata(&mut self, event_metadata: EventMetadata);

    /// Convert a message to bytes for sending to NATS.
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = bytes::BytesMut::with_capacity(self.encoded_len());
        // The only way this can fail is if the buffer doesn't have
        // enough room. We just set that, though, so something would
        // have to be seriously wrong in Prost for this to fail.
        self.encode(&mut buf)
            .expect("UNEXPECTED PROST ERROR: encoded_len() was not long enough!");
        buf.to_vec()
    }
}

// TODO (CM): these are repetitive, and will only get more so as we
// add events. Consider implementing via macro instead.

impl EventMessage for ServiceStartedEvent {
    fn event_metadata(&mut self, event_metadata: EventMetadata) {
        self.event_metadata = Some(event_metadata);
    }
}

impl EventMessage for ServiceStoppedEvent {
    fn event_metadata(&mut self, event_metadata: EventMetadata) {
        self.event_metadata = Some(event_metadata);
    }
}

impl EventMessage for HealthCheckEvent {
    fn event_metadata(&mut self, event_metadata: EventMetadata) {
        self.event_metadata = Some(event_metadata);
    }
}

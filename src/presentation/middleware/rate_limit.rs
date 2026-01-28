use axum::{body::Body, extract::ConnectInfo};
use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder, key_extractor::KeyExtractor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmartIpKeyExtractor;

impl KeyExtractor for SmartIpKeyExtractor {
    type Key = IpAddr;

    fn extract<B>(
        &self,
        req: &axum::http::Request<B>,
    ) -> Result<Self::Key, tower_governor::errors::GovernorError> {
        req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip())
            .or_else(|| {
                // Return local IP if connection info is missing (e.g. in tests)
                Some(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))
            })
            .ok_or(tower_governor::errors::GovernorError::UnableToExtractKey)
    }
}

pub fn rate_limit_layer()
-> anyhow::Result<GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>, Body>> {
    let rate_limit = env::var("RATE_LIMIT_PER_MINUTE")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<u64>()
        .unwrap_or(60);

    custom_rate_limit_layer(rate_limit)
}

pub fn custom_rate_limit_layer(
    requests_per_minute: u64,
) -> anyhow::Result<GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>, Body>> {
    let quota_duration_ms = 60_000 / requests_per_minute;

    let config = Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(quota_duration_ms)
            .burst_size(requests_per_minute as u32)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .ok_or_else(|| anyhow::anyhow!("Failed to finish governor config"))?,
    );

    Ok(GovernorLayer::new(config))
}

use anyhow::Result;
use zcash_keys::keys::{UnifiedAddressRequest, UnifiedFullViewingKey};
use zcash_protocol::consensus;
use zip32::DiversifierIndex;

/// Parses a UFVK from its encoded string representation.
pub fn parse_ufvk<P: consensus::Parameters>(
    params: &P,
    encoded: &str,
) -> Result<UnifiedFullViewingKey> {
    UnifiedFullViewingKey::decode(params, encoded)
        .map_err(|e| anyhow::anyhow!("Failed to decode UFVK: {}", e))
}

/// Generates a unique Orchard-only Unified Address and returns its encoded form
/// for a specific network.
pub fn address_for_index_encoded<P: consensus::Parameters>(
    ufvk: &UnifiedFullViewingKey,
    params: &P,
    index: u32,
) -> Result<String> {
    let div_idx = DiversifierIndex::from(index);

    let ua = ufvk
        .address(div_idx, UnifiedAddressRequest::ORCHARD)
        .map_err(|e| anyhow::anyhow!("Failed to generate address at index {}: {:?}", index, e))?;

    Ok(ua.encode(params))
}

/// Get the raw UnifiedAddress for comparison during scanning.
pub fn unified_address_at(
    ufvk: &UnifiedFullViewingKey,
    index: u32,
) -> Result<zcash_keys::address::UnifiedAddress> {
    let div_idx = DiversifierIndex::from(index);
    ufvk.address(div_idx, UnifiedAddressRequest::ORCHARD)
        .map_err(|e| anyhow::anyhow!("Failed to generate address at index {}: {:?}", index, e))
}

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct FeeConfig {
    pub program_entry_percent: f64,
    pub hosting_percent: f64,
    pub renewal_percent: f64,
    pub operator_address: String,
}

impl FeeConfig {
    pub fn from_config(config: &Config) -> Option<Self> {
        let addr = config.fee_operator_address.clone()?;
        Some(FeeConfig {
            program_entry_percent: config.fee_program_entry_percent,
            hosting_percent: config.fee_hosting_percent,
            renewal_percent: config.fee_renewal_percent,
            operator_address: addr,
        })
    }
}

/// Returns (participant_amount, operator_fee) where both sum to the original amount.
pub fn calculate_fee(amount_zat: u64, fee_percent: f64) -> (u64, u64) {
    if fee_percent <= 0.0 || fee_percent >= 100.0 {
        return (amount_zat, 0);
    }
    let fee = (amount_zat as f64 * (fee_percent / 100.0)).round() as u64;
    let participant = amount_zat.saturating_sub(fee);
    (participant, fee)
}

/// Look up the fee percent for a given invoice type.
pub fn fee_for_invoice_type(invoice_type: &str, config: &FeeConfig) -> f64 {
    match invoice_type {
        "program" => config.program_entry_percent,
        "hosting" => config.hosting_percent,
        "renewal" => config.renewal_percent,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fee_basic() {
        let (participant, fee) = calculate_fee(100_000, 5.0);
        assert_eq!(fee, 5_000);
        assert_eq!(participant, 95_000);
    }

    #[test]
    fn test_calculate_fee_zero_percent() {
        let (participant, fee) = calculate_fee(100_000, 0.0);
        assert_eq!(fee, 0);
        assert_eq!(participant, 100_000);
    }

    #[test]
    fn test_calculate_fee_rounding() {
        // 3% of 1_000_001 = 30000.03 -> rounds to 30000
        let (participant, fee) = calculate_fee(1_000_001, 3.0);
        assert_eq!(fee, 30_000);
        assert_eq!(participant, 970_001);
    }

    #[test]
    fn test_fee_for_invoice_type() {
        let config = FeeConfig {
            program_entry_percent: 3.0,
            hosting_percent: 5.0,
            renewal_percent: 3.0,
            operator_address: "zs1test".to_string(),
        };
        assert_eq!(fee_for_invoice_type("program", &config), 3.0);
        assert_eq!(fee_for_invoice_type("hosting", &config), 5.0);
        assert_eq!(fee_for_invoice_type("renewal", &config), 3.0);
        assert_eq!(fee_for_invoice_type("unknown", &config), 0.0);
    }
}

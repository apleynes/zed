pub mod db;
mod token;

use crate::Cents;

pub use token::*;

pub const AGENT_EXTENDED_TRIAL_FEATURE_FLAG: &str = "agent-extended-trial";

/// The default value to use for maximum spend per month if the user did not
/// explicitly set a maximum spend.
///
/// Used to prevent surprise bills.
pub const DEFAULT_MAX_MONTHLY_SPEND: Cents = Cents::from_dollars(10);

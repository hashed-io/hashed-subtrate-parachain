use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, OnRuntimeUpgrade},
};
use frame_system::pallet_prelude::BlockNumberFor;
use super::*;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::balances_verifier::migration";

pub struct Migrate;

impl OnRuntimeUpgrade for Migrate {

	fn on_runtime_upgrade() -> Weight {
		return Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: TARGET, "In post upgrade");

		let locks_count = pallet_balances::Locks::<Runtime>::iter_keys().count() as u32;
		let decodable_locks_count = pallet_balances::Locks::<Runtime>::iter_values().count() as u32;
		log::info!(target: TARGET, "Num locks: {}, decodable: {}", locks_count, decodable_locks_count);
		// IdentityOf::<Runtime>::get(key)
		pallet_balances::Locks::<Runtime>::iter().take(10).for_each(|(k, v)| {
			let account_bytes: [u8; 32] = k.clone().into();
			log::info!("    ");
			log::info!("    ");
			log::info!("key account bytes: {:?} ", account_bytes);
			log::info!("Locks: {:?}", v);
		});
		log::info!(target: TARGET, "Total Issuance: {}", pallet_balances::TotalIssuance::<Runtime>::get());
		Ok(())
	}
}

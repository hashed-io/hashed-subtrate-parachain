use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::OnRuntimeUpgrade,
};

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::society_verifier::migration";

pub struct Migrate;

impl OnRuntimeUpgrade for Migrate {

	fn on_runtime_upgrade() -> Weight {
		return Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: TARGET, "In post upgrade");

		let candidates_count = pallet_society::Candidates::<Runtime>::iter_keys().count() as u32;
		let decodable_candidates_count = pallet_society::Candidates::<Runtime>::iter_values().count() as u32;
		// IdentityOf::<Runtime>::get(key)
		pallet_society::Candidates::<Runtime>::iter().for_each(|(k, v)| {
			let account_bytes: [u8; 32] = k.clone().into();
			log::info!("    ");
			log::info!("    ");
			log::info!("key account bytes: {:?} ", account_bytes);
			log::info!("Candidacy: {:?}", v);
		});
		log::info!(target: TARGET, "Num candidates: {}, decodable: {}", candidates_count, decodable_candidates_count);
		Ok(())
	}
}

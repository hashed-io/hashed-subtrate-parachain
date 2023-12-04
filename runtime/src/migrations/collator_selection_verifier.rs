use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::OnRuntimeUpgrade,
};

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::collator_selection_verifier::migration";

pub struct Migrate;

impl OnRuntimeUpgrade for Migrate {

	fn on_runtime_upgrade() -> Weight {
		return Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: TARGET, "In post upgrade");

		let authored_blocks_count = pallet_collator_selection::LastAuthoredBlock::<Runtime>::iter_keys().count() as u32;
		let decodable_authored_blocks_count = pallet_collator_selection::LastAuthoredBlock::<Runtime>::iter_values().count() as u32;
		log::info!(target: TARGET, "Num authored_blocks: {}, decodable: {}", authored_blocks_count, decodable_authored_blocks_count);
		// IdentityOf::<Runtime>::get(key)
		pallet_collator_selection::LastAuthoredBlock::<Runtime>::iter().take(10).for_each(|(k, v)| {
			let account_bytes: [u8; 32] = k.clone().into();
			log::info!("    ");
			log::info!("    ");
			// log::info!("key: {} ", k);
			log::info!("key account bytes: {:?} ", account_bytes);
			log::info!("block: {:?}", v);
		});
		log::info!("    ");
		log::info!("    ");
		let invulnerables = pallet_collator_selection::Invulnerables::<Runtime>::get();
		invulnerables.iter().for_each(|v|{
			let account_bytes: [u8; 32] = v.clone().into();
			log::info!("invulnerable: {:?} ", account_bytes);
		});
		log::info!(target: TARGET, "Candidacy Bond: {}", pallet_collator_selection::CandidacyBond::<Runtime>::get());
		Ok(())
	}
}

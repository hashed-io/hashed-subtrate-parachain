use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, OnRuntimeUpgrade},
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_consensus_aura::Slot;
use super::*;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::aura_verifier::migration";


#[frame_support::storage_alias]
pub(super) type Authorities<T: pallet_aura::Config> =
		StorageValue<pallet_aura::Pallet<T>, BoundedVec<<T as pallet_aura::Config>::AuthorityId, <T as pallet_aura::Config>::MaxAuthorities>, ValueQuery>;

#[frame_support::storage_alias]
pub(super) type CurrentSlot<T: pallet_aura::Config> = StorageValue<pallet_aura::Pallet<T>, Slot, ValueQuery>;

pub struct Migrate;

impl OnRuntimeUpgrade for Migrate {

	fn on_runtime_upgrade() -> Weight {
		return Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: TARGET, "In post upgrade");
		let authorities = Authorities::<Runtime>::get();

		authorities.iter().for_each(|v| {
			// let account_bytes: [u8; 32] = v.clone().into();
			// log::info!("authority: {:?} ", account_bytes);
			log::info!("authority1: {} ", v);
		});

		let current_slot = CurrentSlot::<Runtime>::get();
		log::info!("current slot: {:?} ", current_slot);
		Ok(())
	}
}

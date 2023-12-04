use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, OnRuntimeUpgrade},
};
use frame_system::pallet_prelude::BlockNumberFor;
use cumulus_pallet_xcmp_queue::{QueueConfigData, OverweightIndex};
use cumulus_primitives_core::relay_chain::BlockNumber as RelayBlockNumber;
use crate::ParaId;
use super::*;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::xcmp_queue_verifier::migration";


#[frame_support::storage_alias]
pub(super) type Overweight<T: cumulus_pallet_xcmp_queue::Config> =
		CountedStorageMap<cumulus_pallet_xcmp_queue::Pallet<T>, Twox64Concat, OverweightIndex, (ParaId, RelayBlockNumber, Vec<u8>)>;

#[frame_support::storage_alias]
pub(super) type QueueConfig<T: cumulus_pallet_xcmp_queue::Config> = StorageValue<cumulus_pallet_xcmp_queue::Pallet<T>, QueueConfigData, ValueQuery>;

pub struct Migrate;

impl OnRuntimeUpgrade for Migrate {

	fn on_runtime_upgrade() -> Weight {
		return Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: TARGET, "In post upgrade");
		let overweight_tracked_count = Overweight::<Runtime>::count();
		let overweight_count = Overweight::<Runtime>::iter_keys().count() as u32;
		let decodable_overweight_count = Overweight::<Runtime>::iter_values().count() as u32;
		log::info!(target: TARGET, "Num overweight: {}, decodable: {}, tracked: {}", overweight_count, decodable_overweight_count, overweight_tracked_count);
		Overweight::<Runtime>::iter().for_each(|(k, v)| {
			log::info!("    ");
			log::info!("    ");
			log::info!("overweight index: {} ", k);
			log::info!("overweight: {:?}", v);
		});

		log::info!("config data: {:?} ", QueueConfig::<Runtime>::get());
		Ok(())
	}
}

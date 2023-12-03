use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, OnRuntimeUpgrade},
};
use frame_system::pallet_prelude::BlockNumberFor;
use super::*;
use cumulus_pallet_dmp_queue::{ConfigData, PageIndexData};

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::dmp_queue_verifier::migration";

#[frame_support::storage_alias]
pub(super) type Configuration<T: cumulus_pallet_dmp_queue::Config> = StorageValue<cumulus_pallet_dmp_queue::Pallet<T>, ConfigData, ValueQuery>;

#[frame_support::storage_alias]
pub(super) type PageIndex<T: cumulus_pallet_dmp_queue::Config> = StorageValue<cumulus_pallet_dmp_queue::Pallet<T>, PageIndexData, ValueQuery>;

pub struct Migrate;

impl OnRuntimeUpgrade for Migrate {

	fn on_runtime_upgrade() -> Weight {
		return Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: TARGET, "In post upgrade");

		log::info!(target: TARGET, "Configuration: {:?}", Configuration::<Runtime>::get());
		log::info!(target: TARGET, "PageIndex: {:?}", PageIndex::<Runtime>::get());

		Ok(())
	}
}

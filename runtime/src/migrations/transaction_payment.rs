use frame_support::traits::OnRuntimeUpgrade;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use pallet_transaction_payment::{Multiplier, NextFeeMultiplierOnEmpty};
use crate::*;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const TARGET: &'static str = "runtime::transaction-payment::migration";

/// Storage releases of the pallet.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Releases {
	/// Original version of the pallet.
	V1Ancient,
	/// One that bumps the usage to FixedU128 from FixedI128.
	V2,
}

impl Default for Releases {
	fn default() -> Self {
		Releases::V1Ancient
	}
}

pub mod v0 {
	use super::*;
	use frame_support::pallet_prelude::*;
use sp_runtime::traits::Saturating;


	#[frame_support::storage_alias]
	pub(super) type StorageVersion<T: pallet_transaction_payment::Config> = StorageValue<pallet_transaction_payment::Pallet<T>, Releases, ValueQuery>;

	#[frame_support::storage_alias]
	pub type NextFeeMultiplier<T: pallet_transaction_payment::Config> =
		StorageValue<pallet_transaction_payment::Pallet<T>, Multiplier, ValueQuery>;

	pub struct Migrate;

	impl OnRuntimeUpgrade for Migrate {

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			log::info!(target: TARGET, "In pre upgrade");
			let storage_version = StorageVersion::<Runtime>::get();
			if storage_version == Releases::V1Ancient {
				log::info!(target: TARGET, "Correct storage version for  migration, version: {:?}", storage_version);
			} else {
				log::warn!(target: TARGET, "Migration will not run as storage version is: {:?}", storage_version);
			}
			Ok(Vec::new())
		}
		//We are just updating the storage version, as the pallet is already on the latest version
		fn on_runtime_upgrade() -> Weight {
			let storage_version = StorageVersion::<Runtime>::get();
			let mut writes = 0;
			let storage_version = StorageVersion::<Runtime>::get();
			if storage_version == Releases::V1Ancient {
				StorageVersion::<Runtime>::put(Releases::V2);
				writes.saturating_inc();
			} else {
				log::warn!(target: TARGET, "Migration will not run as storage version is: {:?}", storage_version);
			}
			<Runtime as frame_system::Config>::DbWeight::get().reads_writes(writes, 1)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
			log::info!(target: TARGET, "In post upgrade");
			let storage_version = StorageVersion::<Runtime>::get();
			ensure!(storage_version == Releases::V2, "Storage version was not updated");
			log::info!(target: TARGET, "Storage version was correctly updated: {:?}", storage_version);
			log::info!(target: TARGET, "Next fee multiplier: {}", NextFeeMultiplier::<Runtime>::get());
			Ok(())
		}
	}
}

use frame_support::traits::{Currency, OnRuntimeUpgrade};
use frame_system::pallet_prelude::BlockNumberFor;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use pallet_vesting::{VestingInfo, MaxVestingSchedulesGet};
use crate::*;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const TARGET: &'static str = "runtime::vesting::migration";

type BalanceOf<T> = <<T as pallet_identity::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

// A value placed in storage that represents the current version of the Vesting storage.
// This value is used by `on_runtime_upgrade` to determine whether we run storage migration logic.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Releases {
	V0,
	V1,
}

impl Default for Releases {
	fn default() -> Self {
		Releases::V0
	}
}


pub mod v0 {
	use super::*;
	use frame_support::pallet_prelude::*;
use sp_runtime::traits::Saturating;


	/// Storage version of the pallet.
	///
	/// New networks start with latest version, as determined by the genesis build.
	#[frame_support::storage_alias]
	pub(crate) type StorageVersion<T: pallet_vesting::Config> = StorageValue<pallet_vesting::Pallet<T>, Releases, ValueQuery>;

	/// Information regarding the vesting of a given account.
	#[frame_support::storage_alias]
	pub type Vesting<T: pallet_vesting::Config> = StorageMap<
		pallet_vesting::Pallet<T>,
		Blake2_128Concat,
		<T as frame_system::Config>::AccountId,
		BoundedVec<VestingInfo<BalanceOf<T>, BlockNumberFor<T>>, MaxVestingSchedulesGet<T>>,
	>;

	pub struct MigrateToV1;

	impl OnRuntimeUpgrade for MigrateToV1 {

		// #[cfg(feature = "try-runtime")]
		// fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		// 	let result = pallet_vesting::migrations::v1::pre_migrate::<Runtime>();
		// 	match result {
		// 		Ok(_) => {
		// 			log::info!(
		// 				target: TARGET,
		// 				"pre_migrate was successful"
		// 			);
		// 		},
		// 		Err(e) => {
		// 			log::error!(
		// 				target: TARGET,
		// 				"pre_migrate failed: {}",
		// 				e
		// 			);
		// 		}
		// 	}
		// 	Ok(Vec::new())
		// }

		fn on_runtime_upgrade() -> Weight {
			let storage_version = StorageVersion::<Runtime>::get();
			let mut writes = 0;
			if storage_version == Releases::V0 {
				StorageVersion::<Runtime>::put(Releases::V1);
				writes.saturating_inc();
			}
			<Runtime as frame_system::Config>::DbWeight::get().reads_writes(writes, 1)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
			let result = pallet_vesting::migrations::v1::post_migrate::<Runtime>();
			match result {
				Ok(_) => {
					log::info!(
						target: TARGET,
						"pallet post_migrate was successful"
					);
				},
				Err(e) => {
					log::error!(
						target: TARGET,
						"pallet post_migrate failed: {}",
						e
					);
				}
			}
			let vesting_count = Vesting::<Runtime>::iter_keys().count() as u32;
			let decodable_vesting_count = Vesting::<Runtime>::iter_values().count() as u32;
			log::info!(target: TARGET, "Num vesting count: {}, decodable: {}", vesting_count, decodable_vesting_count);
			Vesting::<Runtime>::iter().take(10).for_each(|(k,v)|{
				let account_bytes: [u8; 32] = k.clone().into();
				log::info!("    ");
				log::info!("    ");
				log::info!("key account bytes: {:?} ", account_bytes);
				log::info!("vesting: {:?} ", v);
			});
			Ok(())
		}
	}
}

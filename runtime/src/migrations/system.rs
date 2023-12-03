use super::*;
use crate::*;
use frame_system::EventRecord;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, OnRuntimeUpgrade},
};
use frame_system::pallet_prelude::BlockNumberFor;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::system::migration";

pub mod v0 {
	use cumulus_primitives_core::relay_chain::AsyncBackingParams;
	use sp_runtime::traits::Saturating;

	use super::*;

	// #[frame_support::storage_alias]
	// pub type Account<T: frame_system::Config> = StorageMap<
	// 	frame_system::Pallet<T>,
	// 	Blake2_128Concat,
	// 	<T as frame_system::Config>::AccountId,
	// 	AccountInfo<T::Nonce, T::AccountData>,
	// 	ValueQuery,
	// >;

	// #[frame_support::storage_alias]
	// pub(super) type RelevantMessagingState<T: cumulus_pallet_parachain_system::Config> =
	// 	StorageValue<cumulus_pallet_parachain_system::Pallet<T>, MessagingStateSnapshot>;

	#[frame_support::storage_alias]
	pub(super) type Events<T: frame_system::Config> =
		StorageValue<frame_system::Pallet<T>, Vec<Box<EventRecord<<T as frame_system::Config>::RuntimeEvent, <T as frame_system::Config>::Hash>>>, ValueQuery>;

	pub struct Migrate;

	impl OnRuntimeUpgrade for Migrate {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			log::info!(target: TARGET, "In pre upgrade");
			// let storage_version = StorageVersion::get::<ParachainSystem>();
			// if storage_version == 1 {
			// 	log::info!(target: TARGET, "Correct storage version for  migration, version: {:?}", storage_version);
			// } else {
			// 	log::warn!(target: TARGET, "Migration will not run as storage version is greater than 1, version: {:?}", storage_version);
			// }
			Ok(Vec::new())
		}

		fn on_runtime_upgrade() -> Weight {
			// let storage_version = StorageVersion::get::<ParachainSystem>();
			// let mut reads_writes = 0;
			// if storage_version == 1 {
			// 	let _ = HostConfiguration::<Runtime>::translate::<OldAbridgedHostConfiguration, _>(
			// 		|v| {
			// 			reads_writes.saturating_inc();
			// 			match v {
			// 				Some(c) => Some(AbridgedHostConfiguration {
			// 					max_code_size: c.max_code_size,
			// 					max_head_data_size: c.max_head_data_size,
			// 					max_upward_queue_count: c.max_upward_queue_count,
			// 					max_upward_queue_size: c.max_upward_queue_size,
			// 					max_upward_message_size: c.max_upward_message_size,
			// 					max_upward_message_num_per_candidate: c
			// 						.max_upward_message_num_per_candidate,
			// 					hrmp_max_message_num_per_candidate: c
			// 						.hrmp_max_message_num_per_candidate,
			// 					validation_upgrade_cooldown: c.validation_upgrade_cooldown,
			// 					validation_upgrade_delay: c.validation_upgrade_delay,
			// 					async_backing_params: AsyncBackingParams {
			// 						allowed_ancestry_len: 0,
			// 						max_candidate_depth: 0,
			// 					},
			// 				}),
			// 				None => None,
			// 			}
			// 		},
			// 	);
			// } else {
			// 	log::warn!(target: TARGET, "Migration did not run as storage version is greater than 1, version: {:?}", storage_version);
			// }
			// <Runtime as frame_system::Config>::DbWeight::get()
			// 	.reads_writes(reads_writes + 1, reads_writes)
			Weight::zero()
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
			log::info!(target: TARGET, "In post upgrade");
			let num_account_keys = frame_system::Account::<Runtime>::iter_keys().count() as u32;
			let num_accounts = frame_system::Account::<Runtime>::iter_values().count() as u32;
			log::info!(target: TARGET, "account keys: {}, decodable accounts: {}", num_account_keys, num_accounts);
			frame_system::Account::<Runtime>::iter().take(10).for_each(|(k, v)| {
				let account_bytes: [u8; 32] = k.clone().into();
				log::info!("    ");
				log::info!("    ");
				log::info!("key account bytes: {:?} ", account_bytes);
				log::info!("Nonce: {:?}", v.nonce);
				log::info!("Consumers: {:?}", v.consumers);
				log::info!("Providers: {:?}", v.providers);
				log::info!("Sufficients: {:?}", v.sufficients);
				log::info!("Data: {:?}", v.data);
			});
			// let events = Events::<Runtime>::get();
			// log::info!(target: TARGET, "num_events: {}", events.len());
			// events.iter().take(10).for_each(|v|{
			// 	log::info!("    ");
			// 	log::info!("    ");
			// 	log::info!("event: {:?} ", v);
			// });
			Ok(())
		}
	}
}

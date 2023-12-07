use crate::*;
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use frame_system::{ConsumedWeight, EventRecord};
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::system::migration";

#[frame_support::storage_alias]
pub(super) type Events<T: frame_system::Config> = StorageValue<
	frame_system::Pallet<T>,
	Vec<
		Box<
			EventRecord<
				<T as frame_system::Config>::RuntimeEvent,
				<T as frame_system::Config>::Hash,
			>,
		>,
	>,
	ValueQuery,
>;

#[frame_support::storage_alias]
pub(super) type BlockWeight<T: frame_system::Config> =
	StorageValue<frame_system::Pallet<T>, ConsumedWeight, ValueQuery>;

#[frame_support::storage_alias]
pub(super) type Digest<T: frame_system::Config> =
	StorageValue<frame_system::Pallet<T>, generic::Digest, ValueQuery>;

pub struct Migrate;

impl OnRuntimeUpgrade for Migrate {
	fn on_runtime_upgrade() -> Weight {
		Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: TARGET, "In post upgrade");
		let num_account_keys = frame_system::Account::<Runtime>::iter_keys().count() as u32;
		let num_accounts = frame_system::Account::<Runtime>::iter_values().count() as u32;
		log::info!(target: TARGET, "account keys: {}, decodable accounts: {}", num_account_keys, num_accounts);
		frame_system::Account::<Runtime>::iter().take(10).for_each(|(k, v)| {
			let account_bytes: [u8; 32] = k.clone().into();
			log::info!(target: TARGET, "    ");
			log::info!(target: TARGET, "    ");
			log::info!(target: TARGET, "key account bytes: {:?} ", account_bytes);
			log::info!(target: TARGET, "Nonce: {:?}", v.nonce);
			log::info!(target: TARGET, "Consumers: {:?}", v.consumers);
			log::info!(target: TARGET, "Providers: {:?}", v.providers);
			log::info!(target: TARGET, "Sufficients: {:?}", v.sufficients);
			log::info!(target: TARGET, "Data: {:?}", v.data);
		});
		log::info!(target: TARGET, "block weight: {:?}", BlockWeight::<Runtime>::get());
		log::info!(target: TARGET, "digest: {:?}", Digest::<Runtime>::get());
		log::info!(target: TARGET, "last runtime upgrade: {:?}", frame_system::LastRuntimeUpgrade::<Runtime>::get());
		Ok(())
	}
}

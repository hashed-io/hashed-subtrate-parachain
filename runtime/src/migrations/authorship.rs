use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::OnRuntimeUpgrade
};


/// The log target.
const TARGET: &'static str = "runtime::authorship::migration";

pub mod v0 {
	use super::*;
	/// The actual type isn't important, as we only delete the key in the state.
	#[frame_support::storage_alias]
	pub(super) type Uncles<T: pallet_authorship::Config> =
		StorageValue<pallet_authorship::Pallet<T>, (), ValueQuery>;

	#[frame_support::storage_alias]
	pub(super) type DidSetUncles<T: pallet_authorship::Config> =
		StorageValue<pallet_authorship::Pallet<T>, bool, ValueQuery>;

	pub struct Migrate;

	impl OnRuntimeUpgrade for Migrate {
		fn on_runtime_upgrade() -> Weight {
			Uncles::<Runtime>::kill();
			DidSetUncles::<Runtime>::kill();
			log::info!(target: TARGET, "Killed deprecated storages");
			<Runtime as frame_system::Config>::DbWeight::get().reads_writes(0, 2)
		}
	}
}

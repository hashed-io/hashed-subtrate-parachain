use frame_support::traits::OnRuntimeUpgrade;
use crate::Runtime;


pub mod v0 {
	use super::*;
	use frame_support::pallet_prelude::*;

	pub struct MigrateToV1;

	impl OnRuntimeUpgrade for MigrateToV1 {
		fn on_runtime_upgrade() -> Weight {
			pallet_uniques::migration::migrate_to_v1::<Runtime, (), pallet_uniques::Pallet<Runtime, ()>>()
		}

	}
}

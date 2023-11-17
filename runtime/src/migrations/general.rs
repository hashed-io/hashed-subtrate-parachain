use frame_support::traits::OnRuntimeUpgrade;
use crate::*;


use frame_support::pallet_prelude::*;

pub struct GeneralMigration;

impl OnRuntimeUpgrade for GeneralMigration {
	fn on_runtime_upgrade() -> Weight {
		// Add newly added counts to the society pallet
		pallet_society::RoundCount::<Runtime>::put(0);
		pallet_society::MemberCount::<Runtime>::put(0);
		// Update storage versions for pallets that don't require the migrations because they have no data
		StorageVersion::new(2).put::<Society>();
		StorageVersion::new(4).put::<Bounties>();
		<Runtime as frame_system::Config>::DbWeight::get().reads_writes(0, 4)
	}
}


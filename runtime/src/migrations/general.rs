use frame_support::traits::OnRuntimeUpgrade;
use crate::*;


use frame_support::pallet_prelude::*;

pub struct GeneralMigration;

impl OnRuntimeUpgrade for GeneralMigration {
	fn on_runtime_upgrade() -> Weight {
		pallet_society::RoundCount::<Runtime>::put(0);
		pallet_society::MemberCount::<Runtime>::put(0);
		// <Runtime as pallet_society::RoundCount<>>::RoundCount::<Runtime, ()>::put(0);
		StorageVersion::new(2).put::<Society>();
		StorageVersion::new(4).put::<Bounties>();
		<Runtime as frame_system::Config>::DbWeight::get().reads_writes(0, 4)
	}
}


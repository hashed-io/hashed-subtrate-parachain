use crate::*;
use frame_support::traits::OnRuntimeUpgrade;

use frame_support::pallet_prelude::*;

pub struct GeneralMigration;

impl OnRuntimeUpgrade for GeneralMigration {
	fn on_runtime_upgrade() -> Weight {
		let society_version = StorageVersion::get::<Society>();
		let mut writes: u64 = 0;
		if society_version < 2 {
			// Add newly added counts to the society pallet
			pallet_society::RoundCount::<Runtime>::put(0);
			pallet_society::MemberCount::<Runtime>::put(0);
			StorageVersion::new(2).put::<Society>();
			writes = writes.saturating_add(3);
		} else {
			log::warn!("Society pallet not upgraded, current_version: {:?}", society_version);
		}

		let bounties_version = StorageVersion::get::<Bounties>();
		if bounties_version < 4 {
			StorageVersion::new(4).put::<Bounties>();
			writes = writes.saturating_add(1);
		} else {
			log::warn!("Bounties pallet not upgraded, current_version: {:?}", bounties_version);
		}

		<Runtime as frame_system::Config>::DbWeight::get().reads_writes(2, writes)
	}
}

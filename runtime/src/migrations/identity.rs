use frame_support::traits::{Currency, OnRuntimeUpgrade};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_identity::*;

use crate::{Vec, Runtime};

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::identity::migration";

type BalanceOf<T> =
	<<T as pallet_identity::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub mod v1 {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[frame_support::storage_alias]
	pub(super) type IdentityOf<T: pallet_identity::Config> = StorageMap<
		pallet_identity::Pallet<T>,
		Twox64Concat,
		<T as frame_system::Config>::AccountId,
		Registration<BalanceOf<T>, <T as pallet_identity::Config>::MaxRegistrars, <T as pallet_identity::Config>::IdentityInformation>,
		OptionQuery,
	>;


	pub struct MigrateToV2;

	impl OnRuntimeUpgrade for MigrateToV2 {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			log::info!(target: TARGET, "In preupgrade");
			// let identities = pallet_identity::IdentityOf::<Runtime>::iter_keys().count() as u32;
			// ensure!(false, "Can only upgrade from version 3");
			let identities  = IdentityOf::<Runtime>::iter_keys().count() as u32;
			let decodable_identities  = IdentityOf::<Runtime>::iter_values().count() as u32;
			log::info!(target: TARGET, "Num identities: {}, {}", identities, decodable_identities);

			Ok(Vec::new())
		}

		fn on_runtime_upgrade() -> Weight {
			log::info!(target: TARGET, "In runtime upgrade");
			IdentityOf::<Runtime>::iter_keys().for_each(|k| {
				log::info!("key1: {:?} ", k);
			});
			// IdentityOf::<Runtime>::get(key)
			IdentityOf::<Runtime>::iter().for_each(|(k, v)| {
				log::info!("key: {:?} ", k);
				log::info!("Deposit: {:?} Judgments: {:?}", v.deposit, v.judgements);
				log::info!("additional: {:?}", v.info.additional);
				log::info!("display: {:?}", v.info.display);
				log::info!("legal: {:?}", v.info.legal);
				log::info!("web: {:?}", v.info.web);
				log::info!("riot: {:?}", v.info.riot);
				log::info!("email: {:?}", v.info.email);
				log::info!("pgp_fingerprint: {:?}", v.info.pgp_fingerprint);
				log::info!("image: {:?}", v.info.image);
				log::info!("twitter: {:?}", v.info.twitter);
			});
			return Weight::zero();
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
			log::info!(target: TARGET, "In post upgrade");
			Ok(())
		}
	}
}

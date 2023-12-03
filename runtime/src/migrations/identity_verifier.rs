use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, OnRuntimeUpgrade},
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_identity::*;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::identity_verifier::migration";

type BalanceOf<T> = <<T as pallet_identity::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::storage_alias]
pub(crate) type IdentityOf<T: pallet_identity::Config> = StorageMap<
	pallet_identity::Pallet<T>,
	Twox64Concat,
	<T as frame_system::Config>::AccountId,
	Registration<
		BalanceOf<T>,
		<T as pallet_identity::Config>::MaxRegistrars,
		<T as pallet_identity::Config>::IdentityInformation,
	>,
	OptionQuery,
>;

#[frame_support::storage_alias]
pub(super) type Registrars<T: pallet_identity::Config> = StorageValue<
	pallet_identity::Pallet<T>,
	BoundedVec<
		Option<
			RegistrarInfo<
				BalanceOf<T>,
				<T as frame_system::Config>::AccountId,
				<<T as pallet_identity::Config>::IdentityInformation as IdentityInformationProvider>::IdentityField,
			>,
		>,
		<T as pallet_identity::Config>::MaxRegistrars,
	>,
	ValueQuery,
>;

pub struct Migrate;

impl OnRuntimeUpgrade for Migrate {

	fn on_runtime_upgrade() -> Weight {
		return Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: TARGET, "In identity verifier post upgrade");
		let vesting_version = StorageVersion::get::<Vesting>();
		log::info!(
			"vesting pallet version: {:?}",
			vesting_version
		);
		let society_version = StorageVersion::get::<Society>();
		ensure!(society_version == 2, "Not all pallet migrations have executed");

		let identities_count = IdentityOf::<Runtime>::iter_keys().count() as u32;
		let decodable_identities_count = IdentityOf::<Runtime>::iter_values().count() as u32;
		log::info!(target: TARGET, "Num identities: {}, decodable: {}", identities_count, decodable_identities_count);
		// IdentityOf::<Runtime>::get(key)
		IdentityOf::<Runtime>::iter().for_each(|(k, v)| {
			let account_bytes: [u8; 32] = k.clone().into();
			log::info!("    ");
			log::info!("    ");
			log::info!("key account bytes: {:?} ", account_bytes);
			log::info!("Deposit: {:?}", v.deposit);
			log::info!("Judgments: {:?}", v.judgements);
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

		let identity = IdentityOf::<Runtime>::get(TestAccount::get()).unwrap();
		log::info!("display identity: {:?}", identity.info.display);

		let registrars = Registrars::<Runtime>::get();
		let registrars_count = registrars.len();
		log::info!(target: TARGET, "Num registrars: {}", registrars_count);
		registrars.iter().for_each(|v| match v {
			Some(info) => {
				let account_bytes: [u8; 32] = info.account.clone().into();
				log::info!("   ");
				log::info!("   ");
				log::info!("account bytes: {:?}", account_bytes);
				log::info!("fee: {:?}", info.fee);
				log::info!("fields: {:?}", info.fields.0);
			},
			None => log::info!("empty registrar"),
		});
		Ok(())
	}
}

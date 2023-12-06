use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::OnRuntimeUpgrade,
};

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::proxy_verifier::migration";

pub struct Migrate;

impl OnRuntimeUpgrade for Migrate {

	fn on_runtime_upgrade() -> Weight {
		return Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: TARGET, "In post upgrade");

		let proxies_count = pallet_proxy::Proxies::<Runtime>::iter_keys().count() as u32;
		let decodable_proxies_count = pallet_proxy::Proxies::<Runtime>::iter_values().count() as u32;
		log::info!(target: TARGET, "Num proxies: {}, decodable: {}", proxies_count, decodable_proxies_count);
		// IdentityOf::<Runtime>::get(key)
		pallet_proxy::Proxies::<Runtime>::iter().for_each(|(k, v)| {
			let account_bytes: [u8; 32] = k.clone().into();
			log::info!("    ");
			log::info!("    ");
			log::info!("key account bytes: {:?} ", account_bytes);
			v.0.iter().for_each(|d| {
				let delegate_bytes: [u8; 32] = d.delegate.clone().into();
				log::info!("delegate bytes: {:?}", delegate_bytes);
				log::info!("proxy_type: {:?}", d.proxy_type);
				log::info!("delay: {:?}", d.delay);
			});
			log::info!("value: {:?}", v.1);

		});
		Ok(())
	}
}


use crate::*;
use cumulus_pallet_parachain_system::MessagingStateSnapshot;
use cumulus_primitives_core::{AbridgedHostConfiguration, PersistedValidationData};
use frame_support::{
	pallet_prelude::*,
	traits::OnRuntimeUpgrade,
};
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

/// The log target.
const TARGET: &'static str = "runtime::parachain_system::migration";


pub mod v1 {
	use cumulus_primitives_core::relay_chain::AsyncBackingParams;
	use sp_runtime::traits::Saturating;

	use super::*;
	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct OldAbridgedHostConfiguration {
		/// The maximum validation code size, in bytes.
		pub max_code_size: u32,
		/// The maximum head-data size, in bytes.
		pub max_head_data_size: u32,
		/// Total number of individual messages allowed in the parachain -> relay-chain message queue.
		pub max_upward_queue_count: u32,
		/// Total size of messages allowed in the parachain -> relay-chain message queue before which
		/// no further messages may be added to it. If it exceeds this then the queue may contain only
		/// a single message.
		pub max_upward_queue_size: u32,
		/// The maximum size of an upward message that can be sent by a candidate.
		///
		/// This parameter affects the size upper bound of the `CandidateCommitments`.
		pub max_upward_message_size: u32,
		/// The maximum number of messages that a candidate can contain.
		///
		/// This parameter affects the size upper bound of the `CandidateCommitments`.
		pub max_upward_message_num_per_candidate: u32,
		/// The maximum number of outbound HRMP messages can be sent by a candidate.
		///
		/// This parameter affects the upper bound of size of `CandidateCommitments`.
		pub hrmp_max_message_num_per_candidate: u32,
		/// The minimum period, in blocks, between which parachains can update their validation code.
		pub validation_upgrade_cooldown: BlockNumber,
		/// The delay, in blocks, before a validation upgrade is applied.
		pub validation_upgrade_delay: BlockNumber,
	}

	#[frame_support::storage_alias]
	pub(super) type HostConfiguration<T: cumulus_pallet_parachain_system::Config> =
		StorageValue<cumulus_pallet_parachain_system::Pallet<T>, AbridgedHostConfiguration>;

	#[frame_support::storage_alias]
	pub(super) type RelevantMessagingState<T: cumulus_pallet_parachain_system::Config> =
		StorageValue<cumulus_pallet_parachain_system::Pallet<T>, MessagingStateSnapshot>;

	#[frame_support::storage_alias]
	pub(super) type ValidationData<T: cumulus_pallet_parachain_system::Config> = StorageValue<cumulus_pallet_parachain_system::Pallet<T>, PersistedValidationData>;

	pub struct Migrate;

	impl OnRuntimeUpgrade for Migrate {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			log::info!(target: TARGET, "In pre upgrade");
			let storage_version = StorageVersion::get::<ParachainSystem>();
			if storage_version == 1 {
				log::info!(target: TARGET, "Correct storage version for  migration, version: {:?}", storage_version);
			} else {
				log::warn!(target: TARGET, "Migration will not run as storage version is greater than 1, version: {:?}", storage_version);
			}
			Ok(Vec::new())
		}

		fn on_runtime_upgrade() -> Weight {
			let storage_version = StorageVersion::get::<ParachainSystem>();
			let mut reads_writes = 0;
			if storage_version == 1 {
				let _ = HostConfiguration::<Runtime>::translate::<OldAbridgedHostConfiguration, _>(
					|v| {
						reads_writes.saturating_inc();
						match v {
							Some(c) => Some(AbridgedHostConfiguration {
								max_code_size: c.max_code_size,
								max_head_data_size: c.max_head_data_size,
								max_upward_queue_count: c.max_upward_queue_count,
								max_upward_queue_size: c.max_upward_queue_size,
								max_upward_message_size: c.max_upward_message_size,
								max_upward_message_num_per_candidate: c
									.max_upward_message_num_per_candidate,
								hrmp_max_message_num_per_candidate: c
									.hrmp_max_message_num_per_candidate,
								validation_upgrade_cooldown: c.validation_upgrade_cooldown,
								validation_upgrade_delay: c.validation_upgrade_delay,
								async_backing_params: AsyncBackingParams {
									allowed_ancestry_len: 0,
									max_candidate_depth: 0,
								},
							}),
							None => None,
						}
					},
				);
			} else {
				log::warn!(target: TARGET, "Migration did not run as storage version is greater than 1, version: {:?}", storage_version);
			}
			<Runtime as frame_system::Config>::DbWeight::get()
				.reads_writes(reads_writes + 1, reads_writes)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			log::info!(target: TARGET, "In post upgrade, should be able to decode host configuration with the new structure");
			let host_configuration = HostConfiguration::<Runtime>::get().unwrap();
			log::info!("    ");
			log::info!("    ");
			log::info!("host configuration: {:?} ", host_configuration);

			let relevant_messaging_state = RelevantMessagingState::<Runtime>::get().unwrap();
			log::info!("    ");
			log::info!("    ");
			log::info!("relevant messaging state:");
			log::info!("	dmq mqc head: {:?}", relevant_messaging_state.dmq_mqc_head);
			log::info!("	remaining count: {}", relevant_messaging_state.relay_dispatch_queue_remaining_capacity.remaining_count);
			log::info!("	remaining size: {}", relevant_messaging_state.relay_dispatch_queue_remaining_capacity.remaining_size);
			log::info!("	ingress channels: {:?}", relevant_messaging_state.ingress_channels);
			log::info!("	egress channels: {:?}", relevant_messaging_state.egress_channels);
			log::info!("    ");
			log::info!("    ");
			log::info!("persisted validation data:{:?}", ValidationData::<Runtime>::get().unwrap());
			Ok(())
		}
	}
}

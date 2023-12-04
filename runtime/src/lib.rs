#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod constants;
mod weights;
pub mod xcm_config;
mod migrations;
use constants::*;

use core::marker::PhantomData;
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use cumulus_primitives_core::ParaId;
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, IdentifyAccount, Verify},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};


use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	genesis_builder_helper::{build_config, create_default_config},
	parameter_types,
	pallet_prelude::Get,
	traits::{ AsEnsureOriginWithArg, ConstBool, ConstU32, ConstU64, ConstU128, ConstU8, EitherOfDiverse, Everything, InstanceFilter, WithdrawReasons, tokens::{ConversionFromAssetBalance, PayFromAccount}},
	weights::{
		constants::WEIGHT_REF_TIME_PER_SECOND, ConstantMultiplier, Weight, WeightToFeeCoefficient,
		WeightToFeeCoefficients, WeightToFeePolynomial,
	},
	PalletId,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, EnsureSigned,
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Percent, Permill,RuntimeDebug};
use xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot imports
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};

use weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight};

// XCM Imports
use xcm::latest::prelude::BodyId;
use xcm_executor::XcmExecutor;


/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

pub type AccountIndex = u32;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
parameter_types! {
	pub const Payouts: Vec<(AccountId, Balance)> = Vec::new();
	pub const InactiveAccounts: Vec<AccountId> = Vec::new();
	pub TestAccount: AccountId = hex_literal::hex!["d8033c4d04a502901d24a789da32940085c62eba881c4701a73411288445cc46"].into();
}
pub type RuntimeUpgrades = (
	migrations::system::v0::Migrate,
	migrations::parachain_system::v1::Migrate,
	cumulus_pallet_parachain_system::migration::Migration<Runtime>,
	pallet_collator_selection::migration::v1::MigrateToV1<Runtime>,
	pallet_xcm::migration::v1::VersionCheckedMigrateToV1<Runtime>,
	cumulus_pallet_dmp_queue::migration::Migration<Runtime>,
	cumulus_pallet_xcmp_queue::migration::Migration<Runtime>,
	pallet_balances::migration::MigrateManyToTrackInactive<Runtime, InactiveAccounts>,
	pallet_assets::migration::v1::MigrateToV1<Runtime>,
	pallet_multisig::migrations::v1::MigrateToV1<Runtime>,
	pallet_preimage::migration::v1::Migration<Runtime>,
	migrations::transaction_payment::v0::Migrate,
	migrations::uniques::v0::MigrateToV1,
	migrations::vesting::v0::MigrateToV1,
	migrations::authorship::v0::Migrate,
	pallet_fruniques::migration::v0::MigrateToV1<Runtime>,
	migrations::general::GeneralMigration,
	migrations::identity_verifier::Migrate,
	migrations::aura_verifier::Migrate,
	migrations::balances_verifier::Migrate,
	migrations::collator_selection_verifier::Migrate,
	migrations::dmp_queue_verifier::Migrate,
	migrations::proxy_verifier::Migrate,
	migrations::society_verifier::Migrate,
	migrations::xcmp_queue_verifier::Migrate
);

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	RuntimeUpgrades
>;

pub type RootOrThreeFifthsOfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
>;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
		// in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
		let p = MILLIUNIT / 10;
		let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{generic, traits::{BlakeTwo256, Hash as HashT}};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("luhn"),
	impl_name: create_runtime_str!("luhn"),
	authoring_version: 3,
	spec_version: 3,
	impl_version: 3,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 3,
	state_version: 3,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 12000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLIUNIT: Balance = 1_000_000_000;
pub const MICROUNIT: Balance = 1_000_000;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_div(2),
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
/// into the relay chain.
const UNINCLUDED_SEGMENT_CAPACITY: u32 = 1;
/// How many parachain blocks are processed by the relay chain per parent. Limits the
/// number of blocks authored per slot.
const BLOCK_PROCESSING_VELOCITY: u32 = 1;
/// Relay chain slot duration, in milliseconds.
const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The block type.
	type Block = Block;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = Everything;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<0>;
	type MaxFreezes = ConstU32<0>;
}

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 10 * MICROUNIT;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = ConstU8<5>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = ();
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
	type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
		Runtime,
		RELAY_CHAIN_SLOT_DURATION_MILLIS,
		BLOCK_PROCESSING_VELOCITY,
		UNINCLUDED_SEGMENT_CAPACITY,
	>;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but let's be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	#[cfg(feature = "experimental")]
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Self>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinEligibleCollators: u32 = 5;
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const MaxInvulnerables: u32 = 100;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinEligibleCollators = MinEligibleCollators;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}


// Prebuilt pallets

parameter_types! {
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 200 * CENTS;
	pub const BountyDepositBase: Balance = 100 * CENTS;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = 1 * DOLLARS;
	pub const CuratorDepositMax: Balance = 100 * DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
}

impl pallet_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
	type ChildBountyManager = ChildBounties;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 2000 * CENTS;
	pub const ProposalBondMaximum: Balance = 1 * GRAND;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const Burn: Permill = Permill::from_perthousand(2);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 100 * CENTS;
	pub const DataDepositPerByte: Balance = 1 * CENTS;
	pub const MaximumReasonLength: u32 = 16384;
	pub const MaxApprovals: u32 = 100;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
	pub const SpendPayoutPeriod: BlockNumber = 30 * DAYS;
	pub TreasuryAccount: AccountId = Treasury::account_id();
}

pub struct MulBy<N>(PhantomData<N>);
impl<N: Get<u128>> ConversionFromAssetBalance<u128, (), u128> for MulBy<N> {
	type Error = ();
	fn from_asset_balance(balance: u128, _asset_id: ()) -> Result<u128, Self::Error> {
		return balance.checked_mul(N::get()).ok_or(())
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_successful(_: u32) {}
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ProposalBondMaximum; //set to () on the substrate repo
	type SpendPeriod = SpendPeriod;
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
	type Burn = Burn;
	type BurnDestination = Treasury;
	type MaxApprovals = MaxApprovals;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type SpendFunds = Bounties;
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = Indices;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = MulBy<ConstU128<1>>;
	type PayoutPeriod = SpendPayoutPeriod;
}


parameter_types! {
	pub const ChildBountyValueMinimum: Balance = 1 * DOLLARS;
}

impl pallet_child_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxActiveChildBountyCount = ConstU32<200>;
	type ChildBountyValueMinimum = ChildBountyValueMinimum;
	type WeightInfo = pallet_child_bounties::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const AssetDeposit: Balance = 100 * DOLLARS;
	pub const ApprovalDeposit: Balance = 1 * DOLLARS;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 10 * DOLLARS;
	pub const MetadataDepositPerByte: Balance = 1 * DOLLARS;
	pub const RemoveItemsLimit: u32 = 1000;
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type AssetId = u32;
	type AssetIdParameter = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<DOLLARS>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type RemoveItemsLimit = RemoveItemsLimit;
	type CallbackHandle = ();
}


parameter_types! {
	pub CouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	// pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type MaxProposalWeight = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

parameter_types! {
	pub const GraceStrikes: u32 = 10;
	pub const PeriodSpend: Balance = 50000 * CENTS;
	pub const MaxLockDuration: BlockNumber = 36 * 30 * DAYS;
	pub const VotingPeriod: BlockNumber = 7 * DAYS;
	pub const ClaimPeriod: BlockNumber = 7 * DAYS;
	pub const ChallengePeriod: BlockNumber = 7 * DAYS;
	pub const SocietyPalletId: PalletId = PalletId(*b"py/socie");
}

impl pallet_society::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = SocietyPalletId;
	type Currency = Balances;
	type Randomness = RandomnessCollectiveFlip;
	type GraceStrikes = GraceStrikes;
	type PeriodSpend = PeriodSpend;
	type VotingPeriod = VotingPeriod;
	type ClaimPeriod = ClaimPeriod;
	type MaxLockDuration = MaxLockDuration;
	type FounderSetOrigin = EnsureRoot<AccountId>;
	type ChallengePeriod = ChallengePeriod;
	type MaxPayouts = ConstU32<5>;
	type MaxBids = ConstU32<3>;
	type WeightInfo = pallet_society::weights::SubstrateWeight<Runtime>;

}

parameter_types! {
	// Minimum 100 bytes/KSM deposited (1 CENT/byte)
	pub const BasicDeposit: Balance = 1000 * CENTS;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 * CENTS;        // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 200 * CENTS;   // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type IdentityInformation = pallet_identity::simple::IdentityInfo<MaxAdditionalFields>;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRoot<AccountId>;
	type RegistrarOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ConfigDepositBase: Balance = 500 * CENTS;
	pub const FriendDepositFactor: Balance = 50 * CENTS;
	pub const MaxFriends: u16 = 9;
	pub const RecoveryDeposit: Balance = 500 * CENTS;
}

impl pallet_recovery::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_recovery::weights::SubstrateWeight<Runtime>;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ConfigDepositBase = ConfigDepositBase;
	type FriendDepositFactor = FriendDepositFactor;
	type MaxFriends = MaxFriends;
	type RecoveryDeposit = RecoveryDeposit;
}

parameter_types! {
	pub const IndexDeposit: Balance = 100 * CENTS;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MembershipMaxMembers: u32 = 100_000;
}

impl pallet_membership::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureRoot<AccountId>;
	type RemoveOrigin = EnsureRoot<AccountId>;
	type SwapOrigin = EnsureRoot<AccountId>;
	type ResetOrigin = EnsureRoot<AccountId>;
	type PrimeOrigin = EnsureRoot<AccountId>;
	type MembershipInitialized = Council;
	type MembershipChanged = Council;
	type MaxMembers = MembershipMaxMembers;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU32<100>;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * DOLLARS;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
}

use codec::{Decode, Encode, MaxEncodedLen};

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	Any,
	NonTransfer,
	Governance,
	Marketplaces,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances(..)
					| RuntimeCall::Assets(..)
					| RuntimeCall::Uniques(..)
					| RuntimeCall::Fruniques(..)
					| RuntimeCall::Vesting(pallet_vesting::Call::vested_transfer { .. })
					| RuntimeCall::Indices(pallet_indices::Call::transfer { .. })
			),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Bounties(..)
					| RuntimeCall::ChildBounties(..)
					| RuntimeCall::Council(..)
					| RuntimeCall::Society(..)
					// | RuntimeCall::TechnicalCommittee(..)
					// | RuntimeCall::Elections(..)
					| RuntimeCall::Treasury(..)
			),
			ProxyType::Marketplaces => matches!(c, RuntimeCall::GatedMarketplace(..)),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

impl pallet_remark::Config for Runtime {
	type WeightInfo = pallet_remark::weights::SubstrateWeight<Self>;
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub const PreimageBaseDeposit: Balance = 1 * DOLLARS;
	pub const PreimageByteDeposit: Balance = 1 * CENTS;
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = ();
}

impl pallet_whitelist::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WhitelistOrigin = EnsureRoot<AccountId>;
	type DispatchWhitelistedOrigin = EnsureRoot<AccountId>;
	type Preimages = Preimage;
	type WeightInfo = pallet_whitelist::weights::SubstrateWeight<Runtime>;
}


// Custom Pallets

/// The payload being signed in transactions.

use sp_runtime::SaturatedConversion;

pub type SignedPayload = sp_runtime::generic::SignedPayload<RuntimeCall, SignedExtra>;

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = UncheckedExtrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as sp_runtime::traits::Verify>::Signer,
		account: AccountId,
		index: Index,
	) -> Option<(
		RuntimeCall,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		let period = BlockHashCount::get() as u64;
		let current_block = System::block_number().saturated_into::<u64>().saturating_sub(1);
		let tip = 0;
		let extra: SignedExtra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(index),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		);

		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = account;
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (sp_runtime::MultiAddress::Id(address), signature.into(), extra)))
	}
}



impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}

parameter_types! {
  pub const MaxScopesPerPallet: u32 = 1000;
  pub const MaxRolesPerPallet: u32 = 50;
  pub const RoleMaxLen: u32 = 50;
  pub const PermissionMaxLen: u32 = 50;
  pub const MaxPermissionsPerRole: u32 = 100;
  pub const MaxRolesPerUser: u32 = 10;
  pub const MaxUsersPerRole: u32 = 2500;
}

impl pallet_rbac::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type RemoveOrigin = RootOrThreeFifthsOfCouncil;
  type MaxScopesPerPallet = MaxScopesPerPallet;
  type MaxRolesPerPallet = MaxRolesPerPallet;
  type RoleMaxLen = RoleMaxLen;
  type PermissionMaxLen = PermissionMaxLen;
  type MaxPermissionsPerRole = MaxPermissionsPerRole;
  type MaxRolesPerUser = MaxRolesPerUser;
  type MaxUsersPerRole = MaxUsersPerRole;
}

parameter_types! {
	pub const XPubLen: u32 = XPUB_LEN;
	pub const PSBTMaxLen: u32  = 2048;
	pub const MaxVaultsPerUser: u32 = 10;
	pub const MaxCosignersPerVault: u32 = 7;
	pub const VaultDescriptionMaxLen: u32 = 200;
	pub const OutputDescriptorMaxLen: u32 = 2048;
	pub const MaxProposalsPerVault: u32 = 100;
}

impl pallet_bitcoin_vaults::Config for Runtime {
	type AuthorityId = pallet_bitcoin_vaults::types::crypto::TestAuthId;
	type RuntimeEvent = RuntimeEvent;
	type ChangeBDKOrigin = RootOrThreeFifthsOfCouncil;
	type XPubLen = XPubLen;
	type PSBTMaxLen = PSBTMaxLen;
	type MaxVaultsPerUser = MaxVaultsPerUser;
	type MaxCosignersPerVault = MaxCosignersPerVault;
	type VaultDescriptionMaxLen = VaultDescriptionMaxLen;
	type OutputDescriptorMaxLen = OutputDescriptorMaxLen;
	type MaxProposalsPerVault = MaxProposalsPerVault;
}

parameter_types! {
	pub const MaxOwnedDocs: u32 = 100;
	pub const MaxSharedFromDocs: u32 = 100;
	pub const MaxSharedToDocs: u32 = 100;
	pub const DocNameMinLen: u32 = 3;
	pub const DocNameMaxLen: u32 = 50;
	pub const DocDescMinLen: u32 = 5;
	pub const DocDescMaxLen: u32 = 100;
	pub const GroupNameMinLen: u32 = 3;
	pub const GroupNameMaxLen: u32 = 50;
	pub const MaxMemberGroups: u32 = 100;
}

impl pallet_confidential_docs::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RemoveOrigin = RootOrThreeFifthsOfCouncil;
	type MaxOwnedDocs = MaxOwnedDocs;
	type MaxSharedFromDocs = MaxSharedFromDocs;
	type MaxSharedToDocs = MaxSharedToDocs;
	type DocNameMinLen = DocNameMinLen;
	type DocNameMaxLen = DocNameMaxLen;
	type DocDescMinLen = DocDescMinLen;
	type DocDescMaxLen = DocDescMaxLen;
	type GroupNameMinLen = GroupNameMinLen;
	type GroupNameMaxLen = GroupNameMaxLen;
	type MaxMemberGroups = MaxMemberGroups;
}

parameter_types! {
  pub const MaxReserves: u32 = 200;
}

impl pallet_mapped_assets::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type Balance = u128;
  type AssetId = u32;
  type AssetIdParameter = u32;
  type Currency = Balances;
  type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
  type ForceOrigin = EnsureRoot<AccountId>;
  type AssetDeposit = AssetDeposit;
  type AssetAccountDeposit = ConstU128<DOLLARS>;
  type MetadataDepositBase = MetadataDepositBase;
  type MetadataDepositPerByte = MetadataDepositPerByte;
  type ApprovalDeposit = ApprovalDeposit;
  type StringLimit = StringLimit;
  type Freezer = ();
  type Extra = ();
  type WeightInfo = ();
  type RemoveItemsLimit = RemoveItemsLimit;
  type CallbackHandle = ();
  type Rbac = RBAC;
}

impl pallet_fruniques::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RemoveOrigin = RootOrThreeFifthsOfCouncil;
	type Rbac = RBAC;
	type ChildMaxLen = ChildMaxLen;
	type MaxParentsInCollection = MaxParentsInCollection;
	// type PalletId = FruniquesPalletId;
}

parameter_types! {
  pub const LabelMaxLen: u32 = 32;
  pub const MaxAuthsPerMarket: u32 = 30;
  pub const MaxRolesPerAuth : u32 = 1;
  pub const MaxApplicants: u32 = 3;
  pub const MaxBlockedUsersPerMarket: u32 = 100;
  pub const NotesMaxLen: u32 = 256;
  pub const MaxFeedbackLen: u32 = 256;
  pub const NameMaxLen: u32 = 100;
  pub const MaxFiles: u32 = 10;
  pub const MaxApplicationsPerCustodian: u32 = 2;
  pub const MaxMarketsPerItem: u32 = 10;
  pub const MaxOffersPerMarket: u32 = 100;
}
impl pallet_gated_marketplace::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type MaxAuthsPerMarket = MaxAuthsPerMarket;
  type MaxRolesPerAuth = MaxRolesPerAuth;
  type MaxApplicants = MaxApplicants;
  type MaxBlockedUsersPerMarket = MaxBlockedUsersPerMarket;
  type LabelMaxLen = LabelMaxLen;
  type NotesMaxLen = NotesMaxLen;
  type MaxFeedbackLen = MaxFeedbackLen;
  type NameMaxLen = NameMaxLen;
  type MaxFiles = MaxFiles;
  type MaxApplicationsPerCustodian = MaxApplicationsPerCustodian;
  type MaxOffersPerMarket = MaxOffersPerMarket;
  type MaxMarketsPerItem = MaxMarketsPerItem;
  type Timestamp = Timestamp;
  type Moment = u64;
  //type LocalCurrency = Balances;
  type Rbac = RBAC;
}

impl pallet_afloat::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type Currency = Balances;
  type TimeProvider = Timestamp;
  //type RemoveOrigin = EnsureRoot<AccountId>;
  type Rbac = RBAC;
  type ItemId = u32;
}

parameter_types! {
  pub const MaxRecordsAtTime:u32 = 50;
}

impl pallet_fund_admin_records::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type RemoveOrigin = RootOrThreeFifthsOfCouncil;
  type Timestamp = Timestamp;
  type Moment = u64;
  type MaxRecordsAtTime = MaxRecordsAtTime;
}

parameter_types! {
	pub const MaxRecursions: u32 = 10;
	pub const ResourceSymbolLimit: u32 = 10;
	pub const PartsLimit: u32 = 3;
	pub const MaxPriorities: u32 = 3;
	pub const CollectionSymbolLimit: u32 = 100;
}

parameter_types! {
	pub const CollectionDeposit: Balance = 100 * DOLLARS;
	pub const ItemDeposit: Balance = 1 * DOLLARS;
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 256;
	pub const ChildMaxLen: u32 = 25;
	pub const MaxParentsInCollection: u32 = 4_294_967_295;
}

impl pallet_uniques::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = MetadataDepositBase;
	type DepositPerByte = MetadataDepositPerByte;
	type StringLimit = StringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = pallet_uniques::weights::SubstrateWeight<Runtime>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type Locker = ();
}



// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime
	{
		// System support stuff.
		System: frame_system = 0,
		ParachainSystem: cumulus_pallet_parachain_system = 1,
		Timestamp: pallet_timestamp = 2,
		ParachainInfo: parachain_info = 3,

		// Monetary stuff.
		Balances: pallet_balances = 10,
		TransactionPayment: pallet_transaction_payment = 11,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship = 20,
		CollatorSelection: pallet_collator_selection = 21,
		Session: pallet_session = 22,
		Aura: pallet_aura = 23,
		AuraExt: cumulus_pallet_aura_ext = 24,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue = 30,
		PolkadotXcm: pallet_xcm = 31,
		CumulusXcm: cumulus_pallet_xcm = 32,
		DmpQueue: cumulus_pallet_dmp_queue = 33,

		// Additional pallets
		Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 61,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config<T>, Event<T>} = 62,
		ChildBounties: pallet_child_bounties::{Pallet, Call, Storage, Event<T>} = 63,
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 64,
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>}  = 65,
		// AssetTxPayment: pallet_asset_tx_payment::{Pallet, Call, Storage, Event<T>} = 16,

		// Governance Related
		Council: pallet_collective::<Instance1> = 81,
		RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip::{Pallet, Storage}  = 82,
		Society: pallet_society::{Pallet, Call, Storage, Event<T>}  = 83,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>}  = 84,
		Recovery: pallet_recovery::{Pallet, Call, Storage, Event<T>}  = 85,
		Indices: pallet_indices::{Pallet, Call, Storage, Event<T>}  = 86,
		Membership: pallet_membership::{Pallet, Call, Storage, Event<T>}  = 87,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>}  = 88,
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>}  = 89,
		Sudo: pallet_sudo::{Pallet, Call, Storage, Event<T>, Config<T>}  = 90,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>}  = 91,
		Remark: pallet_remark::{Pallet, Call, Storage, Event<T>}  = 52,
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>}  = 100,
		Whitelist: pallet_whitelist::{Pallet, Call, Storage, Event<T>}  = 101,

		// Custom Pallets
		Fruniques: pallet_fruniques::{Pallet, Call, Storage, Event<T>}  = 151,
		GatedMarketplace: pallet_gated_marketplace::{Pallet, Call, Storage, Event<T>}  = 152,
		BitcoinVaults: pallet_bitcoin_vaults::{Pallet, Call, Storage, Config<T>, Event<T>}  = 154,
		RBAC: pallet_rbac::{Pallet, Call, Storage, Event<T>}  = 155,
		ConfidentialDocs: pallet_confidential_docs::{Pallet, Call, Storage, Event<T>}  = 156,
		FundAdminRecords: pallet_fund_admin_records::{Pallet, Call, Storage, Event<T>}  = 157,
		Afloat: pallet_afloat::{Pallet, Call, Storage, Event<T>}  = 158,
		MappedAssets: pallet_mapped_assets::{Pallet, Call, Storage, Config<T>, Event<T>}  = 159,
	}
);

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_sudo, Sudo]
		[pallet_collator_selection, CollatorSelection]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
	);
}

impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect,
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{BenchmarkError, Benchmarking, BenchmarkBatch};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {
				fn setup_set_code_requirements(code: &sp_std::vec::Vec<u8>) -> Result<(), BenchmarkError> {
					ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
					Ok(())
				}

				fn verify_set_code() {
					System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
				}
			}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_config::<RuntimeGenesisConfig>(config)
		}
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}

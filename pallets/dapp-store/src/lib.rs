#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::traits::Randomness;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::fmt;
	use frame_support::{
		pallet_prelude::*,
		traits::{Randomness, Time},
		BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// Struct for holding dapp information.
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Dapp<T: Config> {
		pub id: T::Hash,
		pub owner: T::AccountId,
		pub name: Vec<u8>,
		pub description: Vec<u8>,
		pub logo_url: Vec<u8>,
		pub link_to_dapp: Vec<u8>,
		pub is_mini_dapp: bool,
		pub tags: Vec<Vec<u8>>,
		pub created_date: <<T as Config>::Time as Time>::Moment,
	}

	// Implement debug for Dapp
	impl<T: Config> fmt::Debug for Dapp<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Dapp")
				.field("id", &self.id)
				.field("owner", &self.owner)
				.field("name", &self.name)
				.field("description", &self.description)
				.field("logo_url", &self.logo_url)
				.field("link_to_dapp", &self.link_to_dapp)
				.field("is_mini_dapp", &self.is_mini_dapp)
				.field("tags", &self.tags)
				.field("created_date", &self.created_date)
				.finish()
		}
	}

	// Enum and implementation to handle Gender type in Kitty struct.
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Gender {
		Male,
		Female,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Time: Time;
		type DappRandomId: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type DappOwnedLimit: Get<u32>;
	}

	// Storage total Dapp in store.
	#[pallet::storage]
	#[pallet::getter(fn dapp_count)]
	pub type CountDapp<T> = StorageValue<_, u32, ValueQuery>;

	// Storage dapp id list in store.
	#[pallet::storage]
	#[pallet::getter(fn dapp_id_list)]
	pub type DappIdList<T: Config> = StorageValue<_, Vec<T::Hash>, ValueQuery>;

	// Storage Dapps map.
	#[pallet::storage]
	#[pallet::getter(fn dapp_list)]
	pub type Dapps<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Dapp<T>>;

	// Storage Dapps owned
	#[pallet::storage]
	#[pallet::getter(fn dapp_owned)]
	pub type DappsOwned<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::DappOwnedLimit>,
		ValueQuery,
	>;

	// Storage Dapps installed
	#[pallet::storage]
	#[pallet::getter(fn dapp_installed)]
	pub type DappsInstalled<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::DappOwnedLimit>,
		ValueQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		DappCreated(T::AccountId, T::Hash),
		DappTransferred(T::AccountId, T::AccountId, T::Hash),
		DappInstalled(T::AccountId, T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		DappDuplicate,
		DappOverflow,
		NoneDapp,
		OverDappOwnedLimit,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.

		// create_dapp
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_dapp(
			origin: OriginFor<T>,
			name: Vec<u8>,
			description: Vec<u8>,
			logo_url: Vec<u8>,
			link_to_dapp: Vec<u8>,
			is_mini_dapp: bool,
			tags: Vec<Vec<u8>>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let owner = ensure_signed(origin)?;

			// Generate id for dapp
			let id = Self::generate_id()?;

			// Get create date
			let created_date = T::Time::now();

			// Create new dapp instance
			let dapp = Dapp::<T> {
				id: id.clone(),
				owner: owner.clone(),
				name,
				description,
				logo_url,
				link_to_dapp,
				is_mini_dapp,
				tags,
				created_date,
			};

			// Log debug dapp instance
			log::info!("==> {:?}", dapp);

			// Check dapp overflow
			let current_id = <CountDapp<T>>::get();
			let next_id = current_id.checked_add(1).ok_or(<Error<T>>::DappOverflow)?;

			// update dapp id list
			let mut dapp_id_list = <DappIdList<T>>::get();
			dapp_id_list.push(id.clone());

			// Update dapps owned
			let mut dapps_owned = DappsOwned::<T>::get(&owner);
			dapps_owned.try_push(id.clone()).map_err(|_| <Error<T>>::OverDappOwnedLimit)?;

			// Update storage
			<DappsOwned<T>>::insert(&owner, dapps_owned);
			<CountDapp<T>>::put(next_id);
			<DappIdList<T>>::put(dapp_id_list);
			<Dapps<T>>::insert(id.clone(), dapp);

			// Emit an event.
			Self::deposit_event(Event::DappCreated(owner.clone(), id));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn install_dapp(origin: OriginFor<T>, id: T::Hash) -> DispatchResult {
			// Check signed
			let owner = ensure_signed(origin)?;

			log::info!("==> {:?}", id);

			// Update dapps owned
			let mut dapps_installed = DappsInstalled::<T>::get(&owner);
			dapps_installed
				.try_push(id.clone())
				.map_err(|_| <Error<T>>::OverDappOwnedLimit)?;

			log::info!("==> {:?}", dapps_installed);

			// Update storage
			<DappsInstalled<T>>::insert(&owner, dapps_installed);

			// Emit an event.
			Self::deposit_event(Event::DappInstalled(owner.clone(), id));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer_dapp(
			origin: OriginFor<T>,
			to: T::AccountId,
			id: T::Hash,
		) -> DispatchResult {
			// Check signed
			let from = ensure_signed(origin)?;

			let mut dapp = <Dapps<T>>::get(id.clone()).ok_or(<Error<T>>::NoneDapp)?;
			let mut from_owned = DappsOwned::<T>::get(&from);

			// Remove dapp from list of owned dapps.
			if let Some(ind) = from_owned.iter().position(|ids| *ids == id) {
				from_owned.swap_remove(ind);
			} else {
				return Err(<Error<T>>::NoneDapp.into());
			}

			// Update new owner for dapp
			let mut to_owned = DappsOwned::<T>::get(&to);
			to_owned.try_push(id.clone()).map_err(|_| <Error<T>>::OverDappOwnedLimit).ok();
			dapp.owner = to.clone();

			// Write updates to storage
			<Dapps<T>>::insert(&id, dapp);
			<DappsOwned<T>>::insert(&to, to_owned);
			<DappsOwned<T>>::insert(&from, from_owned);

			// Emit an event.
			Self::deposit_event(Event::DappTransferred(from.clone(), to.clone(), id.clone()));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	// Generate random id for dapp
	fn generate_id() -> Result<T::Hash, Error<T>> {
		let mut res = T::DappRandomId::random(&b"dappid"[..]).0;
		while Self::dapp_list(res) != None {
			res = T::DappRandomId::random(&b"dappid"[..]).0;
		}
		Ok(res)
	}
}

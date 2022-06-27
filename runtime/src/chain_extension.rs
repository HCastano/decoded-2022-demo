use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;

/// This is the definition of the `Custom` type from our ink! contract. We need this type to match
/// what we have in ink! in order for it to be correctly deserialized when we're reading it out of
/// the buffer.
#[derive(Encode, Decode)]
struct CustomDef {
    inner: sp_std::vec::Vec<u8>,
}

pub struct MyExtension;

// Note, we don't need to have the `SysConfig` trait bound here since the Contract pallet
// Config is required to implement it
//
// Just like when writing runtime code, if we want to access code from specific pallets we need to
// specify that in our generic parameters.
impl<T> ChainExtension<T> for MyExtension
where
    T: pallet_contracts::Config + pallet_template::Config + pallet_scheduler::Config,

    // The Scheduler pallet's `schedule()` dispatchable expects a Scheduler pallet Call. We can't
    // construct this directly from a Contract pallet Call, but we can construct it from a Runtime
    // call (which itself wraps a Contract pallet Call), so we express that requirement here.
    <T as pallet_scheduler::Config>::Call: From<crate::Call>,

    // `pallet_contracts::Call::call()` expects a `MultiAddress`, so we need to make sure this
    // conversion can be done
    sp_runtime::MultiAddress<sp_runtime::AccountId32, ()>:
        From<<T as SysConfig>::AccountId>,
{
    // We will use the `Environment` to get access to the current execution context. What this
    // gives us access to are things like: function arguments and weight information.
    fn call<E>(
        func_id: u32,
        env: Environment<'_, '_, E, InitState>,
    ) -> Result<RetVal, DispatchError>
    where
        E: Ext<T = T>,
        <E::T as SysConfig>::AccountId:
            UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        // When working with chain extensions we "communicate" between our contract and the runtime
        // using a memory buffer.
        //
        // We can read encoded method arguments from this buffer. We can also write the result of
        // our computations into this buffer, which can then get used by ink!.
        let mut env = env.buf_in_buf_out();

        // This is the implementation block of the methods we declared on the ink! side of things.
        //
        // At this point we're writing runtime code, not smart contract code, so we need to be more
        // careful! For instance, we now need to manually track our weight (i.e gas) usage.
        match func_id {
            1 => {
                // This will read some bytes from the memory buffer mentioned above and try to
                // decode them into the specified type. This method should only be used if the size
                // of the type is known ahead of time.
                let something: u32 = env.read_as()?;

                // We need to ensure that we're charging weight to account for the amount of compute
                // used by the call to our pallet. This is something we typically don't have to
                // worry about in the context of smart contracts since they're gas metered.
                //
                // Ideally we should be using benchmarked results here
                // (i.e `T::WeightInfo::do_something()`), but since we know how to calculate the
                // weight manually we're gonna cheat for now.
                let weight = 10_000 + T::DbWeight::get().writes(1);
                env.charge_weight(weight)?;

                // Using `env.ext()` we can access all sorts of info about the execution
                // environment. You can this of this as equivalent to `self.env()` in an ink!
                // contract.
                let caller = env.ext().caller().clone();
                pallet_template::Pallet::<T>::do_something(
                    RawOrigin::Signed(caller).into(),
                    something,
                )?;
            }
            2 => {
                // Since our type interally uses a `Vec` we don't know what the size of it will be
                // ahead of time. This means we can't use `read_as()` which requires the size of the
                // type to be known at compile time (put another way, `read_as()` requires
                // `T: scale::MaxEncodedLen`.
                //
                // We must instead read only the amount of bytes we have gotten as an input.
                let custom: CustomDef = env.read_as_unbounded(env.in_len())?;

                // As mentioned before, we're cheating with this, but it'll do.
                let weight = 10_000 + T::DbWeight::get().writes(1);
                env.charge_weight(weight)?;

                if !custom.inner.len().is_power_of_two() {
                    // Remember that we have a `FromStatusCode` implementation in our contract which
                    // will know to to handle this `RetVal` correctly.
                    //
                    // In our case this maps to our `ExtensionError::CustomCallFailed` error.
                    return Ok(RetVal::Converging(1));
                }

                // Here we don't do anything useful, we just store the length of our vector in
                // storage.
                let caller = env.ext().caller().clone();
                pallet_template::Pallet::<T>::do_something(
                    RawOrigin::Signed(caller).into(),
                    custom.inner.len() as u32,
                )?;
            }
            3 => {
                let at: u32 = env.read_as()?;
                // let weight = T::WeightInfo::schedule(T::MaxScheduledPerBlock::get());
                // env.charge_weight(weight)?;

                let caller = env.ext().caller().clone();
                let dest = env.ext().address().clone().into();
                // let value = env.ext().value_transferred().into();

                // NOTE: About 5% of block weight worked for me here
                let gas_limit = env.ext().gas_meter().gas_left();

                let mut data = crate::Vec::new();

                // If you're unsure about what the selector is, go check out the `metadata.json`
                // file of the contract.
                //
                // TODO: Maybe add an argument here so we can show how to encode it?
                let mut selector: crate::Vec<u8> = [0x00, 0xC0, 0xFF, 0xEE].into();
                data.append(&mut selector);

                let call = crate::Call::Contracts(pallet_contracts::Call::call {
                    dest,
                    value: 0,
                    gas_limit,
                    storage_deposit_limit: None,
                    data,
                })
                .into();

                use frame_support::traits::schedule::MaybeHashed;
                let call = crate::Box::new(MaybeHashed::Value(call));

                pallet_scheduler::Pallet::<T>::schedule(
                    RawOrigin::Signed(caller).into(),
                    at.into(),
                    None,
                    Default::default(),
                    call,
                )?;
            }
            _ => panic!("Unrecognized function ID."),
        }

        Ok(RetVal::Converging(0))
    }
}

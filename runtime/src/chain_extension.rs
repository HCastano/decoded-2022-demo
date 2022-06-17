use frame_support::traits::Get;
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;

pub struct MyExtension;

// Note, we don't need to have the `SysConfig` trait bound here since the Contract pallet
// Config is required to implement it
//
// Just like when writing runtime code, if we want to access code from specific pallets we need to
// specify that in our generic parameters.
//
// We will use the `Environment` to get access to the current execution context. What this gives us
// access to are things like: function arguments and weight information.
impl<T> ChainExtension<T> for MyExtension
where
    T: pallet_contracts::Config + pallet_template::Config,
{
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
            _ => panic!("Unrecognized function ID."),
        }

        Ok(RetVal::Converging(0))
    }
}

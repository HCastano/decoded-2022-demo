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
        // What you should know is that at this point we're writing "real" runtime code, so you need
        // to be careful about tracking Weight and writing to storage.
        match func_id {
            1 => {
                let something: u32 = env.read_as()?;

                // Need to `clone()` this since the `From` implementation doesn't cover references to
                // `AccountId`s
                let caller = env.ext().caller().clone();

                // TODO: Need to charge weight for this. So we'll need to benchmark the
                // `do_something` function and then use the generated weights.
                //
                // let weight = T::WeightInfo::do_something();
                // env.charge_weight(weight);
                // https://crates.parity.io/pallet_contracts/chain_extension/struct.Environment.html#method.charge_weight
                pallet_template::Pallet::<T>::do_something(
                    frame_system::RawOrigin::Signed(caller).into(),
                    something,
                )?;
            }
            _ => panic!("Unrecognized function ID."),
        }

        Ok(RetVal::Converging(0))
    }
}

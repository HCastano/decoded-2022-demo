use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;

struct MyExtension;

// NANDO: Note, we don't need to have the `SysConfig` trait bound here since the Contract pallet
// Config is required to implement it
impl<T: pallet_contracts::Config> ChainExtension<T> for MyExtension {
    fn call<E>(
        _func_id: u32,
        _env: Environment<'_, '_, E, InitState>,
    ) -> Result<RetVal, DispatchError>
    where
        E: Ext<T = T>,
        <E::T as SysConfig>::AccountId:
            UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        todo!()
    }
}

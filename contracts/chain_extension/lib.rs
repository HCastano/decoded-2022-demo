#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::chain_extension]
pub trait MyChainExtension {
    type ErrorCode = ExtensionError;

    /// We can explicitly opt out of returning and handling a `Result` using
    #[ink(extension = 1, returns_result = false, handle_status = false)]
    fn do_something(something: u32);
}

#[derive(Debug, scale::Encode, scale::Decode)]
pub enum ExtensionError {
    SomethingWentWrong,
    EncodingFailed,
}

// NANDO: We need to implement this manually, not clear from the docs that we need to do this
impl ink_env::chain_extension::FromStatusCode for ExtensionError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::SomethingWentWrong),
            _ => panic!("encountered unknown status code"),
        }
    }
}

// NANDO: The rand-extension example doesn't use this, but we need it...
impl From<scale::Error> for ExtensionError {
    fn from(_: scale::Error) -> Self {
        Self::EncodingFailed
    }
}

/// The `Environment` describes the _context_ in which our smart contract is executing in. More
/// concretely it contains the properties of the blockchain in which our smart contracts are being
/// executed. These properties include thing such as the types of accounts being used (`AccountId`)
/// and the type of the block number used by the chain.
///
/// Since Substrate is a generic framework we are not able to make assumptions about these
/// properties. It is prefectly fine for one chain to have `u32` block numbers and another to have
/// `u128`.
///
/// The `DefaultEnvironment` matches the default properties set out in the Subtrate node template.
use ink_env::{DefaultEnvironment, Environment};

/// The default environment assumes that no chain extensions are present. However, we know there is
/// at least one (we're implementing it!) so we need to update our `Environment` to match that.
///
/// We can re-use the rest of the properties from the `Default` environment since we haven't changed
/// those.
pub enum CustomEnvironment {}

/// NANDO: Should probably link to the `Environment` trait docs in the chain extension documentation
impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <DefaultEnvironment as Environment>::AccountId;
    type Balance = <DefaultEnvironment as Environment>::Balance;
    type Hash = <DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = MyChainExtension;
}

/// Now we need to tell our contract to use our custom environment.
///
/// This will give us access to the chain extension that we've defined.
#[ink::contract(env = crate::CustomEnvironment)]
mod chain_extension {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct ChainExtension {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl ChainExtension {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Example of how to use the `do_something` method of our chain extension
        ///
        /// Note, we need to ensure we indicate that this call mutates state, otherwise it won't
        /// work.
        #[ink(message)]
        pub fn do_something(&mut self, something: u32) {
            self.env().extension().do_something(something);
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut chain_extension = ChainExtension::new(false);
        }
    }
}

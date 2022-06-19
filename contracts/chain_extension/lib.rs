#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

/// When we have a custom type we need to make sure that it can be encoded and decoded.
#[derive(scale::Encode, scale::Decode)]
pub struct Custom {
    /// We want to demonstrate how to read dynamically sized types from a chain extension, so we'll
    /// use a type, `Vec`, whose size we cannot necessarily calculate at compile time.
    inner: ink_prelude::vec::Vec<u8>,
}

#[ink::chain_extension]
pub trait MyChainExtension {
    type ErrorCode = ExtensionError;

    /// We can explicitly opt out of returning and handling a `Result` using
    #[ink(extension = 1, returns_result = false, handle_status = false)]
    fn do_something(something: u32);

    /// Here we want to demo what a chain extention with a custom type looks like.
    ///
    /// We also want to see how to handle errors which may arise when calling the extension.
    #[ink(extension = 2)]
    fn custom_type(custom: Custom) -> Result<(), ExtensionError>;

    // TODO: Change to BlockNumber or something
    #[ink(extension = 3)]
    fn schedule_call(at: u32) -> Result<(), ExtensionError>;
}

#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ExtensionError {
    NotAPowerOfTwo,
    EncodingFailed,
}

// NANDO: We need to implement this manually, not clear from the docs that we need to do this
impl ink_env::chain_extension::FromStatusCode for ExtensionError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::NotAPowerOfTwo),
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

    #[ink(event)]
    pub struct SchedulerTriggered {
        at: BlockNumber,
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

        #[ink(message)]
        pub fn custom_type(
            &mut self,
            is_power_of_two: bool,
        ) -> Result<(), crate::ExtensionError> {
            let v = crate::Custom {
                inner: if is_power_of_two {
                    ink_prelude::vec![1, 2]
                } else {
                    ink_prelude::vec![1, 2, 3]
                },
            };

            // Thanks to our `StatusCode` conversion we can easily handle the error using the `?`
            // operator here.
            Ok(self.env().extension().custom_type(v)?)
        }

        #[ink(message)]
        pub fn schedule_call(&mut self, at: u32) -> Result<(), crate::ExtensionError> {
            Ok(self.env().extension().schedule_call(at)?)
        }

        #[ink(message)]
        pub fn scheduler_handler(&mut self) {
            Self::env().emit_event(SchedulerTriggered {
                at: self.env().block_number(),
            });
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
            let _chain_extension = ChainExtension::new(false);
        }
    }
}

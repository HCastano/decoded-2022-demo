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

    /// Our first use case is simple, we just want to write a number to the state of our underlying
    /// Substrate chain.
    ///
    /// By default we have to handle a `Result` from the chain extension, but we can explicitly opt
    /// out of returning and handling a `Result` using these two attributes.
    #[ink(extension = 1, returns_result = false, handle_status = false)]
    fn write_to_storage(value: u32);

    /// Here we want to demo what a chain extention with a custom type looks like.
    ///
    /// We also want to see how to handle errors which may arise when calling the extension.
    ///
    /// One thing to note here is that you can't use the associated type like we would in a normal
    /// Rust trait definition (e.g `Result<(), Self::ErrorCode>`), but we instead have to use the
    /// concrete type.
    #[ink(extension = 2)]
    fn custom_type_with_result(custom: Custom) -> Result<(), ExtensionError>;

    /// For our final trick we will demonstrate bi-directional communication using chain
    /// extensions.
    ///
    /// This means that we will use a chain extension to call the Scheduler pallet in order to
    /// schedule a call which triggers an `#[ink(message)]` at some future point in time.
    #[ink(extension = 3)]
    fn schedule_call(at: u32) -> Result<(), ExtensionError>;
}

#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ExtensionError {
    CustomCallFailed,
    EncodingFailed,
}

impl ink_env::chain_extension::FromStatusCode for ExtensionError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::CustomCallFailed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

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

    #[ink(storage)]
    pub struct ChainExtension {}

    #[ink(event)]
    pub struct SchedulerTriggered {
        at: BlockNumber,
        arg: u32,
    }

    impl ChainExtension {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Note, we need to ensure we indicate that this call mutates state, otherwise it won't
        /// work.
        #[ink(message)]
        pub fn write_to_storage(&mut self, value: u32) {
            self.env().extension().write_to_storage(value);
        }

        #[ink(message)]
        pub fn custom_type_with_result(
            &mut self,
            success: bool,
        ) -> Result<(), crate::ExtensionError> {
            let v = crate::Custom {
                inner: if success {
                    ink_prelude::vec![1, 2]
                } else {
                    ink_prelude::vec![1, 2, 3]
                },
            };

            // Thanks to our `StatusCode` conversion we can easily handle the error using the `?`
            // operator here.
            Ok(self.env().extension().custom_type_with_result(v)?)
        }

        #[ink(message)]
        pub fn schedule_call(&mut self, at: u32) -> Result<(), crate::ExtensionError> {
            Ok(self.env().extension().schedule_call(at)?)
        }

        #[ink(message, selector = 0xC0FFEE)]
        pub fn scheduler_handler(&mut self, arg: u32) {
            Self::env().emit_event(SchedulerTriggered {
                at: self.env().block_number(),
                arg,
            });
        }
    }
}

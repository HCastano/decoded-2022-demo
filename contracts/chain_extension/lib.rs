#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::chain_extension]
pub trait MyChainExtension {
    type ErrorCode = ExtensionError;

    #[ink(extension = 1, returns_result = false)]
    fn read(key: &[u8]) -> Vec<u8>;

    /// By default the chain extension assumes that our method call returns a `Result`.
    #[ink(extension = 2)]
    fn read_small(key: &[u8]) -> Result<u32, ExtensionError>;
}

#[derive(scale::Encode, scale::Decode)]
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
            2 => Err(Self::EncodingFailed),
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

#[ink::contract]
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

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
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

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let chain_extension = ChainExtension::default();
            assert_eq!(chain_extension.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut chain_extension = ChainExtension::new(false);
            assert_eq!(chain_extension.get(), false);
            chain_extension.flip();
            assert_eq!(chain_extension.get(), true);
        }
    }
}

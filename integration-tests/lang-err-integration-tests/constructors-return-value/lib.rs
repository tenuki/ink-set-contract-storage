#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::constructors_return_value::{
    ConstructorError,
    ConstructorsReturnValue,
    ConstructorsReturnValueRef,
};

#[ink::contract]
pub mod constructors_return_value {
    #[ink(storage)]
    pub struct ConstructorsReturnValue {
        value: bool,
    }

    #[derive(Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct ConstructorError;

    impl ConstructorsReturnValue {
        /// Infallible constructor
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Fallible constructor
        #[ink(constructor)]
        pub fn try_new(succeed: bool) -> Result<Self, ConstructorError> {
            if succeed {
                Ok(Self::new(true))
            } else {
                Err(ConstructorError)
            }
        }

        /// A constructor which reverts and fills the output buffer with an erroneously
        /// encoded return value.
        #[ink(constructor)]
        pub fn revert_new(_init_value: bool) -> Self {
            ink::env::return_value::<ink::ConstructorResult<AccountId>>(
                ink::env::ReturnFlags::new_with_reverted(true),
                &Ok(AccountId::from([0u8; 32])),
            )
        }

        /// A constructor which reverts and fills the output buffer with an erroneously
        /// encoded return value.
        #[ink(constructor)]
        pub fn try_revert_new(init_value: bool) -> Result<Self, ConstructorError> {
            let value = if init_value {
                Ok(Ok(AccountId::from([0u8; 32])))
            } else {
                Err(ink::LangError::CouldNotReadInput)
            };

            ink::env::return_value::<
                ink::ConstructorResult<Result<AccountId, ConstructorError>>,
            >(ink::env::ReturnFlags::new_with_reverted(true), &value)
        }

        /// Returns the current value of the contract storage.
        #[ink(message)]
        pub fn get_value(&self) -> bool {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::ConstructorsReturnValue as Contract;
        use std::any::TypeId;

        #[test]
        #[allow(clippy::assertions_on_constants)]
        fn infallible_constructor_reflection() {
            const ID: u32 = ::ink::selector_id!("new");

            assert!(
                !<Contract as ::ink::reflect::DispatchableConstructorInfo<ID>>::IS_RESULT,
            );
            assert_eq!(
                TypeId::of::<
                    <Contract as ::ink::reflect::DispatchableConstructorInfo<ID>>::Error,
                >(),
                TypeId::of::<&()>(),
            )
        }

        #[test]
        #[allow(clippy::assertions_on_constants)]
        fn fallible_constructor_reflection() {
            const ID: u32 = ::ink::selector_id!("try_new");

            assert!(
                <Contract as ::ink::reflect::DispatchableConstructorInfo<ID>>::IS_RESULT,
            );
            assert_eq!(
                TypeId::of::<
                    <Contract as ::ink::reflect::DispatchableConstructorInfo<ID>>::Error,
                >(),
                TypeId::of::<super::ConstructorError>(),
            )
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::scale::Decode as _;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_infallible_constructor<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let mut constructor = ConstructorsReturnValueRef::new(true);
            let infallible_constructor_result = client
                .instantiate(
                    "constructors_return_value",
                    &ink_e2e::alice(),
                    &mut constructor,
                )
                .dry_run()
                .await
                .result
                .expect("Instantiate dry run should succeed");

            let data = infallible_constructor_result.result.data;
            let decoded_result = Result::<(), ink::LangError>::decode(&mut &data[..])
                .expect("Failed to decode constructor Result");
            assert!(
                decoded_result.is_ok(),
                "Constructor dispatch should have succeeded"
            );

            let mut constructor = ConstructorsReturnValueRef::new(true);
            let success = client
                .instantiate(
                    "constructors_return_value",
                    &ink_e2e::alice(),
                    &mut constructor,
                )
                .submit()
                .await
                .is_ok();

            assert!(success, "Contract created successfully");

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_fallible_constructor_succeed<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let mut constructor = ConstructorsReturnValueRef::try_new(true);
            let result = client
                .instantiate(
                    "constructors_return_value",
                    &ink_e2e::bob(),
                    &mut constructor,
                )
                .dry_run()
                .await
                .result
                .expect("Instantiate dry run should succeed");

            let decoded_result = Result::<
                Result<(), super::ConstructorError>,
                ink::LangError,
            >::decode(&mut &result.result.data[..])
            .expect("Failed to decode fallible constructor Result");

            assert!(
                decoded_result.is_ok(),
                "Constructor dispatch should have succeeded"
            );

            assert!(
                decoded_result.unwrap().is_ok(),
                "Fallible constructor should have succeeded"
            );

            let mut constructor = ConstructorsReturnValueRef::try_new(true);
            let contract = client
                .instantiate(
                    "constructors_return_value",
                    &ink_e2e::bob(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let call = contract.call::<ConstructorsReturnValue>();

            let get = call.get_value();
            let value = client
                .call(&ink_e2e::bob(), &get)
                .dry_run()
                .await
                .return_value();

            assert_eq!(
                true, value,
                "Contract success should write to contract storage"
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_fallible_constructor_fails<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let mut constructor = ConstructorsReturnValueRef::try_new(false);

            let result = client
                .instantiate(
                    "constructors_return_value",
                    &ink_e2e::charlie(),
                    &mut constructor,
                )
                .dry_run()
                .await
                .result
                .expect("Instantiate dry run should succeed");

            let decoded_result = Result::<
                Result<(), super::ConstructorError>,
                ink::LangError,
            >::decode(&mut &result.result.data[..])
            .expect("Failed to decode fallible constructor Result");

            assert!(
                decoded_result.is_ok(),
                "Constructor dispatch should have succeeded"
            );

            assert!(
                decoded_result.unwrap().is_err(),
                "Fallible constructor should have failed"
            );

            let mut constructor = ConstructorsReturnValueRef::try_new(false);
            let result = client
                .instantiate(
                    "constructors_return_value",
                    &ink_e2e::charlie(),
                    &mut constructor,
                )
                .submit()
                .await;

            assert!(
                matches!(result, Err(ink_e2e::Error::<ink::env::DefaultEnvironment>::InstantiateExtrinsic(_))),
                "Constructor should fail"
            );

            Ok(())
        }
    }
}

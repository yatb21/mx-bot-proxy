#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::contract]
pub trait YatbProxyContract {
    #[storage_set("allowedExecutor")]
    fn set_allowed_executor(&self, address: ManagedAddress);

    #[view]
    #[storage_get("allowedExecutor")]
    fn get_allowed_executor(&self) -> ManagedAddress;

    #[storage_mapper("whitelist")]
    fn whitelisted_lp(&self) -> WhitelistMapper<ManagedAddress>;

    #[endpoint]
    #[only_owner]
    fn add_allowed_lp(&self, liquidity_pool: ManagedAddress) {
        self.whitelisted_lp().add(&liquidity_pool);
    }

    #[endpoint]
    #[only_owner]
    fn remove_allowed_lp(&self, liquidity_pool: ManagedAddress) {
        self.whitelisted_lp().remove(&liquidity_pool);
    }

    #[view]
    fn is_allowed_lp(&self, liquidity_pool: ManagedAddress) -> bool {
        self.whitelisted_lp().contains(&liquidity_pool)
    }

    #[endpoint]
    #[payable("*")]
    fn deposit(&self) {
        // deposited funds will always remain in the contract and they can only be
        // withdrawn using the withdraw method
    }

    #[endpoint]
    #[only_owner]
    fn withdraw(&self, token: TokenIdentifier) {
        let owner = self.blockchain().get_owner_address();
        let balance =
            self.blockchain()
                .get_esdt_balance(&self.blockchain().get_sc_address(), &token, 0);
        self.send().direct_esdt(&owner, &token, 0, &balance);
    }

    #[proxy]
    fn lp_contract_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;

    #[endpoint]
    #[payable("*")]
    fn swap(
        &self,
        liquidity_pool: ManagedAddress,
        input_token: TokenIdentifier,
        output_token: TokenIdentifier,
        input_amount: BigUint,
        output_amount: BigUint,
    ) {
        // only allowed executor or smart contract owner are allowed make a swap
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.blockchain().get_owner_address()
                || caller == self.get_allowed_executor(),
            "Only owner or allowed executor can make a swap"
        );

        // check that the liquidity pool is whitelisted, otherwise we could be transfering
        // the tokens to another smart contract with the same method signature
        require!(
            self.whitelisted_lp().contains(&liquidity_pool),
            "Liquidity Pool is not whitelisted"
        );

        let _ = self
            .lp_contract_proxy(liquidity_pool)
            .swap_tokens_fixed_input(output_token, output_amount)
            .with_esdt_transfer((input_token, 0, input_amount))
            .async_call()
            .call_and_exit_ignore_callback();
    }

    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}

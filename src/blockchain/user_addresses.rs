use std::collections::HashMap;

use crate::blockchain::prelude::*;

pub struct UserAddresses {
    chain_addresses: HashMap<String, Vec<String>>,
}

impl UserAddresses {
    pub fn new() -> Self {
        Self {
            chain_addresses: HashMap::new(),
        }
    }

    pub fn add_address(&mut self, chain: &Chain, address: String) {
        self.chain_addresses
            .entry(chain.name.to_owned())
            .or_default()
            .push(address);
    }

    pub fn get_addresses(&self, chain: &Chain) -> Option<&Vec<String>> {
        self.chain_addresses.get(chain.name)
    }

    pub fn from_config(config: &BlockchainConfig) -> Self {
        let mut user_addresses = UserAddresses::new();
        for chain in EVM_CHAINS.values() {
            user_addresses.add_address(chain, config.evm.address.to_string());
        }

        user_addresses.add_address(&COSMOS_HUB, config.cosmos.cosmos_address.to_string());
        user_addresses.add_address(&OSMOSIS, config.cosmos.osmosis_address.to_string());
        user_addresses.add_address(&CELESTIA, config.cosmos.celestia_address.to_string());
        // user_addresses.add_address(&KUJIRA, config.cosmos.kujira_address.to_string());
        user_addresses.add_address(&INJECTIVE, config.cosmos.injective_address.to_string());

        user_addresses
    }
}

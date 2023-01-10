
use tokio::runtime::{Builder, Runtime};

use rethnet_eth::remote::{AccountData, RpcClient};
use rethnet_eth::{Address, Bytes, B256, U256};

use revm::{db::DatabaseRef, AccountInfo, Bytecode, KECCAK_EMPTY};

#[derive(std::hash::Hash, std::cmp::Eq, std::cmp::PartialEq)]
struct StorageLocation {
    address: Address,
    index: U256,
}

struct RemoteDatabase {
    client: RpcClient,
    runtime: Runtime,
}

impl RemoteDatabase {
    pub fn _new(url: &str) -> Self {
        Self {
            client: RpcClient::new(url),
            runtime: Builder::new_multi_thread()
                .build()
                .expect("failed to construct async runtime"),
        }
    }
}

impl DatabaseRef for RemoteDatabase {
    type Error = anyhow::Error;

    fn basic(&self, address: Address) -> anyhow::Result<Option<AccountInfo>> {
        let what_block: u64 = 0; // TODO: resolve the question of where we get this input, or how
                                 // we translate from "latest", or whatever other implied behavior
                                 // we're supposed to have here. Or is the block optional for all
                                 // of the underlying RPC methods?

        let account_data: AccountData = self
            .runtime
            .block_on(self.client.get_account_data(&address, what_block))
            .unwrap_or(AccountData {
                balance: U256::ZERO,
                transaction_count: U256::ZERO,
                code: Bytes::from("0x00"),
            });

        let bytecode = Bytecode::new_raw(account_data.code);

        Ok(Some(AccountInfo {
            balance: account_data.balance,
            nonce: account_data.transaction_count.to(),
            code_hash: bytecode.hash(),
            code: Some(bytecode),
        }))
    }

    fn code_by_hash(&self, code_hash: B256) -> anyhow::Result<Bytecode> {
        if code_hash == KECCAK_EMPTY {
            return Ok(Bytecode::new());
        }

        // TODO: decide how we should service this method. "code by hash" is not a use case of
        // Ethereum JSON-RPC. I had previously set up a HashMap cache, to keep track of all
        // code segments seen flowing through this object, but when I converted from Database to
        // DatabaseRef the compiler pointed out to me that updating a HashMap member instance
        // violates immutability.
        Ok(Bytecode::new())
    }

    fn storage(&self, address: Address, index: U256) -> anyhow::Result<U256> {
        Ok(self
            .runtime
            .block_on(self.client.get_storage_at(&address, index, 0))
            .expect("failed to retrieve storage from remote node")
        )
    }
}

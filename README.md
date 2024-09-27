# avail-subxt-light
This repository is separated into three parts
- core - Contains necessary types and definitions in order to interact with Avail Network
- client - Contains a simple client implementation that can construct valid transactions and submit them
- example - Showcases how to use the client

### Transactions
The following transactions are supported out of box:
- create_application_key
- submit_data

### RPCs
The following rpcs are supported out of box:
- `system_accountNextIndex`
- `AccountNonceApi_account_nonce`
- `chain_getBlockHash`
- `chain_getFinalizedHead`
- `chainSpec_v1_genesisHash`
- `state_getRuntimeVersion`
- `chain_getHeader`
- `chain_getBlock`
- `author_submitExtrinsic`
- `kate_blockLength`
- `kate_queryDataProof`
- `kate_queryProof`
- `kate_queryRows`


### Disclaimer
Things that cannot be done:
- Fetching a block will only partially decode it. It's impossible to fully decode without having all the metadata available.
- Fetching events it not supported because it requires all the metadata to be available. This means that currently it's not possible to know if a transaction was successful or not.

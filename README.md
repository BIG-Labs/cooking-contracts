# Start Cooking Contracts
This repository contains the source code for the Start Cooking protocol smart contracts on the [Osmosis](https://osmosis.zone) blockchain.

You can find information about the usage and function of the smart contracts on the official Start Cooking documentation [site](https://https://docs.start.cooking/)

## Contracts

| Contract                                                     | Description                                                                           | Hash                                                                                  |
|--------------------------------------------------------------|---------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|
| [`flambè`](./contracts/flambe)                       | Represents a single Flambe, containing all its code and liquidity                         | TBD|
| [`flambe-factory`](./contracts/flambe-factory)       | Serves as a proxy to create flambè, storing their addresses and global configurations   | TBD|
## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
cargo wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.15.0
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.

## License

**Copyright 2024 BIG Labs**


Start Cooking is Licensed under the Apache License, Version 2.0 with Common Clause License Condition v1.0 and Additional License Condition v1.0 (the "License");


You may not use this file except in compliance with the License.


You may obtain a copy of the Apache License, Version 2.0 license at http://www.apache.org/licenses/LICENSE-2.0


Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.


See the License for the specific language governing permissions and limitations under the License.


**Commons Clause” License Condition v1.0**

The Software is provided to you by the Licensor under the License, as defined below, subject to the following condition.


Without limiting other conditions in the License, the grant of rights under the License will not include, and the License does not grant to you, the right to Sell the Software.


For purposes of the foregoing, “Sell” means practicing any or all of the rights granted to you under the License to provide to third parties or any other persons, for a fee or other monetary or non-monetary consideration (including without limitation fees for hosting or consulting/ support services related to the Software), a product or service whose value derives, entirely, substantially or similarly, from the functionality of the Software. Any license notice or attribution required by the License must also include this Commons Clause License Condition notice.


Software: Start Cooking


License: Apache 2.0 


Licensor: BIG Labs


**Additional License Condition v1.0**


The terms below are in addition to the Apache 2.0 license terms and the Commons Clause License Conditions v1.0. 


Copying Restrictions:
Despite the terms of the License, no person or entity shall be permitted to copy, or reproduce in any form, any portion of the Software.


Redistribution Restrictions:
Despite the terms of the License, no person or entity shall be permitted to redistribute, share, or make publicly available any portion of the Software or derivative works thereof.
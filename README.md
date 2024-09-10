[![Safety Dance](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

# Bitcoin-Wasm


## :ledger: Index

- [Bitcoin-Wasm](#bitcoin-wasm)
  - [:ledger: Index](#ledger-index)
  - [:beginner: Why Bitcoin Wasm](#beginner-why-bitcoin-wasm-?)
    - [Node](#node)
    - [Signer](#signer)
  - [:station: Features](#station-features)
  - [:zap: Usage](#zap-usage)
    - [:electric\_plug: Installation](#electric_plug-installation)
  - [:wrench: Development](#wrench-development)
    - [:notebook: Pre-Requisites](#notebook-pre-requisites)
    - [:nut\_and\_bolt: Development Environment](#nut_and_bolt-development-environment)
    - [:file\_folder: Folder Structure](#file_folder-folder-structure)
    - [:hammer: Build](#hammer-build)
    - [:office: Examples](#examples)
  - [:cherry\_blossom: Community](#cherry_blossom-community)
    - [:fire: Contribution](#fire-contribution)
    - [:cactus: Branches](#cactus-branches)
    - [:exclamation: Guideline](#exclamation-guideline)
  - [:question: FAQ](#question-faq)
  - [:page\_facing\_up: Resources](#page_facing_up-resources)


## :beginner: Why Bitcoin-WASM ?

**Bitcoin Wasm** consists of a  universally pluggable Bitcoin Payment Node  which  is designed to be embedded in a variety of applications using WASI-compliant runtime SDKs like **wasmtime** and **JCO**. This client provides all the necessary functionalities to send, receive, and convert Bitcoin in a non-custodial way using open standards.

```mermaid
flowchart TD
    A[Bitcoin Wasm] --> B(Node)
    A --> C(Signer)
```

### Node

A pluggable Bitcoin Payment Node is  a type of Bitcoin Light Client that is designed to be integrated into other applications while providing such functionalities: 

- **Send Transaction:** Node enables the transmission of Bitcoin transactions to the network through peer-to-peer (P2P) communication.
- **Receive Transaction:** Node effectively receives and processes incoming Bitcoin transactions from the network, utilizing efficient peer-to-peer (P2P) block filtering techniques to minimize data overhead.
- **Currency Conversion:** Node facilitates the seamless conversion of Bitcoin to and from local currencies by leveraging the capabilities of the tbDEX exchange platform. This feature provides users with the flexibility to exchange Bitcoin for their preferred fiat currencies.

Few restrictions need to be in place in order to ensure usability, which are :

- Run In A Sandboxed environemnt .
- Low requirements for compute, memory, and storage..
- Lack of signing procedures.
- Usage of open standards and decentralized protocols.


### Signer

Signer is a powerful utility Application that handles the storing of private details and the provision of signing operations cryptographic aspects of Bitcoin transactions. Its key functionalities include:

- **Bitcoin Transaction Signing:** Signer utilizes the PSBT (Partially Signed Bitcoin Transaction) format to securely sign Bitcoin transactions.
- **Bitcoin Key Management:** It stores and manages private keys, ensuring the safekeeping of Bitcoin signing primitives.
- **tbDEX Message Signing:** Signer supports the signing of tbDEX messages using the JSON format.
- **tbDEX Key Management:** It stores and manages JWKs (JSON Web Keys), which are essential for signing and verifying tbDEX messages.


##  Why Webassembly ?
WebAssembly (Wasm) is a low-level binary format for executable code that can be run in web browsers and other environments which provides sandboxed execution, memory safety, limited access to system resources, validation and verification . These features makes it appealing for the development of plugin. The further Development of `WASI` (WebAssembly System Interface), a standard that defines a set of system calls that WebAssembly modules can use to interact with the underlying operating system, made it quite possible to develop a fully fledged bitcoin light client as plugin.

## :station: Features

- [x] Light Client (Compact Block Filtering)
- [ ] Descriptor Wallet
- [ ] Silent Payment Support
- [x] tbDEX exchange feature
- [ ] Wasm Signer
  - [ ] PSBT support

## :zap: Usage

The executables are WASM modules which are designed to be embedded within applications using WASI runtime like wasmtime.


## :wrench: Development

We warmly welcome your contributions to Bitcoin Wasm! Whether you're a seasoned developer or just starting out, your help can make a significant impact.

### :notebook: Pre-Requisites

- Rust compiler (v1.78 or later) - [Install](https://www.rust-lang.org/tools/install)
- WASI runtime SDK (e.g., wasmtime) - [Install](https://docs.wasmtime.dev/cli-install.html)

### :nut_and_bolt: Development Environment

A. **Setting Up Your Development Environment**

1. **Clone the Repository:**

   ```bash
   $ git clone https://github.com/aruokhai/bitcoin-wasm.git
   ```

2. Navigate to the project directory:

   ```bash
   $ cd bitcoin-wasm
   ```

3. Install Cargo Component:

   ```bash
   $ cargo install cargo-component
   ```

4. Install `wac-cli` used for building wasm  :

   ```bash
   $ cargo install wac-cli
   ```


B. **Running Tests**

To run the project's integration tests, follow these steps:

1. [Install Docker Engine](https://docs.docker.com/engine/install/)

2. Run the integration tests:

   ```bash
   $ cd tests
   $ ./test-runner.sh
   ```

### :file_folder: Folder Structure

```
bitcoin-wasm
├── crates
│   ├── store
│   │   ├── src
│   │   └── wit
|   |   |  └──world.wit
|   |   ├── Cargo.toml
│   ├── tbdex
│   │   ├── src
│   │   └── wit
|   |   |  └──world.wit
|   |   ├── Cargo.toml
├── Node
│   ├── src
│   │   ├── lib.rs
│   │   └── binding.rs
│   └── wit
|   |   └──world.wit
│   └── cargo.toml
├──tests
│   ├── artifacts
│   |   ├── src
│   │   └── wit
|   |   |  └──world.wit
|   |   └── Cargo.toml
│   ├── cli
|   ├── runner
│   |   ├── src
│   │   └── wit
|   |   |  └──world.wit
|   |   └── build.rs
└── Cargo.toml

```

Here's a breakdown of the key folders:

- **crates:** This folder holds common WASI-compliant components used by the project. It contains components like:
   - **store:** A generic key-value store.
   - **tbdex:** The tbDex protocol.
   - **node:** This subfolder contains the source code for the Node plugin, responsible for interacting with the Bitcoin network.
- **test:** This folder holds integration tests used to verify the functionality of the project. It contains two subfolders:
  - **artifacts:** This contains various components test logic.
  - **runner:** The main entry point for the integration test.

### :hammer: Build

There is currently one way to build the Bitcoin-Wasm project:

1. **Building a specific component:**
   You can build a specific component, like the `web5` package, using the following command:

   ```bash
   $ cargo-component build --package=<package-name>
   ```

   Replace `package-name` with the actual name of the package you want to build (e.g `web5`).

### :office: Usage

An example of the usage of the `Node` plugin can be found in the `examples` folder. Which includes a dummy mobile money CLI application created for the TBDX 2024 Hackathon.


## :cherry_blossom: Community

If it's open-source, talk about the community here, ask social media links and other links.

### :fire: Contribution

Your contributions are always welcome and appreciated. Following are the things you can do to contribute to this project.

1.  **Report a bug** <br>
    If you think you have encountered a bug, and I should know about it, feel free to report it [here]() and I will take care of it.

2.  **Request a feature** <br>
    You can also request for a feature [here](), and if it will viable, it will be picked for development.

3.  **Create a pull request** <br>
    It can't get better then this, your pull request will be appreciated by the community. You can get started by picking up any open issues from [here]() and make a pull request.

> If you are new to open-source, make sure to check read more about it [here](https://www.digitalocean.com/community/tutorial_series/an-introduction-to-open-source) and learn more about creating a pull request [here](https://www.digitalocean.com/community/tutorials/how-to-create-a-pull-request-on-github).

### :cactus: Branches

I use an agile continuous integration methodology, so the version is frequently updated and development is really fast.

1. **`stage`** is the development branch.

2. **`master`** is the production branch.

3. No other permanent branches should be created in the main repository, you can create feature branches but they should get merged with the master.

**Steps to work with feature branch**

1. To start working on a new feature, create a new branch prefixed with `feat` and followed by feature name. (ie. `feat-FEATURE-NAME`)
2. Once you are done with your changes, you can raise PR.

**Steps to create a pull request**

1. Make a PR to `stage` branch.
2. Comply with the best practices and guidelines e.g. where the PR concerns visual elements it should have an image showing the effect.
3. It must pass all continuous integration checks and get positive reviews.

After this, changes will be merged.

### :exclamation: Guideline

- Follow Rust's style guidelines and best practices.
- Write clear and concise commit messages.
- Ensure your code passes all tests.
- Review and provide feedback on other contributors' pull requests.

## :page_facing_up: Resources

- **WASM Specification:** [https://webassembly.org/](https://webassembly.org/)
- **WASI Specification:** [https://wasi.dev/](https://wasi.dev/)
- **Bitcoin Documentation:** [https://bitcoin.org/](https://bitcoin.org/)

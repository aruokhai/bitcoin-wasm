[![Safety Dance](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

# Bitcoin-Wasm

**Bitcoin Wasm** is a **WASM-WASI compliant** embedded Bitcoin payment node. It's designed to be embedded in native applications using WASI-compliant runtime SDKs like **wasmtime** and **JCO**. This node provides all the necessary functionalities to send, receive, and convert Bitcoin in a non-custodial way using open standards. 

![image]()


## :ledger: Index

- [About](#beginner-about)
- [Features](#feat-features)
- [Usage](#zap-usage)
  - [Installation](#electric_plug-installation)
  - [Commands](#package-commands)
- [Development](#wrench-development)
  - [Pre-Requisites](#notebook-pre-requisites)
  - [Developmen Environment](#nut_and_bolt-development-environment)
  - [File Structure](#file_folder-file-structure)
  - [Build](#hammer-build)  
  - [Deployment](#rocket-deployment)  
- [Community](#cherry_blossom-community)
  - [Contribution](#fire-contribution)
  - [Branches](#cactus-branches)
  - [Guideline](#exclamation-guideline)  
- [FAQ](#question-faq)
- [Resources](#page_facing_up-resources)
- [Gallery](#camera-gallery)
- [Credit/Acknowledgment](#star2-creditacknowledgment)
- [License](#lock-license)

##  :beginner: About

Bitcoin Wasm contains two sub-projects: the embeddable, self-contained Bitcoin Light Client called **Node** and the Bitcoin Transaction Signing Utility Software called **Signer**.


``` mermaid
flowchart TD
    A[Bitcoin Wasm] --> B(Node)
    A --> C(Signer)
```

### Node

Node is a self-contained Bitcoin Light Client that provides essential functionalities for interacting with the Bitcoin network. Its key features include:

* **Send Transaction:** Node enables the transmission of Bitcoin transactions to the network through peer-to-peer (P2P) communication.
* **Receive Transaction:** Node effectively receives and processes incoming Bitcoin transactions from the network, utilizing efficient peer-to-peer (P2P) block filtering techniques to minimize data overhead.
* **Currency Conversion:** Node facilitates the seamless conversion of Bitcoin to and from local currencies by leveraging the capabilities of the tbDEX exchange platform. This feature provides users with the flexibility to exchange Bitcoin for their preferred fiat currencies.

### Signer

Signer is a powerful utility that handles the cryptographic aspects of Bitcoin transactions. Its key functionalities include:

* **Bitcoin Transaction Signing:** Signer utilizes the PSBT (Partially Signed Bitcoin Transaction) format to securely sign Bitcoin transactions.
* **Bitcoin Key Management:** It stores and manages private keys, ensuring the safekeeping of Bitcoin signing primitives.
* **tbDEX Message Signing:** Signer supports the signing of tbDEX messages using the JSON format.
* **tbDEX Key Management:** It stores and manages JWKs (JSON Web Keys), which are essential for signing and verifying tbDEX messages.


## :station: Features
- [ ] Light Client (Compact Block Filtering)
- [ ] Descriptor Wallet 
- [ ] Silent Payment Support
- [ ] tbDEX exchange feature
- [ ] Wasm Signer
  - [ ] PSBT support
   
## :zap: Usage
This project is designed to be embedded within another project using WASI SDKs like wasmtime.

###  :electric_plug: Installation

1. Clone the Bitcoin-Wasm repository:
   ```bash
   $ git clone https://github.com/aruokhai/bitcoin-wasm.git
   ```

2. Navigate to the project directory:
   ```bash
   $ cd bitcoin-wasm
   ```


###  :package: Commands
- Commands to start the project.

##  :wrench: Development
We warmly welcome your contributions to Bitcoin Wasm! Whether you're a seasoned developer or just starting out, your help can make a significant impact.

### :notebook: Pre-Requisites
* Rust compiler (v1.78 or later) - [Install](https://www.rust-lang.org/tools/install)
* WASI runtime SDK (e.g., wasmtime) - [Install](https://docs.wasmtime.dev/cli-install.html)

###  :nut_and_bolt: Development Environment

A. **Setting Up Your Development Environment**

1. Install WASM runtime (e.g., `wasmtime`):
   ```bash
   $ curl https://wasmtime.dev/install.sh -sSf | bash
   ```

2. **Clone the Repository:**
   ```bash
   $ git clone https://github.com/aruokhai/bitcoin-wasm.git
   ```

3. Navigate to the project directory:
   ```bash
   $ cd bitcoin-wasm
   ```

4. **Install Rust and Dependencies:**
   Ensure you have Rust installed (version 1.78 or later) with the necessary WASM and Bitcoin-related crates. You can use Rustup to manage your Rust installations:

   ```bash
   $ rustup install stable
   ```

5. **Build the Project:**
   Navigate to the project directory and build the project:
   ```bash
   $ cd bitcoin-wasm
   $ cargo build
   ```

B. **Running Tests**

To run the project's integration tests, follow these steps:

1. Install the `cargo-component` and `wac-cli` tools:

   ```bash
   $ cargo install cargo-component
   $ cargo install wac-cli
   ```

2. Navigate to the project directory.

3. Run the integration tests:

   ```bash
   $ cargo run --package runner --bin runner
   ```

###  :file_folder: Folder Structure

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
├──test-programs 
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

* **crates:** This folder holds crates (Rust libraries) used by the project. It contains subfolders like:
    * **store:** A WASI-compliant generic key-value store.
    * **tbdex:** For interacting with the tbDEX platform.
* **node:** This subfolder contains the source code for the Node component, responsible for interacting with the Bitcoin network. 
* **test-programs:** This folder holds test programs used to verify the functionality of the project. It contains subfolders like:
    * **artifacts:** This stores generated test data or artifacts.
    * **runner:** The main entry point for the end-to-end(e2e) testing. It holds code for running and managing the test programs.

###  :hammer: Build

There is currently one way to build the Bitcoin-Wasm project:

1. **Building a specific component:**
   You can build a specific component, like the `web5` package, using the following command:

   ```bash
   $ cargo-component build --package=<package-name>
   ```

   Replace `package-name` with the actual name of the package you want to build (e.g `web5`).


## :cherry_blossom: Community

If it's open-source, talk about the community here, ask social media links and other links.

 ###  :fire: Contribution

 Your contributions are always welcome and appreciated. Following are the things you can do to contribute to this project.

 1. **Report a bug** <br>
 If you think you have encountered a bug, and I should know about it, feel free to report it [here]() and I will take care of it.

 2. **Request a feature** <br>
 You can also request for a feature [here](), and if it will viable, it will be picked for development.  

 3. **Create a pull request** <br>
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
* Follow Rust's style guidelines and best practices.
* Write clear and concise commit messages.
* Ensure your code passes all tests.
* Review and provide feedback on other contributors' pull requests.

## :question: FAQ

**Common Questions and Answers**

* **Can I use Bitcoin Wasm with other programming languages?** Yes, Bitcoin Wasm can be used with any language that can interact with WASM modules.
* **Is Bitcoin Wasm secure?** Bitcoin Wasm is designed to be secure, but it's important to follow best practices for handling private keys and protecting your application from vulnerabilities.
* **How can I contribute to the project?** See the "Contribution" section for more information.

## :page_facing_up: Resources

* **WASM Specification:** [https://webassembly.org/](https://webassembly.org/)
* **WASI Specification:** [https://github.com/WebAssembly/wasi-io](https://github.com/WebAssembly/wasi-io)
* **Bitcoin Documentation:** [https://bitcoin.org/](https://bitcoin.org/)


##  :camera: Gallery
Pictures of your project.

## :star2: Credit/Acknowledgment
Credit the authors here.

##  :lock: License
Add a license here, or a link to it.
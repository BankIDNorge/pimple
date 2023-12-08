# Pimple
Simplified Just in time access in the terminal when using Entra ID Privileged Identity Management

## Installation
Currently installing pimple requires a full nightly rust toolchain. This can be installed using [rustup](https://rustup.rs/).

Then it can be installed using:
```bash
cargo +nightly install --git https://github.com/BankIDNorge/pimple.git
```

As long as the cargo bin folder is in path you should be able to use the command using:
```bash
pimple pim
```

## Features
* Supports multiple types of PIM
  * Entra roles
  * Group membership
  * Azure resources
* Request multiple roles with the same reason
* Super-fast startup using cached Azure responses
* Ability to force refresh Kubernetes tokens before it expires (this is helpful if you use group based RBAC)

## Planned features
* Make it possible to invoke pimple from scripts
* Ability to await activation
* Approve or reject requests

# UNCX Solana Lp Locker Program

A Extension of UNCX Ethereum Lp Locker Protocol onto Solana.

## Building & testing

### Pre-requisites

Before you can build the program, you will first need to install the following:

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana](https://docs.solana.com/cli/install-solana-cli-tools) (>v1.18.11)
- [Anchor](https://www.anchor-lang.com/docs/installation) (v0.30.0)

### Installing

To install the repo, run:

```bash
git clone https://github.com/uncx-private-repos/solana-lp-locker-monorepo
```

### Building

To build, run:

```bash
anchor build
#sync keys
anchor keys sync
```

### Testing

To see whether all of the tests are passing, run:

```bash
RPC="<MAINNET_RPC_KEY>" cargo test-sbf --test test_all --features testing
```

To drill down on a specific test (e.g., test_add_migrator), run:

```bash
 RPC="<MAINNET_RPC_KEY>" cargo test-sbf --features testing -- test_add_migrator
```

### IDL

Anchor idl generation does not work nicely with rust compilation features, since there is only one instance where we need it to work nicely, for the time being while testing integration manually change the initial admin in the initialize ix signer field to false.

## Note:

We are using a custom fork of anchor with a small optimization made to reduce stack space usage during try_accounts deserialization of structs marked with `#[derive(Accounts)]`

### Verifiable Executable Deployed Program Hash

`Hash : 60838187b2c2b92fc6094bba1dcc2a9607ca40f7fb0fdb9e7011e08fa0c863e9`

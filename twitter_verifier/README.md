# Cartesi <> TLSNotary [ Twitter Verifier ]

[TLSN](https://tlsnotary.org/) is a protocol created and supported by Privacy and Scaling Explorations, a team supported by the Ethereum Foundation. It allows users to securely export data from any website. Behind the scenes, this is an implementation of the [zkTLS](https://telah.vc/zktls) protocol, conceived some time ago.

Using Zero Knowledge Proof (ZKP) technology (WIP), data from web2 requests can be selectively shared in the on-chain environment in a cryptographically verifiable manner. The idea behind this integration is to run the verifier, a component of the protocol's architecture, within Cartesi's infrastructure. This enables the verification of data from a "web2" request inside Cartesi dApps.

In this sense, this template allows use cases like the [ones presented](https://tlsnotary.org/use-cases) to run with the same guarantees as an on-chain environment, in a much easier way, using the same [dependencies already employed in the project](https://github.com/tlsnotary/tlsn/blob/main/tlsn/tlsn-verifier/Cargo.toml) through the CVM ( Cartesi virtual machine ).

![image](https://github.com/user-attachments/assets/2a0e1ab5-b61a-4b17-a170-dafc0d7d445c)

## How to run:

```bash
$ cartesi build
```

```bash
$ cartesi run
```

Then, send any input to call the verifier, which will read the proof stored in the machine's file system:

```bash
$ cartesi send generic
```

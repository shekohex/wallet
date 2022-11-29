## ShekozWallet, my personal crypto wallet

This my custom and simple crypto wallet, designed specifically for my needs:

1. Uses a QR Signer, i.e a QR Hardware wallet, like [Keystone](https://keyst.one/) or [Airgap Vault](https://airgap.it/offline-device/).
2. Stateless, does not store anything, does not leave any traces.
3. Works behind Tor Proxy, by default, considered unsafe otherwise.
4. Simple, Small, and fast interactive CLI Based.

This wallet should support these operations:

- [x] Send Native Tokens.
- [x] Send ERC-20 Tokens.

That's it, that is all I need for now, maybe in the future I will extend it more.

### Download

Currently, my wallet only supports Linux, I mean I would only use it on [Tails OS](https://tails.boum.org/) with a USB Stick, so
my only target for now, is Linux.

You can download it from [Github Releases](https://github.com/shekohex/wallet/releases/latest).

### Usage

Just run `shekozwallet`, that's it, the wallet will look for `shekozwallet.json` config file and will load it.
The wallet is interactive, it will first load and verify the config file, then follow the steps:

1. Sync your Account, from your QR Hardware wallet.
2. Ask about what operation you want to do, for example sending ERC-20 Tokens.
3. after following the steps, it will create the Unsigned Transaction as a QR and ask you to sign it with your signer.
4. Scanning the result and broadcasting the transaction to the network.

### Contributing

While this my custom wallet, and I do not expect anyone else using it too, However, I would be happy to see any contribution or suggestions. So feel free to open an issue to ask about any questions.

### License

Really?

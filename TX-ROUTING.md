## Transaction Routing

Transaction Routing is a mechanism to help keeping your privacy in control, to hide your real cold wallet address from the _public_ block chain. This a way that I use to keep my cold wallet address separated from the hot wallet address(es).

Here is what you need to have first:

1. Cold Wallet account (Hardware wallets, QR Wallets, ..etc).
2. Hot wallet(s) (frame.sh, Rabby.io, Metamask.io, ..etc).
3. Two Separate CEX (Binance, MEXC, Kraken, ..etc).

### Transactions Flow(s):

**From your Cold Wallet to one of your Hot Wallets**:

1. You send the Tokens from your cold wallet to one of the CEX.
2. From that CEX you send the tokens to the other CEX.
3. And from there, you send it to your hot wallet account.

**From your Hot Wallet to your Cold Wallet**

1. You send the Tokens from your hot wallet to one of the CEX.
2. From that CEX, you send the tokens to the other CEX.
3. And from there, you send it to your cold wallet account.

This way, to someone observing these accounts, it is super hard (almost impossible) to trace or relate these two wallets to each others (except the CEXs itself).

### Why two CEX? How is that useful?

Lets first imagine if we just used one CEX, the flow would be like the following:

1. To send money from your Cold wallet to the Hot wallet, you would first send the money to the CEX, send the money from the CEX to the Hot wallet.
2. To send money from your Hot wallet to the Cold wallet, you would send that money to the CEX, send the money from the CEX to the Cold wallet.

So, problem with this is that there is a common address between these two flows, which could link your cold and hot wallet accounts, since both of them sent tokens to the same account. Got it? The usage of two CEX here is that your hot wallets would use one CEX and your Cold wallet would use the other CEX, keeping these wallets separated, and not linked to each other.

> This looks familiar? Yup, it is similar to the Onion Routing (TOR), hence the name, Tx Routing.

### Good Tips to always keep in mind

1. Do not use the same CEX for your Hot and Cold wallets, keep one for each type.
2. Try to split your transaction into `N` smaller ones, and maybe send them in different time(s).
3. Try to not use the Cold and Hot wallet from the same IP, use TOR, VPNs, ..etc.
4. Try not to Doxx yourself by using your Cold wallet in DeFi stuff, use your Hot wallets for that.

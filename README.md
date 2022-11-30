## ShekozWallet, my personal crypto wallet

This my custom and simple crypto wallet, designed specifically for my needs:

1. Uses a QR Signer, i.e a QR Hardware wallet, like [Keystone](https://keyst.one/) or [Airgap Vault](https://airgap.it/offline-device/).
2. Privacy focused, does not leave any traces.
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

Just run `shekozwallet`, that's it, the wallet will look for `shekozwallet.json` config file and will load it
if the config file not found, the wallet will create a default one, you can customize it for your needs.

The wallet is interactive, it will first load and verify the config file, then follow the steps:

1. Sync your Account, from your QR Hardware wallet.
2. Ask about what operation you want to do, for example sending ERC-20 Tokens.
3. after following the steps, it will create the Unsigned Transaction as a QR and ask you to sign it with your signer.
4. Scanning the result and broadcasting the transaction to the network.

### Testing Locally

I've added a Small ERC20 token for testing, and since I'm using [foundry](https://github.com/foundry-rs/foundry) toolchain, we can spin up a local node, and deploy the contract
to it for tests.

1. Install Foundry using https://getfoundry.sh/
2. Start Anvil node

```bash
anvil --chain-id 1337 -m "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
```

3. Copy the `tests/contracts/.env.example` file to `tests/contracts/.env` and add change the `PRIVATE_KEY` env to the first private key that anvil printed.
4. Go to `tests/contracts` and run the `forge update` then `forge script` command like the following:

```bash
cd tests/contracts
forge update
forge script DeployToken --rpc-url local -vvv --broadcast
```

it should deploy the token, and also print the contract address, Note that address we will need it later.

4. In your Airgap wallet, import the following Seed:

```
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art
```

5. Compile and run the wallet:

```bash
cargo r
```

6. Sync your account, and after that, do not make anything else, just shutdown the wallet (Ctrl+C).
7. Modify `shekozwallet.json` and in the `networks.local.erc20_tokens` add the token we deployed, like the following:

```json
  "local": {
      "rpc_url": "http://localhost:8545",
      "explorer_url": "http://localhost:3000",
      "chain_id": "1337",
      "currency_symbol": "ETH",
      "erc20_tokens": [
        {
          "address": "0xbba109e735f49fb19fd9765aaa2cb79cc16c38d2",
          "name": "USDTestToken",
          "symbol": "USDT"
        }
      ]
  },
```

8. Finally, run the wallet again, your account should be already synced, so you can use it and send some test ERC20 Tokens.

### Contributing

While this my custom wallet, and I do not expect anyone else using it too, However, I would be happy to see any contribution or suggestions. So feel free to open an issue to ask about any questions.

### License

Really? Okay.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

### Disclaimer

This software is provided as is, and I am not responsible for any loss of funds, or any other damages, caused by using this software.

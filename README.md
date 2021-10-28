Simple solana program for tokenless lamport exchange between two wallets and a central lamport holder.

<img src="https://github.com/dranikpg/simple-solana-two-wallets/blob/media/sc_2.png" max-width="600px"/>

### Running the client

The client **requires** two private keys to be present in the current directory:
- `prog.id` - deployed program private key (make sure an account has been created)
- `payer.id` - some payer account (make sure it has enough lamports to run transactions)

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UncxSolanaLpLocker } from "../target/types/uncx_solana_lp_locker";
import { BN } from "bn.js";

(async () => {
  process.env.ANCHOR_PROVIDER_URL = "http://localhost:8899";
  process.env.ANCHOR_WALLET = "/home/arnau/.config/solana/id.json";
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .UncxSolanaLpLocker as Program<UncxSolanaLpLocker>;
  const pubkey = anchor.getProvider().publicKey;
  const ix = await program.methods
    .initialize(
      {
        adminKey: new anchor.web3.PublicKey(pubkey),
        devAddr: new anchor.web3.PublicKey(pubkey),
        feeConfig: {
          liquidityFee: 20,
          nativeFee: new BN(20),
          referralDiscount: 0,
          referralSharePercent: 0,
          secondaryTokenDiscount: 0,
          secondaryTokenFee: new BN(0),
        },
        minReferralBalance: new BN(0),
        nextLockerUniqueId: new BN(0),
        referralTokenAddress: null,
        secondaryTokenAddress: null,
      },
      [0]
    )
    .accounts({
      payer: new anchor.web3.PublicKey(pubkey),
    })
    .rpc();
  console.log(ix);
})();

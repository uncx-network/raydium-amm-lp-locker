import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UncxSolanaLpLocker } from "../target/types/uncx_solana_lp_locker";
import { BN } from "bn.js";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import fs from "fs";

export function loadWalletKey(keypairFile: string): anchor.web3.Keypair {
  if (!keypairFile || keypairFile == "") {
    throw new Error("Keypair is required!");
  }
  const loaded = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairFile).toString()))
  );
  return loaded;
}

(async () => {
  process.env.ANCHOR_PROVIDER_URL = "http://localhost:8899";
  process.env.ANCHOR_WALLET = "/home/arnau/.config/solana/id.json";
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .UncxSolanaLpLocker as Program<UncxSolanaLpLocker>;
  const pubkey = anchor.getProvider().publicKey;

  const airdropSignature = await anchor
    .getProvider()
    .connection.requestAirdrop(new anchor.web3.PublicKey(pubkey), 1000000000);

  await anchor.getProvider().connection.confirmTransaction(airdropSignature);

  const mint = new anchor.web3.PublicKey(
    "H5vPY967v8DkZRaZVNxDaMrHUdtovRUET8c6AXo3BirF"
  );
  const ata = new anchor.web3.PublicKey(
    "1R8BFjYJYCTgifSwTyPA7gr6HhYPsCr9HHMdXvVLGhm"
  );

  const ammInfoAcc = new anchor.web3.PublicKey(
    "9LfXeYQgTXJWhyTQhykCSnfUDd1ffCYA1LcSdcwaRLBk"
  );

  const [uncxAuthorityAcc] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("uncx_authority")],
    program.programId
  );
  
  const [userInfoLpTrackerAcc] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("user_lp_tracker"),
      new anchor.web3.PublicKey(pubkey).toBuffer(),
      new anchor.web3.PublicKey(ammInfoAcc).toBuffer(),
      new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0]),
    ],
    program.programId
  );

  const feePaymentMethod = { native: {} };

  const tx = await program.methods
    .createAndLockLp(
      pubkey,
      ammInfoAcc,
      null,
      new BN(1_000_001),
      new BN(Date.now() / 1000 + 86400),
      1,
      feePaymentMethod
    )
    .accountsPartial({
      ammInfoAcc,
      tokenProgram: TOKEN_PROGRAM_ID,
      lpMintAcc: mint,
      payer: pubkey,
      uncxAuthorityAcc,
      userLpTokenAcc: ata,
      userInfoLpTrackerAcc,
      devWallet: pubkey,
      referralWallet: null,
      referralSecondaryTokenAccount: null,
      referralTokenAccount: null,
      secondaryTokenMint: null,
      userSecondaryTokenAccount: null,
      userLpTokenAuthorityAcc: null,
      userSecondaryTokenAuthorityAcc: null,
      whitelistAddress: null,
      userWhitelistPdaAcc: null,
    })
    .rpc();
  console.log(tx);
})();

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UncxSolanaLpLocker } from "../target/types/uncx_solana_lp_locker";
import { BN } from "bn.js";
import fs from "fs";
import path from "path";
import {
  MintLayout,
  AccountLayout,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import fsp from "fs/promises";

type Account = {
  account: {
    data: [string, "base64"];
    executable: boolean;
    lamports: number;
    owner: string;
    rentEpoch: number;
    space: number;
  };
  pubkey: string;
};

(async () => {
  process.env.ANCHOR_PROVIDER_URL = "https://api.mainnet-beta.solana.com/";
  process.env.ANCHOR_WALLET = "/home/arnau/.config/solana/id.json";
  // anchor.setProvider(anchor.AnchorProvider.env());

  anchor.setProvider(anchor.AnchorProvider.env());

  const owner = anchor.getProvider().publicKey;

  const decodedData = AccountLayout.decode(
    Buffer.from(
      "7v2uQo9B21VPVpbd7x69UVq+5EkZBNl1mcc8K3e68VxZaYkF6nnfCP+0YrvTlyGwJWhrHNCEuQU/JMmX+sNA+BfaZPEXAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
      "base64"
    )
  );
  console.log({ decodedData });
  decodedData.owner = owner;
  const buf = Buffer.alloc(AccountLayout.span);
  AccountLayout.encode(
    {
      ...decodedData,
    },
    buf
  );
  console.log({
    decodedDataBase64: buf.toString("base64"),
  });
  const ata = getAssociatedTokenAddressSync(decodedData.mint, owner);
  const originalLpAtaJson = await fsp.readFile(
    path.join(__dirname, "lp_ata.json")
  );
  const originalLpAta = JSON.parse(originalLpAtaJson.toString()) as Account;
  originalLpAta.pubkey = ata.toBase58();
  originalLpAta.account.data[0] = buf.toString("base64");
  await fsp.writeFile(
    path.join(__dirname, "lp_ata_new_owner.json"),
    JSON.stringify(originalLpAta, null, 2)
  );
})();

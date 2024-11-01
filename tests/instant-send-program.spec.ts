import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token";

import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { BanksClient } from "solana-bankrun";

import { InstantSendProgram } from "../target/types/instant_send_program";
import * as fs from "fs";
import * as path from "path";
import * as crypto from "crypto";

const IDL = require("../target/idl/instant_send_program");
const programAddress = new PublicKey(
  "4khKXMz3ttSaoxuwJ6nB93SB2PSjvj3FZP4E1gCPGHKW"
);
const directory = path.join(__dirname);

function loadKeypair(filename: string): Keypair {
  const filePath = path.join(directory, `${filename}.json`);
  const secretKey = new Uint8Array(
    JSON.parse(fs.readFileSync(filePath, "utf-8"))
  );
  return Keypair.fromSecretKey(secretKey);
}
// const generateSecret = (length: number = 32): string => {
//     return crypto.randomBytes(length).toString("hex"); // Generates a hexadecimal string
// };

const generateSecret = (): string => {
  return "fixedsecret1234567890abcdef12345678"; // 32-byte hex string
};


const hashSecret = (secret: string): Buffer => {
  return crypto.createHash("sha256").update(secret, "utf8").digest();
};
describe("Instant Transfer", () => {
  const senderWallet = loadKeypair("sender");
  // const receiverWallet = loadKeypair("receiver");
  // const centralFeePayerWallet = loadKeypair("central_fee_payer_wallet");

  let context;
  let provider: BankrunProvider;
  let transferProgram: Program<InstantSendProgram>;
  let payer;
  let banksClient: BanksClient;
  let tokenMint: PublicKey;
  let senderTokenAccount: PublicKey;
  before("set Init vars", async () => {
    await provider.connection.requestAirdrop(senderWallet.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    context = await startAnchor("", [{ name: "instant_send_program", programId: programAddress }], []);
    ({ banksClient, payer } = context);

    provider = new BankrunProvider(context);

    transferProgram = new Program<InstantSendProgram>(
      IDL,
      provider,
    );
  });

  let secret;
  let hashOfSecret;
  let SEED_ESCROW_SOL;
  let SEED_ESCROW_SPL;
  before("set secret, hashed_secret and seed-string buffer", async () => {
    secret = generateSecret();
    console.log(secret);

    hashOfSecret = hashSecret(secret);
    console.log("Hash array length:", hashOfSecret.length);
    console.log("Hash array:", hashOfSecret);

    SEED_ESCROW_SOL = Buffer.from("escrow_sol");
    SEED_ESCROW_SPL = Buffer.from("escrow_spl");


  });
  //it.only
  it.skip("Initialize Transfer SOL", async () => {
    const amount = new anchor.BN(0.2 * anchor.web3.LAMPORTS_PER_SOL); // 0.2 SOL
    console.log("this is the amount: ", amount);
    const expirationTime = new anchor.BN(Math.floor(Date.now() / 1000) + 3600);
    console.log("this is the expirationTime", expirationTime);
    if (hashOfSecret.length !== 32) {
      throw new Error(
        `Hash must be exactly 32 bytes. Got ${hashOfSecret.length} bytes`
      );
    }


    const [escrowAccountPDASol, escrowAccountBumpSol] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [
          SEED_ESCROW_SOL,
          // senderWallet.publicKey.toBuffer(),
          hashOfSecret, // This should be a Buffer
        ],
        programAddress
      );
    console.log("Derived PDA in Test:", escrowAccountPDASol.toBase58());
    console.log("Derived PDA Bump:", escrowAccountBumpSol);

    const txSignature = await transferProgram.methods.initializeTransferSol(amount, expirationTime, hashOfSecret).accounts({ escrowAccount: escrowAccountPDASol } as any).rpc();

    const escrower = await transferProgram.account.escrowSolAccount.fetch(escrowAccountPDASol);

    console.log(escrower)
  });

  it("Initialize Transger SPL", async () => {
    const amount = new anchor.BN(0.2 * anchor.web3.LAMPORTS_PER_SOL); // 0.2 SOL
    console.log("this is the amount: ", amount);
    const expirationTime = new anchor.BN(Math.floor(Date.now() / 1000) + 3600);
    console.log("this is the expirationTime", expirationTime);
    if (hashOfSecret.length !== 32) {
      throw new Error(
        `Hash must be exactly 32 bytes. Got ${hashOfSecret.length} bytes`
      );
    }
    const [escrowAccountPDASpl] = await anchor.web3.PublicKey.findProgramAddressSync(
      [SEED_ESCROW_SPL, hashOfSecret],
      programAddress
    );
    console.log("Derived spl escrow account", escrowAccountPDASpl.toBase58())


  });

});

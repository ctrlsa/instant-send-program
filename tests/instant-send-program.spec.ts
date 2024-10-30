import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { InstantSendProgram } from "../target/types/instant_send_program";
import { BankrunProvider, startAnchor } from "anchor-bankrun";
import * as fs from "fs";
import * as path from "path";
import * as crypto from "crypto";

const IDL = require("../target/idl/instant_send_program");
// const programAddress = new PublicKey(
//   "4khKXMz3ttSaoxuwJ6nB93SB2PSjvj3FZP4E1gCPGHKW"
// );
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

// const hashSecret = (secret: string): Buffer => {
//     const hashHex = anchor.utils.sha256.hash(secret);
//     return Buffer.from(hashHex, 'hex'); // Convert hex string to Buffer
// };
// const hashSecret = (secret: string): number[] => {
//     const hash = crypto.createHash('sha256')
//         .update(Buffer.from(secret, 'utf8'))
//         .digest();
//     return Array.from(hash);
// };
const hashSecret = (secret: string): Buffer => {
  return crypto.createHash("sha256").update(secret, "utf8").digest();
};
describe("Instant Transfer", () => {
  const senderWallet = loadKeypair("sender");
  const receiverWallet = loadKeypair("receiver");
  const centralFeePayerWallet = loadKeypair("central_fee_payer_wallet");

  const secret = generateSecret(); // Generate a 32-byte random secret
  console.log("Generated Secret:", secret);

  const hashOfSecret = hashSecret(secret);
  console.log("Hash array length:", hashOfSecret.length);
  console.log("Hash array:", hashOfSecret);
  const SEED_ESCROW_SOL = Buffer.from("escrow_sol");
  console.log(Array.from(SEED_ESCROW_SOL));

  it("Initialize Transfer SOL", async () => {
    // const context = await startAnchor(
    //   "",
    //   [{ name: "instant_send_program", programId: programAddress }],
    //   []
    // );
    // const provider = new BankrunProvider(context);

    // const transferProgram = new Program<InstantSendProgram>(IDL, provider);
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const transferProgram = anchor.workspace.InstantSendProgram as Program<InstantSendProgram>;
    const programAddress = transferProgram.programId;
    console.log("program address: ", programAddress)
    // const expectedHashHex =
    //   "838e12ed6f077da89f4e3dcec5a9a8e8a21ed0714019176b8f95b935976acf50";
    // const computedHashHex = hashOfSecret.toString("hex");

    // console.log("Expected Hash (Hex):", expectedHashHex);
    // console.log("Computed Hash (Hex):", computedHashHex);

    // if (computedHashHex !== expectedHashHex) {
    //   throw new Error(
    //     `Hash mismatch! Expected: ${expectedHashHex}, Got: ${computedHashHex}`
    //   );
    // }

    const [escrowAccountPDA, escrowAccountBump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [
          SEED_ESCROW_SOL,
          senderWallet.publicKey.toBuffer(),
          hashOfSecret, // This should be a Buffer
        ],
        programAddress
      );
    console.log("Derived PDA in Test:", escrowAccountPDA.toBase58());
    console.log("Derived PDA Bump:", escrowAccountBump);
    const amount = new anchor.BN(0.2 * anchor.web3.LAMPORTS_PER_SOL); // 0.2 SOL
    const expirationTime = new anchor.BN(Math.floor(Date.now() / 1000) + 3600);

    if (hashOfSecret.length !== 32) {
      throw new Error(
        `Hash must be exactly 32 bytes. Got ${hashOfSecret.length} bytes`
      );
    }

    await transferProgram.methods
      .initializeTransferSol(amount, expirationTime, Array.from(hashOfSecret))
      .accounts({
        sender: senderWallet.publicKey,
        escrowAccount: escrowAccountPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      } as any)
      .signers([senderWallet])
      .rpc();
    const escrowAccount = await transferProgram.account.escrowSolAccount.fetch(
      escrowAccountPDA
    );
    console.log("Escrow Account:", escrowAccount);
  });
});

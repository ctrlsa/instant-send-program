import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, Connection } from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    createMint,
    getOrCreateAssociatedTokenAccount,
    mintTo,
    getAssociatedTokenAddressSync,
    ASSOCIATED_TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import { InstantSendProgram } from "../target/types/instant_send_program";
import * as fs from "fs";
import * as path from "path";
import * as crypto from "crypto";
import { Key } from "readline";

const IDL = require("../target/idl/instant_send_program");

const programAddress = new PublicKey("BCLTR5fuCWrMUWc75yKnG35mtrvXt6t2eLuPwCXA93oY")
const directory = path.join(__dirname);

// async function fundWalletFromDefaultWallet(provider: BankrunProvider, payerWallet: Keypair, toBeFundedWallet: Keypair, amount: number) {
async function fundWalletFromDefaultWallet(provider: anchor.AnchorProvider, payerWallet: Keypair, toBeFundedWallet: Keypair, amount: number) {
  const transaction = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
            fromPubkey: payerWallet.publicKey,
            toPubkey: toBeFundedWallet.publicKey,
            lamports: amount * anchor.web3.LAMPORTS_PER_SOL,
        })
    );

    await provider.sendAndConfirm(transaction, [payerWallet]);
    console.log("Funded senderWallet with 2 SOL");
}

function loadKeypair(filename: string): Keypair {
  const filePath = path.join(directory, `${filename}.json`);
  console.log("Trying to load keypair from:", filePath); // Log the file path
  try {
    const secretKey = new Uint8Array(
      JSON.parse(fs.readFileSync(filePath, "utf-8"))
    );
    console.log("File content loaded successfully."); // Log if file is read
    return Keypair.fromSecretKey(secretKey);
  } catch (error) {
    console.error("Error loading keypair:", error.message);
    throw error; // Re-throw to let the test fail
  }
}
// const generateSecret = (length: number = 32): string => {
//     return crypto.randomBytes(length).toString("hex"); // Generates a hexadecimal string
// };

const generateSecret = (): string => {
  return "fixedsecret1234567890sef12345678"; // 32-byte hex string
};


const hashSecret = (secret: string): Buffer => {
  return crypto.createHash("sha256").update(secret, "utf8").digest();
};

async function createMintAndTokenAccounts(
  provider: anchor.AnchorProvider,
  payer: Keypair,
  mintAuthority: Keypair,
  owner: PublicKey,
  amount: number,
  decimals: number
) {
  const mint = await createMint(
    provider.connection,
    payer,
    mintAuthority.publicKey,
    null,
    decimals
  );

  const tokenAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    payer,
    mint,
    owner
  );

  await mintTo(
    provider.connection,
    payer,
    mint,
    tokenAccount.address,
    mintAuthority,
    amount
  );

  return { mint, tokenAccount: tokenAccount.address };
}

describe("Instant Transfer", () => {
  const senderWallet = loadKeypair("sender");
  const receiverWallet = loadKeypair("receiver");
  const centralFeePayerWallet = loadKeypair("central_fee_payer_wallet");
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const connection = provider.connection;
  const wallet = provider.wallet;
  console.log(wallet.publicKey);

  const transferProgram = new Program<InstantSendProgram>(
    IDL,
    provider,
  );

  let secret;
  let hashOfSecret;
  let SEED_ESCROW_SOL;
  let SEED_ESCROW_SPL;
  let mintPubkey: PublicKey;

  before("set secret, hashed_secret and seed-string buffer", async () => {
    secret = generateSecret();
    console.log(secret);

    hashOfSecret = hashSecret(secret);
    console.log("Hash array length:", hashOfSecret.length);
    console.log("Hash array:", hashOfSecret);

    SEED_ESCROW_SOL = Buffer.from("escrow_sol");
    SEED_ESCROW_SPL = Buffer.from("escrow_spl");


  });

  it.skip("init sol transfer devnet", async () => {
    const amount = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL); // 0.1 SOL
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

    const txSignature = await transferProgram.methods.initializeTransferSol(amount, expirationTime, hashOfSecret).accounts({sender: senderWallet.publicKey, escrowAccount: escrowAccountPDASol } as any).signers([senderWallet]).rpc();

    const escrower = await transferProgram.account.escrowSolAccount.fetch(escrowAccountPDASol);
    console.log(escrower)
    
    const escrowAccountInfo = await provider.connection.getAccountInfo(escrowAccountPDASol);
    console.log('Escrow Account Lamports:', escrowAccountInfo.lamports);

  });
  it.skip("Redeem funds from Escrow wallet Native SOL", async() => {
    const [escrowAccountPDASol, escrowAccountBumpSol] = await anchor.web3.PublicKey.findProgramAddressSync(
        [SEED_ESCROW_SOL, hashOfSecret],
        programAddress
    );
    const escrowAccountInfoBefore = await provider.connection.getAccountInfo(escrowAccountPDASol);
    const senderAccountInfoBefore = await provider.connection.getAccountInfo(senderWallet.publicKey);
    //const receiverAccountInfoBefore = await provider.connection.getAccountInfo(receiverWallet.publicKey);

    console.log('Escrow Account Balance Before Redemption (Lamports):', escrowAccountInfoBefore?.lamports);
    console.log('Sender Account Balance Before Redemption (Lamports):', senderAccountInfoBefore?.lamports);
    //console.log('Receiver Account Balance Before Redemption (Lamports):', receiverAccountInfoBefore?.lamports);

    const txSignature = await transferProgram.methods.redeemFundsSol(secret).accounts({signer: centralFeePayerWallet.publicKey, sender: senderWallet.publicKey, recipient: receiverWallet.publicKey, escrowAccount: escrowAccountPDASol}).signers([centralFeePayerWallet]).rpc();
    
    // Fetch and print balances after redemption
    //const escrowAccountInfoAfter = await provider.connection.getAccountInfo(escrowAccountPDASol);
    const senderAccountInfoAfter = await provider.connection.getAccountInfo(senderWallet.publicKey);
    const receiverAccountInfoAfter = await provider.connection.getAccountInfo(receiverWallet.publicKey);

    //console.log('Escrow Account Balance After Redemption (Lamports):', escrowAccountInfoAfter?.lamports);
    console.log('Sender Account Balance After Redemption (Lamports):', senderAccountInfoAfter?.lamports);
    console.log('Receiver Account Balance After Redemption (Lamports):', receiverAccountInfoAfter?.lamports);

  });

    it("Initialize SPL Token Transfer", async () => {
      const { mint, tokenAccount: senderTokenAccount } = await createMintAndTokenAccounts(
        provider,
        senderWallet,
        senderWallet,
        senderWallet.publicKey,
        100,
        0
      );
      mintPubkey = mint;
      const amount = new anchor.BN(50); // Changed from 0.2 * LAMPORTS_PER_SOL to just 50 tokens
      console.log("this is the amount: ", amount);
      const expirationTime = new anchor.BN(Math.floor(Date.now() / 1000) + 3600);
      console.log("this is the expirationTime", expirationTime);
      if (hashOfSecret.length !== 32) {
        throw new Error(
          `Hash must be exactly 32 bytes. Got ${hashOfSecret.length} bytes`
        );
      }
  
      // Derive the escrow account PDA
      const [escrowAccountPDASpl, escrowAccountBump] = PublicKey.findProgramAddressSync(
        [SEED_ESCROW_SPL, hashOfSecret],
        programAddress
      );
  
      // Get escrow token account (ATA)
      const escrowTokenAccount = getAssociatedTokenAddressSync(
        mintPubkey,
        escrowAccountPDASpl,
        true // Allow PDA owner
      );
      console.log("Derived spl escrow account", escrowAccountPDASpl.toBase58())
      const txSignature = await transferProgram.methods.initializeTransferSpl(amount, expirationTime, hashOfSecret).accounts({
          sender: senderWallet.publicKey,
          escrowAccount: escrowAccountPDASpl,
          tokenMint: mintPubkey,
          senderTokenAccount: senderTokenAccount,
          escrowTokenAccount: escrowTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
      } as any).signers([senderWallet]).rpc();
      console.log("Transaction Signature: ", txSignature);

      // Fetch and log escrow account state
      const escrowAccount = await transferProgram.account.escrowAccount.fetch(escrowAccountPDASpl);
      console.log("Escrow Account State:", escrowAccount);
    });

    it("Redeem SPL Token Transfer", async () => {
        // Get the escrow PDA and token accounts
        const [escrowAccountPDASpl] = PublicKey.findProgramAddressSync(
            [SEED_ESCROW_SPL, hashOfSecret],
            programAddress
        );

        // Get the recipient's token account
        const recipientTokenAccount = getAssociatedTokenAddressSync(
            mintPubkey,  // Use the same mint from the initialize test
            receiverWallet.publicKey
        );

        // Get the escrow token account
        const escrowTokenAccount = getAssociatedTokenAddressSync(
            mintPubkey,
            escrowAccountPDASpl,
            true // Allow PDA owner
        );

        const txSignature = await transferProgram.methods
            .redeemFundsSpl(secret)
            .accounts({
                signer: centralFeePayerWallet.publicKey,
                recipient: receiverWallet.publicKey,
                escrowAccount: escrowAccountPDASpl,
                escrowTokenAccount: escrowTokenAccount,
                recipientTokenAccount: recipientTokenAccount,
                tokenMint: mintPubkey,
                sender: senderWallet.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: anchor.web3.SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            }as any)
            .signers([centralFeePayerWallet])
            .rpc();

        console.log("Redeem Transaction Signature:", txSignature);
    });

});
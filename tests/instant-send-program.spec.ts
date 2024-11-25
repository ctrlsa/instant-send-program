import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, Connection } from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    MintLayout,
    createInitializeMintInstruction,
    getAssociatedTokenAddressSync,
    createAssociatedTokenAccountInstruction,
    createMintToInstruction,
  } from "@solana/spl-token";

import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { BanksClient } from "solana-bankrun";

import { InstantSendProgram } from "../target/types/instant_send_program";
import * as fs from "fs";
import * as path from "path";
import * as crypto from "crypto";
import { Key } from "readline";

const IDL = require("../target/idl/instant_send_program");
// const programAddress = new PublicKey(
//   "4khKXMz3ttSaoxuwJ6nB93SB2PSjvj3FZP4E1gCPGHKW"
// );

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

// function loadKeypair(filename: string): Keypair {
//   const filePath = path.join(directory, `${filename}.json`);
//   console.log(filePath)
//   const secretKey = new Uint8Array(
//     JSON.parse(fs.readFileSync(filePath, "utf-8"))
//   );
//   return Keypair.fromSecretKey(secretKey);
// }
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
  return "fixedsecret1234567890abcdef12345678"; // 32-byte hex string
};


const hashSecret = (secret: string): Buffer => {
  return crypto.createHash("sha256").update(secret, "utf8").digest();
};
describe("Instant Transfer", () => {
  const senderWallet = loadKeypair("sender");
  const receiverWallet = loadKeypair("receiver");
  const centralFeePayerWallet = loadKeypair("central_fee_payer_wallet");
  
  let context;
  let provider: anchor.AnchorProvider;
  let transferProgram: Program<InstantSendProgram>;
  let payer;
  //let banksClient: BanksClient;
  let tokenMint: PublicKey;
  let senderTokenAccount: PublicKey;
  let defaultWallet: Keypair;
  before("set Init vars", async () => {
    
    // context = await startAnchor("", [{ name: "instant_send_program", programId: programAddress }], []);
    // ({ banksClient, payer } = context);
    // defaultWallet = payer;
    // provider = new BankrunProvider(context);
    // const connection = new Connection(anchor.web3.clusterApiUrl("devnet"), "confirmed");
    // const wallet = anchor.AnchorProvider.local().wallet;

    // const provider = new anchor.AnchorProvider(connection, wallet, {});
    provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const connection = provider.connection;
    const wallet = provider.wallet;

    // Use the payer from the provider
    payer = wallet;

    
    await fundWalletFromDefaultWallet(provider, defaultWallet, senderWallet, 2);
    await fundWalletFromDefaultWallet(provider, defaultWallet, centralFeePayerWallet, 1);

    const mintAuthority = Keypair.generate();
    const freezeAuthority = null;
    const decimals = 9;

    const mintKeypair = Keypair.generate();
    const lamportsForMint = await provider.connection.getMinimumBalanceForRentExemption(MintLayout.span);

    const mintTransaction = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: mintKeypair.publicKey,
            space: MintLayout.span,
            lamports: lamportsForMint,
            programId: TOKEN_PROGRAM_ID,
        }),
        createInitializeMintInstruction(
            mintKeypair.publicKey,
            decimals,
            mintAuthority.publicKey,
            freezeAuthority,
            TOKEN_PROGRAM_ID,
        )
    );

    await provider.sendAndConfirm(mintTransaction, [payer, mintKeypair])

    senderTokenAccount = getAssociatedTokenAddressSync(
        mintKeypair.publicKey,
        //defaultWallet.publicKey
        senderWallet.publicKey
    )

    const ataTransaction = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
            payer.publicKey,
            senderTokenAccount,
            //defaultWallet.publicKey,
            senderWallet.publicKey,
            mintKeypair.publicKey,
            TOKEN_PROGRAM_ID
        )
    );

    await provider.sendAndConfirm(ataTransaction, [payer])
 
    const mintToTransaction = new anchor.web3.Transaction().add(
        createMintToInstruction(
        mintKeypair.publicKey,
        senderTokenAccount,
        mintAuthority.publicKey,
        1_000_000_000, // Amount to mint
        [],
        TOKEN_PROGRAM_ID
        )
    );

    await provider.sendAndConfirm(mintToTransaction, [payer, mintAuthority]);

    
    tokenMint = mintKeypair.publicKey;

    //console.log("this is the tokenMint address", tokenMint)



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

  it.skip("Initialize Transger SPL", async () => {
    const amount = new anchor.BN(0.2 * anchor.web3.LAMPORTS_PER_SOL); // 0.2 SOL
    console.log("this is the amount: ", amount);
    const expirationTime = new anchor.BN(Math.floor(Date.now() / 1000) + 3600);
    console.log("this is the expirationTime", expirationTime);
    if (hashOfSecret.length !== 32) {
      throw new Error(
        `Hash must be exactly 32 bytes. Got ${hashOfSecret.length} bytes`
      );
    }
    const [escrowAccountPDASpl, escrowAccountBumpSpl] = await anchor.web3.PublicKey.findProgramAddressSync(
      [SEED_ESCROW_SPL, hashOfSecret],
      programAddress
    );
    console.log("Derived spl escrow account", escrowAccountPDASpl.toBase58())
    const txSignature = await transferProgram.methods.initializeTransferSpl(amount, expirationTime, hashOfSecret).accounts({
        sender: senderWallet.publicKey,
        escrowAccount: escrowAccountPDASpl,
        tokenMint: tokenMint,
        senderTokenAccount: senderTokenAccount,
    } as any).signers([senderWallet]).rpc();

    console.log("Transaction Signature: ", txSignature);

    // Fetch and log escrow account state
    const escrowAccount = await transferProgram.account.escrowAccount.fetch(escrowAccountPDASpl);
    console.log("Escrow Account State:", escrowAccount);

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

  it.skip("Redeem funds from Escrow Wallet SPL", async() => {
    const [escrowAccountPDASpl, escrowAccountBumpSpl] = await anchor.web3.PublicKey.findProgramAddressSync(
        [SEED_ESCROW_SPL, hashOfSecret],
        programAddress
    );

    const recipientTokenAccount = getAssociatedTokenAddressSync(
        tokenMint,
        receiverWallet.publicKey
    );

    const escrowTokenAccount = getAssociatedTokenAddressSync(
        tokenMint,
        escrowAccountPDASpl,
        true // This specifies that the escrow PDA is a program-derived address
    );

    // const escrowAccountInfoBefore = await provider.connection.getAccountInfo(escrowAccountPDASpl);
    // const senderAccountInfoBefore = await provider.connection.getAccountInfo(senderWallet.publicKey);
    // const centralFeePayerWalletInfo = await provider.connection.getAccountInfo(centralFeePayerWallet.publicKey)

    // console.log('Escrow Account Balance Before Redemption (Lamports):', escrowAccountInfoBefore?.lamports);
    // console.log('Sender Account Balance Before Redemption (Lamports):', senderAccountInfoBefore?.lamports);
    // console.log('centralFeePayerWalletInfo Account Balance Before Redemption (Lamports):', centralFeePayerWalletInfo?.lamports);

    // console.log('Signer Public Key:', centralFeePayerWallet.publicKey.toBase58());
    // console.log('Recipient Public Key:', receiverWallet.publicKey.toBase58());
    // console.log('Sender Public Key:', senderWallet.publicKey.toBase58());
    // console.log('Escrow Account PDA:', escrowAccountPDASpl.toBase58());
    // console.log('Escrow Token Account:', escrowTokenAccount.toBase58());
    // console.log('Recipient Token Account:', recipientTokenAccount.toBase58());
    
    // Execute the redemption function and get the transaction signature
    const txSignature = await transferProgram.methods
        .redeemFundsSpl(secret)
        .accounts({
            signer: centralFeePayerWallet.publicKey,
            recipient: receiverWallet.publicKey,
            sender: senderWallet.publicKey,
            escrowAccount: escrowAccountPDASpl,
            escrowTokenAccount: escrowTokenAccount,
            recipientTokenAccount: recipientTokenAccount,
            tokenMint: tokenMint,
        }as any)
        .signers([centralFeePayerWallet])
        .rpc();

    console.log("Transaction Signature:", txSignature);

    // // Fetch and print balances after redemption
    // const finalRecipientSolBalance = await provider.connection.getBalance(receiverWallet.publicKey);
    // console.log("Recipient SOL Balance After Redemption (Lamports):", finalRecipientSolBalance);

    // const finalRecipientTokenBalance = await provider.connection.getTokenAccountBalance(recipientTokenAccount);
    // console.log("Recipient SPL Token Balance After Redemption:", finalRecipientTokenBalance.value.amount);
});

});

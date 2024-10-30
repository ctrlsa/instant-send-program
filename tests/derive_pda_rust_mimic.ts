//file: derive_pda_rust_mimic.ts

import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import * as crypto from "crypto";

const SEED_ESCROW_SOL = Buffer.from("escrow_sol");
const PROGRAM_ID = new PublicKey("4khKXMz3ttSaoxuwJ6nB93SB2PSjvj3FZP4E1gCPGHKW");
const SENDER_PUBLIC_KEY = new PublicKey("FPvKC5okno6gu6bTXoTh9W4vzR9CCwZhqdcmuuP6XTCT");

const SECRET = "fixedsecret1234567890abcdef12345678";

// Hash Function
const hashSecret = (secret: string): Buffer => {
    return crypto.createHash('sha256')
        .update(secret, 'utf8')
        .digest(); // Returns a Buffer of 32 bytes
};

// Main Function
const derivePDA_RustMimic = () => {
    console.log("=== Deriving PDA Mimicking Rust Program ===");

    console.log("Fixed Secret:", SECRET);

    const hashOfSecret = hashSecret(SECRET);
    console.log("Hash Buffer Length:", hashOfSecret.length);
    console.log("Hash Buffer:", hashOfSecret);
    console.log("Hash Hex:", hashOfSecret.toString('hex'));

    // Derive PDA
    const [pda, bump] = PublicKey.findProgramAddressSync(
        [
            SEED_ESCROW_SOL,
            SENDER_PUBLIC_KEY.toBuffer(),
            hashOfSecret, // Buffer
        ],
        PROGRAM_ID
    );

    console.log("Derived PDA (Rust Mimic):", pda.toBase58());
    console.log("Derived PDA Bump (Rust Mimic):", bump);
};

derivePDA_RustMimic();


import {
  Keypair,
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";

function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
//const directory = path.join(__dirname, 'tests');
const directory = path.join(__dirname);

const connection = new Connection("https://api.devnet.solana.com", "confirmed");

if (!fs.existsSync(directory)) {
  fs.mkdirSync(directory, { recursive: true });
}

function saveKeypair(keypair: Keypair, filename: string) {
  const filePath = path.join(directory, `${filename}.json`);
  fs.writeFileSync(filePath, JSON.stringify(Array.from(keypair.secretKey)));
  console.log(`Wallet saved: ${filePath}`);
}

async function airdropSol(publicKey: PublicKey, amountSol: number) {
  const airdropSignature = await connection.requestAirdrop(
    publicKey,
    amountSol * LAMPORTS_PER_SOL
  );

  const confirmationStrategy = {
    signature: airdropSignature,
    blockhash: (await connection.getLatestBlockhash()).blockhash,
    lastValidBlockHeight: (await connection.getLatestBlockhash())
      .lastValidBlockHeight,
  };

  await connection.confirmTransaction(confirmationStrategy);
  console.log(`Airdropped ${amountSol} SOL to ${publicKey.toBase58()}`);
}
function loadKeypair(filename: string): Keypair {
  const filePath = path.join(directory, `${filename}.json`);
  const secretKey = new Uint8Array(
    JSON.parse(fs.readFileSync(filePath, "utf-8"))
  );
  return Keypair.fromSecretKey(secretKey);
}

// const senderWallet = Keypair.generate();
// const receiverWallet = Keypair.generate();
// const centralFeePayerWallet = Keypair.generate();

// saveKeypair(senderWallet, 'sender');
// saveKeypair(receiverWallet, 'receiver');
// saveKeypair(centralFeePayerWallet, 'central_fee_payer_wallet');

const senderWallet = loadKeypair("sender");
const receiverWallet = loadKeypair("receiver");
const centralFeePayerWallet = loadKeypair("central_fee_payer_wallet");

(async () => {
  // await airdropSol(senderWallet.publicKey, 5);
  // await delay(10000);
  await airdropSol(centralFeePayerWallet.publicKey, 1);
})();

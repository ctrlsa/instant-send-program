import * as crypto from "crypto";

const generateSecret = (length: number = 32): string => {
  return crypto.randomBytes(length).toString("hex");
};

const hashSecret = (secret: string): string => {
  const hash = crypto.createHash("sha256");
  hash.update(secret);
  return hash.digest("hex");
};

const secret = generateSecret();
console.log("Generated Secret:", secret);

const hashedSecret = hashSecret(secret);
console.log("Hashed Secret:", hashedSecret);

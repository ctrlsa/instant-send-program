import * as crypto from "crypto";

// Step 1: Generate a secure random secret
const generateSecret = (length: number = 32): string => {
    return crypto.randomBytes(length).toString("hex"); // Generates a hexadecimal string
};

// Step 2: Hash the secret using SHA-256
const hashSecret = (secret: string): string => {
    const hash = crypto.createHash("sha256"); // SHA-256 hashing algorithm
    hash.update(secret);
    return hash.digest("hex"); // Returns the hashed secret as a hexadecimal string
};

// Usage
const secret = generateSecret(); // Generate a 32-byte random secret
console.log("Generated Secret:", secret);

const hashedSecret = hashSecret(secret); // Hash the generated secret
console.log("Hashed Secret:", hashedSecret);

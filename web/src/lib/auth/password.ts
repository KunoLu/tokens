import { timingSafeEqual } from "node:crypto";

/**
 * Password hashing for email accounts.
 *
 * bcrypt and argon2 are unavailable on Workers (no native addons), so this is
 * WebCrypto PBKDF2-HMAC-SHA256 at the OWASP-recommended 210,000 iterations
 * with a per-user 16-byte random salt.
 *
 * Stored format: pbkdf2$sha256$210000$<salt_b64>$<hash_b64> (~96 chars,
 * fits varchar(255) with room for a future parameter bump).
 */
const ALGORITHM = "pbkdf2";
const DIGEST = "SHA-256";
const ITERATIONS = 210_000;
const KEY_LENGTH_BITS = 256;
const SALT_BYTES = 16;

async function deriveKey(plain: string, salt: Uint8Array, iterations: number): Promise<Buffer> {
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(plain),
    "PBKDF2",
    false,
    ["deriveBits"]
  );
  const bits = await crypto.subtle.deriveBits(
    {
      name: "PBKDF2",
      hash: DIGEST,
      salt: salt as unknown as BufferSource,
      iterations,
    },
    key,
    KEY_LENGTH_BITS
  );
  return Buffer.from(bits);
}

export async function hashPassword(plain: string): Promise<string> {
  const salt = crypto.getRandomValues(new Uint8Array(SALT_BYTES));
  const hash = await deriveKey(plain, salt, ITERATIONS);
  return [
    ALGORITHM,
    "sha256",
    String(ITERATIONS),
    Buffer.from(salt).toString("base64"),
    hash.toString("base64"),
  ].join("$");
}

/**
 * Verify a plaintext password against a stored hash. Unknown or malformed
 * stored formats fail closed (false) rather than throwing.
 */
export async function verifyPassword(plain: string, stored: string): Promise<boolean> {
  const parts = stored.split("$");
  if (parts.length !== 5 || parts[0] !== ALGORITHM || parts[1] !== "sha256") {
    return false;
  }
  // Strict parse: accept exactly the parameters this deployment writes.
  // parseInt would silently take "210000junk" or "0x33...e"; Buffer.from
  // silently drops invalid base64. Either would let a tampered row verify
  // against weaker parameters, so fail closed on anything unexpected.
  if (!/^[0-9]+$/.test(parts[2]) || Number(parts[2]) !== ITERATIONS) {
    return false;
  }
  const salt = Buffer.from(parts[3], "base64");
  const expected = Buffer.from(parts[4], "base64");
  if (salt.length !== SALT_BYTES || expected.length !== KEY_LENGTH_BITS / 8) {
    return false;
  }
  const actual = await deriveKey(plain, new Uint8Array(salt), ITERATIONS);
  return timingSafeEqual(actual, expected);
}

export interface PasswordValidation {
  error?: string;
  details?: string[];
}

const UPPERCASE = /[A-Z]/;
const LOWERCASE = /[a-z]/;
// Anything that is not a letter or digit counts as a special character.
const SPECIAL = /[^A-Za-z0-9]/;

/**
 * The one password policy, shared by registration and reset: at least 8
 * characters, with an uppercase letter, a lowercase letter, and a special
 * character. Returns {} on success, { error, details } on failure.
 */
export function validatePassword(plain: string): PasswordValidation {
  const details: string[] = [];
  if (plain.length < 8) {
    details.push("At least 8 characters");
  }
  if (!UPPERCASE.test(plain)) {
    details.push("At least one uppercase letter");
  }
  if (!LOWERCASE.test(plain)) {
    details.push("At least one lowercase letter");
  }
  if (!SPECIAL.test(plain)) {
    details.push("At least one special character");
  }
  if (details.length > 0) {
    return {
      error: "Password does not meet the requirements",
      details,
    };
  }
  return {};
}

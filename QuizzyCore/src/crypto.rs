use sha2::{Sha256, Digest};
use rand::Rng;

/// Derive a 32-byte key from a seed string and timestamp.
/// => SHA-256(seed + ":" + timestamp)
pub fn derive_key(seed: &str, timestamp: u64) -> [u8; 32] {
    let payload = format!("{}:{}", seed, timestamp);
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Byte-level XOR cipher. Each byte is XOR'd with key[i % 32].
/// Since XOR is its own inverse, the same operation encrypts and decrypts.
pub fn xor_cipher(input: &[u8], key: &[u8; 32]) -> Vec<u8> {
    input
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % 32])
        .collect()
}

/// Interleave two strings character by character.
/// Both strings MUST have the same length (or deinterleave won't be perfect).
pub fn interleave(a: &str, b: &str) -> String {
    assert_eq!(a.len(), b.len(), "interleave requires equal-length strings");
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut result = String::with_capacity(a.len() * 2);
    for i in 0..a_chars.len() {
        result.push(a_chars[i]);
        result.push(b_chars[i]);
    }
    result
}

/// De-interleave: even indices → a, odd indices → b.
pub fn simple_deinterleave(combined: &str) -> (String, String) {
    let mut a = String::new();
    let mut b = String::new();
    for (i, c) in combined.chars().enumerate() {
        if i % 2 == 0 {
            a.push(c);
        } else {
            b.push(c);
        }
    }
    (a, b)
}

/// Generate a random salt string from the base64 alphabet.
pub fn generate_salt(length: usize) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect()
}

/// SHA-256 → hex string.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let k1 = derive_key("test", 42);
        let k2 = derive_key("test", 42);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cipher_roundtrip() {
        let key = derive_key("secret", 100);
        let plain = b"Hello, world! This is a test with unicode: \xe4\xb8\x96\xe7\x95\x8c!";
        let ciphered = xor_cipher(plain, &key);
        let decrypted = xor_cipher(&ciphered, &key);
        assert_eq!(plain.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_interleave_roundtrip() {
        let a = "abcdefgh";
        let b = "12345678";
        let combined = interleave(a, b);
        let (a2, b2) = simple_deinterleave(&combined);
        assert_eq!(a, a2);
        assert_eq!(b, b2);
    }

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex("hello");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

use crate::errors::EncryptionError;
use aes::{
    cipher::{block_padding::Pkcs7, BlockDecryptMut},
    Aes256,
};
use cbc::{
    cipher::{BlockEncryptMut, KeyIvInit},
    Decryptor, Encryptor,
};
use hex;
use rand::{thread_rng as generate_random_number, Rng};

// IV length is always 16 bytes irrespective of the key size
const IV_LENGTH: usize = 16;

pub fn encrypt_connection_string(plain: &str, key: &[u8]) -> Result<String, EncryptionError> {
    // Check if the key length is valid i.e 32 bytes for AES-256
    if key.len() != 32 {
        return Err(EncryptionError::InvalidKeyLength);
    }
    // prepare the IV
    let mut rng = generate_random_number();
    let mut iv = [0u8; IV_LENGTH];
    rng.fill(&mut iv);

    // encrypt the plain text
    let cipher = Encryptor::<Aes256>::new_from_slices(key, &iv)
        .map_err(|_| EncryptionError::CipherCreationFailed)?;

    let plaintext = plain.as_bytes();
    let plaintext_len = plaintext.len();
    let mut buf = vec![0u8; plaintext_len + IV_LENGTH];
    buf[..plaintext_len].copy_from_slice(plaintext);
    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext_len)
        .map_err(|_| EncryptionError::EncryptionFailed)?;

    let ciphertext_len = ciphertext.len();
    // combine IV and ciphertext to create the encrypted raw data
    let mut result = Vec::with_capacity(iv.len() + ciphertext_len);
    result.extend_from_slice(&iv);
    result.extend_from_slice(&buf[..ciphertext_len]);
    // return the hex encoded value of the raw encrypted data
    Ok(hex::encode(result))
}

pub fn decrypt_connection_string(
    ciphertext_hex: &str,
    key: &[u8],
) -> Result<String, EncryptionError> {
    // Ensure the key is the correct length for AES-256 (32 bytes)
    if key.len() != 32 {
        return Err(EncryptionError::InvalidKeyLength);
    }

    // Decode the hexadecimal ciphertext
    let ciphertext = hex::decode(ciphertext_hex).map_err(|_| EncryptionError::InvalidHex)?;

    // Ensure the ciphertext is long enough to contain the IV
    if ciphertext.len() < 16 {
        return Err(EncryptionError::InvalidCiphertext);
    }

    // Split the IV from the ciphertext
    let (iv, ciphertext) = ciphertext.split_at(16);

    // Create the cipher
    let cipher = Decryptor::<Aes256>::new_from_slices(key, iv)
        .map_err(|_| EncryptionError::DecryptionFailed)?;

    // Create a buffer for the plaintext
    let mut buffer = ciphertext.to_vec();

    // Decrypt the data
    let plaintext_len = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| EncryptionError::DecryptionFailed)?
        .len();

    // Convert the decrypted bytes to a string
    String::from_utf8(buffer[..plaintext_len].to_vec()).map_err(|_| EncryptionError::InvalidUtf8)
}

use std::env;

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{bail, Result};

pub fn identity(data: Vec<u8>) -> Result<Vec<u8>> {
    Ok(data)
}

fn get_key() -> Result<Key<Aes256Gcm>> {
    let key_from_env = env::var("AES_GCM_KEY")?;

    let mut key_bytes = [0u8; 32];
    for (i, &b) in key_from_env.as_bytes().iter().enumerate().take(32) {
        key_bytes[i] = b;
    }

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

    Ok(*key)
}

pub fn encrypt(data: Vec<u8>) -> Result<Vec<u8>> {
    let key = get_key()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = match cipher.encrypt(&nonce, data.as_ref()) {
        Ok(ciphertext) => ciphertext,
        Err(err) => bail!("encryption failed: {:?}", err),
    };

    Ok([nonce.as_slice(), &ciphertext].concat())
}

pub fn decrypt(mut data: Vec<u8>) -> Result<Vec<u8>> {
    let key = get_key()?;
    let cipher = Aes256Gcm::new(&key);

    let nonce_size = 12;
    if data.len() < nonce_size {
        bail!("data too short for nonce");
    }

    let (nonce_bytes, ciphertext) = data.split_at_mut(nonce_size);
    let nonce = Nonce::from_slice(nonce_bytes);

    match cipher.decrypt(nonce, ciphertext.as_ref()) {
        Ok(plaintext) => Ok(plaintext),
        Err(err) => bail!("decryption failed: {:?}", err),
    }
}

use std::{fs, io::Read, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
// use chacha20poly1305::{
//     aead::{Aead, AeadCore, KeyInit},
//     ChaCha20Poly1305, Nonce, XChaCha20Poly1305,
// };
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use super::{process_genpass, GenPassOpt};
use crate::{cli::TextSignFormat, get_reader};

pub trait TextSign {
    // &dyn Read 动态分发，代码体积会小点，但效率比静态分发要低一些，但在业务上相比于IO来说不足一提
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    // impl Read 静态分发，代码体积会大点，但效率比静态分发要高一些，但在业务上相比于IO来说不足一提
    // impl Read 等价于 verify<R: Read>(r: R)
    // impl Read 属于是 owned 的，声明时不需要mut，但使用者实现的时候需要声明mut
    fn verify(&self, reader: impl Read, sig: &[u8]) -> Result<bool>;
}

pub trait KeyLoad {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> Result<String> {
    let mut reader = get_reader(input)?;
    let singed = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
    };
    let signed = URL_SAFE_NO_PAD.encode(singed);
    Ok(signed)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    sig: &str,
) -> Result<bool> {
    let mut reader = get_reader(input)?;
    let sig = URL_SAFE_NO_PAD.decode(sig)?;
    let verified = match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
    };
    Ok(verified)
}

pub fn process_text_generate(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

pub fn process_encrypt(_input: &str, _key: &str, _nonce: &str) -> Result<String> {
    todo!()
}
pub fn process_decrypt(_input: &str, _key: &str, _nonce: &str) -> Result<Vec<u8>> {
    todo!()
}

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        // TODO: improve perf by reading in chunks
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf);
        let hash = hash.as_bytes();
        Ok(hash == sig)
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = self.key.sign(&buf);
        Ok(sig.to_bytes().to_vec())
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = (&sig[..64]).try_into()?;
        let sig = Signature::from_bytes(sig);
        Ok(self.key.verify(&buf, &sig).is_ok())
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key = key.try_into()?;
        Ok(Self::new(key))
    }
}

impl KeyLoad for Blake3 {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl Ed25519Signer {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = (&key[..32]).try_into()?;
        Ok(Self::new(SigningKey::from_bytes(key)))
    }
}

impl KeyLoad for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(key)
    }
}

impl Ed25519Verifier {
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = (&key[..32]).try_into()?;
        Ok(Self::new(VerifyingKey::from_bytes(key)?))
    }
}

impl KeyLoad for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_genpass(GenPassOpt {
            length: 32,
            uppercase: true,
            lowercase: true,
            number: true,
            symbol: true,
        })?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let sk: SigningKey = SigningKey::generate(&mut csprng);
        // pk => public key
        let pk: VerifyingKey = (&sk).into();
        let pk = pk.as_bytes().to_vec();
        let sk = sk.to_bytes().to_vec();
        Ok(vec![pk, sk])
    }
}

// pub struct Chacha20Poly1305 {}

// impl KeyGenerator for ChaCha20Poly1305 {
//     fn generate() -> Result<Vec<Vec<u8>>> {
//         let key = XChaCha20Poly1305::generate_key(&mut OsRng);
//         let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 192-bits; unique per message
//         Ok(vec![key.to_vec(), nonce.to_vec()])
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    use chacha20poly1305::{
        aead::{Aead, AeadCore, KeyInit, OsRng},
        XChaCha20Poly1305,
    };

    #[test]
    fn test_blake3_sign_verify() {
        let blake3 = Blake3::load("fixtures/blake3.txt").unwrap();
        let data = b"hello world";
        let sig = blake3.sign(&mut &data[..]).unwrap();
        assert!(blake3.verify(&mut &data[..], &sig).is_ok())
    }

    #[test]
    fn test_ed25519_sign_verify() {
        // sk 私钥，pk 公钥
        let signer = Ed25519Signer::load("fixtures/ed25519.sk").unwrap();
        let verifier = Ed25519Verifier::load("fixtures/ed25519.pk").unwrap();
        let data = b"hello world";
        let sig = signer.sign(&mut &data[..]).unwrap();
        assert!(verifier.verify(&mut &data[..], &sig).is_ok());

        let sig = URL_SAFE_NO_PAD.decode(b"3yUpPAwTtvmO_Xy3fSkGQEycQTCpWjEbCMJKxWi4fjNR76AZHYlbLb72OemjHXU5woLGEQPr24-1vmaLNDxgCQ").unwrap();

        let data = b"hello!";
        assert!(verifier.verify(&mut &data[..], &sig).is_ok());
    }

    #[test]
    fn test_chacha20poly1035() {
        let key = XChaCha20Poly1305::generate_key(&mut OsRng);
        let cipher = XChaCha20Poly1305::new(&key);
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 192-bits; unique per message
        let ciphertext = cipher
            .encrypt(&nonce, b"plaintext message".as_ref())
            .unwrap();

        let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();
        assert_eq!(&plaintext, b"plaintext message");
    }
}

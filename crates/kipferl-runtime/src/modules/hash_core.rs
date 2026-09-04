use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

#[derive(Clone, Copy)]
pub(super) enum Algorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl Algorithm {
    pub(super) fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().replace('-', "").as_str() {
            "md5" => Some(Self::Md5),
            "sha1" => Some(Self::Sha1),
            "sha256" => Some(Self::Sha256),
            "sha512" => Some(Self::Sha512),
            _ => None,
        }
    }

    pub(super) const fn digest_size(self) -> usize {
        match self {
            Self::Md5 => 16,
            Self::Sha1 => 20,
            Self::Sha256 => 32,
            Self::Sha512 => 64,
        }
    }

    pub(super) const fn block_size(self) -> usize {
        match self {
            Self::Sha512 => 128,
            Self::Md5 | Self::Sha1 | Self::Sha256 => 64,
        }
    }
}

pub(super) fn digest(algorithm: Algorithm, input: &[u8]) -> Vec<u8> {
    match algorithm {
        Algorithm::Md5 => Md5::digest(input).to_vec(),
        Algorithm::Sha1 => Sha1::digest(input).to_vec(),
        Algorithm::Sha256 => Sha256::digest(input).to_vec(),
        Algorithm::Sha512 => Sha512::digest(input).to_vec(),
    }
}

pub(super) fn hmac(algorithm: Algorithm, key: &[u8], message: &[u8]) -> Vec<u8> {
    let block_size = algorithm.block_size();
    let mut normalized_key = if key.len() > block_size {
        digest(algorithm, key)
    } else {
        key.to_vec()
    };
    normalized_key.resize(block_size, 0);

    let mut inner = Vec::with_capacity(block_size.saturating_add(message.len()));
    let mut outer = Vec::with_capacity(block_size.saturating_add(algorithm.digest_size()));
    for byte in normalized_key {
        inner.push(byte ^ 0x36);
        outer.push(byte ^ 0x5c);
    }
    inner.extend_from_slice(message);
    outer.extend_from_slice(&digest(algorithm, &inner));
    digest(algorithm, &outer)
}

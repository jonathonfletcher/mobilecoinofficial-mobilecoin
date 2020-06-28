// Copyright (c) 2018-2020 MobileCoin Inc.

// TODO: Would be nice to use serde_json instead of mc_util_serial but it doesn't
// work with Ristretto keys at time of writing

pub mod config;
pub mod keygen;

use mc_transaction_core::account_keys::PublicAddress;
use mc_transaction_std::identity::RootIdentity;
use std::{fs::File, io::prelude::*, path::Path};

/// Write user root identity to disk
pub fn write_keyfile<P: AsRef<Path>>(
    path: P,
    root_id: &RootIdentity,
) -> Result<(), std::io::Error> {
    File::create(path)?.write_all(&serde_json::to_vec(root_id).map_err(to_io_error)?)?;
    Ok(())
}

/// Read user root identity from disk
pub fn read_keyfile<P: AsRef<Path>>(path: P) -> Result<RootIdentity, std::io::Error> {
    read_keyfile_data(&mut File::open(path)?)
}

/// Read user root identity from any implementor of `Read`
pub fn read_keyfile_data<R: std::io::Read>(buffer: &mut R) -> Result<RootIdentity, std::io::Error> {
    let data = {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;
        data
    };
    let result: RootIdentity = serde_json::from_slice(&data).map_err(to_io_error)?;
    Ok(result)
}

/// Write user public address to disk
pub fn write_pubfile<P: AsRef<Path>>(path: P, addr: &PublicAddress) -> Result<(), std::io::Error> {
    File::create(path)?.write_all(&mc_util_serial::encode(addr))?;
    Ok(())
}

/// Read user public address from disk
pub fn read_pubfile<P: AsRef<Path>>(path: P) -> Result<PublicAddress, std::io::Error> {
    read_pubfile_data(&mut File::open(path)?)
}

/// Read user pubfile from any implementor of `Read`
pub fn read_pubfile_data<R: std::io::Read>(
    buffer: &mut R,
) -> Result<PublicAddress, std::io::Error> {
    let data = {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;
        data
    };
    let result: PublicAddress = mc_util_serial::decode(&data).map_err(prost_to_io_error)?;
    Ok(result)
}

// Helper function maps serde_json::error to io::Error
#[inline]
pub fn to_io_error(err: serde_json::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, Box::new(err))
}

#[inline]
pub fn prost_to_io_error(err: mc_util_serial::DecodeError) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, Box::new(ProstError { err }))
}

// Helper boilerplate mapping prost::DecodeError to io::Error
#[derive(Debug)]
struct ProstError {
    pub err: mc_util_serial::DecodeError,
}

impl std::fmt::Display for ProstError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "prost deserialization failed: {}", self.err)
    }
}

impl std::error::Error for ProstError {
    fn description(&self) -> &str {
        "prost deserialization failed"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    use mc_transaction_core::account_keys::AccountKey;
    use rand::{rngs::StdRng, SeedableRng};
    use tempdir::TempDir;

    #[test]
    fn test_keyfile() {
        let mut rng: StdRng = SeedableRng::from_seed([1u8; 32]);
        let dir = TempDir::new("test").unwrap();

        {
            let entropy = RootIdentity::random(&mut rng, None);
            let f1 = dir.path().join("f1");
            write_keyfile(&f1, &entropy).unwrap();
            let result = read_keyfile(&f1).unwrap();
            assert_eq!(entropy, result);
        }
    }

    #[test]
    fn test_pubfile() {
        let mut rng: StdRng = SeedableRng::from_seed([1u8; 32]);
        let dir = TempDir::new("test").unwrap();

        {
            let acct_key = AccountKey::random(&mut rng);
            let pubaddr = acct_key.default_subaddress();
            let f2 = dir.path().join("f2");
            write_pubfile(&f2, &pubaddr).unwrap();
            let result = read_pubfile(&f2).unwrap();
            assert_eq!(pubaddr, result);
        }

        {
            let acct_key = AccountKey::random_with_fog(&mut rng);
            let pubaddr = acct_key.default_subaddress();
            let f3 = dir.path().join("f3");
            write_pubfile(&f3, &pubaddr).unwrap();
            let result = read_pubfile(&f3).unwrap();
            assert_eq!(pubaddr, result);
        }
    }
}

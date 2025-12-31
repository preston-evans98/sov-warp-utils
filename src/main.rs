use bech32::{Bech32m, Hrp};
use clap::Parser;
use sha2::{Digest, Sha256};
use std::{fmt::Display, str::FromStr};

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexString<T = Vec<u8>>(pub T)
where
    T: AsRef<[u8]>;

type Address = HexString<[u8; 20]>;
type HexHash = HexString<[u8; 32]>;

#[derive(clap::Parser)]
/// Computes the warp route ID and token ID for a warp route mapping native Ether from an EVM chain
/// to a sovereign SDK chain.

struct Args {
    #[clap(long, short)]
    /// The address that will be used to deploy the warp route on the Sovereign SDK chain
    deployer: Address,
    /// The ethereum address of the wrapped token on the EVM chain
    #[clap(long, short)]
    token_address: Address,
}

impl<T> serde::Serialize for HexString<T>
where
    T: AsRef<[u8]>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            let inner_ref = self.0.as_ref();
            let mut seq = serializer.serialize_seq(Some(inner_ref.len()))?;
            for element in inner_ref {
                seq.serialize_element(element)?;
            }
            seq.end()
        }
    }
}

impl<'de, T> serde::Deserialize<'de> for HexString<T>
where
    T: TryFrom<Vec<u8>> + AsRef<[u8]>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = if deserializer.is_human_readable() {
            let string: String = serde::Deserialize::deserialize(deserializer)?;
            parse_vec_u8(&string).map_err(serde::de::Error::custom)?
        } else {
            serde::Deserialize::deserialize(deserializer)?
        };

        Ok(HexString(bytes.try_into().map_err(|_| {
            serde::de::Error::custom("Invalid hex string length")
        })?))
    }
}

impl<T: BorshSerialize + AsRef<[u8]>> BorshSerialize for HexString<T> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.serialize(writer)
    }
}

impl<T: BorshDeserialize + AsRef<[u8]>> BorshDeserialize for HexString<T> {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        T::deserialize_reader(reader).map(Self)
    }
}

impl<T: TryFrom<Vec<u8>> + AsRef<[u8]>> FromStr for HexString<T> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = parse_vec_u8(s)?;
        Ok(HexString(bytes.try_into().map_err(|_| {
            anyhow::anyhow!("Invalid hex string length")
        })?))
    }
}

impl<T> Display for HexString<T>
where
    T: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl<T> std::fmt::Debug for HexString<T>
where
    T: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

/// [`serde`] (de)serialization functions for [`HexString`], to be used with
/// `#[serde(with = "...")]`.
pub mod hex_string_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::HexString;

    /// Serializes `data` as hex string using lowercase characters and prefixing with '0x'.
    ///
    /// Lowercase characters are used (e.g. `f9b4ca`). The resulting string's length
    /// is always even, each byte in data is always encoded using two hex digits.
    /// Thus, the resulting string contains exactly twice as many bytes as the input
    /// data.
    pub fn serialize<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        HexString::<T>(data).serialize(serializer)
    }

    /// Deserializes a hex string into raw bytes.
    ///
    /// Both upper and lower case characters are valid in the input string and can
    /// even be mixed.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: TryFrom<Vec<u8>> + AsRef<[u8]>,
    {
        HexString::<T>::deserialize(deserializer).map(|s| s.0)
    }
}

fn parse_vec_u8(s: &str) -> anyhow::Result<Vec<u8>> {
    let s = s.strip_prefix("0x").unwrap_or(s);

    hex::decode(s).map_err(|e| anyhow::anyhow!("Failed to decode hex string {}, error: {}", s, e))
}

fn main() {
    let Args {
        deployer,
        token_address,
    } = Args::parse();

    let warp_route_id = get_warp_route_id(token_address, deployer);

    let token_id = get_token_id(warp_route_id, 18);
    println!("Warp Route ID: {warp_route_id}",);
    println!("Token ID: {}", format_token_id(token_id));
}

/// `remote_token_id_bytes || 0 || DEPLOYER_ADDRESS`
fn get_warp_route_id(token_address: Address, deployer: Address) -> HexHash {
    let mut hasher = Sha256::default();
    let mut extended_token_address = [0u8; 32];
    extended_token_address[12..].copy_from_slice(&token_address.0);
    hasher.update(&extended_token_address);
    hasher.update(&[0]);
    hasher.update(&deployer.0);
    HexString(hasher.finalize().into())
}

/// WARP_ROUTE_ID || "Synthetic token for 0x{hex(WARP_ROUTE_ID)} || {LOCAL_DECIMALS as u8}
fn get_token_id(warp_route_id: HexHash, decimals: u8) -> HexHash {
    let mut hasher = Sha256::default();
    let token_name = format!("Synthetic token for {warp_route_id}");
    hasher.update(&warp_route_id.0);
    hasher.update(token_name.as_bytes());
    hasher.update(&[decimals]);
    HexString(hasher.finalize().into())
}

fn format_token_id(id: HexHash) -> String {
    let prefix = Hrp::parse("token_").expect("token_ is a valid prefix");
    bech32::encode::<Bech32m>(prefix, &id.0).expect("Failed to format bech32")
}

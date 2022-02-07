use core::fmt;
use std::str::FromStr;

use arkworks_utils::utils::common::Curve as ArkCurve;
use typed_builder::TypedBuilder;
use zeroize::Zeroize;

use crate::error::Error;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NoteVersion {
    V1,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Backend {
    Arkworks,
    Circom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Curve {
    Bls381,
    Bn254,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HashFunction {
    Poseidon,
    MiMCTornado,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NotePrefix {
    Mixer,
    Bridge,
    Anchor,
    VAnchor,
}

impl fmt::Display for NoteVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteVersion::V1 => write!(f, "v1"),
        }
    }
}

impl FromStr for NoteVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1" => Ok(NoteVersion::V1),
            v => Err(Error::UnsupportedNoteVersion(v.into())),
        }
    }
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Backend::Arkworks => write!(f, "Arkworks"),
            Backend::Circom => write!(f, "Circom"),
        }
    }
}

impl FromStr for Backend {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Arkworks" => Ok(Backend::Arkworks),
            "Circom" => Ok(Backend::Circom),
            v => Err(Error::UnsupportedNoteBackend(v.into())),
        }
    }
}

impl fmt::Display for Curve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Curve::Bls381 => write!(f, "Bls381"),
            Curve::Bn254 => write!(f, "Bn254"),
        }
    }
}

impl FromStr for Curve {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bls381" => Ok(Curve::Bls381),
            "Bn254" => Ok(Curve::Bn254),
            v => Err(Error::UnsupportedNoteCurve(v.into())),
        }
    }
}

impl fmt::Display for HashFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashFunction::Poseidon => write!(f, "Poseidon"),
            HashFunction::MiMCTornado => write!(f, "MiMCTornado"),
        }
    }
}

impl FromStr for NotePrefix {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "webb.mixer" => Ok(NotePrefix::Mixer),
            "webb.bridge" => Ok(NotePrefix::Bridge),
            "webb.anchor" => Ok(NotePrefix::Anchor),
            "webb.vanchor" => Ok(NotePrefix::VAnchor),
            v => Err(Error::UnsupportedNotePrefix(v.into())),
        }
    }
}

impl fmt::Display for NotePrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotePrefix::Mixer => write!(f, "webb.mixer"),
            NotePrefix::Bridge => write!(f, "webb.bridge"),
            NotePrefix::Anchor => write!(f, "webb.anchor"),
            NotePrefix::VAnchor => write!(f, "webb.vanchor"),
        }
    }
}

impl FromStr for HashFunction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Poseidon" => Ok(HashFunction::Poseidon),
            "MiMCTornado" => Ok(HashFunction::MiMCTornado),
            v => Err(Error::UnsupportedNoteHashFunction(v.into())),
        }
    }
}

impl From<Curve> for ArkCurve {
    fn from(curve: Curve) -> Self {
        match curve {
            Curve::Bls381 => ArkCurve::Bls381,
            Curve::Bn254 => ArkCurve::Bn254,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, TypedBuilder)]
pub struct Note {
    pub prefix: NotePrefix,
    #[builder(default = NoteVersion::V1)]
    pub version: NoteVersion,
    #[builder(setter(into))]
    pub target_chain_id: u32,
    #[builder(setter(into))]
    pub source_chain_id: u32,
    #[builder(default = Backend::Circom)]
    pub backend: Backend,
    #[builder(default = HashFunction::Poseidon)]
    pub hash_function: HashFunction,
    #[builder(default = Curve::Bn254)]
    pub curve: Curve,
    #[builder(default = 5)]
    pub exponentiation: u8,
    #[builder(default = 5)]
    pub width: usize,
    pub secret: [u8; 64],
    #[builder(setter(into))]
    pub token_symbol: String,
    #[builder(setter(into))]
    pub amount: String,
    #[builder(setter(into))]
    pub denomination: u8,
}

impl Zeroize for Note {
    fn zeroize(&mut self) { self.secret.zeroize() }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let secrets = hex::encode(&self.secret);
        let parts: Vec<String> = vec![
            // 0 => prefix
            self.prefix.to_string(),
            // 1 => version
            self.version.to_string(),
            // 2 => chain
            self.target_chain_id.to_string(),
            // 3 => chain
            self.source_chain_id.to_string(),
            // 4 => backend
            self.backend.to_string(),
            // 5 => curve
            self.curve.to_string(),
            // 6 => hash_function
            self.hash_function.to_string(),
            // 7 => token_symbol
            self.token_symbol.clone(),
            // 8 => denomination
            self.denomination.to_string(),
            // 9 => amount
            self.amount.clone(),
            // 10
            self.exponentiation.to_string(),
            // 11
            self.width.to_string(),
            // 12
            secrets,
        ];
        let note = parts.join(":");
        write!(f, "{}", note)
    }
}

impl FromStr for Note {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(':').collect();
        if parts.len() != 13 {
            return Err(Error::InvalidNoteFormat);
        }
        let prefix = parts[0].parse()?;
        let version = parts[1].parse()?;
        let target_chain_id =
            parts[2].parse().map_err(|_| Error::InvalidChainId)?;
        let source_chain_id =
            parts[3].parse().map_err(|_| Error::InvalidChainId)?;
        let backend = parts[4].parse()?;
        let curve = parts[5].parse()?;
        let hash_function = parts[6].parse()?;
        let token_symbol = parts[7].to_owned();
        let denomination = parts[8].parse().unwrap();
        let amount = parts[9].to_string();
        let exponentiation = parts[10].parse().unwrap();
        let width = parts[11].parse().unwrap();

        let note_val = parts[12];
        let secret = hex::decode(&note_val.replace("0x", ""))?
            .try_into()
            .map_err(|_| Error::InvalidNoteSecrets)?;

        Ok(Note {
            prefix,
            version,
            target_chain_id,
            source_chain_id,
            token_symbol,
            curve,
            hash_function,
            backend,
            denomination,
            amount,
            exponentiation,
            width,
            secret,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mixer;

    #[test]
    fn should_generate_and_parse_note_correctly() {
        let curve = Curve::Bn254;
        let exponentiation = 5;
        let width = 5;
        let rng = &mut rand::thread_rng();
        let secret =
            mixer::generate_secrets(curve, exponentiation, width, rng).unwrap();
        let note = Note::builder()
            .prefix(NotePrefix::Mixer)
            .version(NoteVersion::V1)
            .target_chain_id(1u32)
            .source_chain_id(2u32)
            .backend(Backend::Circom)
            .hash_function(HashFunction::Poseidon)
            .curve(curve)
            .exponentiation(exponentiation)
            .width(width)
            .token_symbol("TEST")
            .amount("1")
            .denomination(1)
            .secret(secret)
            .build();
        let note_str = note.to_string();
        let parsed_note = note_str.parse::<Note>().unwrap();
        assert_eq!(note, parsed_note);
    }
}

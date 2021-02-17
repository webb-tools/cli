use std::{convert::TryInto, fmt, str::FromStr};

use bulletproofs::{r1cs::Prover, BulletproofGens, PedersenGens};
use curve25519_dalek::scalar::Scalar;
use curve25519_gadgets::fixed_deposit_tree::builder::{
    FixedDepositTree, FixedDepositTreeBuilder,
};
use merlin::Transcript;

use crate::{
    error::Error,
    runtime::{Commitment, Data},
};

const NOTE_PREFIX: &str = "webb.mix";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenSymbol {
    Edg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteVersion {
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    prefix: String,
    version: NoteVersion,
    token_symbol: TokenSymbol,
    mixer_id: u8,
    block_number: Option<u32>,
    r: Data,
    nullifier: Data,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkProof {
    comms: Vec<Commitment>,
    nullifier_hash: Data,
    proof_bytes: Vec<u8>,
    leaf_index_commitments: Vec<Commitment>,
    proof_commitments: Vec<Commitment>,
}

impl fmt::Display for TokenSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenSymbol::Edg => write!(f, "EDG"),
        }
    }
}

impl fmt::Display for NoteVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteVersion::V1 => write!(f, "v1"),
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded_r = hex::encode(&self.r.0);
        let encoded_nullifier = hex::encode(&self.nullifier.0);
        let mut parts = vec![
            self.prefix.clone(),
            self.version.to_string(),
            format!("{}", self.token_symbol),
            format!("{}", self.mixer_id),
        ];
        if let Some(bn) = self.block_number {
            parts.push(format!("{}", bn));
        }
        parts.push(format!("{}{}", encoded_r, encoded_nullifier));
        let note = parts.join("-");
        write!(f, "{}", note)
    }
}

impl FromStr for TokenSymbol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EDG" => Ok(TokenSymbol::Edg),
            v => Err(Error::UnsupportedTokenSymbol(v.to_owned())),
        }
    }
}

impl FromStr for NoteVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1" => Ok(NoteVersion::V1),
            v => Err(Error::UnsupportedNoteVersion(v.to_owned())),
        }
    }
}

impl FromStr for Note {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        let partial = parts.len() == 5;
        let full = parts.len() == 6;
        if !partial && !full {
            return Err(Error::InvalidNoteLength);
        }

        if parts[0] != NOTE_PREFIX {
            return Err(Error::InvalidNotePrefix);
        }

        let version: NoteVersion = parts[1].parse()?;
        let token_symbol: TokenSymbol = parts[2].parse()?;
        let mixer_id =
            parts[3].parse().map_err(|_| Error::InvalidNoteMixerId)?;
        let (block_number, note_val) = match partial {
            true => (None, parts[4]),
            false => {
                let bn = parts[4]
                    .parse()
                    .map_err(|_| Error::InvalidNoteBlockNumber)?;
                (Some(bn), parts[5])
            },
        };
        if note_val.len() != 128 {
            return Err(Error::InvalidNoteFooter);
        }

        let r = hex::decode(&note_val[..64]).map(|v| {
            v.try_into().map_err(|_| Error::NotA32BytesArray).map(Data)
        })??;
        let nullifier = hex::decode(&note_val[64..]).map(|v| {
            v.try_into().map_err(|_| Error::NotA32BytesArray).map(Data)
        })??;
        Ok(Note {
            prefix: NOTE_PREFIX.to_owned(),
            version,
            token_symbol,
            mixer_id,
            block_number,
            r,
            nullifier,
        })
    }
}

pub struct Mixer {
    id: u8,
    tree: FixedDepositTree,
}

impl Default for Mixer {
    fn default() -> Self { Self::new(0) }
}

impl Mixer {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            tree: FixedDepositTreeBuilder::new().depth(32).build(),
        }
    }

    pub fn add_leaves(&mut self, leaves: Vec<Data>) {
        let vals = leaves.into_iter().map(|v| v.0).collect();
        self.tree.tree.add_leaves(vals, None);
    }

    pub fn root(&self) -> Data {
        let root = self.tree.tree.root;
        Data(root.to_bytes())
    }

    pub fn generate_note(&mut self, token_symbol: TokenSymbol) -> Note {
        let leaf = self.tree.generate_secrets();
        let (r, nullifier, ..) = self.tree.get_secrets(leaf);
        Note {
            prefix: NOTE_PREFIX.to_owned(),
            version: NoteVersion::V1,
            token_symbol,
            mixer_id: self.id,
            block_number: None,
            r: Data(r.to_bytes()),
            nullifier: Data(nullifier.to_bytes()),
        }
    }

    pub fn save_note(&mut self, note: Note) -> Data {
        let (r, nullifier, nullifier_hash, leaf) =
            self.tree.leaf_data_from_bytes(note.r.0, note.nullifier.0);
        self.tree.add_secrets(leaf, r, nullifier, nullifier_hash);
        Data(leaf.to_bytes())
    }

    pub fn generate_proof(&mut self, root: Data, leaf: Data) -> ZkProof {
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(40960, 1);
        let mut prover_transcript = Transcript::new(b"zk_membership_proof");
        let prover = Prover::new(&pc_gens, &mut prover_transcript);

        let root = Scalar::from_bytes_mod_order(root.0);
        let leaf = Scalar::from_bytes_mod_order(leaf.0);
        let (
            proof_bytes,
            (comms, nullifier_hash, leaf_index_commitments, proof_commitments),
        ) = self.tree.prove_zk(root, leaf, &bp_gens, prover);

        let comms = comms
            .into_iter()
            .map(|v| Commitment(v.to_bytes()))
            .collect();
        let leaf_index_commitments = leaf_index_commitments
            .into_iter()
            .map(|v| Commitment(v.to_bytes()))
            .collect();
        let proof_commitments = proof_commitments
            .into_iter()
            .map(|v| Commitment(v.to_bytes()))
            .collect();
        let nullifier_hash = Data(nullifier_hash.to_bytes());
        let proof_bytes = proof_bytes.to_bytes();

        ZkProof {
            comms,
            leaf_index_commitments,
            proof_commitments,
            nullifier_hash,
            proof_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_note() {
        let mut mixer = Mixer::new(0);
        let note = mixer.generate_note(TokenSymbol::Edg);
        assert_eq!(note.mixer_id, 0);
        assert_eq!(note.token_symbol, TokenSymbol::Edg);
        eprintln!("{:#?}", note);
    }
}

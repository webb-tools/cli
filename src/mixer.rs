use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;

use bulletproofs::r1cs::Prover;
use bulletproofs::{BulletproofGens, PedersenGens};
use bulletproofs_gadgets::fixed_deposit_tree::builder::{
    FixedDepositTree, FixedDepositTreeBuilder,
};
use bulletproofs_gadgets::poseidon::builder::Poseidon;
use bulletproofs_gadgets::poseidon::{PoseidonBuilder, PoseidonSbox};
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

use crate::error::Error;
use crate::pallet::{Commitment, ScalarData};

const NOTE_PREFIX: &str = "webb.mix";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenSymbol {
    Edg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteVersion {
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub prefix: String,
    pub version: NoteVersion,
    pub token_symbol: TokenSymbol,
    pub mixer_id: u32,
    pub block_number: Option<u32>,
    r: ScalarData,
    nullifier: ScalarData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkProof {
    pub comms: Vec<Commitment>,
    pub nullifier_hash: ScalarData,
    pub proof_bytes: Vec<u8>,
    pub leaf_index_commitments: Vec<Commitment>,
    pub proof_commitments: Vec<Commitment>,
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
            v.try_into()
                .map_err(|_| Error::NotA32BytesArray)
                .map(ScalarData)
        })??;
        let nullifier = hex::decode(&note_val[64..]).map(|v| {
            v.try_into()
                .map_err(|_| Error::NotA32BytesArray)
                .map(ScalarData)
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
    id: u32,
    tree: FixedDepositTree,
}

impl Default for Mixer {
    fn default() -> Self { Self::new(0) }
}

/// Default hasher instance used to construct the tree
pub fn default_hasher() -> Poseidon {
    let width = 6;
    // TODO: should be able to pass the number of generators
    let bp_gens = BulletproofGens::new(16400, 1);
    PoseidonBuilder::new(width)
        .bulletproof_gens(bp_gens)
        .sbox(PoseidonSbox::Exponentiation3)
        .build()
}

impl Mixer {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            tree: FixedDepositTreeBuilder::new()
                .hash_params(default_hasher())
                .depth(32)
                .build(),
        }
    }

    pub fn add_leaves(&mut self, leaves: Vec<ScalarData>) {
        let vals = leaves.into_iter().map(|v| v.0).collect();
        self.tree.tree.add_leaves(vals, None);
    }

    pub fn root(&self) -> ScalarData {
        let root = self.tree.tree.root;
        ScalarData(root.to_bytes())
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
            r: ScalarData(r.to_bytes()),
            nullifier: ScalarData(nullifier.to_bytes()),
        }
    }

    pub fn save_note(&mut self, note: Note) -> ScalarData {
        let (r, nullifier, nullifier_hash, leaf) =
            self.tree.leaf_data_from_bytes(note.r.0, note.nullifier.0);
        self.tree.add_secrets(leaf, r, nullifier, nullifier_hash);
        ScalarData(leaf.to_bytes())
    }

    pub fn generate_proof(
        &mut self,
        root: ScalarData,
        leaf: ScalarData,
    ) -> ZkProof {
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(16400, 1);
        let mut prover_transcript = Transcript::new(b"zk_membership_proof");
        let prover = Prover::new(&pc_gens, &mut prover_transcript);

        let root = Scalar::from_bytes_mod_order(root.0);
        let leaf = Scalar::from_bytes_mod_order(leaf.0);
        let recipient = Scalar::default();
        let relayer = Scalar::default();
        let (
            proof_bytes,
            (comms, nullifier_hash, leaf_index_commitments, proof_commitments),
        ) = self
            .tree
            .prove_zk(root, leaf, recipient, relayer, &bp_gens, prover);

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
        let nullifier_hash = ScalarData(nullifier_hash.to_bytes());
        let proof_bytes = proof_bytes.to_bytes();

        ZkProof {
            comms,
            nullifier_hash,
            proof_bytes,
            leaf_index_commitments,
            proof_commitments,
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

use ark_bls12_381::Fr as BlsFr;
use ark_bn254::Fr as Bn254Fr;
use arkworks_circuits::setup::mixer::{
    setup_leaf_with_privates_raw_x5_5, setup_leaf_x5_5,
};
use arkworks_utils::utils::common::Curve as ArkworksCurve;
use rand::RngCore;
use webb::substrate::protocol_substrate_runtime::api::runtime_types::webb_standalone_runtime::Element;

use crate::{
    error::Error,
    note::{Curve, Note},
};

pub fn generate_secrets(
    curve: Curve,
    exponentiation: u8,
    width: usize,
    rng: &mut impl RngCore,
) -> Result<[u8; 64], Error> {
    let (secret_bytes, nullifier_bytes, ..) = match (curve, exponentiation, width) {
        (Curve::Bls381, 5, 5) => setup_leaf_x5_5::<BlsFr, _>(ArkworksCurve::Bls381, rng),
        (Curve::Bn254, 5, 5) => setup_leaf_x5_5::<Bn254Fr, _>(ArkworksCurve::Bn254, rng),
        _ => todo!(
            "mixer for curve {curve}, exponentiation {exponentiation}, and width {width}"
        ),
    }.map_err(|_| Error::FailedToGenerateSecrets)?;
    let mut secrets = [0u8; 64];
    secrets[0..32].copy_from_slice(&secret_bytes);
    secrets[32..].copy_from_slice(&nullifier_bytes);

    Ok(secrets)
}

pub fn get_leaf_from_note(note: &Note) -> Result<(Element, Element), Error> {
    if note.secret.len() < 64 {
        return Err(Error::InvalidNoteSecrets);
    }

    let curve = note.curve;
    let exponentiation = note.exponentiation;
    let width = note.width;

    let secret = note.secret[..32].to_vec();
    let nullifer = note.secret[32..64].to_vec();
    let (leaf, nullifer_hash) = match (curve, exponentiation, width) {
        (Curve::Bls381, 5, 5) => setup_leaf_with_privates_raw_x5_5::<BlsFr>(
            ArkworksCurve::Bls381,
            secret,
            nullifer,
        ),
        (Curve::Bn254, 5, 5) => setup_leaf_with_privates_raw_x5_5::<Bn254Fr>(
            ArkworksCurve::Bn254,
            secret,
            nullifer,
        ),
        _ => todo!(
            "mixer leaf for curve {curve}, exponentiation {exponentiation}, and width {width}"
        ),
    }
    .map(|(l, h)| match (l.try_into(), h.try_into()) {
        (Ok(l), Ok(h)) => Ok((Element(l), Element(h))),
        _ => Err(Error::NotA32BytesArray),
    })
    .map_err(|_| Error::FailedToGenerateLeaf)??;
    Ok((leaf, nullifer_hash))
}

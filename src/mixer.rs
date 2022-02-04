use ark_bls12_381::Fr as BlsFr;
use ark_bn254::Fr as Bn254Fr;
use arkworks_circuits::setup::mixer::{
    setup_leaf_with_privates_raw_x5_5, setup_leaf_x5_5,
};
use arkworks_utils::utils::common::Curve as ArkworksCurve;
use rand::rngs::OsRng;

use crate::error::Error;
use crate::note::Curve;

pub fn generate_secrets(
    curve: Curve,
    exponentiation: u8,
    width: usize,
    rng: &mut OsRng,
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

pub fn get_leaf_with_private_raw(
    curve: Curve,
    exponentiation: u8,
    width: usize,
    raw: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), Error> {
    if raw.len() < 64 {
        return Err(Error::InvalidNoteSecrets);
    }

    let secrets = raw[..32].to_vec();
    let nullifer = raw[32..64].to_vec();
    let sec = match (curve, exponentiation, width) {
        (Curve::Bls381, 5, 5) => setup_leaf_with_privates_raw_x5_5::<BlsFr>(
            ArkworksCurve::Bls381,
            secrets,
            nullifer,
        ),
        (Curve::Bn254, 5, 5) => setup_leaf_with_privates_raw_x5_5::<Bn254Fr>(
            ArkworksCurve::Bn254,
            secrets,
            nullifer,
        ),
        _ => todo!(
            "mixer leaf for curve {curve}, exponentiation {exponentiation}, and width {width}"
        ),
    }
    .map_err(|_| Error::FailedToGenerateSecrets)?;
    Ok(sec)
}

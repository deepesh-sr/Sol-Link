use std::collections::BTreeMap;
use frost_ed25519::{self as frost, keys::dkg::round1::{Package, SecretPackage}, round1::{SigningCommitments, SigningNonces}, round2::SignatureShare} ;
use frost::{
    Identifier, 
    keys::{KeyPackage,PublicKeyPackage, dkg},
    round1,round2,
    Signature, SigningPackage
};

use rand::{rngs, thread_rng};


pub const MAX_SIGNERS : u16 = 3;
pub const MIN_SIGNERS : u16 = 2;


pub fn dkg_part1 ( identifier : Identifier  )-> Result<(SecretPackage, Package), frost::Error>{

    let mut rng = rand::rngs::OsRng;
     let (round1_secret_package, round1_package) = frost::keys::dkg::part1(
        identifier,
        MAX_SIGNERS,
        MIN_SIGNERS,
        &mut rng,
    )?;

    Ok((round1_secret_package,round1_package))
}

pub fn dkg_part2(
    secret_package : SecretPackage,
    round1_packages : &BTreeMap<Identifier , dkg::round1::Package>
)-> Result<(dkg::round2::SecretPackage, BTreeMap<Identifier, dkg::round2::Package>), frost::Error>{
    dkg::part2(secret_package, round1_packages)
}

pub fn dkg_part3(
    round2_secret_package: &dkg::round2::SecretPackage,
    round1_packages: &BTreeMap<Identifier, dkg::round1::Package>,
    round2_packages: &BTreeMap<Identifier, dkg::round2::Package>,
) -> Result<(KeyPackage, PublicKeyPackage), frost::Error> {
    dkg::part3(round2_secret_package, round1_packages, round2_packages)
}

pub fn sign_round1(key_package: KeyPackage) -> Result<(SigningNonces, SigningCommitments) , frost::Error> {
    let mut rng = rngs::OsRng;

    let (nonces, commitments) = frost::round1::commit(key_package.signing_share(), &mut rng);
Ok((nonces,commitments))
}
pub  fn sign_round2(signing_package : SigningPackage , nonces : &SigningNonces , key_package: &KeyPackage)-> Result<SignatureShare, frost::Error>{
        let signature_share = frost::round2::sign(&signing_package, nonces, key_package)?;
Ok(signature_share)
}

pub fn aggregate(signing_package : SigningPackage, signature_shares : &BTreeMap<Identifier, SignatureShare> , pubkey_package : PublicKeyPackage)-> Result<Signature, frost::Error>{
    let group_signature = frost::aggregate(&signing_package, &signature_shares, &pubkey_package)?;
Ok(group_signature)
}
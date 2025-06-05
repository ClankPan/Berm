use ark_bn254::{Fq, Fr};
use ark_ff::{Field, PrimeField};
use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar};
use ark_relations::r1cs::ConstraintSystemRef;

pub trait Alloc<F: PrimeField> {
    type Out;

    fn to_witness(&self, cs: ConstraintSystemRef<F>) -> ark_relations::r1cs::Result<Self::Out>;
    fn to_input(&self, cs: ConstraintSystemRef<F>) -> ark_relations::r1cs::Result<Self::Out>;
}

impl<F> Alloc<F> for F
where
    F: PrimeField,
{
    type Out = FpVar<F>;

    fn to_witness(&self, cs: ConstraintSystemRef<F>) -> ark_relations::r1cs::Result<Self::Out> {
        FpVar::<F>::new_witness(cs.clone(), || Ok(self))
    }
    fn to_input(&self, cs: ConstraintSystemRef<F>) -> ark_relations::r1cs::Result<Self::Out> {
        FpVar::<F>::new_input(cs.clone(), || Ok(self))
    }
}
impl<F> Alloc<F> for Vec<F>
where
    F: PrimeField,
{
    type Out = Vec<FpVar<F>>;

    fn to_witness(&self, cs: ConstraintSystemRef<F>) -> ark_relations::r1cs::Result<Self::Out> {
        Vec::<FpVar<F>>::new_witness(cs.clone(), || Ok(self.clone()))
    }
    fn to_input(&self, cs: ConstraintSystemRef<F>) -> ark_relations::r1cs::Result<Self::Out> {
        Vec::<FpVar<F>>::new_input(cs.clone(), || Ok(self.clone()))
    }
}

// impl Alloc<Fr> for Fr {
//     type Out = FpVar<Fr>;
//
//     fn to_witness(&self, cs: ConstraintSystemRef<Fr>) -> ark_relations::r1cs::Result<Self::Out> {
//         FpVar::<Fr>::new_witness(cs.clone(), || Ok(self))
//     }
// }
//
// impl Alloc<Fr> for Vec<Fr> {
//     type Out = Vec::<FpVar<Fr>>;
//
//     fn to_witness(&self, cs: ConstraintSystemRef<Fr>) -> ark_relations::r1cs::Result<Self::Out> {
//         Vec::<FpVar<Fr>>::new_witness(cs.clone(), || Ok(self.clone()))
//     }
// }
//
//
// impl Alloc<Fq> for Fq {
//     type Out = FpVar<Fq>;
//
//     fn to_witness(&self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<Self::Out> {
//         FpVar::<Fq>::new_witness(cs.clone(), || Ok(self))
//     }
// }
//
// impl Alloc<Fq> for Vec<Fq> {
//     type Out = Vec::<FpVar<Fq>>;
//
//     fn to_witness(&self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<Self::Out> {
//         Vec::<FpVar<Fq>>::new_witness(cs.clone(), || Ok(self.clone()))
//     }
// }

use alloc::Alloc;
use ark_bn254::Fr;
use ark_r1cs_std::{R1CSVar, alloc::AllocVar, eq::EqGadget, fields::fp::FpVar};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef};
use cyclefold::{CommVar, CycleFold};
use zerofold::{NscVar, ZeroFold};

use ark_relations::r1cs::Result;

mod alloc;
mod coeffs;
mod cyclefold;
mod pedersen;
mod poseidon_config;
mod zerofold;

pub const SQ: usize = 4;

pub fn main() {}

pub struct NeutronNova;

pub struct CpuCircuit;

impl CpuCircuit {
    pub fn synthesize(self, cs: ConstraintSystemRef<Fr>, z_in: &Vec<FrVar>) -> Result<Vec<FrVar>> {
        todo!()
    }
}

impl NeutronNova {
    pub fn step() -> ark_relations::r1cs::Result<()> {
        let mut cs = ConstraintSystem::<Fr>::new_ref();

        Ok(())
    }
}

struct AugmentedCircuit {
    zerofold: ZeroFold,
    cyclefold: CycleFold,
    z_i: Vec<Fr>,
    cpu: CpuCircuit,
}

impl ConstraintSynthesizer<Fr> for AugmentedCircuit {
    fn generate_constraints(
        self,
        cs: ark_relations::r1cs::ConstraintSystemRef<Fr>,
    ) -> ark_relations::r1cs::Result<()> {
        let z_in = self
            .z_i
            .into_iter()
            .map(|z| z.to_witness(cs.clone()))
            .collect::<Result<Vec<FrVar>>>()?;
        let z_out = self.cpu.synthesize(cs.clone(), &z_in)?;

        let (zf_r, zf_i, zf_f) = self.zerofold.verify(cs.clone())?;

        let (cf_r, cf_f) = self
            .cyclefold
            .verify(cs.clone(), &zf_r.com, &zf_i.com, &zf_f.com)?;

        let h_r = hash(&zf_r, &cf_r, z_in)?;
        let h_f = hash(&zf_f, &cf_f, z_out)?;

        h_r.enforce_equal(&zf_i.xcc)?;
        h_f.value()?.to_input(cs.clone())?.enforce_equal(&h_r)?;

        Ok(())
    }
}

pub type FrVar = FpVar<Fr>;

pub fn hash(
    zf: &NscVar,
    cf: &cyclefold::U,
    z: Vec<FrVar>,
) -> ark_relations::r1cs::Result<FrVar> {
    todo!()
}

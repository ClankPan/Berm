use std::rc::Rc;

use ark_bn254::{Fq, Fr, G1Projective};
use ark_ff::{Field, PrimeField};
use ark_grumpkin::constraints::GVar;
use ark_r1cs_std::{
    alloc::AllocVar, convert::ToConstraintFieldGadget, eq::EqGadget, fields::{emulated_fp::EmulatedFpVar, fp::FpVar, FieldVar}, prelude::{Boolean, ToBitsGadget}, R1CSVar
};
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, SynthesisError,
};
use nest_struct::nest_struct;

use crate::alloc::Alloc;

#[nest_struct]
pub struct CycleFold {
    // e: nest! {
    //     u_r: U,
    //     u_i: U,
    // },
    // cc: nest! {
    //     u_r: U,
    //     u_i: U,
    // },
    // pc: nest! {
    //     u_r: U,
    //     u_i: U,
    // },
}
pub struct Rlc {
    c_r: G1Projective,
    c_i: G1Projective,
    rb: Fr,
}
pub struct U {
    w: Vec<Fq>,
    e: Vec<Fq>,
    x: Vec<Fq>,
}

pub struct UVar {

}

impl UVar {
    
    pub fn base(cs: ConstraintSystemRef<Fr>) -> ark_relations::r1cs::Result<Self> {
        todo!()
    }
}

pub type EmulatedFrVar = EmulatedFpVar<Fq, Fr>;
#[derive(Clone)]
pub struct AffineVar {
    pub x: EmulatedFrVar,
    pub y: EmulatedFrVar,
}
// pub type ComVar = (AffineVar, AffineVar, AffineVar);
pub struct CommVar {
    e: AffineVar,
    cc: AffineVar,
    pc: AffineVar,
}

impl CommVar {
    pub fn empty() -> Self {
        let zero = AffineVar {
            x: EmulatedFrVar::zero(),
            y: EmulatedFrVar::zero(),
        };
        Self {
            e: zero.clone(),
            cc: zero.clone(),
            pc: zero.clone(),
        }
    }
}


impl Rlc {
    pub fn new(c_r: G1Projective, c_i: G1Projective, rb: Fr) -> Self {
        Self { c_r, c_i, rb }
    }
}

impl CycleFold {
    pub fn prove(
        c_r: (G1Projective, G1Projective, G1Projective),
        c_i: (G1Projective, G1Projective, G1Projective),
        rb: Fr,
    ) -> ark_relations::r1cs::Result<()> {
        let cs = ConstraintSystem::<Fq>::new_ref();

        let rlc_0 = Rlc::new(c_r.0, c_i.0, rb);
        let rlc_1 = Rlc::new(c_r.1, c_i.1, rb);
        let rlc_2 = Rlc::new(c_r.2, c_i.2, rb);

        rlc_0.generate_constraints(cs.clone())?;
        cs.finalize();

        // commitmentを二つ受け取る。その時に、affineでx,yのタプルで受け取る。
        // ランダム線形結合する回路を書く。
        // csを作って、Rlcに加える。
        // cs.finalized()して、witnessとinputを取り出して、witnessに対してコミットして、それをaffineに変換する。
        //

        Ok(())
    }

    pub fn verify(
        self,
        cs: ConstraintSystemRef<Fr>,
        com_r: &CommVar,
        com_i: &CommVar,
        com_f: &CommVar,
        is_base: &Boolean<Fr>,
    ) -> ark_relations::r1cs::Result<(UVar, UVar)> {
        // 引数のcyclefoldのUとこのselfのUを線形結合する。
        // 結果を返す。


        let cf_b = UVar::base(cs.clone())?;

        let rb = FpVar::<Fr>::one(); // 乱数をどこからとってくる？

        let emu_rb = EmulatedFpVar::<Fq, Fr>::new_witness(cs.clone(), || {
            Ok(Fq::from_bigint(rb.value()?.into_bigint())
                .ok_or(SynthesisError::AssignmentMissing)?)
        })?;

        // emu_rbとrbが同じものかをチェクする。
        let constraint_repr = emu_rb.to_constraint_field()?;
        assert_eq!(constraint_repr.len(), 2); // Sanity check (for BN254)
        constraint_repr[0].enforce_equal(&FpVar::zero())?;
        constraint_repr[1].enforce_equal(&rb)?;

        // 結合する、これはNovaのやつ
        // let x_f: Vec<EmulatedFpVar<Fq, Fr>> = self
        //     .e
        //     .u_r
        //     .x
        //     .into_iter()
        //     .zip(self.e.u_i.x)
        //     .map(|(r, i)| {
        //         let r = EmulatedFpVar::<Fq, Fr>::new_witness(cs.clone(), || Ok(r)).unwrap();
        //         let i = EmulatedFpVar::<Fq, Fr>::new_witness(cs.clone(), || Ok(i)).unwrap();
        //         r + &emu_rb * i
        //     })
        //     .collect();
        // //
        todo!()
    }
}

impl ConstraintSynthesizer<Fq> for Rlc {
    fn generate_constraints(
        self,
        cs: ark_relations::r1cs::ConstraintSystemRef<Fq>,
    ) -> ark_relations::r1cs::Result<()> {
        // G1Projectiveをaffineに変換して、rbで(Fq::ONE-rb)*c_r + rb*c_iをやる。
        Ok(())
    }
}

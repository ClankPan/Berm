use std::marker::PhantomData;

use ark_ec::CurveGroup;
use ark_std::rand::RngCore;

pub struct Pedersen<C: CurveGroup> {
    _c: PhantomData<C>,
}

pub struct Param<C: CurveGroup> {
    h: C,
    generators: Vec<C::Affine>,
}

impl<C: CurveGroup> Pedersen<C> {
    pub fn setup(mut rng: impl RngCore, len: usize) -> Param<C> {
        let generators = (0..len).map(|_| C::rand(&mut rng).into_affine()).collect();
        let h = C::rand(&mut rng);
        Param { h, generators }
    }

    pub fn commit(param: &Param<C>, v: &[C::Scalar], r: &C::Scalar) -> C {
        param.h.mul(r) + C::msm(&param.generators, v).unwrap()
    }
}

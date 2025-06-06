use ark_bn254::{Fr, G1Projective};
use ark_crypto_primitives::sponge::{
    CryptographicSponge,
    constraints::CryptographicSpongeVar,
    poseidon::{PoseidonSponge, constraints::PoseidonSpongeVar},
};
use ark_ec::PrimeGroup;
use ark_ff::{AdditiveGroup, Field, PrimeField, UniformRand};
use ark_r1cs_std::{
    alloc::AllocVar, eq::EqGadget, fields::fp::FpVar, prelude::Boolean, select::CondSelectGadget,
};
use ark_relations::r1cs::ConstraintSystemRef;
use ark_std::test_rng;
use coeffs::vandermonde_interpolation;
use pedersen::{Param, Pedersen};
use poseidon_config::poseidon_canonical_config;

use crate::{SQ, alloc::Alloc, coeffs, cyclefold::CommVar, pedersen, poseidon_config};

#[derive(Clone)]
struct Sc {
    cm: (Fr, G1Projective),
    t: Fr,
    w: Vec<Fr>,
    x: Fr,
}

#[derive(Clone)]
struct Nsc {
    cm: (Fr, G1Projective),
    e: Vec<Fr>,
    cc: Sc,
    pc: Sc,
}

type FrVar = FpVar<Fr>;

#[derive(Clone)]
pub struct UVar {
    pub tcc: FrVar,
    pub tpc: FrVar,
    pub xcc: FrVar,
    pub xpc: FrVar,
    pub com: CommVar,
    // todo: commitment
}

impl UVar {
    pub fn new(cs: ConstraintSystemRef<Fr>, nsc: Nsc) -> ark_relations::r1cs::Result<Self> {
        let tcc = nsc.cc.t.to_witness(cs.clone())?;
        let tpc = nsc.pc.t.to_witness(cs.clone())?;
        let xcc = nsc.cc.x.to_witness(cs.clone())?;
        let xpc = nsc.pc.x.to_witness(cs.clone())?;
        let com = CommVar::empty(); // todo
        Ok(Self {
            tcc,
            tpc,
            xcc,
            xpc,
            com,
        })
    }
    pub fn base(cs: ConstraintSystemRef<Fr>) -> ark_relations::r1cs::Result<Self> {
        todo!()
    }
}

pub struct ZeroFold {
    coeffs: Vec<Fr>,
    u_r: Nsc,
    u_i: Nsc,
    u_f: Nsc,
}

impl ZeroFold {
    pub fn prove(
        pp: Param<G1Projective>,
        u_r: Nsc,
        cc: Sc,
        pc: Sc,
        mle: MLE,
    ) -> ark_relations::r1cs::Result<Self> {
        // トランスクリプト
        let mut transcript = PoseidonSponge::<Fr>::new(&poseidon_canonical_config());
        // 今回はテスト用のrng
        let mut rng = test_rng();
        // tauを生成するためのabsorbで、ccとpcのx,comを使う。
        // witnessは回路では扱わないので、commitmentを代わりに使う。
        transcript.absorb(&cc.t);
        transcript.absorb(&cc.x);
        transcript.absorb(&pc.t);
        transcript.absorb(&pc.x);
        // let affine = u_r.c.1.into_affine().xy().unwrap();
        let tau: Fr = transcript.squeeze_field_elements(1)[0];
        // eはtauの冪乗をlowとhighに分解した2倍の長さのベクトル
        let e: Vec<Fr> = (0..2 * SQ)
            .map(|i| {
                // ifで分岐すれば良いのだが、このも数式的に同じ
                let x = i % SQ * (1 - (SQ - 1) * (1 / SQ));
                tau.pow([x as u64])
            })
            .collect();
        // 生成したeに対してコミットする。
        let r = Fr::rand(&mut rng);
        let cm = (r, Pedersen::commit(&pp, &e, &r));
        let u_i = Nsc {
            cm,
            e: e.clone(),
            cc: cc.clone(),
            pc: pc.clone(),
        };

        let _u_r = u_r.clone();

        // bでfoldするclosureを作る。
        // let e = fold(u_r.e, e);
        let e = move |b| {
            u_r.e
                .iter()
                .zip(&e)
                .map(|(v, w)| (Fr::ONE - b) * v + b * w)
                .collect()
        };
        // let x = (fold(u_r.cc.x, cc.x), fold(u_r.pc.x, pc.x));
        let xcc = |b| (Fr::ONE - b) * u_r.cc.x + b * u_i.cc.x;
        let xpc = |b| (Fr::ONE - b) * u_r.pc.x + b * u_i.pc.x;
        let x = (xcc, xpc);
        let wcc = move |b| {
            u_r.cc
                .w
                .iter()
                .zip(&cc.w)
                .map(|(v, w)| (Fr::ONE - b) * v + b * w)
                .collect()
        };
        let wpc = move |b| {
            u_r.pc
                .w
                .iter()
                .zip(&pc.w)
                .map(|(v, w)| (Fr::ONE - b) * v + b * w)
                .collect()
        };
        let w = (wcc, wpc);
        // let w = (fold(u_r.cc.w, cc.w), fold(u_r.pc.w, pc.w));

        // サムチェックの項をclosureで定義する。
        let t_cc_inner = |b, i| {
            let h = MLE::e(e(b));
            let g = mle.cc(x.0(b), w.0(b));
            h.0(i) * h.0(i) * (g.0(i) * g.1(i) - g.2(i))
        };
        let t_pc_inner = |b, i| {
            let h = MLE::e(e(b));
            let g = mle.cc(x.1(b), w.1(b));
            h.0(i) * h.0(i) * (g.0(i) * g.1(i) - g.2(i))
        };

        // サムをとる。10は仮置き。
        let t_cc = |b| (0..10).map(|i| t_cc_inner(b, i)).sum::<Fr>();
        let t_pc = |b| (0..10).map(|i| t_pc_inner(b, i)).sum::<Fr>();
        let gamma: Fr = transcript.squeeze_field_elements(1)[0];
        // γで二つのNSCを一つに結合する。
        let t_gamma = |b| {
            (0..10)
                .map(|i| t_cc_inner(b, i) + gamma + t_pc_inner(b, i))
                .sum::<Fr>()
        };

        // Q(b)のclosureを作る。これは、u_rとu_iのtのselectorになる。
        let rho: Fr = transcript.squeeze_field_elements(1)[0];
        let eq = |i, j| (Fr::ONE - i) * (Fr::ONE - j) + i * j;
        let q = |b| eq(rho, b) * t_gamma(b);

        // ランダムにQを評価うする。これは1変数のsumcheck
        let rb: Fr = transcript.squeeze_field_elements(1)[0];
        // let c = q(rb);

        // Q(b)は現在プログラム的(closure)で定義されているだけなので、
        // verifierに渡すために多項式にする。そのために、評価結果から多項式補完をして、
        // 係数を求める。
        let eval_at_6_points: Vec<Fr> = (0..6).map(|i| q(Fr::from(i))).collect();
        let coeffs = vandermonde_interpolation(&eval_at_6_points);

        // u_r,u_iのwitnessなどを線型結合する。
        let e = e(rb);
        let wcc = w.0(rb);
        let wpc = w.1(rb);
        let xcc = x.0(rb);
        let xpc = x.1(rb);

        // eへのcommentmentを結合
        let cm = (
            u_r.cm.0 + rho * u_i.cm.0,
            u_r.cm.1 + u_i.cm.1.mul_bigint(rho.into_bigint()),
        );

        // ccへのcommentmentを結合
        let cmcc = (
            u_r.cc.cm.0 + rho * u_i.cc.cm.0,
            u_r.cc.cm.1 + u_i.cc.cm.1.mul_bigint(rho.into_bigint()),
        );

        // pcへのcommentmentを結合
        let cmpc = (
            u_r.pc.cm.0 + rho * u_i.pc.cm.0,
            u_r.pc.cm.1 + u_i.pc.cm.1.mul_bigint(rho.into_bigint()),
        );

        let cc = Sc {
            cm: cmcc,
            t: t_cc(rb),
            w: wcc,
            x: xcc,
        };
        let pc = Sc {
            cm: cmpc,
            t: t_pc(rb),
            w: wpc,
            x: xpc,
        };

        let u_f = Nsc { cm, e, cc, pc };

        Ok(Self {
            coeffs,
            u_r: _u_r,
            u_i,
            u_f,
        })
    }

    pub fn verify(
        self,
        cs: ConstraintSystemRef<Fr>,
    ) -> ark_relations::r1cs::Result<(UVar, UVar, UVar)> {
        // Varに割り当てる。
        // todo: base caseを考える。
        let u_r = UVar::new(cs.clone(), self.u_r)?;
        let u_i = UVar::new(cs.clone(), self.u_i)?;
        // let u_f = NscVar::new(cs.clone(), self.u_f)?;
        let coeffs = self.coeffs.to_witness(cs.clone())?;
        let one = FrVar::new_constant(cs.clone(), Fr::ONE)?;

        // Varからgamma, rho, rbなどを生成し直す。
        let mut transcript = PoseidonSpongeVar::<Fr>::new(cs.clone(), &poseidon_canonical_config());
        transcript.absorb(&u_i.tcc)?;
        transcript.absorb(&u_i.tpc)?;
        transcript.absorb(&u_i.xcc)?;
        transcript.absorb(&u_i.xpc)?;
        // tauはverifierで使うんだっけ?
        // let tau = transcript.squeeze_field_elements(1)?[0].clone();

        // gamma, rho, rbは、proverと揃える　todo
        let gamma = transcript.squeeze_field_elements(1)?[0].clone();
        let rho = transcript.squeeze_field_elements(1)?[0].clone();
        let rb = transcript.squeeze_field_elements(1)?[0].clone();

        // inputを畳み込む。
        let xcc = (&one - &rb) * &u_r.xcc + &rb * &u_i.xcc;
        let xpc = (&one - &rb) * &u_r.xpc + &rb * &u_i.xpc;
        // 畳み込んだインスタンスを作る。
        let u_f = UVar {
            tcc: self.u_f.cc.t.to_witness(cs.clone())?,
            tpc: self.u_f.pc.t.to_witness(cs.clone())?,
            xcc,
            xpc,
            com: CommVar::empty(), // todo
        };

        // coeffからqを作る。
        let t_r = &u_r.tcc + &gamma * &u_r.tpc; // tの作り方はあってる？
        let t_i = &u_i.tcc + &gamma * &u_i.tpc; // これも
        let q_0 = coeffs.iter().map(|c| c * Fr::ZERO).sum::<FrVar>();
        let q_1 = coeffs.iter().map(|c| c * Fr::ONE).sum::<FrVar>();
        // t == q(0) + q(1)を確かめる。
        (t_r + t_i).enforce_equal(&(q_0 + q_1))?;

        // c = Q(rb)
        let c = coeffs.iter().map(|c| c * &rb).sum::<FrVar>();
        // t = eq(rho, rb)^-1 * c
        let eq = (&one - &rho) * (&one - &rb) + &rho * &rb;
        let t_f = &u_f.tcc + &gamma * &u_f.tpc; // tの作り方はあってる？
        (t_f * eq).enforce_equal(&c)?;

        // inputを畳み込む。
        let xcc = (&one - &rb) * &u_r.xcc + &rb * &u_i.xcc;
        let xpc = (&one - &rb) * &u_r.xpc + &rb * &u_i.xpc;
        let u_f = UVar {
            tcc: self.u_f.cc.t.to_witness(cs.clone())?,
            tpc: self.u_f.pc.t.to_witness(cs.clone())?,
            xcc,
            xpc,
            com: CommVar::empty(), // todo
        };

        Ok((u_r, u_i, u_f))
    }
}

fn fold(v: Vec<Fr>, w: Vec<Fr>) -> impl Fn(Fr) -> Vec<Fr> {
    move |b| {
        v.iter()
            .zip(&w)
            .map(|(v, w)| (Fr::ONE - b) * v + b * w)
            .collect()
    }
}
#[derive(Default)]
struct MLE {
    a: Vec<Vec<Fr>>,
    b: Vec<Vec<Fr>>,
    c: Vec<Vec<Fr>>,
}

impl MLE {
    pub fn e(e: Vec<Fr>) -> (impl Fn(usize) -> Fr, impl Fn(usize) -> Fr) {
        let mut low = e;
        let high = low.split_off(SQ);

        let a = move |i| low[i % SQ];
        let b = move |i| high[i / SQ];

        (a, b)
    }

    pub fn cc(
        &self,
        x: Fr,
        w: Vec<Fr>,
    ) -> (
        impl Fn(usize) -> Fr,
        impl Fn(usize) -> Fr,
        impl Fn(usize) -> Fr,
    ) {
        let z = [vec![x], w, vec![Fr::ONE]].concat();

        fn mle(m: Vec<Vec<Fr>>, z: Vec<Fr>) -> impl Fn(usize) -> Fr {
            move |i: usize| m[i].iter().zip(&z).map(|(v, z)| v * z).sum::<Fr>()
        }

        let a = mle(self.a.clone(), z.clone());
        let b = mle(self.b.clone(), z.clone());
        let c = mle(self.c.clone(), z.clone());

        (a, b, c)
    }

    pub fn pc(
        x: Vec<Fr>,
        w: Vec<Fr>,
    ) -> (
        impl Fn(usize) -> Fr,
        impl Fn(usize) -> Fr,
        impl Fn(usize) -> Fr,
    ) {
        let e = w;
        let tau = x[0];

        fn mle(e: Vec<Fr>, tau: Fr, gi: usize) -> impl Fn(usize) -> Fr {
            move |i| {
                let g = if i == 0 {
                    [e[i], Fr::ONE, Fr::ONE]
                } else if 1 <= i && i < SQ {
                    [e[i], e[i - 1], tau]
                } else if i == SQ {
                    [e[i], Fr::ONE, Fr::ONE]
                } else if i == SQ + 1 {
                    [e[i], e[i - 2], tau]
                } else if i == SQ + 2 {
                    [e[i], e[i - 1], e[i - 1]]
                } else if SQ + 2 < i && i < 2 * SQ {
                    [e[i], e[SQ + 1], e[i - 1]]
                } else {
                    [Fr::ZERO, Fr::ZERO, Fr::ZERO]
                };
                g[gi]
            }
        }

        let a = mle(e.clone(), tau, 0);
        let b = mle(e.clone(), tau, 1);
        let c = mle(e.clone(), tau, 2);

        (a, b, c)
    }
}

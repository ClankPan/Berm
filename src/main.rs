use ark_bn254::Fr;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
fn main() {
    println!("Hello, world!");
}

struct StepCircuit {
    pub_z: Fr,
    x: Fr,
    y: Fr,
}

impl ConstraintSynthesizer<Fr> for StepCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> ark_relations::r1cs::Result<()> {
        // 変数を配置
        let x_var = cs.new_witness_variable(|| Ok(self.x));
        let y_var = cs.new_witness_variable(|| Ok(self.y));
        let z_var = cs.new_input_variable(|| Ok(self.pub_z));

        todo!()
    }
}

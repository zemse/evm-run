use revm::primitives::U256;

pub struct VecU256(pub Vec<U256>);

impl std::fmt::Display for VecU256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s_vec = vec![];
        for i in &self.0 {
            let limbs = i.as_limbs();
            if limbs[1] == 0 && limbs[2] == 0 && limbs[3] == 0 {
                s_vec.push(format!("{:x}", limbs[0]));
            } else {
                s_vec.push(format!("{:x}", i));
            }
        }
        write!(f, "{}", s_vec.join(","))
    }
}

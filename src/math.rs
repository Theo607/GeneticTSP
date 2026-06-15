use crate::genetic::Permutation;

impl Permutation {
    pub fn invert(&mut self, mut i: usize, mut j: usize) {
        if i > j {
            std::mem::swap(&mut i, &mut j);
        }
        self.perm[i..=j].reverse();
    }
}

pub fn delta(
    adj_mat: &Vec<Vec<f32>>,
    perm: &Permutation,
    i: usize,
    j: usize
) -> f32 {
    let p = &perm.perm;
    let n = p.len();

    let (i, j) = if i > j { (j, i) } else { (i, j) };

    let i_prev = (i + n - 1) % n;
    let j_next = (j + 1) % n;

    let node_i_prev = (p[i_prev] - 1) as usize;
    let node_i      = (p[i] - 1) as usize;
    let node_j      = (p[j] - 1) as usize;
    let node_j_next = (p[j_next] - 1) as usize;

    let old_cost = adj_mat[node_i_prev][node_i] + adj_mat[node_j][node_j_next];

    let new_cost = adj_mat[node_i_prev][node_j] + adj_mat[node_i][node_j_next];
    
    new_cost - old_cost
}

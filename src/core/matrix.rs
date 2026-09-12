//! Small dense matrix operations (row-major `f64`). No external linear-algebra
//! dependency: LU solve/inverse, Cholesky factor/solve, symmetric Jacobi
//! eigensolver, matrix products. Dimensions here are small (Kalman states,
//! factor counts), so O(n^3) routines are fine and allocation-free after setup.

#[derive(Clone, Debug)]
pub struct DMat {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl DMat {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self { rows, cols, data: vec![0.0; rows * cols] }
    }
    pub fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n, n);
        for i in 0..n {
            m.data[i * n + i] = 1.0;
        }
        m
    }
    pub fn from_row_vec(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        assert_eq!(data.len(), rows * cols);
        Self { rows, cols, data }
    }
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.data[i * self.cols + j]
    }
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, v: f64) {
        self.data[i * self.cols + j] = v;
    }
    #[inline]
    pub fn scale(&mut self, k: f64) {
        for d in self.data.iter_mut() {
            *d *= k;
        }
    }
    pub fn transpose(&self) -> DMat {
        let mut t = DMat::zeros(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                t.set(j, i, self.get(i, j));
            }
        }
        t
    }
    /// self * other (matrix product).
    pub fn mul(&self, other: &DMat) -> DMat {
        assert_eq!(self.cols, other.rows, "inner dimension mismatch");
        let mut out = DMat::zeros(self.rows, other.cols);
        for i in 0..self.rows {
            for k in 0..self.cols {
                let aik = self.get(i, k);
                if aik == 0.0 {
                    continue;
                }
                for j in 0..other.cols {
                    let d = out.data[i * out.cols + j];
                    out.data[i * out.cols + j] = d + aik * other.get(k, j);
                }
            }
        }
        out
    }
    /// self * v (matrix-vector).
    pub fn mul_vec(&self, v: &[f64]) -> Vec<f64> {
        assert_eq!(self.cols, v.len());
        let mut out = vec![0.0; self.rows];
        for i in 0..self.rows {
            let mut acc = 0.0;
            let base = i * self.cols;
            for j in 0..self.cols {
                acc += self.data[base + j] * v[j];
            }
            out[i] = acc;
        }
        out
    }
    pub fn add(&self, other: &DMat) -> DMat {
        assert_eq!(self.rows, other.rows);
        assert_eq!(self.cols, other.cols);
        let mut out = self.clone();
        for (a, b) in out.data.iter_mut().zip(other.data.iter()) {
            *a += *b;
        }
        out
    }
    pub fn sub(&self, other: &DMat) -> DMat {
        assert_eq!(self.rows, other.rows);
        assert_eq!(self.cols, other.cols);
        let mut out = self.clone();
        for (a, b) in out.data.iter_mut().zip(other.data.iter()) {
            *a -= *b;
        }
        out
    }
}

/// Solve `A x = b` via LU decomposition with partial pivoting.
/// Returns `None` if `A` is singular. `a` is not modified.
pub fn lu_solve(a: &DMat, b: &[f64]) -> Option<Vec<f64>> {
    let n = a.rows;
    assert_eq!(n, a.cols, "matrix must be square");
    assert_eq!(b.len(), n);
    let mut lu = a.data.clone();
    let mut piv: Vec<usize> = (0..n).collect();
    let mut rhs = b.to_vec();

    for col in 0..n {
        // Partial pivot.
        let mut max_row = col;
        let mut max_val = lu[col * n + col].abs();
        for r in (col + 1)..n {
            let v = lu[r * n + col].abs();
            if v > max_val {
                max_val = v;
                max_row = r;
            }
        }
        if max_val < 1e-300 {
            return None;
        }
        if max_row != col {
            for k in 0..n {
                let t = lu[col * n + k];
                lu[col * n + k] = lu[max_row * n + k];
                lu[max_row * n + k] = t;
            }
            piv.swap(col, max_row);
            rhs.swap(col, max_row);
        }
        let pivot = lu[col * n + col];
        for r in (col + 1)..n {
            let factor = lu[r * n + col] / pivot;
            lu[r * n + col] = factor;
            for k in (col + 1)..n {
                lu[r * n + k] -= factor * lu[col * n + k];
            }
            rhs[r] -= factor * rhs[col];
        }
    }
    // Back substitution.
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut acc = rhs[i];
        for j in (i + 1)..n {
            acc -= lu[i * n + j] * x[j];
        }
        x[i] = acc / lu[i * n + i];
    }
    let _ = &piv;
    Some(x)
}

/// Invert a square matrix. Returns `None` if singular.
pub fn inverse(a: &DMat) -> Option<DMat> {
    let n = a.rows;
    let mut inv = DMat::zeros(n, n);
    for j in 0..n {
        let mut e = vec![0.0; n];
        e[j] = 1.0;
        let col = lu_solve(a, &e)?;
        for i in 0..n {
            inv.set(i, j, col[i]);
        }
    }
    Some(inv)
}

/// Cholesky decomposition of a symmetric positive-definite matrix.
/// Returns the lower-triangular factor `L` such that `L L^T = a`,
/// or `None` if not positive definite.
pub fn cholesky(a: &DMat) -> Option<DMat> {
    let n = a.rows;
    assert_eq!(n, a.cols);
    let mut l = DMat::zeros(n, n);
    for i in 0..n {
        for j in 0..=i {
            let mut sum = a.get(i, j);
            for k in 0..j {
                sum -= l.get(i, k) * l.get(j, k);
            }
            if i == j {
                if sum <= 0.0 {
                    return None;
                }
                l.set(i, i, sum.sqrt());
            } else {
                l.set(i, j, sum / l.get(j, j));
            }
        }
    }
    Some(l)
}

/// Solve `A x = b` for symmetric positive-definite `A` via Cholesky.
pub fn cholesky_solve(a: &DMat, b: &[f64]) -> Option<Vec<f64>> {
    let l = cholesky(a)?;
    let n = l.rows;
    // Forward solve L y = b.
    let mut y = vec![0.0; n];
    for i in 0..n {
        let mut acc = b[i];
        for j in 0..i {
            acc -= l.get(i, j) * y[j];
        }
        y[i] = acc / l.get(i, i);
    }
    // Back solve L^T x = y.
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut acc = y[i];
        for j in (i + 1)..n {
            acc -= l.get(j, i) * x[j];
        }
        x[i] = acc / l.get(i, i);
    }
    Some(x)
}

/// Symmetric eigendecomposition via cyclic Jacobi rotations.
/// Returns `(eigenvalues, eigenvectors)` where eigenvectors is a matrix whose
/// columns are the eigenvectors, sorted by descending eigenvalue.
/// Suitable for small matrices (multitaper DPSS, PCA-style methods).
pub fn jacobi_eigen(a: &DMat, max_sweeps: usize) -> (Vec<f64>, DMat) {
    let n = a.rows;
    assert_eq!(n, a.cols, "matrix must be square");
    let mut m = a.clone();
    let mut v = DMat::identity(n);

    for _ in 0..max_sweeps {
        // Find largest off-diagonal element.
        let mut p = 0usize;
        let mut q = 1usize;
        let mut max_off = 0.0f64;
        for i in 0..n {
            for j in (i + 1)..n {
                let val = m.get(i, j).abs();
                if val > max_off {
                    max_off = val;
                    p = i;
                    q = j;
                }
            }
        }
        if max_off < 1e-14 {
            break;
        }
        // Compute rotation.
        let app = m.get(p, p);
        let aqq = m.get(q, q);
        let apq = m.get(p, q);
        let theta = (aqq - app) / (2.0 * apq);
        let t = if theta >= 0.0 {
            1.0 / (theta + (1.0 + theta * theta).sqrt())
        } else {
            1.0 / (theta - (1.0 + theta * theta).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;

        // Apply rotation to M: M = G^T M G.
        for k in 0..n {
            let mkp = m.get(k, p);
            let mkq = m.get(k, q);
            m.set(k, p, c * mkp - s * mkq);
            m.set(k, q, s * mkp + c * mkq);
        }
        for k in 0..n {
            let mpk = m.get(p, k);
            let mqk = m.get(q, k);
            m.set(p, k, c * mpk - s * mqk);
            m.set(q, k, s * mpk + c * mqk);
        }
        // Accumulate eigenvectors.
        for k in 0..n {
            let vkp = v.get(k, p);
            let vkq = v.get(k, q);
            v.set(k, p, c * vkp - s * vkq);
            v.set(k, q, s * vkp + c * vkq);
        }
    }

    let mut eig: Vec<(f64, usize)> = (0..n).map(|i| (m.get(i, i), i)).collect();
    eig.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let mut values = Vec::with_capacity(n);
    let mut vectors = DMat::zeros(n, n);
    for (new_col, (_, old_col)) in eig.iter().enumerate() {
        values.push(m.get(*old_col, *old_col));
        for r in 0..n {
            vectors.set(r, new_col, v.get(r, *old_col));
        }
    }
    (values, vectors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lu_solve_2x2() {
        let a = DMat::from_row_vec(2, 2, vec![2.0, 1.0, 1.0, 3.0]);
        let x = lu_solve(&a, &[5.0, 10.0]).unwrap();
        // 2x+y=5, x+3y=10 -> x=1, y=3
        assert!((x[0] - 1.0).abs() < 1e-9);
        assert!((x[1] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn inverse_roundtrip() {
        let a = DMat::from_row_vec(2, 2, vec![4.0, 7.0, 2.0, 6.0]);
        let inv = inverse(&a).unwrap();
        let prod = a.mul(&inv);
        assert!((prod.get(0, 0) - 1.0).abs() < 1e-9);
        assert!((prod.get(1, 1) - 1.0).abs() < 1e-9);
        assert!(prod.get(0, 1).abs() < 1e-9);
    }

    #[test]
    fn cholesky_spd() {
        let a = DMat::from_row_vec(2, 2, vec![4.0, 2.0, 2.0, 3.0]);
        let l = cholesky(&a).unwrap();
        assert!((l.get(0, 0) - 2.0).abs() < 1e-12);
        assert!((l.get(1, 0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn jacobi_symmetric() {
        let a = DMat::from_row_vec(2, 2, vec![2.0, 1.0, 1.0, 2.0]);
        let (vals, _) = jacobi_eigen(&a, 100);
        // eigenvalues 3 and 1
        assert!((vals[0] - 3.0).abs() < 1e-9);
        assert!((vals[1] - 1.0).abs() < 1e-9);
    }
}

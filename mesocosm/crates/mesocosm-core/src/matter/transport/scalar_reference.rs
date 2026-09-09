// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
// Frozen ef4828b scalar Soil transport, only for tests and the benchmark.
// Keep independent of the live typed kernel so parity remains a real check.

#[derive(Clone)]
pub struct ScalarSoil {
    extent: i32,
    matter_mg: Vec<u64>,
}

impl ScalarSoil {
    pub fn seeded(extent: i32, mg: u64) -> Self {
        Self {
            extent,
            matter_mg: vec![mg; ((extent * 2 + 1) as usize).pow(2)],
        }
    }
    pub fn column_at(&self, at: [i32; 3]) -> usize {
        ((at[2] + self.extent) * (self.extent * 2 + 1) + at[0] + self.extent) as usize
    }
    pub fn deposit(&mut self, column: usize, mg: u64) {
        self.matter_mg[column] += mg;
    }
    pub fn matter_mg(&self, column: usize) -> u64 {
        self.matter_mg[column]
    }
    pub fn percolate(&mut self) {
        let side = self.extent * 2 + 1;
        let columns = self.matter_mg.len();
        let mut delta = vec![0i64; columns];
        for index in 0..columns {
            let (x, z) = (index as i32 % side, index as i32 / side);
            let neighbours: Vec<usize> = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
                .iter()
                .map(|(dx, dz)| (x + dx, z + dz))
                .filter(|(nx, nz)| (0..side).contains(nx) && (0..side).contains(nz))
                .map(|(nx, nz)| (nz * side + nx) as usize)
                .collect();
            if neighbours.is_empty() {
                continue;
            }
            let out = self.matter_mg[index] / 8;
            if out == 0 {
                continue;
            }
            let share = out / neighbours.len() as u64;
            let extra = out % neighbours.len() as u64;
            delta[index] -= out as i64;
            for (rank, neighbour) in neighbours.into_iter().enumerate() {
                delta[neighbour] += (share + u64::from((rank as u64) < extra)) as i64;
            }
        }
        for (held, change) in self.matter_mg.iter_mut().zip(delta) {
            *held = held.saturating_add_signed(change);
        }
    }
}

pub(super) struct Pattern {
    pub(super) size: usize,
    pub(super) period: i32,
    pub(super) values: Box<[f32]>,
}

fn ranked_cells(size: usize) -> impl Iterator<Item = usize> {
    let center = (size as f32 / 2.0 + 0.1, size as f32 / 2.0 + 0.15);
    let mut cells: Vec<_> = (0..size * size)
        .map(|i| {
            let dy = (i / size) as f32 - center.0;
            let dx = (i % size) as f32 - center.1;
            (i, (dy.powi(2) + dx.powi(2)).sqrt())
        })
        .collect();
    // Equal distances keep their original row-major rank.
    cells.sort_by(|a, b| b.1.total_cmp(&a.1));
    cells.into_iter().map(|(i, _)| i)
}

pub(super) fn patterns(size: usize) -> [Pattern; 2] {
    let period = 2 * size;
    let mut chess = vec![0.0; period * period];
    let step = 0.5 / ((size as f32).powi(2) - 1.0);
    for (rank, i) in ranked_cells(size).enumerate() {
        // For size 1 the historical threshold is NaN (infinity * zero).
        let value = step * rank as f32;
        let top = (i / size) * period + i % size;
        let bottom = top + size * period;
        chess[top] = 0.5 + value;
        chess[top + size] = 0.503 - value;
        chess[bottom] = chess[top + size];
        chess[bottom + size] = chess[top];
    }

    let mut grid = vec![0.0; period * period];
    let step = 1.0 / ((period as f32).powi(2) - 1.0);
    for (rank, i) in ranked_cells(period).enumerate() {
        grid[i] = if rank == 0 { 1e-4 } else { step * rank as f32 };
    }
    [
        Pattern {
            size,
            period: period as i32,
            values: chess.into_boxed_slice(),
        },
        Pattern {
            size: period,
            period: period as i32,
            values: grid.into_boxed_slice(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::patterns;

    #[test]
    fn unit_size_keeps_chess_nan_and_grid_ranks() {
        let [chess, grid] = patterns(1);
        assert_eq!((chess.size, chess.period, chess.values.len()), (1, 2, 4));
        assert!(chess.values.iter().all(|v| v.is_nan()));
        assert_eq!((grid.size, grid.period), (2, 2));
        assert_eq!(&*grid.values, &[1e-4, 2.0 / 3.0, 1.0 / 3.0, 1.0]);
    }
}

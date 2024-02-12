use winterfell::math::FieldElement;

use crate::Felem;

pub fn split_trace_with_padding<const WIDTH: usize, E: FieldElement<BaseField = Felem>>(
    full_trace: &[Vec<E>; WIDTH],
    n_segments: usize,
    segment_len: usize,
) -> Vec<[Vec<E>; WIDTH]> {
    let mut result: Vec<[Vec<E>; WIDTH]> = Vec::new();
    for i in 0..n_segments {
        let mut trace = Vec::new();
        for col in 0..WIDTH {
            let start = i * (segment_len - 2);
            let end = start + segment_len - 1;
            trace.push(full_trace[col][start..end].to_vec());
            // pad the last row of each segment
            use miden_processor::crypto::RandomCoin;
            use miden_processor::crypto::RpoRandomCoin;
            let mut rng = RpoRandomCoin::new(&[(1u32.into())]);
            trace
                .last_mut()
                .unwrap()
                .push(rng.draw().expect("failed to draw a random value"));
        }
        assert_eq!(trace.len(), WIDTH);
        assert!(trace.iter().all(|col| col.len() == segment_len));
        result.push(trace.try_into().unwrap());
    }

    result
}

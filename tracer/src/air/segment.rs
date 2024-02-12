use winterfell::{
    math::{FieldElement, ToElements},
    Assertion,
};

#[derive(Clone, Debug, Copy)]
pub enum SegmentConfig {
    // Proof with a single segment
    Single,
    // Proof split in multiple segments
    Split { segment_len: u32 },
}

#[derive(Clone, Debug, Copy)]
pub struct Segment {
    // which segment this proof is for
    pub segment_n: u32,
}

impl Segment {
    pub fn filter_assertions_for_segment<E: FieldElement>(
        &self,
        segment_len: u32,
        assertions: &[Assertion<E>],
    ) -> Vec<Assertion<E>> {
        // the number for usable rows in a segment is one less than the segmet length, because the last
        // fow is used for padding to ensure the degree of the constraints
        let segment_start = self.segment_n * (segment_len - 1);
        assertions
            .iter()
            .filter(|assertion| {
                assertion.first_step() >= segment_start as usize
                    && assertion.first_step() < (segment_start + segment_len - 1) as usize
            })
            // we need to 'shift' the row of each assertion since this segment first row is actually the segment_start-th row
            // of the complete computation
            .map(|assertion| {
                assert!(assertion.is_single()); // we only support splitting single assertions for now
                Assertion::single(
                    assertion.column(),
                    assertion.first_step() - segment_start as usize,
                    assertion.values()[0],
                )
            })
            .collect()
    }
}

impl<E: FieldElement> ToElements<E> for Segment {
    fn to_elements(&self) -> Vec<E> {
        vec![E::from(self.segment_n)]
    }
}

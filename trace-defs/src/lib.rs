mod defs;
pub use defs::*;

use miden_processor::chiplets::aux_trace::AuxTraceBuilder;
use winterfell::{
    math::{fields::f64::BaseElement, FieldElement, StarkField},
    Air, AuxTraceRandElements, ColMatrix, EvaluationFrame, Trace, TraceInfo, TraceLayout,
};

const NUM_ALPHA_ELEMS: usize = 9;

pub struct TraceTable<E: StarkField> {
    inner: winterfell::TraceTable<E>,
    aux: AuxTraceBuilder,
    layout: TraceLayout,
}

impl<Field: StarkField> TraceTable<Field> {
    pub fn new(trace: winterfell::TraceTable<Field>, aux: AuxTraceBuilder) -> Self {
        let layout = TraceLayout::new(MAIN_TRACE_WIDTH, [AUX_TRACE_WIDTH; 1], [NUM_ALPHA_ELEMS; 1]);
        Self {
            inner: trace,
            layout,
            aux,
        }
    }
}

impl Trace for TraceTable<BaseElement> {
    type BaseField = BaseElement;

    // Required methods
    fn layout(&self) -> &TraceLayout {
        &self.layout
    }
    fn length(&self) -> usize {
        self.inner.length()
    }
    fn meta(&self) -> &[u8] {
        self.inner.meta()
    }
    fn main_segment(&self) -> &ColMatrix<Self::BaseField> {
        self.inner.main_segment()
    }
    fn build_aux_segment<E>(
        &mut self,
        aux_segments: &[ColMatrix<E>],
        rand_elements: &[E],
    ) -> Option<ColMatrix<E>>
    where
        E: FieldElement<BaseField = Self::BaseField>,
    {
        // we only have one auxiliary segment
        if !aux_segments.is_empty() {
            return None;
        }

        // add the running product columns for the chiplets
        let mut bus = self
            .aux
            .build_aux_columns(self.main_segment(), rand_elements)
            // (we only need the bus column)
            .remove(0);
        println!("{:?}", bus.len());
        // // inject random values into the last rows of the trace
        use miden_processor::crypto::RandomCoin;
        use miden_processor::crypto::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(&[(1u32.into())]);
        for i in self.length() - 1..self.length() {
            bus[i] = rng.draw().expect("failed to draw a random value");
        }

        Some(ColMatrix::new(vec![bus]))
    }

    fn read_main_frame(&self, row_idx: usize, frame: &mut EvaluationFrame<Self::BaseField>) {
        self.inner.read_main_frame(row_idx, frame)
    }

    // Provided methods
    fn get_info(&self) -> TraceInfo {
        TraceInfo::new_multi_segment(self.layout.clone(), self.length(), vec![])
    }
    fn main_trace_width(&self) -> usize {
        assert!(self.inner.main_trace_width() == MAIN_TRACE_WIDTH);
        self.inner.main_trace_width()
    }
    fn aux_trace_width(&self) -> usize {
        AUX_TRACE_WIDTH
    }
    fn validate<A, E>(
        &self,
        air: &A,
        aux_segments: &[ColMatrix<E>],
        aux_rand_elements: &AuxTraceRandElements<E>,
    ) where
        A: Air<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField>,
    {
        // we can't use inner.validate as it is as it will try to validate aux constraints for columns which are missing
        // in 'inner'
        // self.inner
        //     .validate(air, &[], &<AuxTraceRandElements<E>>::new())
        // TODO: validate
    }
}

impl<Field: StarkField> TraceTable<Field> {
    /// Reads a single row from this execution trace into the provided target.
    pub fn read_row_into(&self, step: usize, target: &mut [Field]) {
        self.inner.read_row_into(step, target);
    }

    pub fn update_row(&mut self, step: usize, state: &[Field]) {
        self.inner.update_row(step, state);
    }
}

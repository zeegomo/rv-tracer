use trace_defs::*;

use winterfell::{
    math::{fields::f64::BaseElement, FieldElement, StarkField},
    matrix::ColMatrix,
    Air, AuxTraceRandElements, EvaluationFrame, Trace, TraceInfo, TraceLayout,
};

const NUM_ALPHA_ELEMS: usize = 0;

#[derive(Debug, Clone)]
pub struct TraceTable<E: StarkField> {
    inner: winterfell::TraceTable<E>,
    layout: TraceLayout,
}

impl<Field: StarkField> TraceTable<Field> {
    pub fn new(trace: Vec<Vec<Field>>) -> Self {
        let layout = TraceLayout::new(MAIN_TRACE_WIDTH, [AUX_TRACE_WIDTH; 1], [NUM_ALPHA_ELEMS; 1]);
        Self {
            inner: winterfell::TraceTable::init(trace),
            layout,
        }
    }
}

impl Trace for TraceTable<BaseElement> {
    type BaseField = BaseElement;

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
        _aux_segments: &[ColMatrix<E>],
        _rand_elements: &[E],
    ) -> Option<ColMatrix<E>>
    where
        E: FieldElement<BaseField = Self::BaseField>,
    {
        panic!("no aux segment in table")
    }

    fn read_main_frame(&self, row_idx: usize, frame: &mut EvaluationFrame<Self::BaseField>) {
        self.inner.read_main_frame(row_idx, frame)
    }

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
        _aux_segments: &[ColMatrix<E>],
        aux_rand_elements: &AuxTraceRandElements<E>,
    ) where
        A: Air<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField>,
    {
        self.inner.validate(air, &[], aux_rand_elements)
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

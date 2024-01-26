use miden_processor::{chiplets, range};
use trace_defs::*;

use winterfell::{
    math::{fields::f64::BaseElement, FieldElement, StarkField},
    Air, AuxTraceRandElements, ColMatrix, EvaluationFrame, Trace, TraceInfo, TraceLayout,
};

const NUM_ALPHA_ELEMS: usize = 9;
const NUM_RAND_ROWS: usize = 1;

pub struct TraceTable<E: StarkField> {
    inner: winterfell::TraceTable<E>,
    layout: TraceLayout,
    aux_builder: AuxTraceBuilder,
}

pub struct AuxTraceBuilder {
    mem: chiplets::aux_trace::AuxTraceBuilder,
    range: range::AuxTraceBuilder,
}

impl AuxTraceBuilder {
    pub fn new(mem: chiplets::aux_trace::AuxTraceBuilder, range: range::AuxTraceBuilder) -> Self {
        Self { mem, range }
    }
}

impl<Field: StarkField> TraceTable<Field> {
    pub fn new(trace: Vec<Vec<Field>>, aux_builder: AuxTraceBuilder) -> Self {
        let layout = TraceLayout::new(MAIN_TRACE_WIDTH, [AUX_TRACE_WIDTH; 1], [NUM_ALPHA_ELEMS; 1]);
        Self {
            inner: winterfell::TraceTable::init(trace),
            layout,
            aux_builder,
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
        let bus = self
            .aux_builder
            .mem
            .build_memory_aux_column(self.main_segment(), rand_elements);

        // add the range check columns
        let range = self
            .aux_builder
            .range
            .build_aux_columns(self.main_segment(), rand_elements);

        let mut aux_columns = vec![bus].into_iter().chain(range).collect::<Vec<_>>();
        // // inject random values into the last rows of the trace
        use miden_processor::crypto::RandomCoin;
        use miden_processor::crypto::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(&[(1u32.into())]);

        // inject random values into the last rows of the trace
        for i in self.length() - NUM_RAND_ROWS..self.length() {
            for column in aux_columns.iter_mut() {
                column[i] = rng.draw().expect("failed to draw a random value");
            }
        }

        Some(ColMatrix::new(aux_columns))
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
        _aux_rand_elements: &AuxTraceRandElements<E>,
    ) where
        A: Air<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField>,
    {
        // println!("{:?}", air.get_assertions());
        // first, check assertions against the main segment of the execution trace
        for assertion in air.get_assertions() {
            assertion.apply(self.length(), |step, value| {
                assert!(
                    value == self.main_segment().get(assertion.column(), step),
                    "trace does not satisfy assertion main_trace({}, {}) == {}",
                    assertion.column(),
                    step,
                    value
                );
            });
        }
        // self.inner.validate(air, &[], aux_rand_elements)
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

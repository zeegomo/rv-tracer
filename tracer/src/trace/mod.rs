use miden_processor::{chiplets, range};
use trace_defs::*;

use std::rc::Rc;
use winterfell::{
    math::{fields::f64::BaseElement, FieldElement, StarkField},
    Air, AuxTraceRandElements, ColMatrix, EvaluationFrame, Trace, TraceInfo, TraceLayout,
};

const NUM_ALPHA_ELEMS: usize = 9;
const NUM_RAND_ROWS: usize = 1;

pub struct TraceTable<E: StarkField> {
    inner: winterfell::TraceTable<E>,
    layout: TraceLayout,
    aux_builder: AuxTraceBuilder<E>,
}

#[derive(Clone)]
pub struct AuxTraceBuilder<E: StarkField> {
    mem: chiplets::aux_trace::AuxTraceBuilder,
    range: range::AuxTraceBuilder,
    full_trace: Option<Rc<winterfell::TraceTable<E>>>,
    // if this trace is segment of a bigger trace, we build the full columns and then skip the first `skip` elements
    // and take the next `length` elements
    skip: Option<usize>,
    length: Option<usize>,
}

impl<E: StarkField> AuxTraceBuilder<E> {
    pub fn new(mem: chiplets::aux_trace::AuxTraceBuilder, range: range::AuxTraceBuilder) -> Self {
        Self {
            mem,
            range,
            full_trace: None,
            skip: None,
            length: None,
        }
    }

    pub fn segmented(
        mut self,
        full_trace: Rc<winterfell::TraceTable<E>>,
        segment_n: usize,
        segment_len: usize,
    ) -> Self {
        self.full_trace = Some(full_trace);
        self.skip = Some((segment_len - 2) * segment_n);
        self.length = Some(segment_len);
        self
    }
}

impl<Field: StarkField> TraceTable<Field> {
    pub fn new(trace: Vec<Vec<Field>>, aux_builder: AuxTraceBuilder<Field>) -> Self {
        let layout = TraceLayout::new(MAIN_TRACE_WIDTH, [AUX_TRACE_WIDTH; 1], [NUM_ALPHA_ELEMS; 1]);
        Self {
            inner: winterfell::TraceTable::init(trace),
            layout,
            aux_builder,
        }
    }
}

impl<E: StarkField> std::fmt::Display for TraceTable<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "TraceTable")?;
        let length = self.inner.get_column(0).len();
        for i in 0..MAIN_TRACE_WIDTH {
            for j in 0..length {
                write!(f, "{} ", self.inner.get(i, j))?;
            }
            writeln!(f)?;
        }
        Ok(())
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
        &[]
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

        let main_segment = if self.aux_builder.full_trace.is_some() {
            self.aux_builder.full_trace.as_ref().unwrap().main_segment()
        } else {
            self.main_segment()
        };

        // add the running product columns for the chiplets
        let bus = self
            .aux_builder
            .mem
            .build_memory_aux_column(main_segment, rand_elements);

        // add the range check columns
        let range = self
            .aux_builder
            .range
            .build_aux_columns(main_segment, rand_elements);

        let dummy_column = vec![E::ZERO; self.length()];
        let mut aux_columns = vec![bus].into_iter().chain(range).collect::<Vec<_>>();
        // // inject random values into the last rows of the trace
        use miden_processor::crypto::RandomCoin;
        use miden_processor::crypto::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(&[(1u32.into())]);

        for column in &mut aux_columns {
            if let Some(skip) = self.aux_builder.skip {
                *column = column
                    .iter()
                    .skip(skip)
                    .take(self.aux_builder.length.unwrap())
                    .copied()
                    .collect();

                if column.len() < self.length() {
                    column.extend(core::iter::repeat(E::ONE).take(self.length() - column.len()));
                }
            }
        }

        aux_columns.push(dummy_column);
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

impl TraceTable<BaseElement> {
    /// Reads a single row from this execution trace into the provided target.
    pub fn read_row_into(&self, step: usize, target: &mut [BaseElement]) {
        self.inner.read_row_into(step, target);
    }

    pub fn get(&self, column: usize, step: usize) -> BaseElement {
        self.inner.get(column, step)
    }

    pub fn update_row(&mut self, step: usize, state: &[BaseElement]) {
        self.inner.update_row(step, state);
    }
}

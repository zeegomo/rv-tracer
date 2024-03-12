use miden_processor::{chiplets, range};
use trace_defs::*;

use std::rc::Rc;
use winterfell::{
    math::{fields::f64::BaseElement, FieldElement, StarkField},
    Air, AuxTraceRandElements, ColMatrix, EvaluationFrame, Trace, TraceInfo, TraceLayout,
};

const NUM_ALPHA_ELEMS: usize = 9;
const NUM_RAND_ROWS: usize = 1;

#[derive(Clone)]
pub struct TraceTable<E: StarkField> {
    inner: Rc<winterfell::TraceTable<E>>,
    segment_n: usize,
    segment_len: usize,
    layout: TraceLayout,
    aux_builder: AuxTraceBuilder<E>,
    seg: ColMatrix<E::BaseField>,
}

#[derive(Clone)]
pub struct AuxTraceBuilder<E: StarkField> {
    mem: Rc<chiplets::aux_trace::AuxTraceBuilder>,
    range: Rc<range::AuxTraceBuilder>,
    full_trace: Option<Rc<winterfell::TraceTable<E>>>,
    // if this trace is segment of a bigger trace, we build the full columns and then skip the first `skip` elements
    // and take the next `length` elements
    skip: Option<usize>,
    length: Option<usize>,
}

impl<E: StarkField> AuxTraceBuilder<E> {
    pub fn new(mem: chiplets::aux_trace::AuxTraceBuilder, range: range::AuxTraceBuilder) -> Self {
        Self {
            mem: Rc::new(mem),
            range: Rc::new(range),
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
            segment_len: trace[0].len(),
            inner: Rc::new(winterfell::TraceTable::init(trace.clone())),
            layout,
            aux_builder,
            segment_n: 0,
            seg: ColMatrix::new(trace),
        }
    }

    pub fn new_segmented(
        trace: Rc<winterfell::TraceTable<Field>>,
        aux_builder: AuxTraceBuilder<Field>,
        segment_n: usize,
        segment_len: usize,
    ) -> Self {
        let layout = TraceLayout::new(MAIN_TRACE_WIDTH, [AUX_TRACE_WIDTH; 1], [NUM_ALPHA_ELEMS; 1]);
        Self {
            inner: trace.clone(),
            layout,
            aux_builder: aux_builder.segmented(trace, segment_n, segment_len),
            segment_n,
            segment_len,
            seg: ColMatrix::new(vec![vec![Field::BaseField::ZERO; 2]]),
        }
    }
}

impl<E: StarkField> std::fmt::Display for TraceTable<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "TraceTable")?;
        for i in 0..MAIN_TRACE_WIDTH {
            for j in 0..self.segment_len {
                write!(
                    f,
                    "{} ",
                    self.inner
                        .get(i, j + (self.segment_len - 2) * self.segment_n)
                )?;
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
        self.segment_len
    }

    fn meta(&self) -> &[u8] {
        &[]
    }

    fn main_segment(&self) -> &ColMatrix<Self::BaseField> {
        // TODO: single segment proofs should not pay the cost of cloning the segment
        self.assert_segment_built();
        if self.segment_len == self.inner.length() {
            return self.inner.main_segment();
        }
        &self.seg
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
        let main_frame = self.main_segment();
        let next_row_idx = (row_idx + 1) % self.length();
        main_frame.read_row_into(row_idx, frame.current_mut());
        main_frame.read_row_into(next_row_idx, frame.next_mut());
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
        self.main_segment().read_row_into(step, target);
    }

    fn assert_segment_built(&self) {
        if self.segment_len == self.inner.length() {
            return;
        }
        assert_eq!(self.segment_len, self.seg.num_rows(), "segment not built");
    }

    pub fn get(&self, column: usize, step: usize) -> BaseElement {
        self.main_segment().get(column, step)
    }

    pub fn update_row(&mut self, step: usize, state: &[BaseElement]) {
        self.assert_segment_built();
        if self.segment_len == self.seg.num_rows() {
            Rc::get_mut(&mut self.inner)
                .unwrap()
                .update_row(step, state);
            return;
        }
        self.seg.update_row(step, state);
    }

    // Clone the this segment trace table from the full trace table and overwrite the last row
    // with random padding
    // This is necessary because we must return a ColMatrix which has to own its data
    pub fn build_segment(&mut self) {
        if self.segment_len == self.seg.num_rows() {
            return;
        }
        let mut cols = vec![vec![BaseElement::ZERO; self.segment_len]; MAIN_TRACE_WIDTH];
        for (i, col) in cols.iter_mut().enumerate() {
            for (j, elem) in col.iter_mut().enumerate() {
                *elem = self
                    .inner
                    .get(i, j + (self.segment_len - 2) * self.segment_n);
            }
        }

        use miden_processor::crypto::RandomCoin;
        use miden_processor::crypto::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(&[(1u32.into())]);

        for col in &mut cols {
            *col.last_mut().unwrap() = rng.draw().expect("failed to draw a random value");
        }

        self.seg = ColMatrix::new(cols);
    }

    // Drop the segment trace table to free up memory
    pub fn drop_segment(&mut self) {
        self.seg = ColMatrix::new(vec![vec![BaseElement::ZERO; 2]]);
    }
}

use miden_processor::{chiplets, range};
use trace_defs::*;

use winterfell::{
    math::{fields::f64::BaseElement, FieldElement, StarkField},
    Air, AuxTraceRandElements, ColMatrix, EvaluationFrame, Trace, TraceInfo, TraceLayout,
};

const NUM_ALPHA_ELEMS: usize = 9;
const NUM_RAND_ROWS: usize = 1;

#[derive(Clone)]
pub struct TraceTable<E: StarkField> {
    inner: winterfell::TraceTable<E>,
    layout: TraceLayout,
    aux_builder: AuxTraceBuilder,
}

#[derive(Clone)]
pub struct AuxTraceBuilder {
    mem: chiplets::aux_trace::AuxTraceBuilder,
    full_main: winterfell::TraceTable<BaseElement>,
    range: range::AuxTraceBuilder,
    // if this trace is segment of a bigger trace, we build the full columns and then skip the first `skip` elements
    // and take the next `length` elements
    skip: Option<usize>,
    length: Option<usize>,
}

impl AuxTraceBuilder {
    pub fn new(
        mem: chiplets::aux_trace::AuxTraceBuilder,
        range: range::AuxTraceBuilder,
        full_main: Vec<Vec<BaseElement>>,
    ) -> Self {
        Self {
            mem,
            range,
            full_main: winterfell::TraceTable::init(full_main),
            skip: None,
            length: None,
        }
    }

    pub fn set_skip(&mut self, skip: usize) {
        self.skip = Some(skip);
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = Some(length);
    }
}

impl<Field: StarkField> TraceTable<Field> {
    pub fn new(trace: Vec<Vec<Field>>, mut aux_builder: AuxTraceBuilder) -> Self {
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
            .build_memory_aux_column(self.aux_builder.full_main.main_segment(), rand_elements);

        // add the range check columns
        let range = self
            .aux_builder
            .range
            .build_aux_columns(self.aux_builder.full_main.main_segment(), rand_elements);

        let mut aux_columns = vec![bus].into_iter().chain(range).collect::<Vec<_>>();
        // // inject random values into the last rows of the trace
        use miden_processor::crypto::RandomCoin;
        use miden_processor::crypto::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(&[(1u32.into())]);

        println!("{:?} {:?}", self.aux_builder.skip, self.aux_builder.length);

        for column in &mut aux_columns {
            if let Some(skip) = self.aux_builder.skip {
                println!("skipping");
                *column = column
                    .iter()
                    .skip(skip)
                    .take(self.aux_builder.length.unwrap())
                    .copied()
                    .collect();
            }
        }

        // inject random values into the last rows of the trace
        for i in self.length() - NUM_RAND_ROWS..self.length() {
            for column in aux_columns.iter_mut() {
                column[i] = rng.draw().expect("failed to draw a random value");
            }
        }

        println!(
            "{:?}",
            aux_columns.iter().map(|c| c.len()).collect::<Vec<_>>()
        );

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

impl TraceTable<BaseElement> {
    /// Reads a single row from this execution trace into the provided target.
    pub fn read_row_into(&self, step: usize, target: &mut [BaseElement]) {
        self.inner.read_row_into(step, target);
    }

    pub fn update_row(&mut self, step: usize, state: &[BaseElement]) {
        self.inner.update_row(step, state);
    }

    pub fn split(self, segment_size: usize) -> Vec<Self> {
        assert!(segment_size.is_power_of_two());
        println!("trace_info: {:?}", self.get_info());
        // we need at least 1 row in each table that needs to be filled with random values
        let available_segment_size = segment_size - 1;
        // TODO: we don't have to replicate all padding rows in the, it would be enough to calculate this
        // using the length of the 'real' trace
        let num_tables = (self.length() + available_segment_size - 1) / available_segment_size;
        assert!(num_tables > 0);
        let mut tables: Vec<Vec<_>> = vec![Vec::new(); num_tables];
        let main_trace_width = self.main_trace_width();
        for (n_table, table) in tables.iter_mut().enumerate() {
            for i in 0..main_trace_width {
                table.push(
                    self.inner
                        .get_column(i)
                        .iter()
                        .skip(n_table * (available_segment_size))
                        .take(available_segment_size)
                        .copied()
                        .collect::<Vec<_>>(),
                );

                use miden_processor::crypto::RandomCoin;
                use miden_processor::crypto::RpoRandomCoin;
                let mut rng = RpoRandomCoin::new(&[(1u32.into())]);
                let col = table.last_mut().unwrap();
                let col_size = col.len();
                for _ in 0..segment_size - col_size {
                    col.push(rng.draw().expect("failed to draw a random value"));
                }
            }
        }

        let tables = tables
            .into_iter()
            .enumerate()
            .map(|(i, table)| {
                let mut aux_builder = self.aux_builder.clone();
                aux_builder.set_length(segment_size);
                aux_builder.set_skip(i * available_segment_size);
                println!("width: {}, len: {}", table.len(), table[0].len());
                Self::new(table, aux_builder)
            })
            .collect::<Vec<_>>();

        for table in tables.iter() {
            println!(
                "trace_info: {:?} {:?}",
                table.get_info(),
                &table.aux_builder.skip
            );
        }

        tables
    }
}

use crate::{
    air::{self, proof::StarkProof, Inputs, RiscvAir},
    trace::{self, TraceTable},
};
mod channel;
use channel::ProverChannel;
use core::marker::PhantomData;
use std::vec;
use trace_defs::MAIN_TRACE_WIDTH;
use winter_air::proof::Queries;
use winter_fri::FriProof;
use winter_prover::*;
use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::{fft::infer_degree, fields::f64::BaseElement, FieldElement, StarkField},
    Air, AuxTraceRandElements, ColMatrix, ProofOptions, Prover, ProverError, Trace,
    TraceCommitment, TraceInfo,
};

// This should be AUX_TRACE_WIDTH * EXTENSION_DEGREE
// TODO: use constant evaluation when it's stable
const AUX_SEGMENT_WIDTH: usize = 6;
const CONSTRAINTS_SEGMENT_WIDTH: usize = 32;

type Channel<'a, E, P> =
    ProverChannel<'a, <P as Prover>::Air, E, <P as Prover>::HashFn, <P as Prover>::RandomCoin>;

pub struct RiscvProver<
    H: ElementHasher<BaseField = BaseElement>,
    E: FieldElement<BaseField = BaseElement>,
> {
    options: ProofOptions,
    inputs: Inputs,
    _hasher: PhantomData<H>,
    cache: Vec<TraceInterpolations<E>>,
    trace_info: Option<TraceInfo>,
}

struct TraceInterpolations<E: FieldElement<BaseField = BaseElement>> {
    rand_elements: AuxTraceRandElements<E>,
}

impl<H, E> RiscvProver<H, E>
where
    H: ElementHasher<BaseField = BaseElement>,
    E: FieldElement<BaseField = BaseElement>,
{
    pub fn new(options: ProofOptions, inputs: Inputs) -> Self {
        Self {
            options,
            _hasher: PhantomData,
            inputs,
            cache: Vec::new(),
            trace_info: None,
        }
    }

    pub fn prove_segmented(
        &mut self,
        mut traces: Vec<TraceTable<E::BaseField>>,
    ) -> Result<(StarkProof, Vec<StarkProof>), ProverError> {
        let t = &traces[0];
        let air = <Self as Prover>::Air::new(
            t.get_info(),
            self.get_pub_inputs(t),
            self.options().clone(),
        );
        let mut channel = ProverChannel::<
            <Self as Prover>::Air,
            E,
            <Self as Prover>::HashFn,
            <Self as Prover>::RandomCoin,
        >::new(&air, vec![]);

        self.commit_main_traces(&mut channel, &mut traces);

        let mut rand_elements = AuxTraceRandElements::new();
        rand_elements.add_segment_elements(channel.get_aux_trace_segment_rand_elements(0));
        self.commit_aux_traces(
            &mut channel,
            rand_elements.get_segment_elements(0),
            &mut traces,
        );

        let constraint_coeffs = channel.get_constraint_composition_coeffs();
        self.commit_constraints(
            &mut channel,
            rand_elements.clone(),
            constraint_coeffs.clone(),
            &mut traces,
        );

        let deep_evaluations = self.build_deep_composition_poly(
            &mut channel,
            rand_elements.clone(),
            constraint_coeffs.clone(),
            &mut traces,
        );

        let (fri_proof, trace_queries, constraint_queries) = self.build_fri_proof(
            &mut channel,
            &air,
            &mut traces,
            rand_elements.clone(),
            constraint_coeffs.clone(),
            deep_evaluations,
        );

        Ok((
            channel.build_proof(trace_queries, constraint_queries, fri_proof),
            vec![],
        ))
    }

    fn commit_main_traces(
        &mut self,
        channel: &mut ProverChannel<
            <Self as Prover>::Air,
            E,
            <Self as Prover>::HashFn,
            <Self as Prover>::RandomCoin,
        >,
        traces: &mut [TraceTable<E::BaseField>],
    ) {
        let t = &traces[0];
        let air = <Self as Prover>::Air::new(
            t.get_info(),
            self.get_pub_inputs(t),
            self.options().clone(),
        );
        let domain = StarkDomain::new(&air);
        for t in traces {
            t.build_segment();
            channel.commit_trace(
                *self
                    .build_trace_commitment::<<Self as Prover>::BaseField, MAIN_TRACE_WIDTH>(
                        t.main_segment(),
                        &domain,
                    )
                    .1
                    .root(),
            );
            t.drop_segment()
        }
    }

    fn commit_aux_traces(
        &mut self,
        channel: &mut ProverChannel<
            <Self as Prover>::Air,
            E,
            <Self as Prover>::HashFn,
            <Self as Prover>::RandomCoin,
        >,
        rand_elements: &[E],
        traces: &mut [TraceTable<E::BaseField>],
    ) {
        let t = &traces[0];
        let air = <Self as Prover>::Air::new(
            t.get_info(),
            self.get_pub_inputs(t),
            self.options().clone(),
        );
        let domain = StarkDomain::new(&air);
        for t in traces {
            t.build_segment();
            let aux_segment = t.build_aux_segment(&[], rand_elements).unwrap();
            let (lde, tree, _) =
                self.build_trace_commitment::<E, AUX_SEGMENT_WIDTH>(&aux_segment, &domain);
            channel.commit_trace(*tree.root());
            t.drop_segment();
        }
    }

    fn commit_constraints(
        &mut self,
        channel: &mut ProverChannel<
            <Self as Prover>::Air,
            E,
            <Self as Prover>::HashFn,
            <Self as Prover>::RandomCoin,
        >,
        rand_elements: AuxTraceRandElements<E>,
        constraint_coeffs: ConstraintCompositionCoefficients<E>,
        traces: &mut [TraceTable<E::BaseField>],
    ) {
        let t = &traces[0];
        let air = <Self as Prover>::Air::new(
            t.get_info(),
            self.get_pub_inputs(t),
            self.options().clone(),
        );
        let domain = StarkDomain::new(&air);
        let rand_elems = rand_elements.get_segment_elements(0);
        println!("prover rand_elems: {rand_elems:?}");

        for (i, trace) in traces.iter_mut().enumerate() {
            let mut inputs = self.get_pub_inputs(trace);
            inputs.segment.segment_n = i as u32;
            println!("{:?}", trace.get_info());
            let air = <Self as Prover>::Air::new(trace.get_info(), inputs, self.options().clone());
            let mut constraint_coeffs = constraint_coeffs.clone();
            constraint_coeffs
                .boundary
                .truncate(air.context().num_assertions());
            let commitment = self.get_commitment(trace, rand_elems, &domain);
            let evaluator =
                ConstraintEvaluator::new(&air, rand_elements.clone(), constraint_coeffs);
            let constraint_evaluations = evaluator.evaluate(commitment.trace_table(), &domain);
            #[cfg(feature = "std")]
            debug!(
                "Evaluated constraints over domain of 2^{} elements in {} ms",
                constraint_evaluations.num_rows().ilog2(),
                now.elapsed().as_millis()
            );

            // 3 ----- commit to constraint evaluations -----------------------------------------------

            // first, build constraint composition polynomial from the constraint evaluation table:
            // - divide all constraint evaluation columns by their respective divisors
            // - combine them into a single column of evaluations,
            // - interpolate the column into a polynomial in coefficient form
            // - "break" the polynomial into a set of column polynomials each of degree equal to
            //   trace_length - 1
            #[cfg(feature = "std")]
            let now = Instant::now();
            let composition_poly = constraint_evaluations
                .into_poly(air.context().num_constraint_composition_columns())
                .unwrap();
            #[cfg(feature = "std")]
            debug!(
            "Converted constraint evaluations into {} composition polynomial columns of degree {} in {} ms",
            composition_poly.num_columns(),
            composition_poly.column_degree(),
            now.elapsed().as_millis()
        );

            // then, build a commitment to the evaluations of the composition polynomial columns
            let constraint_commitment: ConstraintCommitment<E, H> = self
                .build_constraint_commitment::<E, CONSTRAINTS_SEGMENT_WIDTH>(
                    &composition_poly,
                    &domain,
                );

            // then, commit to the evaluations of constraints by writing the root of the constraint
            // Merkle tree into the channel
            channel.commit_constraints(constraint_commitment.root());
        }
    }

    fn build_deep_composition_poly(
        &mut self,
        channel: &mut ProverChannel<
            <Self as Prover>::Air,
            E,
            <Self as Prover>::HashFn,
            <Self as Prover>::RandomCoin,
        >,
        rand_elements: AuxTraceRandElements<E>,
        constraint_coeffs: ConstraintCompositionCoefficients<E>,
        traces: &mut [TraceTable<E::BaseField>],
    ) -> Vec<E> {
        let t = &traces[0];
        let air = <Self as Prover>::Air::new(
            t.get_info(),
            self.get_pub_inputs(t),
            self.options().clone(),
        );
        let domain = StarkDomain::new(&air);
        // draw an out-of-domain point z. Depending on the type of E, the point is drawn either
        // from the base field or from an extension field defined by E.
        //
        // The purpose of sampling from the extension field here (instead of the base field) is to
        // increase security. Soundness is limited by the size of the field that the random point
        // is drawn from, and we can potentially save on performance by only drawing this point
        // from an extension field, rather than increasing the size of the field overall.
        let z = channel.get_ood_point();

        println!("prover z: {z}");
        let mut deep_composition_poly =
            DeepCompositionPoly::new(z, channel.get_deep_composition_coeffs());

        for (i, trace) in traces.iter_mut().enumerate() {
            let trace_polys = self.get_polys(trace, rand_elements.get_segment_elements(0), &domain);
            let commitment =
                self.get_commitment(trace, rand_elements.get_segment_elements(0), &domain);
            let mut inputs = self.get_pub_inputs(trace);
            inputs.segment.segment_n = i as u32;
            let air = <Self as Prover>::Air::new(trace.get_info(), inputs, self.options().clone());
            // evaluate trace and constraint polynomials at the OOD point z, and send the results to
            // the verifier. the trace polynomials are actually evaluated over two points: z and z * g,
            // where g is the generator of the trace domain.
            let ood_trace_states = trace_polys.get_ood_frame(z);
            channel.send_ood_trace_states(&ood_trace_states);

            let mut constraint_coeffs = constraint_coeffs.clone();
            constraint_coeffs
                .boundary
                .truncate(air.context().num_assertions());
            let composition_poly = self.get_constraint_poly(
                &air,
                commitment.trace_table(),
                rand_elements.clone(),
                constraint_coeffs,
                &domain,
            );

            let ood_evaluations = composition_poly.evaluate_at(z);
            channel.send_ood_constraint_evaluations(&ood_evaluations);

            // draw random coefficients to use during DEEP polynomial composition, and use them to
            // initialize the DEEP composition polynomial
            let deep_coefficients = channel.get_deep_composition_coeffs();
            deep_composition_poly.cc = deep_coefficients;
            // combine all trace polynomials together and merge them into the DEEP composition
            // polynomial
            deep_composition_poly.add_trace_polys(&trace_polys, ood_trace_states);

            // merge columns of constraint composition polynomial into the DEEP composition polynomial;
            deep_composition_poly.add_composition_poly(composition_poly, ood_evaluations);
        }

        #[cfg(feature = "std")]
        debug!(
            "Built DEEP composition polynomial of degree {} in {} ms",
            deep_composition_poly.degree(),
            now.elapsed().as_millis()
        );

        // make sure the degree of the DEEP composition polynomial is equal to trace polynomial
        // degree minus 1.
        assert_eq!(domain.trace_length() - 2, deep_composition_poly.degree());

        // 5 ----- evaluate DEEP composition polynomial over LDE domain ---------------------------
        #[cfg(feature = "std")]
        let now = Instant::now();
        let deep_evaluations = deep_composition_poly.evaluate(&domain);
        // we check the following condition in debug mode only because infer_degree is an expensive
        // operation
        debug_assert_eq!(
            domain.trace_length() - 2,
            infer_degree(&deep_evaluations, domain.offset())
        );
        #[cfg(feature = "std")]
        debug!(
            "Evaluated DEEP composition polynomial over LDE domain (2^{} elements) in {} ms",
            domain.lde_domain_size().ilog2(),
            now.elapsed().as_millis()
        );
        deep_evaluations
    }

    fn build_fri_proof(
        &mut self,
        channel: &mut ProverChannel<
            <Self as Prover>::Air,
            E,
            <Self as Prover>::HashFn,
            <Self as Prover>::RandomCoin,
        >,
        air: &<Self as Prover>::Air,
        traces: &mut [TraceTable<E::BaseField>],
        rand_elements: AuxTraceRandElements<E>,
        constraint_coeffs: ConstraintCompositionCoefficients<E>,
        deep_evaluations: Vec<E>,
    ) -> (FriProof, Vec<Vec<Queries>>, Vec<Queries>) {
        let mut fri_prover = FriProver::new(air.options().to_fri_options());
        fri_prover.build_layers(channel, deep_evaluations);
        channel.grind_query_seed();
        let query_positions = channel.get_query_positions();
        let domain = StarkDomain::new(air);

        // generate FRI proof
        let fri_proof = fri_prover.build_proof(&query_positions);

        let mut trace_queries = Vec::with_capacity(traces.len());
        let mut constraint_queries = Vec::with_capacity(traces.len());

        for (i, trace) in traces.iter_mut().enumerate() {
            let commitment =
                self.get_commitment(trace, rand_elements.get_segment_elements(0), &domain);
            let mut inputs = self.get_pub_inputs(trace);
            inputs.segment.segment_n = i as u32;
            let air = <Self as Prover>::Air::new(trace.get_info(), inputs, self.options().clone());
            let mut constraint_coeffs = constraint_coeffs.clone();
            constraint_coeffs
                .boundary
                .truncate(air.context().num_assertions());
            let constraint_commitment = self.get_constraint_commitment(
                &air,
                commitment.trace_table(),
                rand_elements.clone(),
                constraint_coeffs,
                &domain,
            );
            trace_queries.push(commitment.query(&query_positions));
            constraint_queries.push(constraint_commitment.query(&query_positions));
        }

        (fri_proof, trace_queries, constraint_queries)
    }

    fn get_polys(
        &self,
        t: &mut <Self as Prover>::Trace,
        rand_elements: &[E],
        _domain: &StarkDomain<BaseElement>,
    ) -> TracePolyTable<E> {
        t.build_segment();
        let t1_polys = t.main_segment().interpolate_columns();
        let mut t1_polys = TracePolyTable::new(t1_polys);
        t.drop_segment();
        let t1_aux_polys = t
            .build_aux_segment(&[], rand_elements)
            .unwrap()
            .interpolate_columns();
        t1_polys.add_aux_segment(t1_aux_polys);

        t1_polys
    }

    fn get_commitment(
        &self,
        t: &mut <Self as Prover>::Trace,
        rand_elements: &[E],
        domain: &StarkDomain<BaseElement>,
    ) -> TraceCommitment<E, H> {
        t.build_segment();
        let (t1_lde, t1_tree, _) = self
            .build_trace_commitment::<<Self as Prover>::BaseField, MAIN_TRACE_WIDTH>(
                t.main_segment(),
                domain,
            );
        t.drop_segment();
        let t1_aux_segment = t.build_aux_segment(&[], rand_elements).unwrap();
        let (t1_aux_lde, t1_aux_tree, _) =
            self.build_trace_commitment::<E, AUX_SEGMENT_WIDTH>(&t1_aux_segment, domain);

        let mut commitment = TraceCommitment::new(t1_lde, t1_tree, domain.trace_to_lde_blowup());
        commitment.add_segment(t1_aux_lde, t1_aux_tree);
        commitment
    }

    fn get_constraint_poly(
        &self,
        air: &<Self as Prover>::Air,
        t_main_lde: &TraceLde<E>,
        rand_elements: AuxTraceRandElements<E>,
        constraint_coeffs: ConstraintCompositionCoefficients<E>,
        domain: &StarkDomain<BaseElement>,
    ) -> CompositionPoly<E> {
        let evaluator = ConstraintEvaluator::new(air, rand_elements.clone(), constraint_coeffs);
        let constraint_evaluations = evaluator.evaluate(t_main_lde, &domain);
        constraint_evaluations
            .into_poly(air.context().num_constraint_composition_columns())
            .unwrap()
    }

    fn get_constraint_commitment(
        &self,
        air: &<Self as Prover>::Air,
        t_main_lde: &TraceLde<E>,
        rand_elements: AuxTraceRandElements<E>,
        constraint_coeffs: ConstraintCompositionCoefficients<E>,
        domain: &StarkDomain<BaseElement>,
    ) -> ConstraintCommitment<E, H> {
        let composition_poly =
            self.get_constraint_poly(air, t_main_lde, rand_elements, constraint_coeffs, domain);
        self.build_constraint_commitment::<E, CONSTRAINTS_SEGMENT_WIDTH>(&composition_poly, domain)
    }

    #[doc(hidden)]
    fn generate_link_proof(
        &mut self,
        trace_1: &TraceInterpolations<E>,
        trace_2: &TraceInterpolations<E>,
        mut t1: <Self as Prover>::Trace,
        mut t2: <Self as Prover>::Trace,
    ) -> Result<StarkProof, ProverError> {
        let pub_inputs = self.inputs.clone();
        let air: RiscvAir = <Self as Prover>::Air::new(
            self.trace_info.clone().unwrap(),
            pub_inputs.clone(),
            self.options().clone(),
        );
        let main_trace_width = air.trace_layout().main_trace_width();
        let aux_trace_width = air.trace_layout().aux_trace_width();
        // create a channel which is used to simulate interaction between the prover and the
        // verifier; the channel will be used to commit to values and to draw randomness that
        // should come from the verifier.
        let mut channel = ProverChannel::<
            <Self as Prover>::Air,
            E,
            <Self as Prover>::HashFn,
            <Self as Prover>::RandomCoin,
        >::new(&air, vec![]);
        let domain = StarkDomain::new(&air);
        // 4 ----- build DEEP composition polynomial ----------------------------------------------
        #[cfg(feature = "std")]
        let now = Instant::now();

        // We construct a polynomial B(x) = (proof_1_poly(x*g^(l-1)) - proof_2_poly(x) ) / (x - 1) and show that
        // it's low degree, where l is the length of the proofs
        let g = E::BaseField::get_root_of_unity(air.trace_length().ilog2());
        let mut b_evals = vec![vec![E::BaseField::ZERO; air.trace_length()]; main_trace_width];
        let mut b_aux_evals = vec![vec![E::ZERO; air.trace_length()]; aux_trace_width];

        // evaluatins are at g^0 - g^(size -1) but the last row is padding
        let offset = (air.trace_length() - 2) as u32;
        t1.build_segment();
        let (t1_lde, t1_tree, _) = self
            .build_trace_commitment::<<Self as Prover>::BaseField, MAIN_TRACE_WIDTH>(
                t1.main_segment(),
                &domain,
            );
        t1.drop_segment();
        let t1_root = *t1_tree.root();
        drop(t1_tree);
        let t1_aux_segment = t1
            .clone()
            .build_aux_segment(&[], trace_1.rand_elements.get_segment_elements(0))
            .unwrap();
        let (t1_aux_lde, t1_aux_tree, _) =
            self.build_trace_commitment::<E, AUX_gSEGMENT_WIDTH>(&t1_aux_segment, &domain);
        let t1_aux_root = *t1_aux_tree.root();
        drop(t1_aux_tree);

        // add t1 contribution
        for row in 0..air.trace_length() {
            let blowup_row_offset = ((row + offset as usize) * air.lde_blowup_factor())
                % (air.trace_length() * air.lde_blowup_factor());
            for (col, b_col) in b_evals.iter_mut().enumerate() {
                b_col[row] = t1_lde.get(col, blowup_row_offset);
            }
            for (col, b_col) in b_aux_evals.iter_mut().enumerate() {
                b_col[row] = t1_aux_lde.get(col, blowup_row_offset);
            }
        }

        drop(t1_aux_segment);
        drop(t1_lde);
        drop(t1_aux_lde);
        t2.build_segment();
        let (t2_lde, t2_tree, _) = self
            .build_trace_commitment::<<Self as Prover>::BaseField, MAIN_TRACE_WIDTH>(
                t2.main_segment(),
                &domain,
            );
        t2.drop_segment();
        let t2_root = *t2_tree.root();
        drop(t2_tree);
        let t2_aux_segment = t2
            .clone()
            .build_aux_segment(&[], trace_2.rand_elements.get_segment_elements(0))
            .unwrap();

        let (t2_aux_lde, t2_aux_tree, _) =
            self.build_trace_commitment::<E, AUX_SEGMENT_WIDTH>(&t2_aux_segment, &domain);
        let t2_aux_root = *t2_aux_tree.root();
        drop(t2_aux_tree);

        // add t2 contribution
        for row in 0..air.trace_length() {
            let blowup_row =
                (row * air.lde_blowup_factor()) % (air.trace_length() * air.lde_blowup_factor());
            for (col, b_col) in b_evals.iter_mut().enumerate() {
                b_col[row] -= t2_lde.get(col, blowup_row);
            }
            for (col, b_col) in b_aux_evals.iter_mut().enumerate() {
                b_col[row] -= t2_aux_lde.get(col, blowup_row);
            }
        }

        drop(t2_aux_segment);
        drop(t2_lde);
        drop(t2_aux_lde);

        // divide by x - 1
        for row in 0..air.trace_length() {
            let x = g.exp_vartime((row as u32).into()) * E::BaseField::GENERATOR;
            for b_col in &mut b_evals {
                b_col[row] /= x - E::BaseField::ONE;
            }
            for b_col in &mut b_aux_evals {
                b_col[row] /= E::from(x) - E::ONE;
            }
        }

        let b_evals = ColMatrix::new(b_evals);
        let b_aux_evals = ColMatrix::new(b_aux_evals);

        // extend the main execution trace and build a Merkle tree from the extended trace
        let (b_trace_lde, b_trace_tree, b_trace_polys) = self
            .build_trace_commitment_with_offset::<<Self as Prover>::BaseField, MAIN_TRACE_WIDTH>(
                &b_evals,
                &domain,
                E::BaseField::GENERATOR,
            );

        let (b_aux_trace_lde, b_aux_trace_tree, b_aux_trace_polys) = self
            .build_trace_commitment_with_offset::<E, AUX_SEGMENT_WIDTH>(
                &b_aux_evals,
                &domain,
                E::BaseField::GENERATOR,
            );

        // commit to the LDE of the main trace by writing the root of its Merkle tree into
        // the channel
        // println!("adding link commitments");
        channel.commit_trace(t1_root);
        channel.commit_trace(t1_aux_root);
        channel.commit_trace(t2_root);
        channel.commit_trace(t2_aux_root);

        channel.commit_trace(*b_trace_tree.root());

        // initialize trace commitment and trace polynomial table structs with the main trace
        // data; for multi-segment traces these structs will be used as accumulators of all
        // trace segments
        let mut b_commitment =
            <TraceCommitment<E, _>>::new(b_trace_lde, b_trace_tree, domain.trace_to_lde_blowup());
        let mut b_polys = TracePolyTable::new(b_trace_polys);

        channel.commit_trace(*b_aux_trace_tree.root());
        b_commitment.add_segment(b_aux_trace_lde, b_aux_trace_tree);
        b_polys.add_aux_segment(b_aux_trace_polys);

        // draw an out-of-domain point z. Depending on the type of E, the point is drawn either
        // from the base field or from an extension field defined by E.
        //
        // The purpose of sampling from the extension field here (instead of the base field) is to
        // increase security. Soundness is limited by the size of the field that the random point
        // is drawn from, and we can potentially save on performance by only drawing this point
        // from an extension field, rather than increasing the size of the field overall.
        let z = channel.get_ood_point();
        let next_z = z * E::from(g.exp_vartime(offset.into()));

        let t1_polys = self.get_polys(
            &mut t1,
            trace_1.rand_elements.get_segment_elements(0),
            &domain,
        );
        let trace_states_1 = t1_polys.evaluate_at(next_z);
        drop(t1_polys);

        let t2_polys = self.get_polys(
            &mut t2,
            trace_2.rand_elements.get_segment_elements(0),
            &domain,
        );
        let trace_states_2 = t2_polys.evaluate_at(z);
        drop(t2_polys);

        let b_states = b_polys.evaluate_at(z);

        channel.send_ood_trace_states(&[
            trace_states_1.clone(),
            trace_states_2.clone(),
            b_states.clone(),
        ]);
        // draw random coefficients to use during DEEP polynomial composition, and use them to
        // initialize the DEEP composition polynomial
        let deep_coefficients = channel.get_deep_composition_coeffs();
        let mut deep_composition_poly = DeepCompositionPoly::new(z, deep_coefficients);

        // combine all trace polynomials together and merge them into the DEEP composition
        // polynomial
        let t1_polys = self.get_polys(
            &mut t1,
            trace_1.rand_elements.get_segment_elements(0),
            &domain,
        );
        deep_composition_poly.add_link_trace_polys(&t1_polys, vec![trace_states_1], next_z);
        drop(t1_polys);

        let t2_polys = self.get_polys(
            &mut t2,
            trace_2.rand_elements.get_segment_elements(0),
            &domain,
        );
        deep_composition_poly.add_link_trace_polys(&t2_polys, vec![trace_states_2], z);
        drop(t2_polys);

        deep_composition_poly.add_link_trace_polys(&b_polys, vec![b_states], z);

        drop(b_polys);
        #[cfg(feature = "std")]
        debug!(
            "Built DEEP composition polynomial of degree {} in {} ms",
            deep_composition_poly.degree(),
            now.elapsed().as_millis()
        );

        // make sure the degree of the DEEP composition polynomial is equal to trace polynomial
        // degree minus 1.
        assert_eq!(domain.trace_length() - 2, deep_composition_poly.degree());

        // 5 ----- evaluate DEEP composition polynomial over LDE domain ---------------------------
        #[cfg(feature = "std")]
        let now = Instant::now();
        let deep_evaluations = deep_composition_poly.evaluate(&domain);
        // we check the following condition in debug mode only because infer_degree is an expensive
        // operation
        debug_assert_eq!(
            domain.trace_length() - 2,
            infer_degree(&deep_evaluations, domain.offset())
        );
        #[cfg(feature = "std")]
        debug!(
            "Evaluated DEEP composition polynomial over LDE domain (2^{} elements) in {} ms",
            domain.lde_domain_size().ilog2(),
            now.elapsed().as_millis()
        );

        // 6 ----- compute FRI layers for the composition polynomial ------------------------------3
        #[cfg(feature = "std")]
        let now = Instant::now();
        let mut fri_prover = FriProver::new(air.options().to_fri_options());
        fri_prover.build_layers(&mut channel, deep_evaluations);
        #[cfg(feature = "std")]
        debug!(
            "Computed {} FRI layers from composition polynomial evaluations in {} ms",
            fri_prover.num_layers(),
            now.elapsed().as_millis()
        );

        // 7 ----- determine query positions ------------------------------------------------------
        #[cfg(feature = "std")]
        let now = Instant::now();

        // apply proof-of-work to the query seed
        channel.grind_query_seed();

        // generate pseudo-random query positions
        let query_positions = channel.get_query_positions();
        #[cfg(feature = "std")]
        debug!(
            "Determined {} query positions in {} ms",
            query_positions.len(),
            now.elapsed().as_millis()
        );

        // 8 ----- build proof object -------------------------------------------------------------
        #[cfg(feature = "std")]
        let now = Instant::now();

        // query the constraint commitment at the selected positions; for each query, we need just
        // a Merkle authentication path. this is because constraint evaluations for each step are
        // merged into a single value and Merkle authentication paths contain these values already

        let b_queries = b_commitment.query(&query_positions);
        drop(b_commitment);
        let t1_commitment = self.get_commitment(
            &mut t1,
            trace_1.rand_elements.get_segment_elements(0),
            &domain,
        );
        let trace_1_queries = t1_commitment.query(&query_positions);
        drop(t1_commitment);

        let t2_commitment = self.get_commitment(
            &mut t2,
            trace_2.rand_elements.get_segment_elements(0),
            &domain,
        );
        let trace_2_queries = t2_commitment.query(&query_positions);
        drop(t2_commitment);

        // generate FRI proof
        let fri_proof = fri_prover.build_proof(&query_positions);

        let placeholder = b_queries[0].clone();
        let queries = trace_1_queries
            .into_iter()
            .chain(trace_2_queries)
            .chain(b_queries)
            .collect::<Vec<_>>();
        // build the proof object
        // trace proofs are the same as the one in the original proofs, so we just ignore them
        let proof = channel.build_proof(vec![queries], vec![placeholder], fri_proof);
        #[cfg(feature = "std")]
        debug!("Built proof object in {} ms", now.elapsed().as_millis());

        Ok(proof)
    }
}

impl<H: ElementHasher, E: FieldElement> Prover for RiscvProver<H, E>
where
    H: ElementHasher<BaseField = BaseElement>,
    E: FieldElement<BaseField = BaseElement>,
{
    type BaseField = BaseElement;
    type Air = RiscvAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = H;
    type RandomCoin = DefaultRandomCoin<Self::HashFn>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> <Self::Air as winterfell::Air>::PublicInputs {
        self.inputs.clone()
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}

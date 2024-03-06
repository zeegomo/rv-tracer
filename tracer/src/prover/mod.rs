pub mod cache;

use crate::{
    air::{Inputs, RiscvAir},
    trace::TraceTable,
};
use cache::Cache;
use core::marker::PhantomData;
use serde::{de::DeserializeOwned, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use trace_defs::MAIN_TRACE_WIDTH;
use winter_prover::*;
use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher, MerkleTree},
    math::{fft::infer_degree, fields::f64::BaseElement, FieldElement, StarkField},
    Air, AuxTraceRandElements, ColMatrix, ProofOptions, Prover, ProverError, StarkProof, Trace,
    TraceCommitment, TraceInfo,
};

// This should be AUX_TRACE_WIDTH * EXTENSION_DEGREE
// TODO: use constant evaluation when it's stable
const AUX_SEGMENT_WIDTH: usize = 6;
const CONSTRAINTS_SEGMENT_WIDTH: usize = 32;

pub struct RiscvProver<
    H: ElementHasher<BaseField = BaseElement>,
    E: FieldElement<BaseField = BaseElement>,
    C,
> {
    options: ProofOptions,
    inputs: Inputs,
    _hasher: PhantomData<H>,
    rand_elems: Vec<E>,
    trace_info: Option<TraceInfo>,
    cache: C,
}
impl<H, E, C> RiscvProver<H, E, C>
where
    H: ElementHasher<BaseField = BaseElement>,
    E: FieldElement<BaseField = BaseElement>,

    C: Cache,
{
    pub fn new(options: ProofOptions, inputs: Inputs, cache: C) -> Self {
        Self {
            options,
            _hasher: PhantomData,
            inputs,
            rand_elems: Vec::new(),
            trace_info: None,
            cache,
        }
    }
}

impl<H, E, C> RiscvProver<H, E, C>
where
    H: ElementHasher<BaseField = BaseElement>,
    E: FieldElement<BaseField = BaseElement>,
    E: Serialize + DeserializeOwned,
    H::Digest: Serialize + DeserializeOwned,
    C: Cache,
{
    pub fn prove_segmented(
        &mut self,
        mut traces: Vec<TraceTable<E::BaseField>>,
    ) -> Result<(Vec<StarkProof>, Vec<StarkProof>), ProverError> {
        let mut proofs = Vec::new();
        let mut link_proofs = Vec::new();
        let main_traces_commitments = traces
            .iter_mut()
            .enumerate()
            .map(|(i, t)| {
                let air = <Self as Prover>::Air::new(
                    t.get_info(),
                    self.get_pub_inputs(t),
                    self.options().clone(),
                );
                let domain = StarkDomain::new(&air);
                t.build_segment();
                let res = *self.get_main_commitment(t, i, &domain).1.root();
                t.drop_segment();
                res
            })
            .collect::<Vec<_>>();

        proofs.push(self.generate_proof_with_cache(&mut traces[0], 0, &main_traces_commitments)?);
        for i in 1..traces.len() {
            // generate proof
            self.inputs.segment.segment_n += 1;
            proofs.push(self.generate_proof_with_cache(
                &mut traces[i],
                i,
                &main_traces_commitments,
            )?);

            // pattern matching so that we can get mutable reference to two elements at once
            let [ref mut t1, ref mut t2, ..] = traces[i - 1..] else {
                unreachable!();
            };

            // generate link proof
            // assert_eq!(self.cache.len(), 2);
            link_proofs.push(self.generate_link_proof(i - 1, t1, t2)?);
        }

        Ok((proofs, link_proofs))
    }

    // /// Performs the actual proof generation procedure, generating the proof that the provided
    // /// execution `trace` is valid against this prover's AIR.
    // /// TODO: make this function un-callable externally?
    // #[doc(hidden)]
    fn generate_proof_with_cache(
        &mut self,
        trace: &mut <Self as Prover>::Trace,
        trace_n: usize,
        main_trace_commitments: &[H::Digest],
    ) -> Result<StarkProof, ProverError> {
        // 0 ----- instantiate AIR and prover channel ---------------------------------------------

        // serialize public inputs; these will be included in the seed for the public coin
        let pub_inputs = self.get_pub_inputs(&trace);

        // create an instance of AIR for the provided parameters. this takes a generic description
        // of the computation (provided via AIR type), and creates a description of a specific
        // execution of the computation for the provided public inputs.
        let air = <Self as Prover>::Air::new(trace.get_info(), pub_inputs, self.options().clone());

        // create a channel which is used to simulate interaction between the prover and the
        // verifier; the channel will be used to commit to values and to draw randomness that
        // should come from the verifier.
        // TODO: add pub inputs to coin seed
        let mut channel = ProverChannel::<
            <Self as Prover>::Air,
            E,
            <Self as Prover>::HashFn,
            <Self as Prover>::RandomCoin,
        >::new(&air, vec![]);

        // 1 ----- Commit to the execution trace --------------------------------------------------

        // build computation domain; this is used later for polynomial evaluations
        #[cfg(feature = "std")]
        let now = Instant::now();
        let domain = StarkDomain::new(&air);
        #[cfg(feature = "std")]
        debug!(
            "Built domain of 2^{} elements in {} ms",
            domain.lde_domain_size().ilog2(),
            now.elapsed().as_millis()
        );

        for commitment in main_trace_commitments {
            channel.commit_trace(*commitment);
        }

        // extend the main execution trace and build a Merkle tree from the extended trace
        let (main_trace_lde, main_trace_tree, main_trace_polys) =
            self.get_main_commitment(trace, trace_n, &domain);

        // initialize trace commitment and trace polynomial table structs with the main trace
        // data; for multi-segment traces these structs will be used as accumulators of all
        // trace segments
        let mut trace_commitment = TraceCommitment::new(
            main_trace_lde,
            main_trace_tree,
            domain.trace_to_lde_blowup(),
        );
        let mut trace_polys = TracePolyTable::new(main_trace_polys);

        // build auxiliary trace segments (if any), and append the resulting segments to trace
        // commitment and trace polynomial table structs
        let mut aux_trace_segments = Vec::new();
        let mut aux_trace_rand_elements = AuxTraceRandElements::new();

        #[cfg(feature = "std")]
        let now = Instant::now();

        // draw a set of random elements required to build an auxiliary trace segment
        let rand_elements = channel.get_aux_trace_segment_rand_elements(0);

        if !self.rand_elems.is_empty() {
           assert_eq!(self.rand_elems, rand_elements);
        }
        self.rand_elems = rand_elements.clone();

        // build the trace segment
        let aux_segment = trace
            .build_aux_segment(&aux_trace_segments, &rand_elements)
            .expect("failed build auxiliary trace segment");
        #[cfg(feature = "std")]
        debug!(
            "Built auxiliary trace segment of {} columns and 2^{} steps in {} ms",
            aux_segment.num_cols(),
            aux_segment.num_rows().ilog2(),
            now.elapsed().as_millis()
        );

        // extend the auxiliary trace segment and build a Merkle tree from the extended trace
        let (aux_segment_lde, aux_segment_tree, aux_segment_polys) =
            self.get_aux_commitment(trace, trace_n, &domain);

        // commit to the LDE of the extended auxiliary trace segment  by writing the root of
        // its Merkle tree into the channel
        channel.commit_trace(*aux_segment_tree.root());

        // append the segment to the trace commitment and trace polynomial table structs
        trace_commitment.add_segment(aux_segment_lde, aux_segment_tree);
        trace_polys.add_aux_segment(aux_segment_polys);
        aux_trace_rand_elements.add_segment_elements(rand_elements.clone());
        aux_trace_segments.push(aux_segment);

        // make sure the specified trace (including auxiliary segments) is valid against the AIR.
        // This checks validity of both, assertions and state transitions. We do this in debug
        // mode only because this is a very expensive operation.
        #[cfg(debug_assertions)]
        {
            trace.build_segment();
            trace.validate(&air, &aux_trace_segments, &aux_trace_rand_elements);
            trace.drop_segment();
        }

        // 2 ----- evaluate constraints -----------------------------------------------------------
        // evaluate constraints specified by the AIR over the constraint evaluation domain, and
        // compute random linear combinations of these evaluations using coefficients drawn from
        // the channel; this step evaluates only constraint numerators, thus, only constraints with
        // identical denominators are merged together. the results are saved into a constraint
        // evaluation table where each column contains merged evaluations of constraints with
        // identical denominators.
        #[cfg(feature = "std")]
        let now = Instant::now();
        let constraint_coeffs = channel.get_constraint_composition_coeffs();
        let evaluator = ConstraintEvaluator::new(&air, aux_trace_rand_elements, constraint_coeffs);
        let constraint_evaluations = evaluator.evaluate(trace_commitment.trace_table(), &domain);
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
        let composition_poly =
            constraint_evaluations.into_poly(air.context().num_constraint_composition_columns())?;
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

        // 4 ----- build DEEP composition polynomial ----------------------------------------------
        #[cfg(feature = "std")]
        let now = Instant::now();

        // draw an out-of-domain point z. Depending on the type of E, the point is drawn either
        // from the base field or from an extension field defined by E.
        //
        // The purpose of sampling from the extension field here (instead of the base field) is to
        // increase security. Soundness is limited by the size of the field that the random point
        // is drawn from, and we can potentially save on performance by only drawing this point
        // from an extension field, rather than increasing the size of the field overall.
        let z = channel.get_ood_point();

        // evaluate trace and constraint polynomials at the OOD point z, and send the results to
        // the verifier. the trace polynomials are actually evaluated over two points: z and z * g,
        // where g is the generator of the trace domain.
        let ood_trace_states = trace_polys.get_ood_frame(z);
        channel.send_ood_trace_states(&ood_trace_states);

        let ood_evaluations = composition_poly.evaluate_at(z);
        channel.send_ood_constraint_evaluations(&ood_evaluations);

        // draw random coefficients to use during DEEP polynomial composition, and use them to
        // initialize the DEEP composition polynomial
        let deep_coefficients = channel.get_deep_composition_coeffs();
        let mut deep_composition_poly = DeepCompositionPoly::new(z, deep_coefficients);

        // combine all trace polynomials together and merge them into the DEEP composition
        // polynomial
        deep_composition_poly.add_trace_polys(&trace_polys, ood_trace_states);

        // merge columns of constraint composition polynomial into the DEEP composition polynomial;
        deep_composition_poly.add_composition_poly(composition_poly, ood_evaluations);

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

        // 6 ----- compute FRI layers for the composition polynomial ------------------------------
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

        // generate FRI proof
        let fri_proof = fri_prover.build_proof(&query_positions);

        // query the execution trace at the selected position; for each query, we need the
        // state of the trace at that position + Merkle authentication path
        let trace_queries = trace_commitment.query(&query_positions);

        // query the constraint commitment at the selected positions; for each query, we need just
        // a Merkle authentication path. this is because constraint evaluations for each step are
        // merged into a single value and Merkle authentication paths contain these values already
        let constraint_queries = constraint_commitment.query(&query_positions);

        self.trace_info = Some(trace.get_info());

        // build the proof object
        let proof = channel.build_proof(trace_queries, constraint_queries, fri_proof);
        #[cfg(feature = "std")]
        debug!("Built proof object in {} ms", now.elapsed().as_millis());

        Ok(proof)
    }

    fn get_or_init<T: DeserializeOwned + Serialize + PartialEq + core::fmt::Debug>(
        &self,
        key: &str,
        mut init: impl FnMut() -> T,
    ) -> T {
        match self.cache.get(key) {
            Some(res) => {
                res
            }
            None => {
                let res = init();
                self.cache.put(key, &res);
                res
            }
        }
    }

    fn get_main_polys(&self, t: &mut <Self as Prover>::Trace, trace_n: usize) -> TracePolyTable<E> {
        let polys = self.get_or_init(
            &format!(
                "main_polys-{}-{}",
                calculate_hash(&self.inputs.program),
                trace_n
            ),
            || {
                t.build_segment();
                let polys = t.main_segment().interpolate_columns();
                t.drop_segment();
                polys
            },
        );
        TracePolyTable::new(polys)
    }

    fn get_polys(&self, t: &mut <Self as Prover>::Trace, trace_n: usize) -> TracePolyTable<E> {
        let mut polys = self.get_main_polys(t, trace_n);
        let aux_polys = self.get_or_init(
            &format!(
                "aux_polys-{}-{}",
                calculate_hash(&self.inputs.program),
                trace_n
            ),
            || {
                t.build_aux_segment(&[], &self.rand_elems)
                    .unwrap()
                    .interpolate_columns()
            },
        );

        polys.add_aux_segment(aux_polys);
        polys
    }

    fn get_main_commitment(
        &self,
        t: &mut <Self as Prover>::Trace,
        trace_n: usize,
        domain: &StarkDomain<BaseElement>,
    ) -> (
        RowMatrix<E::BaseField>,
        MerkleTree<H>,
        ColMatrix<E::BaseField>,
    ) {
        let (lde, tree, polys) = self.get_or_init(
            &format!(
                "main_commitments-{}-{}",
                calculate_hash(&self.inputs.program),
                trace_n
            ),
            || {
                t.build_segment();
                let (lde, tree, polys) = self
                    .build_trace_commitment::<<Self as Prover>::BaseField, MAIN_TRACE_WIDTH>(
                        t.main_segment(),
                        domain,
                    );
                t.drop_segment();
                (lde, tree, polys)
            },
        );
        self.cache.put(
            &format!(
                "main_polys-{}-{}",
                calculate_hash(&self.inputs.program),
                trace_n
            ),
            &polys,
        );

        (lde, tree, polys)
    }

    fn get_aux_commitment(
        &self,
        t: &mut <Self as Prover>::Trace,
        trace_n: usize,
        domain: &StarkDomain<BaseElement>,
    ) -> (RowMatrix<E>, MerkleTree<H>, ColMatrix<E>) {
        let (lde, tree, polys) = self.get_or_init(
            &format!(
                "aux_commitments-{}-{}",
                calculate_hash(&self.inputs.program),
                trace_n
            ),
            || {
                let aux_segment = t.build_aux_segment(&[], &self.rand_elems).unwrap();
                self.build_trace_commitment::<E, AUX_SEGMENT_WIDTH>(&aux_segment, domain)
            },
        );

        self.cache.put(
            &format!(
                "aux_polys-{}-{}",
                calculate_hash(&self.inputs.program),
                trace_n
            ),
            &polys,
        );

        (lde, tree, polys)
    }

    fn get_commitment(
        &self,
        t: &mut <Self as Prover>::Trace,
        trace_n: usize,
        domain: &StarkDomain<BaseElement>,
    ) -> TraceCommitment<E, H> {
        let (lde, tree, _) = self.get_main_commitment(t, trace_n, domain);
        let mut commitment = TraceCommitment::new(lde, tree, domain.trace_to_lde_blowup());
        let (aux_lde, aux_tree, _) = self.get_aux_commitment(t, trace_n, domain);
        commitment.add_segment(aux_lde, aux_tree);
        commitment
    }

    #[doc(hidden)]
    fn generate_link_proof(
        &mut self,
        proof_n: usize,
        t1: &mut <Self as Prover>::Trace,
        t2: &mut <Self as Prover>::Trace,
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

        let (t1_lde, t1_tree, _) = self.get_main_commitment(t1, proof_n, &domain);
        let t1_root = *t1_tree.root();
        drop(t1_tree);
        let (t1_aux_lde, t1_aux_tree, _) = self.get_aux_commitment(t1, proof_n, &domain);
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

        drop(t1_lde);
        drop(t1_aux_lde);

        let (t2_lde, t2_tree, _) = self.get_main_commitment(t2, proof_n + 1, &domain);
        let t2_root = *t2_tree.root();
        drop(t2_tree);

        let (t2_aux_lde, t2_aux_tree, _) = self.get_aux_commitment(t2, proof_n + 1, &domain);
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

        let t1_polys = self.get_polys(t1, proof_n);
        let trace_states_1 = t1_polys.evaluate_at(next_z);
        drop(t1_polys);

        let t2_polys = self.get_polys(t2, proof_n + 1);
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
        let t1_polys = self.get_polys(t1, proof_n);
        deep_composition_poly.add_link_trace_polys(&t1_polys, vec![trace_states_1], next_z);
        drop(t1_polys);

        let t2_polys = self.get_polys(t2, proof_n + 1);
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
        let t1_commitment = self.get_commitment(t1, proof_n, &domain);
        let trace_1_queries = t1_commitment.query(&query_positions);
        drop(t1_commitment);

        let t2_commitment = self.get_commitment(t2, proof_n + 1, &domain);
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
        let proof = channel.build_proof(queries, placeholder, fri_proof);
        #[cfg(feature = "std")]
        debug!("Built proof object in {} ms", now.elapsed().as_millis());

        Ok(proof)
    }
}

impl<H: ElementHasher, E: FieldElement, C> Prover for RiscvProver<H, E, C>
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

fn calculate_hash(x: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

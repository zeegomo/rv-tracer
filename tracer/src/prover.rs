use crate::{
    air::{Inputs, RiscvAir},
    trace::TraceTable,
};
use core::marker::PhantomData;

use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::{fft::infer_degree, fields::f64::BaseElement, FieldElement, StarkField, ToElements},
    prover::*,
    Air, AuxTraceRandElements, ColMatrix, ProofOptions, Prover, ProverError, StarkProof, Trace,
    TraceCommitment, TraceInfo,
};

pub struct RiscvProver<
    H: ElementHasher<BaseField = BaseElement>,
    E: FieldElement<BaseField = BaseElement>,
> {
    options: ProofOptions,
    inputs: Inputs,
    _hasher: PhantomData<H>,
    cache: Vec<TraceInterpolations<H, E>>,
    trace_info: Option<TraceInfo>,
}

struct TraceInterpolations<
    H: ElementHasher<BaseField = BaseElement>,
    E: FieldElement<BaseField = BaseElement>,
> {
    commitment: TraceCommitment<E, H>,
    polys: TracePolyTable<E>,
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

    pub fn prove_with_splits(
        &mut self,
        traces: Vec<TraceTable<E::BaseField>>,
    ) -> Result<(Vec<StarkProof>, Vec<StarkProof>), ProverError> {
        let mut proofs = Vec::new();
        let mut link_proofs = Vec::new();
        for trace in traces {
            let proof = self.generate_proof_with_cache(trace)?;
            self.inputs.segment.segment_n += 1;
            proofs.push(proof);
        }
        // reset segment counter
        self.inputs.segment.segment_n = 0;
        let cache = core::mem::take(&mut self.cache);

        for traces in cache.windows(2) {
            let (prev, next) = (&traces[0], &traces[1]);
            let link_proof = self.generate_link_proof(prev, next)?;
            link_proofs.push(link_proof);
        }

        Ok((proofs, link_proofs))
    }

    // /// Performs the actual proof generation procedure, generating the proof that the provided
    // /// execution `trace` is valid against this prover's AIR.
    // /// TODO: make this function un-callable externally?
    // #[doc(hidden)]
    fn generate_proof_with_cache(
        &mut self,
        mut trace: <Self as Prover>::Trace,
    ) -> Result<StarkProof, ProverError> {
        // 0 ----- instantiate AIR and prover channel ---------------------------------------------

        // serialize public inputs; these will be included in the seed for the public coin
        let pub_inputs = self.get_pub_inputs(&trace);
        let pub_inputs_elements = pub_inputs.to_elements();

        // create an instance of AIR for the provided parameters. this takes a generic description
        // of the computation (provided via AIR type), and creates a description of a specific
        // execution of the computation for the provided public inputs.
        let air = <Self as Prover>::Air::new(trace.get_info(), pub_inputs, self.options().clone());

        // create a channel which is used to simulate interaction between the prover and the
        // verifier; the channel will be used to commit to values and to draw randomness that
        // should come from the verifier.
        let mut channel = ProverChannel::<
            <Self as Prover>::Air,
            E,
            <Self as Prover>::HashFn,
            <Self as Prover>::RandomCoin,
        >::new(&air, pub_inputs_elements);

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

        // extend the main execution trace and build a Merkle tree from the extended trace
        let (main_trace_lde, main_trace_tree, main_trace_polys) = self
            .build_trace_commitment::<<Self as Prover>::BaseField>(trace.main_segment(), &domain);

        // commit to the LDE of the main trace by writing the root of its Merkle tree into
        // the channel
        channel.commit_trace(*main_trace_tree.root());

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
        for i in 0..trace.layout().num_aux_segments() {
            #[cfg(feature = "std")]
            let now = Instant::now();

            // draw a set of random elements required to build an auxiliary trace segment
            let rand_elements = channel.get_aux_trace_segment_rand_elements(i);

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
                self.build_trace_commitment::<E>(&aux_segment, &domain);

            // commit to the LDE of the extended auxiliary trace segment  by writing the root of
            // its Merkle tree into the channel
            channel.commit_trace(*aux_segment_tree.root());

            // append the segment to the trace commitment and trace polynomial table structs
            trace_commitment.add_segment(aux_segment_lde, aux_segment_tree);
            trace_polys.add_aux_segment(aux_segment_polys);
            aux_trace_rand_elements.add_segment_elements(rand_elements);
            aux_trace_segments.push(aux_segment);
        }

        // make sure the specified trace (including auxiliary segments) is valid against the AIR.
        // This checks validity of both, assertions and state transitions. We do this in debug
        // mode only because this is a very expensive operation.
        #[cfg(debug_assertions)]
        trace.validate(&air, &aux_trace_segments, &aux_trace_rand_elements);

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
        let constraint_commitment =
            self.build_constraint_commitment::<E>(&composition_poly, &domain);

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

        self.cache.push(TraceInterpolations {
            commitment: trace_commitment,
            polys: trace_polys,
        });
        self.trace_info = Some(trace.get_info());

        // build the proof object
        let proof = channel.build_proof(trace_queries, constraint_queries, fri_proof);
        #[cfg(feature = "std")]
        debug!("Built proof object in {} ms", now.elapsed().as_millis());

        Ok(proof)
    }

    #[doc(hidden)]
    fn generate_link_proof(
        &mut self,
        trace_1: &TraceInterpolations<H, E>,
        trace_2: &TraceInterpolations<H, E>,
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
        >::new(&air, pub_inputs.to_elements());
        let domain = StarkDomain::new(&air);
        // 4 ----- build DEEP composition polynomial ----------------------------------------------
        #[cfg(feature = "std")]
        let now = Instant::now();

        // We construct a polynomial B(x) = (proof_1_poly(x*g^(l-1)) - proof_2_poly(x) ) / (x - 1) and show that
        // it's low degree, where l is the length of the proofs
        let g = E::BaseField::get_root_of_unity(trace_1.polys.poly_size().ilog2());
        let mut b_evals = vec![vec![E::BaseField::ZERO; air.trace_length()]; main_trace_width];
        debug_assert_eq!(trace_1.polys.poly_size(), trace_2.polys.poly_size());
        debug_assert_eq!(trace_1.polys.poly_size(), domain.trace_length());

        // evaluatins are at g^0 - g^(size -1) but the last row is padding
        let offset = (trace_1.polys.poly_size() - 2) as u32;
        debug_assert_eq!(
            trace_1
                .polys
                .evaluate_base_field_at(g.exp_vartime(offset.into()))[0],
            trace_2.polys.evaluate_base_field_at(E::BaseField::ONE)[0]
        );

        let trace_evals = (0..air.trace_length())
            .map(|row| {
                let x = g.exp_vartime((row as u32).into()) * E::BaseField::GENERATOR;
                let x_l = x * g.exp_vartime(offset.into());
                let trace_1_evals = trace_1.polys.evaluate_base_field_at(x_l);
                let trace_2_evals = trace_2.polys.evaluate_base_field_at(x);
                (x, trace_1_evals, trace_2_evals)
            })
            .collect::<Vec<_>>();
        for (i, (x, t1, t2)) in trace_evals.into_iter().enumerate() {
            for (j, (a, b)) in t1.into_iter().zip(t2).enumerate() {
                b_evals[j][i] = (a - b) / (x - E::BaseField::ONE);
            }
        }

        let mut b_aux_evals = vec![vec![E::ZERO; air.trace_length()]; aux_trace_width];
        let g = E::from(g);
        let aux_trace_evals = (0..air.trace_length())
            .map(|row| {
                let x = g.exp_vartime((row as u32).into()) * E::BaseField::GENERATOR.into();
                let x_l = x * g.exp_vartime(offset.into());
                let trace_1_evals = trace_1.polys.evaluate_aux_at(x_l);
                let trace_2_evals = trace_2.polys.evaluate_aux_at(x);
                (x, trace_1_evals, trace_2_evals)
            })
            .collect::<Vec<_>>();
        for (i, (x, t1, t2)) in aux_trace_evals.into_iter().enumerate() {
            for (j, (a, b)) in t1.into_iter().zip(t2).enumerate() {
                b_aux_evals[j][i] = (a - b) / (x - E::ONE);
            }
        }
        let b_evals = ColMatrix::new(b_evals);
        let b_aux_evals = ColMatrix::new(b_aux_evals);

        // extend the main execution trace and build a Merkle tree from the extended trace
        let (b_trace_lde, b_trace_tree, b_trace_polys) = self
            .build_trace_commitment_with_offset::<<Self as Prover>::BaseField>(
                &b_evals,
                &domain,
                E::BaseField::GENERATOR,
            );

        let (b_aux_trace_lde, b_aux_trace_tree, b_aux_trace_polys) = self
            .build_trace_commitment_with_offset::<E>(
                &b_aux_evals,
                &domain,
                E::BaseField::GENERATOR,
            );

        // commit to the LDE of the main trace by writing the root of its Merkle tree into
        // the channel
        // println!("adding link commitments");
        channel.commit_trace(trace_1.commitment.main_trace_root());
        channel.commit_trace(trace_1.commitment.aux_trace_roots()[0]);
        channel.commit_trace(trace_2.commitment.main_trace_root());
        channel.commit_trace(trace_2.commitment.aux_trace_roots()[0]);

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
        let next_z = z * g.exp_vartime(offset.into());
        let trace_states_1 = trace_1.polys.evaluate_at(next_z);
        let trace_states_2 = trace_2.polys.evaluate_at(z);
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
        deep_composition_poly.add_link_trace_polys(&trace_1.polys, vec![trace_states_1], next_z);
        deep_composition_poly.add_link_trace_polys(&trace_2.polys, vec![trace_states_2], z);
        deep_composition_poly.add_link_trace_polys(&b_polys, vec![b_states], z);

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

        // generate FRI proof
        let fri_proof = fri_prover.build_proof(&query_positions);

        // query the constraint commitment at the selected positions; for each query, we need just
        // a Merkle authentication path. this is because constraint evaluations for each step are
        // merged into a single value and Merkle authentication paths contain these values already
        let trace_1_queries = trace_1.commitment.query(&query_positions);
        let trace_2_queries = trace_2.commitment.query(&query_positions);
        let b_queries = b_commitment.query(&query_positions);
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

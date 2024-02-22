mod channel;
use crate::air::RiscvAir;
use channel::{LinkVerifierChannel, VerifierChannel};
use winter_air::{proof::StarkProof, Air, FieldExtension};
use winter_crypto::{ElementHasher, RandomCoin};
use winter_fri::FriVerifier;
use winter_math::{
    fields::{CubeExtension, QuadExtension},
    FieldElement, StarkField, ToElements,
};
use winter_verifier::{evaluate_constraints, AuxTraceRandElements, DeepComposer, VerifierError};

pub fn verify_segmented<HashFn, RandCoin>(
    proofs: Vec<StarkProof>,
    link_proofs: Vec<StarkProof>,
    mut pub_inputs: <RiscvAir as Air>::PublicInputs,
) -> Result<(), VerifierError>
where
    HashFn: ElementHasher<BaseField = <RiscvAir as Air>::BaseField>,
    RandCoin: RandomCoin<BaseField = <RiscvAir as Air>::BaseField, Hasher = HashFn>,
{
    assert_eq!(proofs.len(), link_proofs.len() + 1);
    let n_segments = proofs.len();

    for (segment_n, (proofs, link_proof)) in proofs.windows(2).zip(link_proofs).enumerate() {
        let prev = proofs[0].clone();
        let next = proofs[1].clone();
        verify_link::<HashFn, RandCoin>(
            prev,
            next,
            link_proof,
            pub_inputs.clone(),
            n_segments,
            segment_n,
        )
        .unwrap();
    }
    for proof in proofs {
        verify::<HashFn, RandCoin>(proof, n_segments, pub_inputs.clone()).unwrap();
        pub_inputs.segment.segment_n += 1;
    }

    Ok(())
}

fn verify_link<HashFn, RandCoin>(
    proof_1: StarkProof,
    proof_2: StarkProof,
    link_proof: StarkProof,
    pub_inputs: <RiscvAir as Air>::PublicInputs,
    n_segments: usize,
    segment_n: usize,
) -> Result<(), VerifierError>
where
    HashFn: ElementHasher<BaseField = <RiscvAir as Air>::BaseField>,
    RandCoin: RandomCoin<BaseField = <RiscvAir as Air>::BaseField, Hasher = HashFn>,
{
    let proof = link_proof;

    // build a seed for the public coin; the initial seed is a hash of the proof context and the
    // public inputs, but as the protocol progresses, the coin will be reseeded with the info
    // received from the prover
    // TODO: include public inputs in the coin seed
    let public_coin_seed = proof.context.to_elements();
    // public_coin_seed.append(&mut pub_inputs.to_elements());

    // create AIR instance for the computation specified in the proof
    let air = RiscvAir::new(proof.get_trace_info(), pub_inputs, proof.options().clone());

    // figure out which version of the generic proof verification procedure to run. this is a sort
    // of static dispatch for selecting two generic parameter: extension field and hash function.
    match air.options().field_extension() {
        FieldExtension::None => {
            let public_coin = RandCoin::new(&public_coin_seed);
            let channel = LinkVerifierChannel::new(&air, proof)?;
            let channel_1: VerifierChannel<<RiscvAir as Air>::BaseField, HashFn> =
                VerifierChannel::new(&air, proof_1, n_segments, segment_n)?;
            let channel_2: VerifierChannel<<RiscvAir as Air>::BaseField, HashFn> =
                VerifierChannel::new(&air, proof_2, n_segments, segment_n + 1)?;
            if channel_1.read_trace_commitments() != channel.read_trace_1_commitments()
                || channel_2.read_trace_commitments() != channel.read_trace_2_commitments()
            {
                return Err(VerifierError::InconsistentTraceCommitments);
            }

            perform_link_verification::<RiscvAir, <RiscvAir as Air>::BaseField, HashFn, RandCoin>(
                air,
                channel,
                public_coin,
            )
        }
        FieldExtension::Quadratic => {
            if !<QuadExtension<<RiscvAir as Air>::BaseField>>::is_supported() {
                return Err(VerifierError::UnsupportedFieldExtension(2));
            }
            let public_coin = RandCoin::new(&public_coin_seed);
            let channel = LinkVerifierChannel::new(&air, proof).unwrap();
            let channel_1: VerifierChannel<QuadExtension<<RiscvAir as Air>::BaseField>, HashFn> =
                VerifierChannel::new(&air, proof_1, n_segments, segment_n).unwrap();
            let channel_2: VerifierChannel<QuadExtension<<RiscvAir as Air>::BaseField>, HashFn> =
                VerifierChannel::new(&air, proof_2, n_segments, segment_n + 1).unwrap();

            if channel_1.read_trace_commitments() != channel.read_trace_1_commitments()
                || channel_2.read_trace_commitments() != channel.read_trace_2_commitments()
            {
                return Err(VerifierError::InconsistentTraceCommitments);
            }
            perform_link_verification::<
                RiscvAir,
                QuadExtension<<RiscvAir as Air>::BaseField>,
                HashFn,
                RandCoin,
            >(air, channel, public_coin)
        }
        FieldExtension::Cubic => {
            if !<CubeExtension<<RiscvAir as Air>::BaseField>>::is_supported() {
                return Err(VerifierError::UnsupportedFieldExtension(3));
            }
            let public_coin = RandCoin::new(&public_coin_seed);
            let channel = LinkVerifierChannel::new(&air, proof)?;
            let channel_1: VerifierChannel<CubeExtension<<RiscvAir as Air>::BaseField>, HashFn> =
                VerifierChannel::new(&air, proof_1, n_segments, segment_n)?;
            let channel_2: VerifierChannel<CubeExtension<<RiscvAir as Air>::BaseField>, HashFn> =
                VerifierChannel::new(&air, proof_2, n_segments, segment_n + 1)?;
            if channel_1.read_trace_commitments() != channel.read_trace_1_commitments()
                || channel_2.read_trace_commitments() != channel.read_trace_2_commitments()
            {
                return Err(VerifierError::InconsistentTraceCommitments);
            }
            perform_link_verification::<
                RiscvAir,
                CubeExtension<<RiscvAir as Air>::BaseField>,
                HashFn,
                RandCoin,
            >(air, channel, public_coin)
        }
    }
}

pub fn verify<HashFn, RandCoin>(
    proof: StarkProof,
    n_segments: usize,
    pub_inputs: <RiscvAir as Air>::PublicInputs,
) -> Result<(), VerifierError>
where
    HashFn: ElementHasher<BaseField = <RiscvAir as Air>::BaseField>,
    RandCoin: RandomCoin<BaseField = <RiscvAir as Air>::BaseField, Hasher = HashFn>,
{
    // build a seed for the public coin; the initial seed is a hash of the proof context and the
    // public inputs, but as the protocol progresses, the coin will be reseeded with the info
    // received from the prover
    // TODO: include public inputs in the coin seed
    let public_coin_seed = proof.context.to_elements();
    // public_coin_seed.append(&mut pub_inputs.to_elements());

    let segment_n = pub_inputs.segment.segment_n as usize;

    // create AIR instance for the computation specified in the proof
    let air = RiscvAir::new(proof.get_trace_info(), pub_inputs, proof.options().clone());

    // figure out which version of the generic proof verification procedure to run. this is a sort
    // of static dispatch for selecting two generic parameter: extension field and hash function.
    match air.options().field_extension() {
        FieldExtension::None => {
            let public_coin = RandCoin::new(&public_coin_seed);
            let channel = VerifierChannel::new(&air, proof, n_segments, segment_n)?;
            perform_verification::<<RiscvAir as Air>::BaseField, HashFn, RandCoin>(
                air,
                channel,
                public_coin,
            )
        }
        FieldExtension::Quadratic => {
            if !<QuadExtension<<RiscvAir as Air>::BaseField>>::is_supported() {
                return Err(VerifierError::UnsupportedFieldExtension(2));
            }
            let public_coin = RandCoin::new(&public_coin_seed);
            let channel = VerifierChannel::new(&air, proof, n_segments, segment_n)?;
            perform_verification::<QuadExtension<<RiscvAir as Air>::BaseField>, HashFn, RandCoin>(
                air,
                channel,
                public_coin,
            )
        }
        FieldExtension::Cubic => {
            if !<CubeExtension<<RiscvAir as Air>::BaseField>>::is_supported() {
                return Err(VerifierError::UnsupportedFieldExtension(3));
            }
            let public_coin = RandCoin::new(&public_coin_seed);
            let channel = VerifierChannel::new(&air, proof, n_segments, segment_n)?;
            perform_verification::<CubeExtension<<RiscvAir as Air>::BaseField>, HashFn, RandCoin>(
                air,
                channel,
                public_coin,
            )
        }
    }
}

// VERIFICATION PROCEDURE
// ================================================================================================
/// Performs the actual verification by reading the data from the `channel` and making sure it
/// attests to a correct execution of the computation specified by the provided `air`.
fn perform_verification<E, H, R>(
    air: RiscvAir,
    mut channel: VerifierChannel<E, H>,
    mut public_coin: R,
) -> Result<(), VerifierError>
where
    E: FieldElement<BaseField = <RiscvAir as Air>::BaseField>,
    H: ElementHasher<BaseField = <RiscvAir as Air>::BaseField>,
    R: RandomCoin<BaseField = <RiscvAir as Air>::BaseField, Hasher = H>,
{
    // 1 ----- trace commitment -------------------------------------------------------------------
    // Read the commitments to evaluations of the trace polynomials over the LDE domain sent by the
    // prover. The commitments are used to update the public coin, and draw sets of random elements
    // from the coin (in the interactive version of the protocol the verifier sends these random
    // elements to the prover after each commitment is made). When there are multiple trace
    // commitments (i.e., the trace consists of more than one segment), each previous commitment is
    // used to draw random elements needed to construct the next trace segment. The last trace
    // commitment is used to draw a set of random coefficients which the prover uses to compute
    // constraint composition polynomial.
    let trace_commitments = channel.read_trace_commitments();

    // reseed the coin with the commitment to the main trace segment
    for commitment in channel.main_trace_commitments() {
        public_coin.reseed(*commitment);
    }

    // process auxiliary trace segments (if any), to build a set of random elements for each segment
    let mut aux_trace_rand_elements = AuxTraceRandElements::<E>::new();
    for (i, commitment) in trace_commitments.iter().skip(1).enumerate() {
        let rand_elements = air
            .get_aux_trace_segment_random_elements(i, &mut public_coin)
            .map_err(|_| VerifierError::RandomCoinError)?;
        aux_trace_rand_elements.add_segment_elements(rand_elements);
        public_coin.reseed(*commitment);
    }

    // build random coefficients for the composition polynomial
    let constraint_coeffs = air
        .get_constraint_composition_coefficients(&mut public_coin)
        .map_err(|_| VerifierError::RandomCoinError)?;

    // 2 ----- constraint commitment --------------------------------------------------------------
    // read the commitment to evaluations of the constraint composition polynomial over the LDE
    // domain sent by the prover, use it to update the public coin, and draw an out-of-domain point
    // z from the coin; in the interactive version of the protocol, the verifier sends this point z
    // to the prover, and the prover evaluates trace and constraint composition polynomials at z,
    // and sends the results back to the verifier.
    let constraint_commitment = channel.read_constraint_commitment();
    public_coin.reseed(constraint_commitment);
    let z = public_coin
        .draw::<E>()
        .map_err(|_| VerifierError::RandomCoinError)?;

    // 3 ----- OOD consistency check --------------------------------------------------------------
    // make sure that evaluations obtained by evaluating constraints over the out-of-domain frame
    // are consistent with the evaluations of composition polynomial columns sent by the prover

    // read the out-of-domain trace frames (the main trace frame and auxiliary trace frame, if
    // provided) sent by the prover and evaluate constraints over them; also, reseed the public
    // coin with the OOD frames received from the prover.
    let ood_trace_frame = channel.read_ood_trace_frame();
    let ood_main_trace_frame = ood_trace_frame.main_frame();
    let ood_aux_trace_frame = ood_trace_frame.aux_frame();
    let ood_constraint_evaluation_1 = evaluate_constraints(
        &air,
        constraint_coeffs,
        &ood_main_trace_frame,
        &ood_aux_trace_frame,
        aux_trace_rand_elements,
        z,
    );
    public_coin.reseed(H::hash_elements(ood_trace_frame.values()));

    // read evaluations of composition polynomial columns sent by the prover, and reduce them into
    // a single value by computing \sum_{i=0}^{m-1}(z^(i * l) * value_i), where value_i is the
    // evaluation of the ith column polynomial H_i(X) at z, l is the trace length and m is
    // the number of composition column polynomials. This computes H(z) (i.e.
    // the evaluation of the composition polynomial at z) using the fact that
    // H(X) = \sum_{i=0}^{m-1} X^{i * l} H_i(X).
    // Also, reseed the public coin with the OOD constraint evaluations received from the prover.
    let ood_constraint_evaluations = channel.read_ood_constraint_evaluations();
    let ood_constraint_evaluation_2 =
        ood_constraint_evaluations
            .iter()
            .enumerate()
            .fold(E::ZERO, |result, (i, &value)| {
                result + z.exp_vartime(((i * (air.trace_length())) as u32).into()) * value
            });
    public_coin.reseed(H::hash_elements(&ood_constraint_evaluations));

    // finally, make sure the values are the same
    if ood_constraint_evaluation_1 != ood_constraint_evaluation_2 {
        return Err(VerifierError::InconsistentOodConstraintEvaluations);
    }

    // 4 ----- FRI commitments --------------------------------------------------------------------
    // draw coefficients for computing DEEP composition polynomial from the public coin; in the
    // interactive version of the protocol, the verifier sends these coefficients to the prover
    // and the prover uses them to compute the DEEP composition polynomial. the prover, then
    // applies FRI protocol to the evaluations of the DEEP composition polynomial.
    let deep_coefficients = air
        .get_deep_composition_coefficients::<E, R>(&mut public_coin)
        .map_err(|_| VerifierError::RandomCoinError)?;

    // instantiates a FRI verifier with the FRI layer commitments read from the channel. From the
    // verifier's perspective, this is equivalent to executing the commit phase of the FRI protocol.
    // The verifier uses these commitments to update the public coin and draw random points alpha
    // from them; in the interactive version of the protocol, the verifier sends these alphas to
    // the prover, and the prover uses them to compute and commit to the subsequent FRI layers.
    let fri_verifier = FriVerifier::new(
        &mut channel,
        &mut public_coin,
        air.options().to_fri_options(),
        air.trace_poly_degree(),
    )
    .map_err(VerifierError::FriVerificationFailed)?;
    // TODO: make sure air.lde_domain_size() == fri_verifier.domain_size()

    // 5 ----- trace and constraint queries -------------------------------------------------------
    // read proof-of-work nonce sent by the prover and update the public coin with it
    let pow_nonce = channel.read_pow_nonce();
    public_coin.reseed_with_int(pow_nonce);

    // make sure the proof-of-work specified by the grinding factor is satisfied
    if public_coin.leading_zeros() < air.options().grinding_factor() {
        return Err(VerifierError::QuerySeedProofOfWorkVerificationFailed);
    }

    // draw pseudo-random query positions for the LDE domain from the public coin; in the
    // interactive version of the protocol, the verifier sends these query positions to the prover,
    // and the prover responds with decommitments against these positions for trace and constraint
    // composition polynomial evaluations.
    let query_positions = public_coin
        .draw_integers(air.options().num_queries(), air.lde_domain_size())
        .map_err(|_| VerifierError::RandomCoinError)?;

    // read evaluations of trace and constraint composition polynomials at the queried positions;
    // this also checks that the read values are valid against trace and constraint commitments
    let (queried_main_trace_states, queried_aux_trace_states) =
        channel.read_queried_trace_states(&query_positions)?;
    let queried_constraint_evaluations = channel.read_constraint_evaluations(&query_positions)?;

    // 6 ----- DEEP composition -------------------------------------------------------------------
    // compute evaluations of the DEEP composition polynomial at the queried positions
    let composer = DeepComposer::new(&air, &query_positions, z, deep_coefficients);
    let t_composition = composer.compose_trace_columns(
        queried_main_trace_states,
        queried_aux_trace_states,
        ood_main_trace_frame,
        ood_aux_trace_frame,
    );
    let c_composition = composer
        .compose_constraint_evaluations(queried_constraint_evaluations, ood_constraint_evaluations);
    let deep_evaluations = composer.combine_compositions(t_composition, c_composition);

    // 7 ----- Verify low-degree proof -------------------------------------------------------------
    // make sure that evaluations of the DEEP composition polynomial we computed in the previous
    // step are in fact evaluations of a polynomial of degree equal to trace polynomial degree
    fri_verifier
        .verify(&mut channel, &deep_evaluations, &query_positions)
        .map_err(VerifierError::FriVerificationFailed)
}

// VERIFICATION PROCEDURE
// ================================================================================================
/// Performs the actual verification by reading the data from the `channel` and making sure it
/// attests to a correct execution of the computation specified by the provided `air`.
fn perform_link_verification<A, E, H, R>(
    air: A,
    mut channel: LinkVerifierChannel<E, H>,
    mut public_coin: R,
) -> Result<(), VerifierError>
where
    A: Air,
    E: FieldElement<BaseField = A::BaseField>,
    H: ElementHasher<BaseField = A::BaseField>,
    R: RandomCoin<BaseField = A::BaseField, Hasher = H>,
{
    // 1 ----- trace commitment -------------------------------------------------------------------
    // Read the commitments to evaluations of the trace polynomials over the LDE domain sent by the
    // prover. The commitments are used to update the public coin, and draw sets of random elements
    // from the coin (in the interactive version of the protocol the verifier sends these random
    // elements to the prover after each commitment is made). When there are multiple trace
    // commitments (i.e., the trace consists of more than one segment), each previous commitment is
    // used to draw random elements needed to construct the next trace segment. The last trace
    // commitment is used to draw a set of random coefficients which the prover uses to compute
    // constraint composition polynomial.
    let main_trace_width = air.trace_layout().main_trace_width();
    let _aux_trace_width = air.trace_layout().aux_trace_width();
    let trace_1_commitments = channel.read_trace_1_commitments();
    let trace_2_commitments = channel.read_trace_2_commitments();
    let b_commitments = channel.read_b_commitments();

    // reseed the coin with the commitment to the main trace segment
    public_coin.reseed(trace_1_commitments[0]);
    public_coin.reseed(trace_1_commitments[1]);
    public_coin.reseed(trace_2_commitments[0]);
    public_coin.reseed(trace_2_commitments[1]);
    public_coin.reseed(b_commitments[0]);
    public_coin.reseed(b_commitments[1]);

    // // 2 ----- constraint commitment --------------------------------------------------------------
    // // read the commitment to evaluations of the constraint composition polynomial over the LDE
    // // domain sent by the prover, use it to update the public coin, and draw an out-of-domain point
    // // z from the coin; in the interactive version of the protocol, the verifier sends this point z
    // // to the prover, and the prover evaluates trace and constraint composition polynomials at z,
    // // and sends the results back to the verifier.
    let z = public_coin
        .draw::<E>()
        .map_err(|_| VerifierError::RandomCoinError)?;
    let trace_length = air.trace_info().length() as u32;
    let g = E::BaseField::get_root_of_unity(trace_length.ilog2());
    let next_z = z * g.exp_vartime((trace_length - 2).into()).into();
    // 3 ----- OOD consistency check --------------------------------------------------------------
    // make sure that evaluations obtained by evaluating constraints over the out-of-domain frame
    // are consistent with the evaluations of composition polynomial columns sent by the prover

    // read the out-of-domain trace frames (the main trace frame and auxiliary trace frame, if
    // provided) sent by the prover and evaluate constraints over them; also, reseed the public
    // coin with the OOD frames received from the prover.
    let mut result = Vec::new();
    let mut trace_1_ood_main_evals = channel.trace_1();
    let mut trace_2_ood_main_evals = channel.trace_2();
    let mut b_ood_main_evals = channel.b();

    for (i, ((trace_1_eval, trace_2_eval), b_actual)) in trace_1_ood_main_evals
        .iter()
        .zip(trace_2_ood_main_evals.iter())
        .zip(b_ood_main_evals.iter())
        .enumerate()
    {
        let b_expected = (*trace_1_eval - *trace_2_eval) / (z - E::ONE);
        // let b_actual = b_ood_main_evals[i];
        // finally, make sure the values are the same
        if b_expected != *b_actual {
            println!("constraint: {i} {} {}", b_expected, b_actual);
            return Err(VerifierError::InconsistentOodConstraintEvaluations);
        }
    }

    for ((a, b), c) in trace_1_ood_main_evals
        .iter()
        .zip(trace_2_ood_main_evals.iter())
        .zip(b_ood_main_evals.iter())
    {
        result.push(*a);
        result.push(*b);
        result.push(*c);
    }

    let trace_1_ood_aux_evals = trace_1_ood_main_evals.split_off(main_trace_width);
    let trace_2_ood_aux_evals = trace_2_ood_main_evals.split_off(main_trace_width);
    let b_ood_aux_evals = b_ood_main_evals.split_off(main_trace_width);

    public_coin.reseed(H::hash_elements(&result));

    // read evaluations of composition polynomial columns sent by the prover, and reduce them into
    // a single value by computing \sum_{i=0}^{m-1}(z^(i * l) * value_i), where value_i is the
    // evaluation of the ith column polynomial H_i(X) at z, l is the trace length and m is
    // the number of composition column polynomials. This computes H(z) (i.e.
    // the evaluation of the composition polynomial at z) using the fact that
    // H(X) = \sum_{i=0}^{m-1} X^{i * l} H_i(X).
    // Also, reseed the public coin with the OOD constraint evaluations received from the prover.
    // let b_evals = channel.b();

    // 4 ----- FRI commitments --------------------------------------------------------------------
    // draw coefficients for computing DEEP composition polynomial from the public coin; in the
    // interactive version of the protocol, the verifier sends these coefficients to the prover
    // and the prover uses them to compute the DEEP composition polynomial. the prover, then
    // applies FRI protocol to the evaluations of the DEEP composition polynomial.
    let deep_coefficients = air
        .get_deep_composition_coefficients::<E, R>(&mut public_coin)
        .map_err(|_| VerifierError::RandomCoinError)?;

    // instantiates a FRI verifier with the FRI layer commitments read from the channel. From the
    // verifier's perspective, this is equivalent to executing the commit phase of the FRI protocol.
    // The verifier uses these commitments to update the public coin and draw random points alpha
    // from them; in the interactive version of the protocol, the verifier sends these alphas to
    // the prover, and the prover uses them to compute and commit to the subsequent FRI layers.
    let fri_verifier = FriVerifier::new(
        &mut channel,
        &mut public_coin,
        air.options().to_fri_options(),
        air.trace_poly_degree(),
    )
    .map_err(VerifierError::FriVerificationFailed)?;
    // TODO: make sure air.lde_domain_size() == fri_verifier.domain_size()

    // 5 ----- trace and constraint queries -------------------------------------------------------
    // read proof-of-work nonce sent by the prover and update the public coin with it
    let pow_nonce = channel.read_pow_nonce();
    public_coin.reseed_with_int(pow_nonce);

    // make sure the proof-of-work specified by the grinding factor is satisfied
    if public_coin.leading_zeros() < air.options().grinding_factor() {
        return Err(VerifierError::QuerySeedProofOfWorkVerificationFailed);
    }

    // draw pseudo-random query positions for the LDE domain from the public coin; in the
    // interactive version of the protocol, the verifier sends these query positions to the prover,
    // and the prover responds with decommitments against these positions for trace and constraint
    // composition polynomial evaluations.
    let query_positions = public_coin
        .draw_integers(air.options().num_queries(), air.lde_domain_size())
        .map_err(|_| VerifierError::RandomCoinError)?;

    let (queried_trace_1_main_states, queried_trace_1_aux_states) =
        channel.read_queried_trace_1_states(&query_positions)?;

    let (queried_trace_2_main_states, queried_trace_2_aux_states) =
        channel.read_queried_trace_2_states(&query_positions)?;
    // read evaluations of trace and constraint composition polynomials at the queried positions;
    // this also checks that the read values are valid against trace and constraint commitments
    let (queried_b_states, queried_b_aux_trace_states) =
        channel.read_queried_b_states(&query_positions)?;
    // 6 ----- DEEP composition -------------------------------------------------------------------
    // compute evaluations of the DEEP composition polynomial at the queried positions
    let composer = DeepComposer::new(&air, &query_positions, z, deep_coefficients);
    let t1_composition = composer.compose_trace_columns_link(
        queried_trace_1_main_states,
        queried_trace_1_aux_states,
        trace_1_ood_main_evals,
        Some(trace_1_ood_aux_evals),
        next_z,
    );
    let t2_composition = composer.compose_trace_columns_link(
        queried_trace_2_main_states,
        queried_trace_2_aux_states,
        trace_2_ood_main_evals,
        Some(trace_2_ood_aux_evals),
        z,
    );
    let b_composition = composer.compose_trace_columns_link(
        queried_b_states,
        queried_b_aux_trace_states,
        b_ood_main_evals,
        Some(b_ood_aux_evals),
        z,
    );
    let mut deep_evaluations = composer.combine_compositions(t1_composition, t2_composition);
    deep_evaluations = composer.combine_compositions(deep_evaluations, b_composition);

    // 7 ----- Verify low-degree proof -------------------------------------------------------------
    // make sure that evaluations of the DEEP composition polynomial we computed in the previous
    // step are in fact evaluations of a polynomial of degree equal to trace polynomial degree
    fri_verifier
        .verify(&mut channel, &deep_evaluations, &query_positions)
        .map_err(VerifierError::FriVerificationFailed)
}

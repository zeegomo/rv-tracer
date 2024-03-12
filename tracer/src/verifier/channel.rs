// From winterfell
// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.
use crate::air::{
    proof::{Commitments, StarkProof},
    RiscvAir,
};
use winter_air::proof::Table;
use winter_crypto::{BatchMerkleProof, ElementHasher, MerkleTree};
use winter_fri::VerifierChannel as FriVerifierChannel;
use winter_math::{FieldElement, StarkField};
use winter_utils::{collections::Vec, string::ToString};
use winter_verifier::{Air, ConstraintQueries, TraceOodFrame, TraceQueries, VerifierError};

// VERIFIER CHANNEL
// ================================================================================================

/// A view into a [StarkProof] for a computation structured to simulate an "interactive" channel.
///
/// A channel is instantiated for a specific proof, which is parsed into structs over the
/// appropriate field (specified by type parameter `E`). This also validates that the proof is
/// well-formed in the context of the computation for the specified [Air].
pub struct VerifierChannel<E: FieldElement, H: ElementHasher<BaseField = E::BaseField>> {
    // trace queries
    trace_roots: Vec<H::Digest>,
    trace_queries: Option<Vec<TraceQueries<E, H>>>,
    // constraint queries
    constraint_roots: Vec<H::Digest>,
    constraint_queries: Option<Vec<ConstraintQueries<E, H>>>,
    // FRI proof
    fri_roots: Option<Vec<H::Digest>>,
    fri_layer_proofs: Vec<BatchMerkleProof<H>>,
    fri_layer_queries: Vec<Vec<E>>,
    fri_remainder: Option<Vec<E>>,
    fri_num_partitions: usize,
    // out-of-domain frame
    ood_trace_frames: Option<Vec<TraceOodFrame<E>>>,
    ood_constraint_evaluations: Option<Vec<Vec<E>>>,
    // query proof-of-work
    pow_nonce: u64,
}

impl<
        E: FieldElement<BaseField = <RiscvAir as Air>::BaseField>,
        H: ElementHasher<BaseField = E::BaseField>,
    > VerifierChannel<E, H>
{
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Creates and returns a new [VerifierChannel] initialized from the specified `proof`.
    pub fn new(
        air: &RiscvAir,
        proof: StarkProof,
        n_segments: usize,
        segment_n: usize,
    ) -> Result<Self, VerifierError> {
        let StarkProof {
            context,
            commitments,
            trace_queries,
            constraint_queries,
            ood_frame,
            fri_proof,
            pow_nonce,
        } = proof;

        // make sure AIR and proof base fields are the same
        if E::BaseField::get_modulus_le_bytes() != context.field_modulus_bytes() {
            return Err(VerifierError::InconsistentBaseField);
        }
        let constraint_frame_width = air.context().num_constraint_composition_columns();
        let mut num_trace_segments = n_segments;
        let main_trace_width = air.trace_layout().main_trace_width();
        let aux_trace_width = air.trace_layout().aux_trace_width();
        let lde_domain_size = air.lde_domain_size();
        let fri_options = air.options().to_fri_options();

        // --- parse commitments ------------------------------------------------------------------
        let (trace_roots, constraint_roots, fri_roots) = commitments
            .parse::<H>(
                num_trace_segments,
                fri_options.num_fri_layers(lde_domain_size),
            )
            .map_err(|err| VerifierError::ProofDeserializationError(err.to_string()))
            .unwrap();

        // --- parse trace and constraint queries -------------------------------------------------

        let trace_queries = trace_queries
            .into_iter()
            .map(|q| <TraceQueries<E, H>>::new(q, air).unwrap())
            .collect::<Vec<_>>();
        let constraint_queries = constraint_queries
            .into_iter()
            .map(|q| ConstraintQueries::new(q, air).unwrap())
            .collect();

        // --- parse FRI proofs -------------------------------------------------------------------
        let fri_num_partitions = fri_proof.num_partitions();
        let fri_remainder = fri_proof
            .parse_remainder()
            .map_err(|err| VerifierError::ProofDeserializationError(err.to_string()))
            .unwrap();
        let (fri_layer_queries, fri_layer_proofs) = fri_proof
            .parse_layers::<H, E>(lde_domain_size, fri_options.folding_factor())
            .map_err(|err| VerifierError::ProofDeserializationError(err.to_string()))
            .unwrap();

        // --- parse out-of-domain evaluation frame -----------------------------------------------
        let res = ood_frame
            .parse(
                main_trace_width,
                aux_trace_width,
                constraint_frame_width,
                n_segments,
            )
            .map_err(|err| VerifierError::ProofDeserializationError(err.to_string()))
            .unwrap();

        let mut ood_trace_frames = Vec::new();
        let mut ood_constraint_evaluations = Vec::new();

        for (trace_frame, constraint_evaluations) in res {
            ood_trace_frames.push(TraceOodFrame::new(
                trace_frame,
                main_trace_width,
                aux_trace_width,
            ));
            ood_constraint_evaluations.push(constraint_evaluations);
        }

        Ok(VerifierChannel {
            // trace queries
            trace_roots,
            trace_queries: Some(trace_queries),
            // constraint queries
            constraint_roots,
            constraint_queries: Some(constraint_queries),
            // FRI proof
            fri_roots: Some(fri_roots),
            fri_layer_proofs,
            fri_layer_queries,
            fri_remainder: Some(fri_remainder),
            fri_num_partitions,
            // out-of-domain evaluation
            ood_trace_frames: Some(ood_trace_frames),
            ood_constraint_evaluations: Some(ood_constraint_evaluations),
            // query seed
            pow_nonce,
        })
    }

    // DATA READERS
    // --------------------------------------------------------------------------------------------

    /// Returns execution trace commitments sent by the prover.
    ///
    /// For computations requiring multiple trace segment, the returned slice will contain a
    /// commitment for each trace segment.
    pub fn read_trace_commitments(&self) -> &[H::Digest] {
        &self.trace_roots
    }

    /// Returns constraint evaluation commitment sent by the prover.
    pub fn read_constraint_commitments(&self) -> &[H::Digest] {
        &self.constraint_roots
    }

    /// Returns trace polynomial evaluations at out-of-domain points z and z * g, where g is the
    /// generator of the LDE domain.
    ///
    /// For computations requiring multiple trace segments, evaluations of auxiliary trace
    /// polynomials are also included.
    pub fn read_ood_trace_frames(&mut self) -> Vec<TraceOodFrame<E>> {
        self.ood_trace_frames.take().expect("already read")
    }

    /// Returns evaluations of composition polynomial columns at z^m, where z is the out-of-domain
    /// point, and m is the number of composition polynomial columns.
    pub fn read_ood_constraint_evaluations(&mut self) -> Vec<Vec<E>> {
        self.ood_constraint_evaluations
            .take()
            .expect("already read")
    }

    /// Returns query proof-of-work nonce sent by the prover.
    pub fn read_pow_nonce(&self) -> u64 {
        self.pow_nonce
    }

    /// Returns trace states at the specified positions of the LDE domain. This also checks if
    /// the trace states are valid against the trace commitment sent by the prover.
    ///
    /// For computations requiring multiple trace segments, trace states for auxiliary segments
    /// are also included as the second value of the returned tuple (trace states for all auxiliary
    /// segments are merged into a single table). Otherwise, the second value is None.
    #[allow(clippy::type_complexity)]
    pub fn read_queried_trace_states(
        &mut self,
        positions: &[usize],
    ) -> Result<Vec<(Table<E::BaseField>, Option<Table<E>>)>, VerifierError> {
        let queries = self.trace_queries.take().expect("already read");
        let mut trace_roots = Vec::new();
        let n_main_roots = self.trace_roots.len() / 2;
        for i in 0..n_main_roots {
            trace_roots.push(self.trace_roots[i]);
            trace_roots.push(self.trace_roots[i + n_main_roots]);
        }

        let mut res = Vec::new();
        for (queries, roots) in queries.into_iter().zip(trace_roots.chunks(2)) {
            // make sure the states included in the proof correspond to the trace commitment
            for (root, proof) in roots.iter().zip(queries.query_proofs().iter()) {
                MerkleTree::verify_batch(root, positions, proof)
                    .map_err(|_| VerifierError::TraceQueryDoesNotMatchCommitment)
                    .unwrap();
            }
            res.push(queries.states());
        }

        Ok(res)
    }

    /// Returns constraint evaluations at the specified positions of the LDE domain. This also
    /// checks if the constraint evaluations are valid against the constraint commitment sent by
    /// the prover.
    pub fn read_constraint_evaluations(
        &mut self,
        positions: &[usize],
    ) -> Result<Vec<Table<E>>, VerifierError> {
        let queries = self.constraint_queries.take().expect("already read");

        let mut res = Vec::new();

        for (i, queries) in queries.into_iter().enumerate() {
            MerkleTree::verify_batch(&self.constraint_roots[i], positions, queries.query_proofs())
                .map_err(|_| VerifierError::ConstraintQueryDoesNotMatchCommitment)?;
            res.push(queries.evaluations());
        }

        Ok(res)
    }
}

// FRI VERIFIER CHANNEL IMPLEMENTATION
// ================================================================================================

impl<E, H> FriVerifierChannel<E> for VerifierChannel<E, H>
where
    E: FieldElement,
    H: ElementHasher<BaseField = E::BaseField>,
{
    type Hasher = H;

    fn read_fri_num_partitions(&self) -> usize {
        self.fri_num_partitions
    }

    fn read_fri_layer_commitments(&mut self) -> Vec<H::Digest> {
        self.fri_roots.take().expect("already read")
    }

    fn take_next_fri_layer_proof(&mut self) -> BatchMerkleProof<H> {
        self.fri_layer_proofs.remove(0)
    }

    fn take_next_fri_layer_queries(&mut self) -> Vec<E> {
        self.fri_layer_queries.remove(0)
    }

    fn take_fri_remainder(&mut self) -> Vec<E> {
        self.fri_remainder.take().expect("already read")
    }
}

pub struct LinkVerifierChannel<E: FieldElement, H: ElementHasher<BaseField = E::BaseField>> {
    trace_1_roots: Vec<H::Digest>,
    trace_2_roots: Vec<H::Digest>,
    b_roots: Vec<H::Digest>,
    trace_1_queries: Option<TraceQueries<E, H>>,
    trace_2_queries: Option<TraceQueries<E, H>>,
    b_queries: Option<TraceQueries<E, H>>,
    // FRI proof
    fri_roots: Option<Vec<H::Digest>>,
    fri_layer_proofs: Vec<BatchMerkleProof<H>>,
    fri_layer_queries: Vec<Vec<E>>,
    fri_remainder: Option<Vec<E>>,
    fri_num_partitions: usize,
    // out-of-domain frame
    ood_evaluations: Option<Vec<E>>,
    // query proof-of-work
    pow_nonce: u64,
}

impl<E: FieldElement, H: ElementHasher<BaseField = E::BaseField>> LinkVerifierChannel<E, H> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Creates and returns a new [VerifierChannel] initialized from the specified `proof`.
    pub fn new<A: Air<BaseField = E::BaseField>>(
        air: &A,
        proof: StarkProof,
    ) -> Result<Self, VerifierError> {
        let StarkProof {
            context,
            commitments,
            mut trace_queries,
            constraint_queries: _,
            ood_frame,
            fri_proof,
            pow_nonce,
        } = proof;

        // make sure AIR and proof base fields are the same
        if E::BaseField::get_modulus_le_bytes() != context.field_modulus_bytes() {
            return Err(VerifierError::InconsistentBaseField);
        }
        let num_trace_segments = air.trace_layout().num_segments();
        let main_trace_width = air.trace_layout().main_trace_width();
        let aux_trace_width = air.trace_layout().aux_trace_width();
        let lde_domain_size = air.lde_domain_size();
        let fri_options = air.options().to_fri_options();

        // --- parse commitments ------------------------------------------------------------------
        let (trace_1_roots, trace_2_roots, b_roots, fri_roots) = commitments
            .parse_link::<H>(
                num_trace_segments,
                fri_options.num_fri_layers(lde_domain_size),
            ) // TODO: read aux segments
            .map_err(|err| VerifierError::ProofDeserializationError(err.to_string()))?;

        // --- parse trace and constraint queries -------------------------------------------------
        let mut trace_queries = trace_queries.pop().unwrap();
        let queries_length = trace_queries.len() / 3;
        let mut trace_2_queries = trace_queries.split_off(queries_length);
        let b_queries = trace_2_queries.split_off(queries_length);
        let trace_1_queries = TraceQueries::new(trace_queries, air)?;
        let trace_2_queries = TraceQueries::new(trace_2_queries, air)?;
        let b_queries = TraceQueries::new(b_queries, air)?;

        // --- parse FRI proofs -------------------------------------------------------------------
        let fri_num_partitions = fri_proof.num_partitions();
        let fri_remainder = fri_proof
            .parse_remainder()
            .map_err(|err| VerifierError::ProofDeserializationError(err.to_string()))?;
        let (fri_layer_queries, fri_layer_proofs) = fri_proof
            .parse_layers::<H, E>(lde_domain_size, fri_options.folding_factor())
            .map_err(|err| VerifierError::ProofDeserializationError(err.to_string()))?;

        // --- parse out-of-domain evaluation frame -----------------------------------------------
        let (ood_evaluations, _) = ood_frame
            .parse_link(main_trace_width, aux_trace_width)
            .map_err(|err| VerifierError::ProofDeserializationError(err.to_string()))?;

        Ok(Self {
            trace_1_roots,
            trace_2_roots,
            b_roots,
            trace_1_queries: Some(trace_1_queries),
            trace_2_queries: Some(trace_2_queries),
            b_queries: Some(b_queries),
            // FRI proof
            fri_roots: Some(fri_roots),
            fri_layer_proofs,
            fri_layer_queries,
            fri_remainder: Some(fri_remainder),
            fri_num_partitions,
            ood_evaluations: Some(ood_evaluations),
            // query seed
            pow_nonce,
        })
    }

    // DATA READERS
    // --------------------------------------------------------------------------------------------

    /// Returns execution trace commitments sent by the prover.
    ///
    /// For computations requiring multiple trace segment, the returned slice will contain a
    /// commitment for each trace segment.
    pub fn read_trace_1_commitments(&self) -> &[H::Digest] {
        &self.trace_1_roots
    }

    pub fn read_trace_2_commitments(&self) -> &[H::Digest] {
        &self.trace_2_roots
    }

    pub fn read_b_commitments(&self) -> &[H::Digest] {
        &self.b_roots
    }

    pub fn trace_1(&self) -> Vec<E> {
        self.ood_evaluations
            .as_ref()
            .unwrap()
            .chunks(3)
            .map(|values| values[0])
            .collect()
    }

    pub fn trace_2(&self) -> Vec<E> {
        self.ood_evaluations
            .as_ref()
            .unwrap()
            .chunks(3)
            .map(|values| values[1])
            .collect()
    }

    pub fn b(&self) -> Vec<E> {
        self.ood_evaluations
            .as_ref()
            .unwrap()
            .chunks(3)
            .map(|values| values[2])
            .collect()
    }

    /// Retur
    /// Returns query proof-of-work nonce sent by the prover.
    pub fn read_pow_nonce(&self) -> u64 {
        self.pow_nonce
    }

    #[allow(clippy::type_complexity)]
    pub fn read_queried_trace_1_states(
        &mut self,
        positions: &[usize],
    ) -> Result<(Table<E::BaseField>, Option<Table<E>>), VerifierError> {
        let queries = self.trace_1_queries.take().expect("already read");

        // make sure the states included in the proof correspond to the trace commitment
        for (root, proof) in self.trace_1_roots.iter().zip(queries.query_proofs().iter()) {
            MerkleTree::verify_batch(root, positions, proof)
                .map_err(|_| VerifierError::TraceQueryDoesNotMatchCommitment)?;
        }

        Ok(queries.states())
    }

    #[allow(clippy::type_complexity)]
    pub fn read_queried_trace_2_states(
        &mut self,
        positions: &[usize],
    ) -> Result<(Table<E::BaseField>, Option<Table<E>>), VerifierError> {
        let queries = self.trace_2_queries.take().expect("already read");

        // make sure the states included in the proof correspond to the trace commitment
        for (root, proof) in self.trace_2_roots.iter().zip(queries.query_proofs().iter()) {
            MerkleTree::verify_batch(root, positions, proof)
                .map_err(|_| VerifierError::TraceQueryDoesNotMatchCommitment)?;
        }

        Ok(queries.states())
    }

    /// Returns trace states at the specified positions of the LDE domain. This also checks if
    /// the trace states are valid against the trace commitment sent by the prover.
    ///
    /// For computations requiring multiple trace segments, trace states for auxiliary segments
    /// are also included as the second value of the returned tuple (trace states for all auxiliary
    /// segments are merged into a single table). Otherwise, the second value is None.
    #[allow(clippy::type_complexity)]
    pub fn read_queried_b_states(
        &mut self,
        positions: &[usize],
    ) -> Result<(Table<E::BaseField>, Option<Table<E>>), VerifierError> {
        let queries = self.b_queries.take().expect("already read");

        // make sure the states included in the proof correspond to the trace commitment
        for (root, proof) in self.b_roots.iter().zip(queries.query_proofs().iter()) {
            MerkleTree::verify_batch(root, positions, proof)
                .map_err(|_| VerifierError::TraceQueryDoesNotMatchCommitment)?;
        }

        Ok(queries.states())
    }
}

// FRI VERIFIER CHANNEL IMPLEMENTATION
// ================================================================================================

impl<E, H> FriVerifierChannel<E> for LinkVerifierChannel<E, H>
where
    E: FieldElement,
    H: ElementHasher<BaseField = E::BaseField>,
{
    type Hasher = H;

    fn read_fri_num_partitions(&self) -> usize {
        self.fri_num_partitions
    }

    fn read_fri_layer_commitments(&mut self) -> Vec<H::Digest> {
        self.fri_roots.take().expect("already read")
    }

    fn take_next_fri_layer_proof(&mut self) -> BatchMerkleProof<H> {
        self.fri_layer_proofs.remove(0)
    }

    fn take_next_fri_layer_queries(&mut self) -> Vec<E> {
        self.fri_layer_queries.remove(0)
    }

    fn take_fri_remainder(&mut self) -> Vec<E> {
        self.fri_remainder.take().expect("already read")
    }
}

pub mod air {
    mod ops {
        use constraint_macros::air;
        pub mod lui {
            use trace_defs::*;
            use core::ops::*;
            use winterfell::{
                EvaluationFrame, TransitionConstraintDegree, math::FieldElement,
            };
            const OPCODE_FLAG_DEG: usize = 7;
            const FUNCT3_FLAG_DEG: usize = 0i32 as usize;
            const RS1_FLAG_DEG: usize = 0i32 as usize;
            const RS2_FLAG_DEG: usize = 0i32 as usize;
            const RD_FLAG_DEG: usize = 5i32 as usize;
            const RD_CNT: usize = 1 << 5i32 as usize;
            const RS1_CNT: usize = 1 << 0i32 as usize;
            const RS2_CNT: usize = 1 << 0i32 as usize;
            const TOT_CNT: usize = RD_CNT * RS1_CNT * RS2_CNT;
            const BODY_FLAG_DEG: usize = 1;
            const CONSTRAINT_DEGS: [usize; 1usize] = [1u8 as usize];
            pub fn evaluate_transitions<E: FieldElement>(
                frame: &EvaluationFrame<E>,
                periodic_values: &[E],
                result: &mut [E],
            ) -> usize {
                let current = frame.current();
                let next = frame.next();
                let is_body = current[BODY];
                let mut index = 0;
                for rd in 0..RD_CNT {
                    for rs1 in 0..RS1_CNT {
                        for rs2 in 0..RS2_CNT {
                            let rd_flag = rd_flag(
                                rd as u8,
                                &current[RD_END..RD_END + 5],
                            );
                            let rs1_flag = rs1_flag(
                                rs1 as u8,
                                &current[RS1_END..RS1_END + 5],
                            );
                            let rs2_flag = rs2_flag(
                                rs2 as u8,
                                &current[RS2_END..RS2_END + 5],
                            );
                            let funct3_flag = funct3_flag(
                                &current[FUNCT3_END..FUNCT3_END + 3],
                            );
                            let op_flag = op_flag(&current[OPCODE_END..OPCODE_END + 7]);
                            let body_flag = current[BODY];
                            let cumulative_flag = op_flag * rd_flag * rs1_flag * rs2_flag
                                * body_flag * funct3_flag;
                            let imm = get_immediate(&current[UIMM_END..UIMM_END + 12]);
                            let uimm = get_immediate(&current[UIMM_END..UIMM_END + 20]);
                            let pc = current[PC];
                            let h0 = current[H_0];
                            let h1 = current[H_1];
                            let h2 = current[H_2];
                            let h3 = current[H_3];
                            let h4 = current[H_4];
                            let h5 = current[H_5];
                            let rs = next[REGISTER_START + rd];
                            let rs1 = current[REGISTER_START + rs1];
                            let rs2 = current[REGISTER_START + rs2];
                            result[index] = (rd - uimm) * cumulative_flag;
                            index += 1;
                        }
                    }
                }
                1usize
            }
            pub fn constraint_degrees() -> Vec<TransitionConstraintDegree> {
                let mut degrees = Vec::with_capacity(TOT_CNT);
                for _ in 0..TOT_CNT {
                    for deg in CONSTRAINT_DEGS.iter() {
                        degrees
                            .push(
                                TransitionConstraintDegree::new(
                                    OPCODE_FLAG_DEG + RD_FLAG_DEG + RS1_FLAG_DEG + RS2_FLAG_DEG
                                        + FUNCT3_FLAG_DEG + BODY_FLAG_DEG + deg,
                                ),
                            );
                    }
                }
                degrees
            }
            fn evaluate_constraints<E: FieldElement>(
                current: &[E],
                next: &[E],
                rd: usize,
                rs1: usize,
                rs2: usize,
                result: &mut [E],
            ) {}
            fn rd_flag<E: FieldElement>(reg: u8, test: &[E]) -> E {
                match (&test.len(), &5) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            let kind = ::core::panicking::AssertKind::Eq;
                            ::core::panicking::assert_failed(
                                kind,
                                &*left_val,
                                &*right_val,
                                ::core::option::Option::None,
                            );
                        }
                    }
                };
                binary_flag(&to_binary(reg, E::ZERO, E::ONE), test, E::ONE)
            }
            fn rs1_flag<E: FieldElement>(reg: u8, test: &[E]) -> E {
                match (&test.len(), &5) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            let kind = ::core::panicking::AssertKind::Eq;
                            ::core::panicking::assert_failed(
                                kind,
                                &*left_val,
                                &*right_val,
                                ::core::option::Option::None,
                            );
                        }
                    }
                };
                E::ONE
            }
            fn rs2_flag<E: FieldElement>(reg: u8, test: &[E]) -> E {
                match (&test.len(), &5) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            let kind = ::core::panicking::AssertKind::Eq;
                            ::core::panicking::assert_failed(
                                kind,
                                &*left_val,
                                &*right_val,
                                ::core::option::Option::None,
                            );
                        }
                    }
                };
                E::ONE
            }
            fn funct3_flag<E>(test: &[E]) -> E
            where
                E: FieldElement,
            {
                match (&test.len(), &3) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            let kind = ::core::panicking::AssertKind::Eq;
                            ::core::panicking::assert_failed(
                                kind,
                                &*left_val,
                                &*right_val,
                                ::core::option::Option::None,
                            );
                        }
                    }
                };
                E::ONE
            }
            fn op_flag<E>(test: &[E]) -> E
            where
                E: FieldElement,
            {
                match (&test.len(), &7) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            let kind = ::core::panicking::AssertKind::Eq;
                            ::core::panicking::assert_failed(
                                kind,
                                &*left_val,
                                &*right_val,
                                ::core::option::Option::None,
                            );
                        }
                    }
                };
                binary_flag(
                    &[E::ZERO, E::ONE, E::ONE, E::ZERO, E::ONE, E::ONE, E::ONE],
                    test,
                    E::ONE,
                )
            }
            fn reg_flag<E>(reg: u8, test: &[E]) -> E
            where
                E: FieldElement,
            {
                match (&test.len(), &5) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            let kind = ::core::panicking::AssertKind::Eq;
                            ::core::panicking::assert_failed(
                                kind,
                                &*left_val,
                                &*right_val,
                                ::core::option::Option::None,
                            );
                        }
                    }
                };
                binary_flag(&to_binary(reg, E::ZERO, E::ONE), test, E::ONE)
            }
            pub fn binary_flag<E>(expected: &[E], test: &[E], one: E) -> E
            where
                E: Mul<Output = E> + Sub<Output = E> + Copy + FieldElement,
            {
                let mut result = one;
                for (i, bit) in expected.iter().enumerate() {
                    result *= if bit == &one { test[i] } else { one - test[i] };
                }
                result
            }
            fn to_binary<E: Copy>(reg: u8, zero: E, one: E) -> [E; 5] {
                let mut result = [zero; 5];
                for i in 5..0 {
                    if reg & (1 << i) != 0 {
                        result[i] = one;
                    }
                }
                result
            }
            fn get_immediate<E: FieldElement>(op: &[E]) -> E {
                let mut result = E::ZERO;
                match (&op.len(), &12) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            let kind = ::core::panicking::AssertKind::Eq;
                            ::core::panicking::assert_failed(
                                kind,
                                &*left_val,
                                &*right_val,
                                ::core::option::Option::None,
                            );
                        }
                    }
                };
                for (i, bit) in op.iter().enumerate() {
                    result += *bit * E::from(1u32 << i);
                }
                result
            }
        }
    }
    mod utils {
        use core::ops::{Mul, MulAssign, Sub};
        use winterfell::math::FieldElement;
    }
    use winterfell::{
        math::{fields::f128::BaseElement, FieldElement},
        Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
        TransitionConstraintDegree,
    };
    pub type BaseField = winterfell::math::fields::f128::BaseElement;
    pub struct RiscvAir {
        context: AirContext<BaseElement>,
    }
    use trace_defs::TRACE_WIDTH;
    impl Air for RiscvAir {
        type BaseField = BaseElement;
        type PublicInputs = ();
        fn new(trace_info: TraceInfo, _pub_inputs: (), options: ProofOptions) -> Self {
            let mut degrees = Vec::new();
            degrees.push(TransitionConstraintDegree::new(2));
            match (&TRACE_WIDTH, &trace_info.width()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::None,
                        );
                    }
                }
            };
            let num_assertions = 1;
            Self {
                context: AirContext::new(trace_info, degrees, num_assertions, options),
            }
        }
        fn context(&self) -> &AirContext<Self::BaseField> {
            &self.context
        }
        fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
            &self,
            frame: &EvaluationFrame<E>,
            periodic_values: &[E],
            result: &mut [E],
        ) {
            let mut index = 0;
            let current = frame.current();
            let next = frame.next();
        }
        fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
            <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([
                    Assertion::single(0, 0, Self::BaseField::ZERO),
                ]),
            )
        }
    }
}

use constraint_macros::air;

air! {
    name = lui,
    opcode = "0110111",
    parse = rd / uimm,
    constraints = rd - upper_imm => 1
}

// h0 is the overflow bit
air! {
    name = auipc,
    opcode = "0010111",
    parse = rd,
    constraints = (rd + h0 * E::from(1u64 << 32)) - (current[PC] + upper_imm) => 1
}

// h0 is the overflowing bit
air! {
    name = addi,
    opcode = "0010011",
    funct3 = "000",
    parse = rs1 / rd,
    constraints =
        (rd + h0 * E::from(1u64 << 32)) - (rs1 + simm) => 1,
        h0 * (h0 - E::ONE) * (h0 + E::ONE) => 3
}

// FIX: we ignore constraints for rd = 0 but we should not do that
// h0, h1 overflow bits
air! {
    name = jal,
    opcode = "1101111",
    parse = rd,
    constraints =
        rd + h0 * E::from(1u64 << 32) - (current[PC] + E::from(4u32)) => 1,
        (next[PC] + h1 * E::from(1u64 << 32)) - (current[PC] + jal_offset) => 1
}

// FIX: we ignore constraints for rd = 0 but we should not do that
// h0, h1 overflow bits
// overflow before or after adding h2?
// TODO: h2 can be directly substituted in the second constraint
air! {
    name = jalr,
    opcode = "1100111",
    funct3 = "000",
    parse = rs1 / rd,
    constraints =
        rd  + h0 * E::from(1u64 << 32) - (current[PC] + E::from(4u32)) => 1,
        (next[PC] + h1 * E::from(1u64 << 32)) - (rs1 + simm - h2) => 1,
        h2 - (current[RS1_BITS_END + 31] + current[IMM_END + 11] - E::from(2u32) * current[RS1_BITS_END + 31] * current[IMM_END + 11]) => 2
}

// If rs1 < immediate, then rd = 1. This means there is 0 < C < 2^32 s.t. rs1 + C = immediate. H0 is C.
// If rs1 >= immediate, then rd = 0. This means there is 0 <= C' < 2^32 s.t. immediate + C = rs1. H1 is C'.
air!(
    name = slti,
    opcode = "0010011",
    funct3 = "010",
    parse = rs1 / rd,
    constraints =
        rd * (h0 + rs1 - simm) => 2,
        (E::ONE - rd) * (h1 + simm - rs1) => 2,
        rd * rd - rd => 2
);

// if the branch is not taken, then rs1 = rs2
// if the branch is taken, then (a - b) != 0, which means the reciprocal 1 / (a - b) exists. We ask the user to provide
// such value in h0, so that we can check that (rs1 - rs2) * h0 - 1 = 0
air!(
    name = bne,
    opcode = "1100011",
    funct3 = "001",
    parse = rs1 / rs2,
    constraints =
        (next[PC] + h1 * E::from(1u64 << 32)  - pc - E::from(4u32)) * ((rs1 - rs2) * h0 - E::ONE) => 3,
        (next[PC] + h1 * E::from(1u64 << 32) - branch_offset - pc) * (rs1 - rs2) => 2
);

// // TODO: add range checks to H0 and H1
// // This check uses 2 additional helper registers to ensure the computation was
// // performed correctly.
// // If rs1 < immediate, then rd = 1. This means there is 0 < C < 2^32 s.t. rs1 + C = immediate. H0 is C.
// // If rs1 >= immediate, then rd = 0. This means there is 0 <= C' < 2^32 s.t. immediate + C = rs1. H1 is C'.
// air!(
//     name = sltui,
//     opcode = "0010011",
//     funct3 = "011",
//     parse = rs1 / rd,
//     constraints =
//         rd * h0 + rs1 - uimm => 2,
//         (E::ONE - rd) * h1 + uimm - rs1 => 2
// );

// air!(
//     name = xori,
//     opcode = "0010011",
//     funct3 = "100",
//     parse = rs1 / rd,
//     constraints = bitwise rd - (rs1 + simm - E::from(2u32) * rs1 * simm) => 2
// );

// air!(
//     name = ori,
//     opcode = "0010011",
//     funct3 = "110",
//     parse = rs1 / rd,
//     constraints = bitwise rd - (rs1 + simm - rs1 * simm) => 2
// );

// air!(
//     name = andi,
//     opcode = "0010011",
//     funct3 = "111",
//     parse = rs1 / rd,
//     constraints = bitwise rd - (rs1 * simm) => 2
// );

// // TODO Fix
// air!(
//     name = slli,
//     opcode = "0010011",
//     funct3 = "001",
//     parse = rs1 / rd / shamt,
//     constraints = shift rs1 -> rd,

// );

// // TODO check the next instructions
// bitwise_air!(
//     name = srli,
//     opcode = "0010011",
//     funct3 = "101",
//     parse = rs1 / rd / shamt,
//     constraints =
//         rd[shamt..] - rs1[..32 - shamt] => 1,
//         rd[..shamt] => 1
// );

// bitwise_air!(
//     name = srai,
//     opcode = "0010011",
//     funct3 = "101",
//     funct7 = "0100000",
//     parse = rs1 / rd / shamt,
//     constraints =
//         rd[shamt..] - rs1[..32 - shamt] => 1,
//         rd[..shamt] - rs1[32] => 1
// );

air!(
    name = add,
    opcode = "0110011",
    funct3 = "000",
    funct7 = "0000000",
    parse = rs1 / rs2 / rd,
    constraints =
        (rd + h0 * E::from(1u64 << 32)) - (rs1 + rs2) => 1,
        h0 * (h0 - E::ONE) * (h0 + E::ONE) => 3
);

// air!(
//     name = sub,
//     opcode = "0110011",
//     funct3 = "000",
//     funct5 = "01000",
//     parse = rs1 / rs2 / rd,
//     constraints = rd - (rs1 - rs2) => 1
// );

// air!(
//     name = sll,
//     opcode = "0110011",
//     funct3 = "001",
//     parse = rs1 / rs2 / rd,
//     constraints = rd - (rs1 * rs2) => 1
// );

// air!(
//     name = slt,
//     opcode = "0110011",
//     funct3 = "010",
//     parse = rs1 / rs2 / rd,
//     constraints =
//         rd * h0 + rs1 - rs2 => 2,
//         (E::ONE - rd) * h1 + rs1 - rs2 => 2
// );

// air!(
//     name = sltu,
//     opcode = "0110011",
//     funct3 = "011",
//     parse = rs1 / rs2 / rd,
//     constraints =
//         rd * h0 + rs1 - rs2 => 2,
//         (E::ONE - rd) * h1 + rs1 - rs2 => 2
// );

// air!(
//     name = xor,
//     opcode = "0110011",
//     funct3 = "100",
//     parse = rs1 / rs2 / rd,
//     constraints = rd - (rs1 + rs2 - E::from(2u32) * rs1 * rs2) => 2
// );

// air!(
//     name = srl,
//     opcode = "0110011",
//     funct3 = "101",
//     parse = rs1 / rs2 / rd,
//     constraints = rd - (rs1 * rs2) => 2
// );

// air!(
//     name = sra,
//     opcode = "0110011",
//     funct3 = "101",
//     funct7 = "0100000",
//     parse = rs1 / rs2 / rd,
//     constraints = rd - (rs1 * rs2) => 2
// );

// air!(
//     name = or,
//     opcode = "0110011",
//     funct3 = "110",
//     parse = rs1 / rs2 / rd,
//     constraints = bitwise rd - (rs1 + rs2 - rs1 * rs2) => 2
// );

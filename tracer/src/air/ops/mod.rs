use constraint_macros::air;

air! {
    name = lui,
    opcode = "0110111",
    parse = rd / uimm,
    constraints = rd - uimm => 1
}

air! {
    name = auipc,
    opcode = "0010111",
    parse = rd,
    constraints = next[PC] - (current[PC] + imm) => 1
}

air! {
    name = addi,
    opcode = "0010011",
    funct3 = "000",
    parse = rs1 / rd,
    constraints = rd - (rs1 + imm) => 1
}

air!(
    name = sltui,
    opcode = "0010011",
    funct3 = "010",
    parse = rs1 / rd,
    constraints =
        rd * h0 + rs1 - imm => 2,
        (E::ONE - rd) * h1 + rs1 - imm => 2
);

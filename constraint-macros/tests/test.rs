use constraint_macros::air;

#[test]
fn test() {
    air!(
        name = addi,
        opcode = "0010011",
        funct3 = "000",
        parse = rs1 / rd / imm,
        constraints =
            next[rd] == prev[rs1] + imm => 1,
            next[rd] == prev[rs1] + imm => 1
    );
}

pub const TRACE_WIDTH: usize = 254;
// 0-31: registers
pub const REGISTER_START: usize = 0;
pub const PC: usize = 32;
// 33-64: instruction at pc
pub const INS_END: usize = 33;
pub const UIMM_END: usize = INS_END;
pub const IMM_END: usize = INS_END;
pub const RS1_END: usize = IMM_END + 12;
pub const RS2_END: usize = INS_END + 7;
pub const RD_END: usize = UIMM_END + 20;
pub const OPCODE_END: usize = RD_END + 5;
pub const FUNCT3_END: usize = IMM_END + 17;
pub const SHAMT_END: usize = INS_END + 7;
// 65-96: rs1
pub const RS1_START: usize = 65;
// 97-128: rs2
pub const RS2_START: usize = 97;
// 129-160: rd
pub const RD_START: usize = 129;
// are we executing riscv code or is this padding?
pub const BODY: usize = 247;
// helper registers
pub const H_0: usize = 248;
pub const H_1: usize = 249;
pub const H_2: usize = 250;
pub const H_3: usize = 251;
pub const H_4: usize = 252;
pub const H_5: usize = 253;

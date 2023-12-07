// 0-31: registers
// 32: pc
// 33-64: instruction at pc
pub const TRACE_WIDTH: usize = 128;
pub const REGISTER_START: usize = 0;
pub const PC: usize = 32;
pub const INS_END: usize = 33;
pub const UIMM_END: usize = INS_END;
pub const IMM_END: usize = INS_END;
pub const RS1_END: usize = IMM_END + 12;
pub const RS2_END: usize = INS_END + 7;
pub const RD_END: usize = UIMM_END + 20;
pub const OPCODE_END: usize = RD_END + 5;
pub const FUNCT3_END: usize = IMM_END + 17;
// are we executing riscv code or is this padding?
pub const BODY: usize = 120;
// helper registers
pub const H_0: usize = 100;
pub const H_1: usize = 101;
pub const H_2: usize = 102;
pub const H_3: usize = 103;
pub const H_4: usize = 104;
pub const H_5: usize = 105;

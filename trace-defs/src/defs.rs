pub const MAIN_TRACE_WIDTH: usize = 254;
pub const AUX_TRACE_WIDTH: usize = 1;
// 0-31: registers
pub const REGISTER_START: usize = 0;
pub const PC: usize = 32;
pub const UNSIGNED_PC: usize = 198;
pub const PC_CONTENTS: usize = 199;
// 1 when we are loading the ELF in memory and 0 otherwise
pub const LOADING: usize = 200;
// TODO: can we remove this?
pub const READING_PC: usize = 201;
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
pub const JAL_OFFSET_END: usize = INS_END;
// 65-96: rs1
pub const RS1_BITS_END: usize = 65;
// 97-128: rs2
pub const RS2_BITS_END: usize = 97;
// 129-160: rd
pub const RD_BITS_END: usize = 129;
pub const CHIPLETS_START: usize = 161;
pub const CHIPLETS_WIDTH: usize = 17;
// are we executing riscv code or is this padding?
pub const BODY: usize = 190;
pub const CYCLE: usize = 191;
// helper registers
pub const H_0: usize = 192;
pub const H_1: usize = 193;
pub const H_2: usize = 194;
pub const H_3: usize = 195;
pub const H_4: usize = 196;
pub const H_5: usize = 197;

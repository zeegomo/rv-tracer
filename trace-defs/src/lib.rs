pub const CPU_TRACE_WIDTH: usize = RD_ZERO + 1;
pub const MEMORY_TRACE_WIDTH: usize = CHIPLETS_WIDTH;
pub const MAIN_TRACE_WIDTH: usize = CHIPLETS_WIDTH + CPU_TRACE_WIDTH + RANGE_WIDTH;
pub const AUX_TRACE_WIDTH: usize = 3;

// ------------------------CPU trace------------------------
pub const CYCLE: usize = 0;
pub const BODY: usize = 1;
// 1 when we are loading the ELF in memory and 0 otherwise
pub const LOADING: usize = 2;
pub const PC: usize = 3;
pub const INSN: usize = 4;
// 6-37: instruction at pc
pub const INS_END: usize = INSN + 1;
pub const UIMM_END: usize = INS_END;
pub const IMM_END: usize = INS_END;
pub const RS1_END: usize = IMM_END + 12;
pub const RS2_END: usize = INS_END + 7;
pub const RD_END: usize = UIMM_END + 20;
pub const OPCODE_END: usize = RD_END + 5;
pub const FUNCT3_END: usize = IMM_END + 17;
pub const FUNCT7_END: usize = INS_END;
pub const SHAMT_END: usize = INS_END + 7;
pub const JAL_OFFSET_END: usize = INS_END;
pub const BRANCH_OFFSET_31_25_END: usize = INS_END;
pub const BRANCH_OFFSET_11_7_END: usize = INS_END + 20;
// 38-69: rs1
pub const RS1_BITS_END: usize = INS_END + 32;
// 70-101: rs2
pub const RS2_BITS_END: usize = RS1_BITS_END + 32;
// 102-133: rd
pub const RD_BITS_END: usize = RS2_BITS_END + 32;
// are we executing riscv code or is this padding?
// helper registers
pub const H_0: usize = RD_BITS_END + 32;
pub const H_1: usize = H_0 + 1;
pub const H_2: usize = H_1 + 1;
pub const H_3: usize = H_2 + 1;
pub const FUNCT7_ZERO: usize = H_3 + 1;
pub const RD_ZERO: usize = FUNCT7_ZERO + 1;

// ------------------------Memory trace------------------------
pub const CHIPLETS_START: usize = CPU_TRACE_WIDTH;
pub const CHIPLETS_WIDTH: usize = 10;

// ------------------------Range check------------------------
pub const RANGE_START: usize = CHIPLETS_START + CHIPLETS_WIDTH;
pub const RANGE_WIDTH: usize = 2;

// ------------------------Aux trace -------------------------
// Winterfell requires an assertion for each segment in the aux trace.
// Since we can't predicat values of aux columns in the middle of the computation
// we need to introduce another columsn which is always zero
pub const AUX_DUMMY: usize = 2;

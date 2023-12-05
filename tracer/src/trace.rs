// 0-31: registers
// 32: pc
// 33-64: instruction at pc
// 65: is_r
// 66: is_i
// 67: is_s
// 68: is_u
// 69-100: is_rd_i // is 'i' the destination register
// 101-132: is_rs1_i // is 'i' the first source register
// 133-164: is_rs2_i // is 'i' the second source register
// 165: u_imm // immediate in a u-type instruction
pub const TRACE_WIDTH: usize = 128;
pub const REGISTER_START: usize = 0;
pub const PC: usize = 32;
pub const UIMM_END: usize = 33;
pub const IMM_END: usize = UIMM_END;
pub const RS1_END: usize = IMM_END + 12;
pub const RD_END: usize = UIMM_END + 20;
pub const OPCODE_END: usize = RD_END + 5;
pub const FUNCT3_END: usize = IMM_END + 17;
//
pub const BODY: usize = 120;
pub const H_0: usize = 100;
pub const H_1: usize = 101;
pub const H_2: usize = 102;
pub const H_3: usize = 103;
pub const H_4: usize = 104;
pub const H_5: usize = 105;


// #[derive(Debug, Clone, Copy)]
// pub struct Trace<E>(pub [E; 128]);

// impl<E> Trace<E>
// where
//     E: Copy,
// {
//     pub fn reg(&self, index: usize) -> E {
//         assert!(index < 32);
//         self.0[index + 2]
//     }

//     pub fn pc(&self) -> E {
//         self.0[1]
//     }

//     pub fn cycle(&self) -> E {
//         self.0[0]
//     }

//     pub fn opcode_0(&self) -> E {
//         self.0[34]
//     }

//     pub fn opcode_1(&self) -> E {
//         self.0[35]
//     }

//     pub fn opcode_2(&self) -> E {
//         self.0[36]
//     }

//     pub fn opcode_3(&self) -> E {
//         self.0[37]
//     }

//     pub fn opcode_4(&self) -> E {
//         self.0[38]
//     }

//     pub fn opcode_5(&self) -> E {
//         self.0[39]
//     }

//     pub fn opcode_6(&self) -> E {
//         self.0[40]
//     }

//     pub fn is_u(&self) -> E {
//         todo!()
//     }

//     // Return the immediate value in a U-type instruction
//     pub fn u_imm(&self) -> E {
//         // assert!(self.is_u() == E::ONE);
//         todo!()
//     }
// }

// impl<E> From<[E; 64]> for Trace<E> {
//     fn from(inner: [E; 64]) -> Self {
//         Self(inner)
//     }
// }

// seq_macro::seq!(rd in 0..32 {
//     impl<E> Trace<E> {
//         #(
//             pub fn is_rd_~rd(&self) -> E {
//                 todo!()
//             }
//         )*
//     }
// });

// seq_macro::seq!(rd in 0..32 {
//     impl<E> Trace<E> {
//         #(
//             pub fn is_rd_~rd(&self) -> E {
//                 todo!()
//             }
//         )*
//     }
// });

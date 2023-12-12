use clap::Parser;
use rv_tracer::*;
use rvsim::elf::Elf32;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use winterfell::{math::fields::f128::BaseElement, FieldExtension, ProofOptions};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

const NUM_QUERIES: usize = 10;
const BLOWUP_FACTOR: usize = 32;
const GRINDING_FACTOR: u32 = 5;
const FRI_FOLDING_FACTOR: usize = 4;
const FRI_REMAINDER_MAX_DEGREE: usize = 255;

type Blake3_192 = winterfell::crypto::hashers::Blake3_192<BaseElement>;

fn main() {
    env_logger::init();
    let args = Args::parse();

    let path = args.path.canonicalize().unwrap();
    let mut elf = Vec::new();
    File::open(path).unwrap().read_to_end(&mut elf).unwrap();
    let elf = Elf32::parse(&elf).unwrap();

    let proof = prove_from_elf::<Blake3_192>(
        elf,
        ProofOptions::new(
            NUM_QUERIES,
            BLOWUP_FACTOR,
            GRINDING_FACTOR,
            FieldExtension::None,
            FRI_FOLDING_FACTOR,
            FRI_REMAINDER_MAX_DEGREE,
        ),
    )
    .unwrap();
    verify::<Blake3_192>(proof).unwrap();
}

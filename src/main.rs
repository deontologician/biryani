use std::{ffi::CString, io::Write, os::unix::prelude::AsRawFd};

use memfile::{CreateOptions, MemFile, Seal};
use nix::unistd::fexecve;

#[link_section = ".ls.binary"]
static LS_BINARY: &[u8] = include_bytes!("../binaries/ls");

#[link_section = ".rg.binary"]
static RG_BINARY: &[u8] = include_bytes!("../binaries/rg");

fn main() {
    let args = vec!["in-memory-ls"];
    let env = vec![];
    analyze_elf(LS_BINARY);
    analyze_elf(RG_BINARY);
    exec_binary(LS_BINARY, args, env);
    println!("You shouldn't see this!");
}

fn analyze_elf(ls_binary: &[u8]) {
    match Elf::from_bytes(ls_binary).expect("parsing ELF failed") {
        Elf::Elf64(elfgen) => {
            let header = elfgen.elf_header();
            println!("Entry point: {}", header.entry_point());
            println!("Elfgen: {header:#?}");
            for section_header in elfgen.section_header_iter() {
                println!(
                    "{typ:?}: {name}",
                    name = std::str::from_utf8(section_header.section_name()).unwrap(),
                    typ = section_header.sh_type(),
                );
            }
        }
        _ => unreachable!("We only include elf64 binaries"),
    }
}

fn exec_binary(binary_bytes: &[u8], args: Vec<&str>, env: Vec<&str>) {
    let mut file = MemFile::create("in-memory-ls", CreateOptions::new().allow_sealing(true))
        .expect("Couldnt create memfile");
    file.write_all(binary_bytes)
        .expect("Couldn't write to the memfile");
    file.add_seals(Seal::Write | Seal::Grow | Seal::Shrink)
        .expect("Couldn't seal the memfile");
    exec_memfile(&file, args, env);
}

fn exec_memfile(memfile: &MemFile, args: Vec<&str>, env: Vec<&str>) {
    let c_args = to_cstrings(args);
    let c_env = to_cstrings(env);
    fexecve(memfile.as_raw_fd(), &c_args, &c_env).unwrap();
}

fn to_cstrings(arr: Vec<&str>) -> Vec<CString> {
    arr.iter()
        .map(|&x| CString::new(x).expect("Couldn't coerce to CString"))
        .collect()
}

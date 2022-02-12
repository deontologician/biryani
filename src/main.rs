use std::{
    ffi::{CString},
    io::Write,
    os::unix::prelude::AsRawFd,
};

use memfile::{CreateOptions, MemFile, Seal};
use nix::unistd::fexecve;

fn main() {
    let ls_binary = include_bytes!("../assets/ls");

    let mut file = MemFile::create("in-memory-ls", CreateOptions::new().allow_sealing(true))
        .expect("Couldnt create memfile");
    file.write_all(ls_binary)
        .expect("Couldn't write to the memfile");
    file.add_seals(Seal::Write | Seal::Grow | Seal::Shrink).expect("Couldn't seal the memfile");
    exec_memfile(&file, vec!["in-memory-ls", "--help"], vec![]);
    println!("You shouldn't see this!");
}

fn exec_memfile(memfile: &MemFile, args: Vec<&str>, env: Vec<&str>) {
    let c_args = to_cstrings(args);
    let c_env = to_cstrings(env);
    fexecve(memfile.as_raw_fd(), &c_args, &c_env).unwrap();
}

fn to_cstrings(arr: Vec<&str>) -> Vec<CString> {
    arr.iter().map(|&x| CString::new(x).expect("Couldn't coerce to CString")).collect()
}

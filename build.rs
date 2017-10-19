use std::env;
use std::process::Command;
use std::fs::File;
use std::string::String;
use std::io::prelude::*;

extern crate rand;

fn process_asm(asm: &str) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Split file into lines.
    let mut lines: Vec<String> = asm.split("\n")
        .map(|x| x.trim().to_string())
        .collect();

    // Strip SEH crap.
    lines.retain(|ref x| !x.starts_with(".seh") && !x.starts_with(".Lcfi"));

    // Locate & extract code section within assembly.
    // We force-inline everything, so this code just has to deal with a single
    // code section.
    let code_start = lines
        .iter()
        .position(|ref x| x.starts_with(".section\t.text"))
        .expect("Unable to find code section start");
    let code_end = lines[code_start + 1..]
        .iter()
        .position(|ref x| x.starts_with(".section"))
        .expect("Unable to find code section end") + code_start;

    // Rewrite code.
    let sane_code: Vec<_> = lines.drain(code_start + 1..code_end + 1).collect();
    let mut mad_code = Vec::<String>::with_capacity(sane_code.len());
    let mut i = 0usize;
    while i < sane_code.len() {
        let mut line = sane_code[i].to_owned();

        let is_label = line.ends_with(':');
        let is_directive = line.starts_with('.') && !is_label;
        let is_insn = !is_label && !is_directive;

        if line.starts_with("ret") {
            line = ".word 0x0B0F".to_string();
        }

        if is_insn || is_label {
            line = format!(".align 16\n{}", line);
        }

        if is_insn {
            // Make IDA and co recognize code around the instruction as data
            let rnd = rng.gen_range(std::i8::MIN, std::i8::MAX);
            line = format!("{}\njmp [rip + {}]", line, rnd);
        }

        if is_label {
            mad_code.push(format!("{}\n{}", line, sane_code[i + 1]));
            i += 2;
        }
        else {
            mad_code.push(line);
            i += 1;
        }
    }

    // Insert new code section.
    for (i, line) in mad_code.drain(..).rev().enumerate() {
        lines.insert(i + code_start + 1, line);
    }

    lines.join("\n")
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let asm_out = format!("{}/flag-logic.asm", out_dir);

    // Compile flag-logic.rs to native assembly.
    let status = Command::new("rustc").args(&[
        "src/flag-logic.rs",
        "-o", asm_out.as_str(),
        "--emit", "asm",
        "-C", "llvm-args=-x86-asm-syntax=intel", // FCK ATT
        "-C", "opt-level=s",
        "-C", "panic=abort",
        "--crate-type=lib",
    ]).status().expect("Unable to invoke rustc");

    if !status.success() {
        panic!("Unable to build flag logic");
    }

    // Pre-process assembly.
    let processed_asm_path = format!("{}/{}", out_dir, "flag-logic.proc.asm");
    let processed_asm = {
        let asm_file = File::open(&asm_out);
        let mut asm_content = String::new();
        asm_file
            .expect("Can't open ASM")
            .read_to_string(&mut asm_content)
            .expect("Can't read ASM");
        process_asm(&asm_content)
    };

    {
        let mut processed_asm_file = File::create(
            &processed_asm_path
        ).expect("Can't create file");
        processed_asm_file.write_all(processed_asm.as_bytes()).expect(
            "Can't write assembly to file"
        );
    }

    // Assemble again.
    let lib_path = format!("{}/{}", out_dir, "flag-logic.lib");
    let status = Command::new("clang").args(&[
        "-c", processed_asm_path.as_str(),
        "-o", lib_path.as_str(),
    ]).status().expect("Unable to invoke Clang");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=flag-logic");

    if !status.success() {
        panic!("Unable to reassemble");
    }
}
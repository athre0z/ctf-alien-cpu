#![feature(
    compiler_builtins_lib,
    core_intrinsics,
    lang_items,
    start,
    asm,
)]

extern crate winapi;

use std::{ptr,mem,env};
use std::path::Path;
use winapi::vc::excpt;
use winapi::um::{
    errhandlingapi,
    winnt,
    minwinbase,
    libloaderapi,
};

// ========================================================================== //
// [Utils]                                                                    //
// ========================================================================== //

unsafe fn calc_rip(rip: u64) -> u64 {
    let row = ((rip & (1 << 3)) >> 2) | (rip & 1);
    let col = ((rip & (1 << 2)) >> 2) | (rip & (1 << 1));

    let idx = ((row ^ 2) << 2) | (col ^ 1);
    let table = [12, 8, 14, 10, 13, 9, 15, 11, 4, 0, 6, 2, 5, 1, 7, 3];

    if idx == 9 { rip } else { (rip & !table[idx as usize]) - 16 }
}

// ========================================================================== //
// [Exception handler]                                                        //
// ========================================================================== //

const UD1: u16 = 0xB90F;
const UD2: u16 = 0x0B0F;
const TRAP_MASK: u32 = 1 << 8;

unsafe extern "system" fn vectored_handler(
    ep: *mut winnt::EXCEPTION_POINTERS,
) -> winnt::LONG {
    let cr = &mut *(*ep).ContextRecord;
    let er = &mut *(*ep).ExceptionRecord;

    match er.ExceptionCode {
        minwinbase::EXCEPTION_ILLEGAL_INSTRUCTION => {
            let ins_word = *mem::transmute::<_, *mut u16>(cr.Rip);

            match ins_word {
                UD1 => {
                    // Enter the matrix!
                    if cfg!(debug_assertions) {
                        println!("ud1!");
                    }

                    // Push current RIP + 10.
                    cr.Rsp -= 8;
                    *mem::transmute::<_, *mut u64>(cr.Rsp) = cr.Rip + 10;

                    // Update RIP.
                    cr.Rip = *mem::transmute::<_, *mut u64>(cr.Rip + 2);

                    // Set trap flag.
                    cr.EFlags |= TRAP_MASK;
                },
                UD2 => {
                    // Leave the matrix!
                    if cfg!(debug_assertions) {
                        println!("ud2!");
                    }

                    // Pop new RIP.
                    cr.Rip = *mem::transmute::<_, *mut u64>(cr.Rsp);
                    cr.Rsp += 8;

                    // Clear trap flag.
                    cr.EFlags &= !TRAP_MASK;
                },
                _ => panic!("unexpected SIGILL")
            }

            excpt::EXCEPTION_CONTINUE_EXECUTION
        },
        minwinbase::EXCEPTION_SINGLE_STEP => {
            // Keep up the matrix!
            if cfg!(debug_assertions) {
                println!(
                    "single step @ 0x{:016X}",
                    cr.Rip - libloaderapi::GetModuleHandleA(ptr::null()) as u64 + 0x140000000
                );
            }
            cr.Rip = calc_rip(cr.Rip);
            cr.EFlags |= TRAP_MASK;
            excpt::EXCEPTION_CONTINUE_EXECUTION
        },
        _ => panic!("unexpected exception")
    }
}

// ========================================================================== //
// [The upside down]                                                          //
// ========================================================================== //

unsafe fn call_flag_logic(flag: &str) -> bool {
    errhandlingapi::AddVectoredExceptionHandler(0, Some(vectored_handler));

    let result: u32;
    asm!(
        r#"
        mov     rcx, $1
        mov     rdx, $2
        .2byte  $3
        .8byte  upside_down_ep
        "#
        : "={eax}"(result)
        : "X"(flag.as_ptr())
          "X"(flag.len())
          "i"(UD1)
        : "rcx" "rdx"
        : "intel"
    );

    result != 0
}

// ========================================================================== //
// [Entry point]                                                              //
// ========================================================================== //


fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        eprintln!(
            "Usage: ./{} <flag>",
            Path::new(&args[0])
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
        );
        return;
    }

    println!("{}", match unsafe { call_flag_logic(args[1].as_str()) } {
        true  => "Yep, congrats.",
        false => "Try again.",
    });
}

// ========================================================================== //

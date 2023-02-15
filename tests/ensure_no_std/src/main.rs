// ensure_no_std/src/main.rs
#![no_std]
#![feature(core_intrinsics, lang_items, start, alloc_error_handler)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// This function is called on panic.
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &::core::panic::PanicInfo) -> ! { ::core::intrinsics::abort(); }

#[no_mangle]
extern "C" fn test(_argc: isize, _argv: *const *const u8) -> isize {
    let bytes = b"\xa6source";
    let expected = b"source";

    let actual: &[u8] = wasm_msgpack::decode::from_slice(bytes).unwrap();
    assert_eq!(expected, actual);

    0
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    panic!();
}

#[no_mangle]
pub extern "C" fn _start() -> ! { loop {} }

#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;
use esp_hal::init as _;
use postcard_schema::Schema;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOC: DummyAllocator = DummyAllocator;

#[esp_hal::main]
fn main() -> ! {
    loop {}
}

#[allow(unused)]
#[derive(Schema)]
struct UnitStruct;

#[allow(unused)]
#[derive(Schema)]
struct NewTypeStruct(String);

#[allow(unused)]
#[derive(Schema)]
struct TupStruct(u64, String);

#[allow(unused)]
#[derive(Schema)]
enum Enums {
    Unit,
    Nt(u64),
    Tup(u32, bool),
    Str { a: u32, b: u16, c: bool },
}

#[allow(unused)]
#[derive(Schema)]
struct Classic {
    a: u32,
    b: u16,
    c: bool,
}

#[allow(unused)]
#[derive(Schema)]
struct ClassicGen<T: Schema> {
    a: u32,
    b: T,
}

destore::export_schema!(_DESTORE_SCHEMA_BOOL, bool);
destore::export_schema!(_DESTORE_SCHEMA_I8, i8);
destore::export_schema!(_DESTORE_SCHEMA_U8, u8);
destore::export_schema!(_DESTORE_SCHEMA_I16, i16);
destore::export_schema!(_DESTORE_SCHEMA_I32, i32);
destore::export_schema!(_DESTORE_SCHEMA_I64, i64);
destore::export_schema!(_DESTORE_SCHEMA_I128, i128);
destore::export_schema!(_DESTORE_SCHEMA_U16, u16);
destore::export_schema!(_DESTORE_SCHEMA_U32, u32);
destore::export_schema!(_DESTORE_SCHEMA_U64, u64);
destore::export_schema!(_DESTORE_SCHEMA_U128, u128);
//destore::export_schema!(_DESTORE_SCHEMA_USIZE, usize); // Schema not implemented for usize ?!
//destore::export_schema!(_DESTORE_SCHEMA_ISIZE, isize); // Schema not implemented for isize ?!
destore::export_schema!(_DESTORE_SCHEMA_F32, f32);
destore::export_schema!(_DESTORE_SCHEMA_F64, f64);
destore::export_schema!(_DESTORE_SCHEMA_CHAR, char);
destore::export_schema!(_DESTORE_SCHEMA_STRING, String);
//destore::export_schema!(_DESTORE_SCHEMA_BYTEARRAY, &[u8]); // only ever used for uuids. &[u8] is type seq instead
destore::export_schema!(_DESTORE_SCHEMA_OPTION, Option<u8>);
destore::export_schema!(_DESTORE_SCHEMA_UNIT, ());
destore::export_schema!(_DESTORE_SCHEMA_SEQUENCE, &[u16]);
destore::export_schema!(_DESTORE_SCHEMA_TUPLE, (u8, u16, u32));
destore::export_schema!(_DESTORE_SCHEMA_MAP, alloc::collections::BTreeMap<u32, String>);
destore::export_schema!(_DESTORE_SCHEMA_UNITSTRUCT, UnitStruct);
destore::export_schema!(_DESTORE_SCHEMA_NEWTYPESTRUCT, NewTypeStruct);
destore::export_schema!(_DESTORE_SCHEMA_TUPLESTRUCT, TupStruct);
destore::export_schema!(_DESTORE_SCHEMA_STRUCTSTRUCT, Classic);
destore::export_schema!(_DESTORE_SCHEMA_ENUM, Enums);
destore::export_schema!(
    _DESTORE_SCHEMA_SCHEMA,
    postcard_schema::schema::DataModelType
);

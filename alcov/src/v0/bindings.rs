//! Bindings to the C reference implementation

#![expect(non_camel_case_types)]
#![expect(unsafe_op_in_unsafe_fn)]
#![expect(clippy::missing_safety_doc)]
#![expect(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

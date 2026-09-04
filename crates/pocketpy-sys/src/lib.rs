#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals)]

#[expect(
    clippy::pub_underscore_fields,
    clippy::doc_markdown,
    reason = "Generated bindgen declarations preserve upstream C ABI field names and documentation verbatim"
)]
mod bindings;

pub use bindings::*;

#[cfg(test)]
mod tests {
    use core::mem::{align_of, offset_of, size_of};

    use super::py_TValue;

    #[test]
    fn public_value_layout_matches_the_pocketpy_c_abi() {
        assert_eq!(size_of::<py_TValue>(), 24);
        assert_eq!(align_of::<py_TValue>(), 8);
        assert_eq!(offset_of!(py_TValue, type_), 0);
        assert_eq!(offset_of!(py_TValue, is_ptr), 2);
        assert_eq!(offset_of!(py_TValue, extra), 4);
        assert_eq!(offset_of!(py_TValue, __bindgen_anon_1), 8);
    }
}

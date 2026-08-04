use std::ffi::{CStr, c_int, c_void};
use std::mem;
use std::ptr;
use std::slice;

use ucharm_pocketpy_sys as ffi;

pub(crate) type Callback = unsafe extern "C" fn(c_int, ffi::py_StackRef) -> bool;

pub(crate) struct NativeFunction {
    pub name: &'static CStr,
    pub callback: Callback,
}

pub(crate) struct NativeSignature {
    pub signature: &'static CStr,
    pub callback: Callback,
}

pub(crate) struct NativeIntConstant {
    pub name: &'static CStr,
    pub value: i64,
}

pub(crate) struct NativeTypeAlias {
    pub name: &'static CStr,
    pub value_type: ffi::py_PredefinedType,
}

pub(crate) enum NativeModuleKind {
    Create,
    Extend,
}

pub(crate) struct NativeModule {
    pub name: &'static CStr,
    pub kind: NativeModuleKind,
    pub functions: &'static [NativeFunction],
    pub signatures: &'static [NativeSignature],
    pub int_constants: &'static [NativeIntConstant],
    pub type_aliases: &'static [NativeTypeAlias],
    pub initializer: Option<fn(Value)>,
}

pub(crate) fn register_modules(modules: &[NativeModule]) {
    for module in modules {
        // SAFETY: registration runs while the uniquely owned VM is active.
        // Module, function, constant, and alias names have static storage;
        // every callback uses PocketPy's required C ABI, and predefined type
        // objects remain valid for the lifetime of the VM.
        unsafe {
            let object = match module.kind {
                NativeModuleKind::Create => ffi::py_newmodule(module.name.as_ptr()),
                NativeModuleKind::Extend => {
                    let object = ffi::py_getmodule(module.name.as_ptr());
                    assert!(
                        !object.is_null(),
                        "native module extension target is missing"
                    );
                    object
                }
            };
            if let Some(initializer) = module.initializer {
                initializer(Value { raw: object });
            }
            for function in module.functions {
                ffi::py_bindfunc(object, function.name.as_ptr(), Some(function.callback));
            }
            for function in module.signatures {
                ffi::py_bind(object, function.signature.as_ptr(), Some(function.callback));
            }
            for constant in module.int_constants {
                let value = ffi::py_pushtmp();
                ffi::py_newint(value, constant.value);
                ffi::py_setdict(object, ffi::py_name(constant.name.as_ptr()), value);
                ffi::py_pop();
            }
            for alias in module.type_aliases {
                ffi::py_setdict(
                    object,
                    ffi::py_name(alias.name.as_ptr()),
                    ffi::py_tpobject(alias.value_type as ffi::py_Type),
                );
            }
        }
    }
}

pub(crate) fn create_type(module: Value, name: &'static CStr) -> ffi::py_Type {
    // SAFETY: `module` is the active module object, `name` has static storage,
    // and the object base type and null destructor require no custom lifetime.
    unsafe {
        ffi::py_newtype(
            name.as_ptr(),
            ffi::py_PredefinedType_tp_object as ffi::py_Type,
            module.raw,
            None,
        )
    }
}

pub(crate) fn type_object(value_type: ffi::py_Type) -> Value {
    // SAFETY: PocketPy type objects are process-global and remain valid for the
    // lifetime of the active VM.
    Value {
        raw: unsafe { ffi::py_tpobject(value_type) },
    }
}

pub(crate) fn bind_type_signature(
    value_type: ffi::py_Type,
    signature: &'static CStr,
    callback: Callback,
) {
    // SAFETY: the type object is VM-global, the signature has static storage,
    // and the callback uses PocketPy's C ABI.
    unsafe {
        ffi::py_bind(
            ffi::py_tpobject(value_type),
            signature.as_ptr(),
            Some(callback),
        )
    };
}

pub(crate) fn bind_type_method(value_type: ffi::py_Type, name: &'static CStr, callback: Callback) {
    // SAFETY: the type is active, the name has static storage, and the callback
    // uses PocketPy's C ABI.
    unsafe { ffi::py_bindmethod(value_type, name.as_ptr(), Some(callback)) };
}

pub(crate) struct Arguments {
    count: usize,
    values: ffi::py_StackRef,
}

impl Arguments {
    /// # Safety
    ///
    /// `values` must point to the active callback's PocketPy argument stack,
    /// which contains at least `count` initialized values.
    pub unsafe fn from_raw(count: c_int, values: ffi::py_StackRef) -> Self {
        Self {
            count: usize::try_from(count).unwrap_or(0),
            values,
        }
    }

    pub fn require_arity(&self, minimum: usize, maximum: usize) -> bool {
        if self.count < minimum {
            return type_error(c"too few arguments");
        }
        if self.count > maximum {
            return type_error(c"too many arguments");
        }
        true
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        if index >= self.count {
            return None;
        }
        // SAFETY: `from_raw` establishes a stack containing `count` values,
        // and the bounds check above keeps this pointer within that stack.
        Some(Value {
            raw: unsafe { self.values.add(index) },
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Value {
    raw: ffi::py_Ref,
}

#[derive(Clone, Copy)]
pub(crate) struct ValueSnapshot {
    raw: ffi::py_TValue,
}

impl Value {
    /// # Safety
    ///
    /// `raw` must point to an initialized PocketPy value that remains alive
    /// for every use of the returned wrapper.
    pub unsafe fn from_raw(raw: ffi::py_Ref) -> Self {
        Self { raw }
    }

    pub fn integer(self) -> Option<i64> {
        if !self.is_type(ffi::py_PredefinedType_tp_int) {
            return None;
        }
        // SAFETY: the exact type check above establishes an integer value.
        Some(unsafe { ffi::py_toint(self.raw) })
    }

    pub fn boolean(self) -> Option<bool> {
        if !self.is_type(ffi::py_PredefinedType_tp_bool) {
            return None;
        }
        // SAFETY: the exact type check above establishes a boolean value.
        Some(unsafe { ffi::py_tobool(self.raw) })
    }

    pub fn number(self) -> Option<f64> {
        if self.is_type(ffi::py_PredefinedType_tp_int) {
            // SAFETY: the exact type check above establishes an integer value.
            return Some(unsafe { ffi::py_toint(self.raw) } as f64);
        }
        if self.is_type(ffi::py_PredefinedType_tp_float) {
            // SAFETY: the exact type check above establishes a float value.
            return Some(unsafe { ffi::py_tofloat(self.raw) });
        }
        None
    }

    pub fn cast_number(self) -> Result<f64, ()> {
        let mut output = 0.0;
        // SAFETY: `self.raw` is initialized and `output` is writable. PocketPy
        // accepts integers or floats and raises TypeError for other types.
        if unsafe { ffi::py_castfloat(self.raw, &mut output) } {
            Ok(output)
        } else {
            Err(())
        }
    }

    pub fn equals(self, other: Self) -> Result<bool, ()> {
        // SAFETY: both values remain VM-rooted for the duration of equality,
        // which may invoke Python code and allocate.
        match unsafe { ffi::py_equal(self.raw, other.raw) } {
            -1 => Err(()),
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(()),
        }
    }

    pub fn less_than(self, other: Self) -> Result<bool, ()> {
        // SAFETY: both values remain VM-rooted for the duration of comparison,
        // which may invoke Python code and allocate.
        match unsafe { ffi::py_less(self.raw, other.raw) } {
            -1 => Err(()),
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(()),
        }
    }

    pub fn snapshot(self) -> ValueSnapshot {
        // SAFETY: `self.raw` points to one initialized value. The snapshot is
        // not a VM root and must not cross an allocating call before being
        // restored into a `RootFrame`.
        ValueSnapshot {
            raw: unsafe { *self.raw },
        }
    }

    pub fn is_none(self) -> bool {
        self.is_type(ffi::py_PredefinedType_tp_NoneType)
    }

    pub fn string(self) -> Option<String> {
        if !self.is_type(ffi::py_PredefinedType_tp_str) {
            return None;
        }
        let mut length = 0;
        // SAFETY: the exact type check establishes a string. PocketPy owns the
        // returned UTF-8 buffer for at least the duration of this callback.
        let data = unsafe { ffi::py_tostrn(self.raw, &mut length) };
        let length = usize::try_from(length).ok()?;
        if length == 0 {
            return Some(String::new());
        }
        // SAFETY: PocketPy returned `length` initialized bytes. Python strings
        // are represented as valid UTF-8 by PocketPy.
        let bytes = unsafe { slice::from_raw_parts(data.cast::<u8>(), length) };
        String::from_utf8(bytes.to_vec()).ok()
    }

    pub fn bytes(self) -> Option<Vec<u8>> {
        if !self.is_type(ffi::py_PredefinedType_tp_bytes) {
            return None;
        }
        let mut length = 0;
        // SAFETY: the exact type check establishes bytes. The returned buffer
        // remains valid until the next allocating VM operation, so copy it
        // before returning to safe Rust.
        let data = unsafe { ffi::py_tobytes(self.raw, &mut length) };
        let length = usize::try_from(length).ok()?;
        if length == 0 {
            return Some(Vec::new());
        }
        // SAFETY: PocketPy returned `length` initialized bytes.
        Some(unsafe { slice::from_raw_parts(data, length) }.to_vec())
    }

    pub fn truthy(self) -> bool {
        // SAFETY: every initialized PocketPy value supports truth conversion.
        unsafe { ffi::py_tobool(self.raw) }
    }

    pub fn is_type(self, value_type: ffi::py_PredefinedType) -> bool {
        // SAFETY: `self.raw` points to an initialized callback argument, and
        // predefined type identifiers fit PocketPy's `py_Type` representation.
        unsafe { ffi::py_istype(self.raw, value_type as ffi::py_Type) }
    }

    pub fn is_type_object(self, value_type: ffi::py_PredefinedType) -> bool {
        // SAFETY: `self.raw` is initialized and `py_tpobject` returns a
        // process-global type object for a predefined type.
        unsafe { ffi::py_isidentical(self.raw, ffi::py_tpobject(value_type as ffi::py_Type)) }
    }

    pub fn is_instance(self, value_type: ffi::py_Type) -> bool {
        // SAFETY: `self.raw` is initialized and `value_type` identifies a live
        // type in the active VM.
        unsafe { ffi::py_isinstance(self.raw, value_type) }
    }

    pub fn cast_integer(self) -> Result<i64, ()> {
        let mut output = 0;
        // SAFETY: `self.raw` is initialized and `output` is writable. PocketPy
        // raises TypeError when the value cannot be interpreted as an integer.
        if unsafe { ffi::py_castint(self.raw, &mut output) } {
            Ok(output)
        } else {
            Err(())
        }
    }

    pub fn list_len(self) -> Option<usize> {
        if !self.is_type(ffi::py_PredefinedType_tp_list) {
            return None;
        }
        // SAFETY: the exact type check above establishes a list.
        usize::try_from(unsafe { ffi::py_list_len(self.raw) }).ok()
    }

    pub fn list_item(self, index: usize) -> Option<Self> {
        let length = self.list_len()?;
        if index >= length {
            return None;
        }
        let index = c_int::try_from(index).ok()?;
        // SAFETY: the exact type and bounds checks establish a valid item.
        Some(Self {
            raw: unsafe { ffi::py_list_getitem(self.raw, index) },
        })
    }

    pub fn list_append(self, value: Self) {
        debug_assert!(self.is_type(ffi::py_PredefinedType_tp_list));
        // SAFETY: `self` is a list and both values remain VM-rooted for the
        // duration of the operation.
        unsafe { ffi::py_list_append(self.raw, value.raw) };
    }

    pub fn list_set(self, index: usize, value: Self) -> bool {
        let Some(length) = self.list_len() else {
            return false;
        };
        if index >= length {
            return false;
        }
        let Ok(index) = c_int::try_from(index) else {
            return false;
        };
        // SAFETY: the exact type and bounds checks establish a valid list slot.
        // PocketPy copies `value` before the mutation can invalidate item refs.
        unsafe { ffi::py_list_setitem(self.raw, index, value.raw) };
        true
    }

    pub fn list_delete(self, index: usize) -> bool {
        let Some(length) = self.list_len() else {
            return false;
        };
        if index >= length {
            return false;
        }
        let Ok(index) = c_int::try_from(index) else {
            return false;
        };
        // SAFETY: the exact type and bounds checks establish a valid list slot.
        unsafe { ffi::py_list_delitem(self.raw, index) };
        true
    }

    pub fn list_swap(self, first: usize, second: usize) -> bool {
        let Some(length) = self.list_len() else {
            return false;
        };
        if first >= length || second >= length {
            return false;
        }
        let (Ok(first), Ok(second)) = (c_int::try_from(first), c_int::try_from(second)) else {
            return false;
        };
        // SAFETY: the exact type and bounds checks establish two valid slots.
        unsafe { ffi::py_list_swap(self.raw, first, second) };
        true
    }

    pub fn tuple_len(self) -> Option<usize> {
        if !self.is_type(ffi::py_PredefinedType_tp_tuple) {
            return None;
        }
        // SAFETY: the exact type check above establishes a tuple.
        usize::try_from(unsafe { ffi::py_tuple_len(self.raw) }).ok()
    }

    pub fn tuple_item(self, index: usize) -> Option<Self> {
        let length = self.tuple_len()?;
        if index >= length {
            return None;
        }
        let index = c_int::try_from(index).ok()?;
        // SAFETY: the exact type and bounds checks establish a valid item.
        Some(Self {
            raw: unsafe { ffi::py_tuple_getitem(self.raw, index) },
        })
    }

    pub fn tuple_set(self, index: usize, value: Self) -> bool {
        let Some(length) = self.tuple_len() else {
            return false;
        };
        if index >= length {
            return false;
        }
        let Ok(index) = c_int::try_from(index) else {
            return false;
        };
        // SAFETY: the exact type and bounds checks establish a valid tuple
        // slot, and both values remain rooted during the assignment.
        unsafe { ffi::py_tuple_setitem(self.raw, index, value.raw) };
        true
    }

    pub fn dict_set(self, key: Self, value: Self) -> bool {
        debug_assert!(self.is_type(ffi::py_PredefinedType_tp_dict));
        // SAFETY: `self` is a dictionary and all values remain VM-rooted for
        // the duration of the operation.
        unsafe { ffi::py_dict_setitem(self.raw, key.raw, value.raw) }
    }

    pub fn attribute(self, name: &'static CStr) -> Option<Self> {
        // SAFETY: `self` is initialized and `name` has static storage. The
        // returned item remains valid until this object's dictionary changes.
        let raw = unsafe { ffi::py_getdict(self.raw, ffi::py_name(name.as_ptr())) };
        (!raw.is_null()).then_some(Self { raw })
    }

    pub fn set_attribute(self, name: &'static CStr, value: Self) {
        // SAFETY: both values are initialized and `name` has static storage.
        unsafe { ffi::py_setdict(self.raw, ffi::py_name(name.as_ptr()), value.raw) };
    }

    pub fn slot(self, index: usize) -> Self {
        let index = c_int::try_from(index).expect("native object slot index fits c_int");
        // SAFETY: callers use this only with native types and slot indices
        // established when those objects were created.
        Self {
            raw: unsafe { ffi::py_getslot(self.raw, index) },
        }
    }

    pub fn set_slot(self, index: usize, value: Self) {
        let index = c_int::try_from(index).expect("native object slot index fits c_int");
        // SAFETY: callers use this only with native types and slot indices
        // established when those objects were created.
        unsafe { ffi::py_setslot(self.raw, index, value.raw) };
    }

    pub fn set_slot_snapshot(self, index: usize, value: ValueSnapshot) {
        let index = c_int::try_from(index).expect("native object slot index fits c_int");
        // SAFETY: the snapshot contains one initialized value and the native
        // object owns the requested slot.
        unsafe {
            ffi::py_setslot(
                self.raw,
                index,
                (&value.raw as *const ffi::py_TValue).cast_mut(),
            )
        };
    }

    /// # Safety
    ///
    /// The value must be an object created with at least `size_of::<T>()`
    /// bytes of userdata initialized as `T`.
    pub unsafe fn userdata<T>(self) -> *mut T {
        // SAFETY: upheld by the caller as documented above.
        unsafe { ffi::py_touserdata(self.raw).cast::<T>() }
    }
}

impl ValueSnapshot {
    pub fn value(&self) -> Value {
        Value {
            raw: (&self.raw as *const ffi::py_TValue).cast_mut(),
        }
    }
}

pub(crate) fn global_list(index: c_int) -> Value {
    assert!((0..8).contains(&index), "PocketPy global register index");
    // SAFETY: the VM is active and its eight global scratch registers have
    // stable addresses. Replacing one with a list keeps that list GC-rooted.
    let raw = unsafe { ffi::py_getreg(index) };
    unsafe { ffi::py_newlist(raw) };
    Value { raw }
}

pub(crate) fn global_string_bytes(index: c_int, value: &[u8]) -> Option<Value> {
    assert!((0..8).contains(&index), "PocketPy global register index");
    let length = c_int::try_from(value.len()).ok()?;
    // SAFETY: the register has a stable address and PocketPy reserves exactly
    // `length` bytes plus a trailing NUL for the new string.
    let raw = unsafe { ffi::py_getreg(index) };
    let destination = unsafe { ffi::py_newstrn(raw, length) };
    if !value.is_empty() {
        // SAFETY: both regions cover `value.len()` bytes and do not overlap.
        unsafe { ptr::copy_nonoverlapping(value.as_ptr(), destination.cast(), value.len()) };
    }
    Some(Value { raw })
}

pub(crate) fn global_integer(index: c_int, value: i64) -> Value {
    assert!((0..8).contains(&index), "PocketPy global register index");
    // SAFETY: the VM is active and the selected global register has a stable
    // writable address.
    let raw = unsafe { ffi::py_getreg(index) };
    unsafe { ffi::py_newint(raw, value) };
    Value { raw }
}

pub(crate) fn call_one_bool(function: Value, argument: Value) -> Result<bool, ()> {
    // Copy both values before PocketPy grows its stack for the call. Their
    // owning Python objects remain rooted by the active callback arguments or
    // containers, while these local trivial values give `py_call` stable refs.
    let mut function = unsafe { *function.raw };
    let mut argument = unsafe { *argument.raw };
    // SAFETY: both local values are initialized and form a one-element argv.
    if !unsafe { ffi::py_call(&mut function, 1, &mut argument) } {
        return Err(());
    }
    // SAFETY: a successful call initializes PocketPy's return register.
    let returned = Value {
        raw: unsafe { ffi::py_retval() },
    };
    let Some(value) = returned.boolean() else {
        type_error(c"predicate must return bool");
        return Err(());
    };
    Ok(value)
}

/// A LIFO frame for PocketPy temporary stack roots.
///
/// Values created by native callbacks must live on PocketPy's stack—not only
/// on Rust's stack—while later VM calls may allocate. This frame owns those
/// roots and releases them in one place when the callback finishes.
pub(crate) struct RootFrame {
    count: usize,
}

impl RootFrame {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    fn push(&mut self) -> Value {
        // SAFETY: native callbacks run with the VM active. Each successful
        // push is paired with one `py_pop` in this frame's `Drop`.
        let raw = unsafe { ffi::py_pushtmp() };
        self.count += 1;
        Value { raw }
    }

    pub fn top(&self) -> Option<Value> {
        if self.count == 0 {
            return None;
        }
        // SAFETY: this frame has at least one live root. Callers use `top`
        // only when no later frame is active, so the frame's last root is TOS.
        Some(Value {
            raw: unsafe { ffi::py_peek(-1) },
        })
    }

    pub fn copy_returned(&mut self) -> Value {
        // Copy the return register before any other PocketPy call can replace
        // it. The copied value is then installed in a VM-visible root.
        let returned = unsafe { *ffi::py_retval() };
        let root = self.push();
        // SAFETY: `root.raw` points to writable storage for one value.
        unsafe { ptr::write(root.raw, returned) };
        root
    }

    pub fn copy(&mut self, value: Value) -> Value {
        // Copy before pushing because a VM stack growth could otherwise move
        // the source slot supplied by another temporary root.
        let copied = unsafe { *value.raw };
        let root = self.push();
        // SAFETY: `root.raw` points to writable storage for one value.
        unsafe { ptr::write(root.raw, copied) };
        root
    }

    pub fn restore(&mut self, snapshot: ValueSnapshot) -> Value {
        let root = self.push();
        // SAFETY: `root.raw` points to writable storage for one value. The
        // caller guarantees no allocation occurred while the snapshot was
        // temporarily unrooted.
        unsafe { ptr::write(root.raw, snapshot.raw) };
        root
    }

    pub fn string(&mut self, value: &str) -> Option<Value> {
        let length = c_int::try_from(value.len()).ok()?;
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage and PocketPy reserves
        // exactly `length` bytes plus its trailing NUL.
        let destination = unsafe { ffi::py_newstrn(root.raw, length) };
        if !value.is_empty() {
            // SAFETY: both buffers are valid for `value.len()` bytes and do
            // not overlap.
            unsafe {
                ptr::copy_nonoverlapping(value.as_ptr(), destination.cast::<u8>(), value.len())
            };
        }
        Some(root)
    }

    pub fn bytes(&mut self, value: &[u8]) -> Option<Value> {
        let length = c_int::try_from(value.len()).ok()?;
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage and PocketPy reserves
        // exactly `length` bytes.
        let destination = unsafe { ffi::py_newbytes(root.raw, length) };
        if !value.is_empty() {
            // SAFETY: both buffers are valid for `value.len()` bytes and do
            // not overlap.
            unsafe { ptr::copy_nonoverlapping(value.as_ptr(), destination, value.len()) };
        }
        Some(root)
    }

    pub fn integer(&mut self, value: i64) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newint(root.raw, value) };
        root
    }

    pub fn number(&mut self, value: f64) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newfloat(root.raw, value) };
        root
    }

    pub fn boolean(&mut self, value: bool) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newbool(root.raw, value) };
        root
    }

    pub fn none(&mut self) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newnone(root.raw) };
        root
    }

    pub fn list(&mut self) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newlist(root.raw) };
        root
    }

    pub fn tuple(&mut self, length: usize) -> Option<Value> {
        let length = c_int::try_from(length).ok()?;
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage and every tuple slot is
        // initialized by the caller before the tuple becomes observable.
        unsafe { ffi::py_newtuple(root.raw, length) };
        Some(root)
    }

    pub fn dict(&mut self) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newdict(root.raw) };
        root
    }

    pub fn object(&mut self, value_type: ffi::py_Type) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage. A slot count of -1 gives
        // the object a normal attribute dictionary and no userdata is needed.
        unsafe { ffi::py_newobject(root.raw, value_type, -1, 0) };
        root
    }

    pub fn object_with_userdata<T: Copy>(
        &mut self,
        value_type: ffi::py_Type,
        slots: c_int,
        userdata: T,
    ) -> Option<Value> {
        if slots < -1 {
            return None;
        }
        let size = c_int::try_from(mem::size_of::<T>()).ok()?;
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage. PocketPy allocates the
        // requested slots followed by `size_of::<T>()` userdata bytes, whose
        // alignment supports PocketPy's own pointer-sized native states.
        let destination = unsafe { ffi::py_newobject(root.raw, value_type, slots, size) };
        if !(destination as usize).is_multiple_of(mem::align_of::<T>()) {
            return None;
        }
        // SAFETY: the allocation above reserved enough suitably aligned bytes
        // and `T: Copy` requires no destructor registration.
        unsafe { ptr::write(destination.cast::<T>(), userdata) };
        Some(root)
    }

    pub fn dict_get(&mut self, dict: Value, key: Value) -> Result<Option<Value>, ()> {
        debug_assert!(dict.is_type(ffi::py_PredefinedType_tp_dict));
        // SAFETY: `dict` and `key` remain rooted throughout the operation.
        match unsafe { ffi::py_dict_getitem(dict.raw, key.raw) } {
            -1 => Err(()),
            0 => Ok(None),
            1 => Ok(Some(self.copy_returned())),
            _ => Err(()),
        }
    }
}

impl Drop for RootFrame {
    fn drop(&mut self) {
        for _ in 0..self.count {
            // SAFETY: `count` exactly tracks roots pushed by this frame and
            // frames are dropped in lexical LIFO order.
            unsafe { ffi::py_pop() };
        }
    }
}

pub(crate) type DictCallback = unsafe extern "C" fn(ffi::py_Ref, ffi::py_Ref, *mut c_void) -> bool;

pub(crate) fn dict_apply(dict: Value, callback: DictCallback, context: *mut c_void) -> bool {
    debug_assert!(dict.is_type(ffi::py_PredefinedType_tp_dict));
    // SAFETY: `dict` remains rooted for the call and `context` is supplied by
    // the caller with a lifetime covering the synchronous traversal.
    unsafe { ffi::py_dict_apply(dict.raw, Some(callback), context) }
}

pub(crate) fn return_value(value: Value) -> bool {
    // SAFETY: the value and return register are initialized storage for one
    // `py_TValue`; `ptr::copy` permits overlap.
    unsafe { ptr::copy(value.raw, ffi::py_retval(), 1) };
    true
}

pub(crate) fn return_string(value: &str) -> bool {
    return_string_bytes(value.as_bytes())
}

pub(crate) fn return_string_bytes(value: &[u8]) -> bool {
    let Ok(length) = c_int::try_from(value.len()) else {
        return type_error(c"return string is too large");
    };
    // SAFETY: the VM is active during a callback. `py_newstrn` reserves exactly
    // `length` writable bytes in the return register.
    let destination = unsafe { ffi::py_newstrn(ffi::py_retval(), length) };
    if !value.is_empty() {
        // SAFETY: both regions are valid for `value.len()` bytes and do not
        // overlap; PocketPy owns the destination after this copy.
        unsafe { ptr::copy_nonoverlapping(value.as_ptr(), destination.cast::<u8>(), value.len()) };
    }
    true
}

pub(crate) fn return_string_list(values: &[String]) -> bool {
    // Global scratch registers have stable addresses and remain VM roots while
    // list appends allocate. This avoids retaining pointers into PocketPy's
    // movable value stack when a callback returns many strings.
    let list = unsafe { ffi::py_getreg(0) };
    let item = unsafe { ffi::py_getreg(1) };
    unsafe { ffi::py_newlist(list) };
    for value in values {
        let Ok(length) = c_int::try_from(value.len()) else {
            return type_error(c"return string is too large");
        };
        let destination = unsafe { ffi::py_newstrn(item, length) };
        if !value.is_empty() {
            // SAFETY: PocketPy reserved exactly `value.len()` writable bytes
            // and the source and destination do not overlap.
            unsafe {
                ptr::copy_nonoverlapping(value.as_ptr(), destination.cast::<u8>(), value.len())
            };
        }
        // SAFETY: both scratch registers contain initialized, globally rooted
        // values and `list` is a list for the duration of this callback.
        unsafe { ffi::py_list_append(list, item) };
    }
    // SAFETY: both locations contain initialized values and do not overlap.
    unsafe { ptr::copy_nonoverlapping(list, ffi::py_retval(), 1) };
    true
}

pub(crate) fn return_bytes(value: &[u8]) -> bool {
    let mut roots = RootFrame::new();
    let Some(value) = roots.bytes(value) else {
        return value_error(c"return bytes are too large");
    };
    return_value(value)
}

pub(crate) fn return_number(value: f64) -> bool {
    let mut roots = RootFrame::new();
    let value = roots.number(value);
    return_value(value)
}

pub(crate) fn type_error(message: &'static CStr) -> bool {
    // SAFETY: the VM is active during a callback. The format string and message
    // have static storage and match PocketPy's `%s` vararg contract.
    unsafe {
        ffi::py_exception(
            ffi::py_PredefinedType_tp_TypeError as ffi::py_Type,
            c"%s".as_ptr(),
            message.as_ptr(),
        )
    }
}

pub(crate) fn runtime_error(message: &'static CStr) -> bool {
    // SAFETY: the VM is active during a callback. The format string and message
    // have static storage and match PocketPy's `%s` vararg contract.
    unsafe {
        ffi::py_exception(
            ffi::py_PredefinedType_tp_RuntimeError as ffi::py_Type,
            c"%s".as_ptr(),
            message.as_ptr(),
        )
    }
}

pub(crate) fn value_error(message: &'static CStr) -> bool {
    // SAFETY: the VM is active during a callback. The format string and message
    // have static storage and match PocketPy's `%s` vararg contract.
    unsafe {
        ffi::py_exception(
            ffi::py_PredefinedType_tp_ValueError as ffi::py_Type,
            c"%s".as_ptr(),
            message.as_ptr(),
        )
    }
}

pub(crate) fn stop_iteration() -> bool {
    // SAFETY: the VM is active during a callback and the empty message has
    // static storage.
    unsafe {
        ffi::py_exception(
            ffi::py_PredefinedType_tp_StopIteration as ffi::py_Type,
            c"%s".as_ptr(),
            c"".as_ptr(),
        )
    }
}

pub(crate) fn index_error(message: &'static CStr) -> bool {
    // SAFETY: the VM is active during a callback. The format string and message
    // have static storage and match PocketPy's `%s` vararg contract.
    unsafe {
        ffi::py_exception(
            ffi::py_PredefinedType_tp_IndexError as ffi::py_Type,
            c"%s".as_ptr(),
            message.as_ptr(),
        )
    }
}

use std::ffi::c_int;
use std::sync::atomic::{AtomicI16, Ordering};

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, RootFrame, Value, bind_type_method,
    bind_type_signature, call_one_bool, create_type, global_integer, global_list,
    global_string_bytes, return_value, stop_iteration, type_error, value_error,
};

static COUNT_TYPE: AtomicI16 = AtomicI16::new(0);
static CYCLE_TYPE: AtomicI16 = AtomicI16::new(0);
static REPEAT_TYPE: AtomicI16 = AtomicI16::new(0);

#[derive(Clone, Copy)]
#[repr(C)]
struct CountState {
    current: i64,
    step: i64,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct CycleState {
    index: usize,
    length: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct RepeatState {
    times: i64,
}

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"count(start=0, step=1)",
        callback: count,
    },
    NativeSignature {
        signature: c"cycle(iterable)",
        callback: cycle,
    },
    NativeSignature {
        signature: c"repeat(object, times=None)",
        callback: repeat,
    },
    NativeSignature {
        signature: c"chain(*iterables)",
        callback: chain,
    },
    NativeSignature {
        signature: c"islice(iterable, *args)",
        callback: islice,
    },
    NativeSignature {
        signature: c"takewhile(predicate, iterable)",
        callback: takewhile,
    },
    NativeSignature {
        signature: c"dropwhile(predicate, iterable)",
        callback: dropwhile,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"itertools",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    let count_type = create_type(module, c"count");
    COUNT_TYPE.store(count_type, Ordering::Release);
    bind_type_signature(count_type, c"__new__(cls, start=0, step=1)", count_new);
    bind_type_method(count_type, c"__iter__", iterator_self);
    bind_type_method(count_type, c"__next__", count_next);

    let cycle_type = create_type(module, c"cycle");
    CYCLE_TYPE.store(cycle_type, Ordering::Release);
    bind_type_method(cycle_type, c"__iter__", iterator_self);
    bind_type_method(cycle_type, c"__next__", cycle_next);

    let repeat_type = create_type(module, c"repeat");
    REPEAT_TYPE.store(repeat_type, Ordering::Release);
    bind_type_method(repeat_type, c"__iter__", iterator_self);
    bind_type_method(repeat_type, c"__next__", repeat_next);
}

unsafe extern "C" fn count_new(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 3) {
        return false;
    }
    let Some((start, step)) = count_arguments(&arguments, 1) else {
        return false;
    };
    new_count(start, step)
}

unsafe extern "C" fn count(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 2) {
        return false;
    }
    let Some((start, step)) = count_arguments(&arguments, 0) else {
        return false;
    };
    new_count(start, step)
}

fn count_arguments(arguments: &Arguments, offset: usize) -> Option<(i64, i64)> {
    let start = match arguments.get(offset) {
        Some(value) => value.integer().or_else(|| {
            type_error(c"count() arguments must be integers");
            None
        })?,
        None => 0,
    };
    let step = match arguments.get(offset + 1) {
        Some(value) => value.integer().or_else(|| {
            type_error(c"count() arguments must be integers");
            None
        })?,
        None => 1,
    };
    Some((start, step))
}

fn new_count(current: i64, step: i64) -> bool {
    let mut roots = RootFrame::new();
    let Some(instance) =
        roots.object_with_userdata(load_type(&COUNT_TYPE), -1, CountState { current, step })
    else {
        return value_error(c"count state is too large");
    };
    return_value(instance)
}

unsafe extern "C" fn iterator_self(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    return_value(arguments.get(0).expect("arity checked"))
}

unsafe extern "C" fn count_next(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let instance = arguments.get(0).expect("arity checked");
    // SAFETY: this method is bound only to objects created with `CountState`.
    let state = unsafe { &mut *instance.userdata::<CountState>() };
    let current = state.current;
    state.current = state.current.wrapping_add(state.step);
    return_value(global_integer(6, current))
}

unsafe extern "C" fn cycle(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let iterable_snapshot = arguments.get(0).expect("arity checked").snapshot();
    let iterable = iterable_snapshot.value();
    let items = global_list(7);
    if let Some(length) = iterable.list_len() {
        for index in 0..length {
            items.list_append(iterable.list_item(index).expect("index is in bounds"));
        }
    } else if let Some(value) = iterable.string() {
        for byte in value.bytes() {
            let Some(item) = global_string_bytes(6, &[byte]) else {
                return value_error(c"cycle() string item is too large");
            };
            items.list_append(item);
        }
    } else {
        return type_error(c"cycle() argument must be list or string");
    }

    let length = items.list_len().expect("scratch value is a list");
    let mut roots = RootFrame::new();
    let Some(instance) =
        roots.object_with_userdata(load_type(&CYCLE_TYPE), 1, CycleState { index: 0, length })
    else {
        return value_error(c"cycle state is too large");
    };
    instance.set_slot(0, items);
    return_value(instance)
}

unsafe extern "C" fn cycle_next(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let instance = arguments.get(0).expect("arity checked");
    // SAFETY: this method is bound only to objects created with `CycleState`.
    let state = unsafe { &mut *instance.userdata::<CycleState>() };
    if state.length == 0 {
        return stop_iteration();
    }
    let item = instance
        .slot(0)
        .list_item(state.index)
        .expect("cycle index is in bounds");
    state.index = (state.index + 1) % state.length;
    return_value(item)
}

unsafe extern "C" fn repeat(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 2) {
        return false;
    }
    let value = arguments.get(0).expect("arity checked").snapshot();
    let mut times = -1;
    if let Some(candidate) = arguments.get(1)
        && !candidate.is_none()
    {
        let Ok(candidate) = candidate.cast_integer() else {
            return false;
        };
        times = candidate.max(0);
    }

    let mut roots = RootFrame::new();
    let Some(instance) =
        roots.object_with_userdata(load_type(&REPEAT_TYPE), 1, RepeatState { times })
    else {
        return value_error(c"repeat state is too large");
    };
    instance.set_slot_snapshot(0, value);
    return_value(instance)
}

unsafe extern "C" fn repeat_next(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let instance = arguments.get(0).expect("arity checked");
    // SAFETY: this method is bound only to objects created with `RepeatState`.
    let state = unsafe { &mut *instance.userdata::<RepeatState>() };
    if state.times == 0 {
        return stop_iteration();
    }
    if state.times > 0 {
        state.times -= 1;
    }
    return_value(instance.slot(0))
}

unsafe extern "C" fn chain(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let mut roots = RootFrame::new();
    roots.list();
    let output = roots.top().expect("output list remains rooted");
    if argc == 1 {
        let iterables = arguments.get(0).expect("argc is one");
        if let Some(length) = iterables.tuple_len() {
            for index in 0..length {
                if !append_iterable(
                    output,
                    iterables.tuple_item(index).expect("index is in bounds"),
                ) {
                    return type_error(c"chain() arguments must be iterable");
                }
            }
            return return_value(output);
        }
        if append_iterable(output, iterables) {
            return return_value(output);
        }
    }
    for index in 0..usize::try_from(argc).unwrap_or(0) {
        if !append_iterable(output, arguments.get(index).expect("index is in bounds")) {
            return type_error(c"chain() arguments must be iterable");
        }
    }
    return_value(output)
}

fn append_iterable(output: Value, iterable: Value) -> bool {
    if let Some(length) = iterable.list_len() {
        for index in 0..length {
            output.list_append(iterable.list_item(index).expect("index is in bounds"));
        }
        return true;
    }
    if let Some(length) = iterable.tuple_len() {
        for index in 0..length {
            output.list_append(iterable.tuple_item(index).expect("index is in bounds"));
        }
        return true;
    }
    if let Some(value) = iterable.string() {
        for byte in value.bytes() {
            let Some(item) = global_string_bytes(6, &[byte]) else {
                return false;
            };
            output.list_append(item);
        }
        return true;
    }
    false
}

unsafe extern "C" fn islice(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let iterable_snapshot = arguments.get(0).expect("arity checked").snapshot();
    let packed = arguments.get(1).expect("arity checked");
    let Some(count) = packed.tuple_len() else {
        return type_error(c"islice() args must be tuple");
    };
    if !(1..=3).contains(&count) {
        return type_error(c"islice() requires 1 to 3 positional args after iterable");
    }
    let mut values = [0_i64, 0, 1];
    for (index, destination) in values.iter_mut().enumerate().take(count) {
        let Some(value) = packed.tuple_item(index).and_then(Value::integer) else {
            return type_error(c"islice() arguments must be integers");
        };
        *destination = value;
    }
    let (start, stop, step) = match count {
        1 => (0, values[0], 1),
        2 => (values[0], values[1], 1),
        3 => (values[0], values[1], values[2]),
        _ => unreachable!(),
    };
    if step < 1 {
        return value_error(c"step must be >= 1");
    }

    let iterable = iterable_snapshot.value();
    let mut roots = RootFrame::new();
    roots.list();
    let output = roots.top().expect("output list remains rooted");
    if iterable.is_instance(load_type(&COUNT_TYPE)) {
        // SAFETY: the instance check establishes `CountState` userdata.
        let state = unsafe { &mut *iterable.userdata::<CountState>() };
        for index in 0..stop {
            if selected(index, start, step) {
                output.list_append(global_integer(6, state.current));
            }
            state.current = state.current.wrapping_add(state.step);
        }
        return return_value(output);
    }
    if iterable.is_instance(load_type(&CYCLE_TYPE)) {
        // SAFETY: the instance check establishes `CycleState` userdata.
        let state = unsafe { &mut *iterable.userdata::<CycleState>() };
        if state.length == 0 {
            return return_value(output);
        }
        let items = iterable.slot(0);
        for index in 0..stop {
            if selected(index, start, step) {
                output.list_append(
                    items
                        .list_item(state.index)
                        .expect("cycle index is in bounds"),
                );
            }
            state.index = (state.index + 1) % state.length;
        }
        return return_value(output);
    }
    if let Some(length) = iterable.list_len() {
        let available = i64::try_from(length).unwrap_or(i64::MAX);
        for index in 0..stop.min(available) {
            if selected(index, start, step) {
                output.list_append(
                    iterable
                        .list_item(usize::try_from(index).expect("index is non-negative"))
                        .expect("index is in bounds"),
                );
            }
        }
        return return_value(output);
    }
    type_error(c"islice() iterable must be list, count, or cycle")
}

fn selected(index: i64, start: i64, step: i64) -> bool {
    index >= start && (index - start) % step == 0
}

unsafe extern "C" fn takewhile(argc: c_int, argv: ffi::py_StackRef) -> bool {
    predicate_list(argc, argv, true)
}

unsafe extern "C" fn dropwhile(argc: c_int, argv: ffi::py_StackRef) -> bool {
    predicate_list(argc, argv, false)
}

fn predicate_list(argc: c_int, argv: ffi::py_StackRef, take: bool) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let predicate_snapshot = arguments.get(0).expect("arity checked").snapshot();
    let iterable_snapshot = arguments.get(1).expect("arity checked").snapshot();
    let predicate = predicate_snapshot.value();
    let iterable = iterable_snapshot.value();
    let Some(length) = iterable.list_len() else {
        return type_error(if take {
            c"takewhile() iterable must be a list"
        } else {
            c"dropwhile() iterable must be a list"
        });
    };
    let mut roots = RootFrame::new();
    roots.list();
    let mut dropping = !take;
    for index in 0..length {
        let item = iterable
            .list_item(index)
            .expect("index is in bounds")
            .snapshot();
        if take {
            match call_one_bool(predicate, item.value()) {
                Ok(true) => roots
                    .top()
                    .expect("output list remains rooted")
                    .list_append(item.value()),
                Ok(false) => break,
                Err(()) => return false,
            }
        } else if dropping {
            match call_one_bool(predicate, item.value()) {
                Ok(true) => {}
                Ok(false) => {
                    dropping = false;
                    roots
                        .top()
                        .expect("output list remains rooted")
                        .list_append(item.value());
                }
                Err(()) => return false,
            }
        } else {
            roots
                .top()
                .expect("output list remains rooted")
                .list_append(item.value());
        }
    }
    return_value(roots.top().expect("output list remains rooted"))
}

fn load_type(value: &AtomicI16) -> ffi::py_Type {
    let value = value.load(Ordering::Acquire);
    assert_ne!(value, 0, "itertools type used before module initialization");
    value
}

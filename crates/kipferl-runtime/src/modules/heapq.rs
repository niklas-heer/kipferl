use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, index_error,
    return_value, type_error, value_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"heapify",
        callback: heapify,
    },
    NativeFunction {
        name: c"heappush",
        callback: heappush,
    },
    NativeFunction {
        name: c"heappop",
        callback: heappop,
    },
    NativeFunction {
        name: c"heapreplace",
        callback: heapreplace,
    },
    NativeFunction {
        name: c"heappushpop",
        callback: heappushpop,
    },
    NativeFunction {
        name: c"nlargest",
        callback: nlargest,
    },
    NativeFunction {
        name: c"nsmallest",
        callback: nsmallest,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"heapq",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

unsafe extern "C" fn heapify(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Some(list) = list_argument(argc, stack, 1) else {
        return false;
    };
    let length = list.list_len().unwrap_or(0);
    if length > 1 {
        for index in (0..(length / 2)).rev() {
            if sift_down(list, length, index).is_err() {
                return false;
            }
        }
    }
    return_none()
}

unsafe extern "C" fn heappush(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Some((list, item)) = list_and_item(argc, stack) else {
        return false;
    };
    list.list_append(item);
    let length = list.list_len().unwrap_or(0);
    if sift_up(list, length.saturating_sub(1)).is_err() {
        return false;
    }
    return_none()
}

unsafe extern "C" fn heappop(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Some(list) = list_argument(argc, stack, 1) else {
        return false;
    };
    let length = list.list_len().unwrap_or(0);
    if length == 0 {
        return index_error(c"pop from empty heap");
    }

    let mut roots = RootFrame::new();
    let Some(first) = list.list_item(0) else {
        return index_error(c"heap is empty");
    };
    let minimum = roots.copy(first);
    if length == 1 {
        list.list_delete(0);
        return return_value(minimum);
    }
    let Some(final_item) = list.list_item(length.saturating_sub(1)) else {
        return index_error(c"heap is empty");
    };
    list.list_set(0, final_item);
    list.list_delete(length.saturating_sub(1));
    if sift_down(list, length.saturating_sub(1), 0).is_err() {
        return false;
    }
    return_value(minimum)
}

unsafe extern "C" fn heapreplace(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Some((list, item)) = list_and_item(argc, stack) else {
        return false;
    };
    let length = list.list_len().unwrap_or(0);
    if length == 0 {
        return index_error(c"heap is empty");
    }

    let mut roots = RootFrame::new();
    let Some(first) = list.list_item(0) else {
        return index_error(c"heap is empty");
    };
    let minimum = roots.copy(first);
    list.list_set(0, item);
    if sift_down(list, length, 0).is_err() {
        return false;
    }
    return_value(minimum)
}

unsafe extern "C" fn heappushpop(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Some((list, item)) = list_and_item(argc, stack) else {
        return false;
    };
    let length = list.list_len().unwrap_or(0);
    if length == 0 {
        return return_value(item);
    }

    let Some(root) = list.list_item(0) else {
        return index_error(c"heap is empty");
    };
    let mut roots = RootFrame::new();
    let root = roots.copy(root);
    match item.less_than(root) {
        Ok(true) => return return_value(item),
        Ok(false) => {}
        Err(()) => return false,
    }
    match item.equals(root) {
        Ok(true) => return return_value(item),
        Ok(false) => {}
        Err(()) => return false,
    }

    if list.list_len() != Some(length) {
        return crate::native::runtime_error(c"heap changed during comparison");
    }
    let minimum = roots.copy(root);
    list.list_set(0, item);
    if sift_down(list, length, 0).is_err() {
        return false;
    }
    return_value(minimum)
}

unsafe extern "C" fn nlargest(argc: c_int, stack: ffi::py_StackRef) -> bool {
    select_extreme(argc, stack, true)
}

unsafe extern "C" fn nsmallest(argc: c_int, stack: ffi::py_StackRef) -> bool {
    select_extreme(argc, stack, false)
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The selection path rejects lists above 256 items, so advancing the outer index by one cannot overflow."
)]
fn select_extreme(argc: c_int, stack: ffi::py_StackRef, largest: bool) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(count) = arguments.get(0).and_then(Value::integer) else {
        return type_error(c"expected int for n");
    };
    if count < 0 {
        return value_error(c"n must be non-negative");
    }
    if count == 0 {
        return empty_list();
    }
    let Some(values) = arguments.get(1) else {
        crate::native::type_error(c"missing native argument");
        return false;
    };
    let Some(length) = values.list_len() else {
        return type_error(c"expected list");
    };
    if length == 0 {
        return empty_list();
    }
    if length > 256 {
        return value_error(c"data too large");
    }

    let mut indices: Vec<usize> = (0..length).collect();
    for start in 0..length {
        let mut extreme = start;
        for candidate in (start + 1)..length {
            let mut operands = RootFrame::new();
            let Some(candidate_value) = indices
                .get(candidate)
                .and_then(|index| values.list_item(*index))
            else {
                return crate::native::runtime_error(c"heap changed during comparison");
            };
            let candidate_value = operands.copy(candidate_value);
            let Some(extreme_value) = indices
                .get(extreme)
                .and_then(|index| values.list_item(*index))
            else {
                return crate::native::runtime_error(c"heap changed during comparison");
            };
            let extreme_value = operands.copy(extreme_value);
            let comparison = if largest {
                extreme_value.less_than(candidate_value)
            } else {
                candidate_value.less_than(extreme_value)
            };
            let Ok(smaller) = comparison else {
                return false;
            };
            if require_heap_size(values, length).is_err() {
                return false;
            }
            if smaller {
                extreme = candidate;
            }
        }
        indices.swap(start, extreme);
    }

    let result_count = usize::try_from(count).unwrap_or(usize::MAX).min(length);
    let mut roots = RootFrame::new();
    let result = roots.list();
    for index in indices.into_iter().take(result_count) {
        let Some(item) = values.list_item(index) else {
            return crate::native::runtime_error(c"heap changed during comparison");
        };
        result.list_append(item);
    }
    return_value(result)
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "PocketPy list lengths fit a signed C int; twice a valid index plus two fits usize on all supported targets."
)]
fn sift_down(list: Value, length: usize, mut position: usize) -> Result<(), ()> {
    loop {
        let mut smallest = position;
        let left = position * 2 + 1;
        let right = position * 2 + 2;
        if left < length {
            let mut operands = RootFrame::new();
            let left_value = operands.copy(heap_item(list, left)?);
            let smallest_value = operands.copy(heap_item(list, smallest)?);
            let smaller = left_value.less_than(smallest_value)?;
            require_heap_size(list, length)?;
            if smaller {
                smallest = left;
            }
        }
        if right < length {
            let mut operands = RootFrame::new();
            let right_value = operands.copy(heap_item(list, right)?);
            let smallest_value = operands.copy(heap_item(list, smallest)?);
            let smaller = right_value.less_than(smallest_value)?;
            require_heap_size(list, length)?;
            if smaller {
                smallest = right;
            }
        }
        if smallest == position {
            return Ok(());
        }
        if !list.list_swap(position, smallest) {
            crate::native::runtime_error(c"heap changed during comparison");
            return Err(());
        }
        position = smallest;
    }
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The loop checks position > 0 before subtracting one; division is by the nonzero constant two."
)]
fn sift_up(list: Value, mut position: usize) -> Result<(), ()> {
    let length = list.list_len().ok_or(())?;
    while position > 0 {
        let parent = (position - 1) / 2;
        let mut operands = RootFrame::new();
        let value = operands.copy(heap_item(list, position)?);
        let parent_value = operands.copy(heap_item(list, parent)?);
        let smaller = value.less_than(parent_value)?;
        require_heap_size(list, length)?;
        if !smaller {
            break;
        }
        if !list.list_swap(position, parent) {
            crate::native::runtime_error(c"heap changed during comparison");
            return Err(());
        }
        position = parent;
    }
    Ok(())
}

fn list_argument(argc: c_int, stack: ffi::py_StackRef, arity: usize) -> Option<Value> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(arity, arity) {
        return None;
    }
    let Some(list) = arguments.get(0) else {
        crate::native::type_error(c"missing native argument");
        return None;
    };
    if list.list_len().is_none() {
        type_error(c"expected list");
        return None;
    }
    Some(list)
}

fn list_and_item(argc: c_int, stack: ffi::py_StackRef) -> Option<(Value, Value)> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return None;
    }
    let Some(list) = arguments.get(0) else {
        crate::native::type_error(c"missing native argument");
        return None;
    };
    if list.list_len().is_none() {
        type_error(c"expected list");
        return None;
    }
    Some((list, arguments.get(1)?))
}

fn return_none() -> bool {
    let mut roots = RootFrame::new();
    let value = roots.none();
    return_value(value)
}

fn empty_list() -> bool {
    let mut roots = RootFrame::new();
    let value = roots.list();
    return_value(value)
}

fn heap_item(list: Value, index: usize) -> Result<Value, ()> {
    list.list_item(index).ok_or_else(|| {
        crate::native::runtime_error(c"heap changed during comparison");
    })
}

fn require_heap_size(list: Value, expected: usize) -> Result<(), ()> {
    if list.list_len() == Some(expected) {
        return Ok(());
    }
    crate::native::runtime_error(c"heap changed during comparison");
    Err(())
}

use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

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
};

unsafe extern "C" fn heapify(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(list) = list_argument(argc, argv, 1) else {
        return false;
    };
    let length = list.list_len().unwrap_or(0);
    if length > 1 {
        for index in (0..=(length / 2 - 1)).rev() {
            if sift_down(list, length, index).is_err() {
                return false;
            }
        }
    }
    return_none()
}

unsafe extern "C" fn heappush(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some((list, item)) = list_and_item(argc, argv) else {
        return false;
    };
    list.list_append(item);
    let length = list.list_len().unwrap_or(0);
    if sift_up(list, length - 1).is_err() {
        return false;
    }
    return_none()
}

unsafe extern "C" fn heappop(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(list) = list_argument(argc, argv, 1) else {
        return false;
    };
    let length = list.list_len().unwrap_or(0);
    if length == 0 {
        return index_error(c"pop from empty heap");
    }

    let mut roots = RootFrame::new();
    let minimum = roots.copy(list.list_item(0).expect("non-empty list"));
    if length == 1 {
        list.list_delete(0);
        return return_value(minimum);
    }
    let last = list.list_item(length - 1).expect("non-empty list");
    list.list_set(0, last);
    list.list_delete(length - 1);
    if sift_down(list, length - 1, 0).is_err() {
        return false;
    }
    return_value(minimum)
}

unsafe extern "C" fn heapreplace(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some((list, item)) = list_and_item(argc, argv) else {
        return false;
    };
    let length = list.list_len().unwrap_or(0);
    if length == 0 {
        return index_error(c"heap is empty");
    }

    let mut roots = RootFrame::new();
    let minimum = roots.copy(list.list_item(0).expect("non-empty list"));
    list.list_set(0, item);
    if sift_down(list, length, 0).is_err() {
        return false;
    }
    return_value(minimum)
}

unsafe extern "C" fn heappushpop(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some((list, item)) = list_and_item(argc, argv) else {
        return false;
    };
    let length = list.list_len().unwrap_or(0);
    if length == 0 {
        return return_value(item);
    }

    let root = list.list_item(0).expect("non-empty list");
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

    let mut roots = RootFrame::new();
    let minimum = roots.copy(root);
    list.list_set(0, item);
    if sift_down(list, length, 0).is_err() {
        return false;
    }
    return_value(minimum)
}

unsafe extern "C" fn nlargest(argc: c_int, argv: ffi::py_StackRef) -> bool {
    select_extreme(argc, argv, true)
}

unsafe extern "C" fn nsmallest(argc: c_int, argv: ffi::py_StackRef) -> bool {
    select_extreme(argc, argv, false)
}

fn select_extreme(argc: c_int, argv: ffi::py_StackRef, largest: bool) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
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
    let values = arguments.get(1).expect("arity checked");
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
            let candidate_value = values.list_item(indices[candidate]).expect("valid index");
            let extreme_value = values.list_item(indices[extreme]).expect("valid index");
            let comparison = if largest {
                extreme_value.less_than(candidate_value)
            } else {
                candidate_value.less_than(extreme_value)
            };
            match comparison {
                Ok(true) => extreme = candidate,
                Ok(false) => {}
                Err(()) => return false,
            }
        }
        indices.swap(start, extreme);
    }

    let result_count = usize::try_from(count).unwrap_or(usize::MAX).min(length);
    let mut roots = RootFrame::new();
    let result = roots.list();
    for index in indices.into_iter().take(result_count) {
        result.list_append(values.list_item(index).expect("valid index"));
    }
    return_value(result)
}

fn sift_down(list: Value, length: usize, mut position: usize) -> Result<(), ()> {
    loop {
        let mut smallest = position;
        let left = position * 2 + 1;
        let right = position * 2 + 2;
        if left < length {
            let left_value = list.list_item(left).ok_or(())?;
            let smallest_value = list.list_item(smallest).ok_or(())?;
            if left_value.less_than(smallest_value)? {
                smallest = left;
            }
        }
        if right < length {
            let right_value = list.list_item(right).ok_or(())?;
            let smallest_value = list.list_item(smallest).ok_or(())?;
            if right_value.less_than(smallest_value)? {
                smallest = right;
            }
        }
        if smallest == position {
            return Ok(());
        }
        if !list.list_swap(position, smallest) {
            return Err(());
        }
        position = smallest;
    }
}

fn sift_up(list: Value, mut position: usize) -> Result<(), ()> {
    while position > 0 {
        let parent = (position - 1) / 2;
        let value = list.list_item(position).ok_or(())?;
        let parent_value = list.list_item(parent).ok_or(())?;
        if !value.less_than(parent_value)? {
            break;
        }
        if !list.list_swap(position, parent) {
            return Err(());
        }
        position = parent;
    }
    Ok(())
}

fn list_argument(argc: c_int, argv: ffi::py_StackRef, arity: usize) -> Option<Value> {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(arity, arity) {
        return None;
    }
    let list = arguments.get(0).expect("arity checked");
    if list.list_len().is_none() {
        type_error(c"expected list");
        return None;
    }
    Some(list)
}

fn list_and_item(argc: c_int, argv: ffi::py_StackRef) -> Option<(Value, Value)> {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return None;
    }
    let list = arguments.get(0).expect("arity checked");
    if list.list_len().is_none() {
        type_error(c"expected list");
        return None;
    }
    Some((list, arguments.get(1).expect("arity checked")))
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

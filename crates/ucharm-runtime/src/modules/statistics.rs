use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use super::statistics_core;
use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, return_number,
    return_value, type_error, value_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"mean",
        callback: mean,
    },
    NativeFunction {
        name: c"median",
        callback: median,
    },
    NativeFunction {
        name: c"median_low",
        callback: median_low,
    },
    NativeFunction {
        name: c"median_high",
        callback: median_high,
    },
    NativeFunction {
        name: c"mode",
        callback: mode,
    },
    NativeFunction {
        name: c"variance",
        callback: variance,
    },
    NativeFunction {
        name: c"pvariance",
        callback: pvariance,
    },
    NativeFunction {
        name: c"stdev",
        callback: stdev,
    },
    NativeFunction {
        name: c"pstdev",
        callback: pstdev,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"statistics",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

#[derive(Clone, Copy)]
enum Sequence {
    List(Value),
    Tuple(Value),
}

impl Sequence {
    fn from_value(value: Value) -> Option<Self> {
        if value.list_len().is_some() {
            Some(Self::List(value))
        } else if value.tuple_len().is_some() {
            Some(Self::Tuple(value))
        } else {
            None
        }
    }

    fn len(self) -> usize {
        match self {
            Self::List(value) => value.list_len().unwrap_or(0),
            Self::Tuple(value) => value.tuple_len().unwrap_or(0),
        }
    }

    fn item(self, index: usize) -> Option<Value> {
        match self {
            Self::List(value) => value.list_item(index),
            Self::Tuple(value) => value.tuple_item(index),
        }
    }
}

unsafe extern "C" fn mean(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(sequence) = sequence_argument(argc, argv) else {
        return false;
    };
    if sequence.len() == 0 {
        return type_error(c"mean requires at least one data point");
    }
    let mut values = Vec::with_capacity(sequence.len());
    for index in 0..sequence.len() {
        let Some(item) = sequence.item(index) else {
            return type_error(c"mean requires at least one data point");
        };
        let Ok(value) = item.cast_number() else {
            return false;
        };
        values.push(value);
    }
    return_number(statistics_core::mean(&values))
}

unsafe extern "C" fn median(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(sequence) = sequence_argument(argc, argv) else {
        return false;
    };
    if sequence.len() == 0 {
        return type_error(c"median requires at least one data point");
    }
    if sequence.len() > 256 {
        return value_error(c"data too large");
    }
    let Some(mut values) = numeric_values(sequence) else {
        return type_error(c"data must be numeric");
    };
    return_number(statistics_core::median(&mut values))
}

unsafe extern "C" fn median_low(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(sequence) = sequence_argument(argc, argv) else {
        return false;
    };
    if sequence.len() == 0 || sequence.len() > 256 {
        return type_error(c"median_low requires numeric data");
    }
    let Some(mut values) = numeric_values(sequence) else {
        return type_error(c"median_low requires numeric data");
    };
    return_number(statistics_core::median_low(&mut values))
}

unsafe extern "C" fn median_high(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(sequence) = sequence_argument(argc, argv) else {
        return false;
    };
    if sequence.len() == 0 || sequence.len() > 256 {
        return type_error(c"median_high requires numeric data");
    }
    let Some(mut values) = numeric_values(sequence) else {
        return type_error(c"median_high requires numeric data");
    };
    return_number(statistics_core::median_high(&mut values))
}

unsafe extern "C" fn mode(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(sequence) = sequence_argument(argc, argv) else {
        return false;
    };
    if sequence.len() == 0 {
        return type_error(c"mode requires at least one data point");
    }

    let mut unique_roots = RootFrame::new();
    let mut values: Vec<(Value, usize)> = Vec::new();
    for index in 0..sequence.len() {
        let Some(item) = sequence.item(index) else {
            return type_error(c"mode requires at least one data point");
        };
        let new_value = {
            let mut candidate_root = RootFrame::new();
            let candidate = candidate_root.copy(item);
            let mut found = false;
            for entry in &mut values {
                match entry.0.equals(candidate) {
                    Ok(true) => {
                        entry.1 += 1;
                        found = true;
                        break;
                    }
                    Ok(false) => {}
                    Err(()) => return false,
                }
            }
            if found {
                None
            } else {
                Some(candidate.snapshot())
            }
        };
        if let Some(snapshot) = new_value {
            if values.len() >= 256 {
                return value_error(c"too many unique values");
            }
            // No PocketPy operation occurs between dropping the candidate
            // frame and restoring its snapshot into the persistent frame.
            values.push((unique_roots.restore(snapshot), 1));
        }
    }

    let mut mode_index = 0;
    let mut maximum_count = 0;
    for (index, (_, count)) in values.iter().enumerate() {
        if *count > maximum_count {
            maximum_count = *count;
            mode_index = index;
        }
    }
    return_value(values[mode_index].0)
}

unsafe extern "C" fn variance(argc: c_int, argv: ffi::py_StackRef) -> bool {
    dispersion(argc, argv, true, false)
}

unsafe extern "C" fn pvariance(argc: c_int, argv: ffi::py_StackRef) -> bool {
    dispersion(argc, argv, false, false)
}

unsafe extern "C" fn stdev(argc: c_int, argv: ffi::py_StackRef) -> bool {
    dispersion(argc, argv, true, true)
}

unsafe extern "C" fn pstdev(argc: c_int, argv: ffi::py_StackRef) -> bool {
    dispersion(argc, argv, false, true)
}

fn dispersion(argc: c_int, argv: ffi::py_StackRef, sample: bool, square_root: bool) -> bool {
    let Some(sequence) = sequence_argument(argc, argv) else {
        return false;
    };
    let minimum = if sample { 2 } else { 1 };
    if sequence.len() < minimum {
        return type_error(if sample {
            c"variance requires at least two data points"
        } else {
            c"pvariance requires at least one data point"
        });
    }
    let Some(values) = numeric_values(sequence) else {
        return type_error(c"data must be numeric");
    };
    let variance = statistics_core::variance(&values, sample);
    return_number(if square_root {
        variance.sqrt()
    } else {
        variance
    })
}

fn sequence_argument(argc: c_int, argv: ffi::py_StackRef) -> Option<Sequence> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return None;
    }
    let value = arguments.get(0).expect("arity checked");
    if let Some(sequence) = Sequence::from_value(value) {
        return Some(sequence);
    }
    type_error(c"expected data");
    None
}

fn numeric_values(sequence: Sequence) -> Option<Vec<f64>> {
    let mut values = Vec::with_capacity(sequence.len());
    for index in 0..sequence.len() {
        values.push(sequence.item(index)?.cast_number().ok()?);
    }
    Some(values)
}

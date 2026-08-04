use std::ffi::{c_int, c_void};
use std::ptr;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicI16, Ordering};

use rusqlite::types::Value as SqlValue;
use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, RootFrame, Value, bind_type_method,
    create_type_with_destructor, return_value, runtime_error, type_error, value_error,
};

static CONNECTION_TYPE: AtomicI16 = AtomicI16::new(0);
static CURSOR_TYPE: AtomicI16 = AtomicI16::new(0);

const MAX_RESULT_ROWS: usize = 100_000;
const MAX_RESULT_BYTES: usize = 64 * 1024 * 1024;

const SIGNATURES: &[NativeSignature] = &[NativeSignature {
    signature: c"connect(database, timeout=5.0, detect_types=0, isolation_level=None, check_same_thread=True, factory=None, cached_statements=128, uri=False)",
    callback: connect,
}];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"sqlite3",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

#[derive(Clone, Copy)]
#[repr(C)]
struct ConnectionState {
    connection: *mut rusqlite::Connection,
}

impl ConnectionState {
    const CLOSED: Self = Self {
        connection: ptr::null_mut(),
    };
}

#[derive(Clone, Copy)]
#[repr(C)]
struct CursorState {
    index: usize,
    closed: bool,
}

fn initialize(module: Value) {
    let connection =
        create_type_with_destructor(module, c"Connection", Some(connection_destructor));
    CONNECTION_TYPE.store(connection, Ordering::Release);
    bind_type_method(connection, c"cursor", connection_cursor);
    bind_type_method(connection, c"execute", connection_execute);
    bind_type_method(connection, c"commit", connection_commit);
    bind_type_method(connection, c"close", connection_close);

    let cursor = create_type_with_destructor(module, c"Cursor", None);
    CURSOR_TYPE.store(cursor, Ordering::Release);
    bind_type_method(cursor, c"execute", cursor_execute);
    bind_type_method(cursor, c"fetchone", cursor_fetchone);
    bind_type_method(cursor, c"fetchall", cursor_fetchall);
    bind_type_method(cursor, c"close", cursor_close);

    let mut roots = RootFrame::new();
    let version = roots
        .string(rusqlite::version())
        .expect("SQLite version string fits PocketPy");
    module.set_attribute(c"sqlite_version", version);
}

unsafe extern "C" fn connection_destructor(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    // SAFETY: every Connection instance has exactly this userdata layout.
    let state = unsafe { &mut *userdata.cast::<ConnectionState>() };
    close_connection(state);
}

fn close_connection(state: &mut ConnectionState) {
    if !state.connection.is_null() {
        // SAFETY: `connect` transfers one Box into this pointer exactly once.
        unsafe { drop(Box::from_raw(state.connection)) };
        *state = ConnectionState::CLOSED;
    }
}

unsafe extern "C" fn connect(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 8) {
        return false;
    }
    let Some(database) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"database must be str");
    };
    let Ok(connection) = rusqlite::Connection::open(database) else {
        return runtime_error(c"unable to open sqlite3 database");
    };
    let state = ConnectionState {
        connection: Box::into_raw(Box::new(connection)),
    };
    let mut roots = RootFrame::new();
    let instance = roots
        .object_with_userdata(CONNECTION_TYPE.load(Ordering::Acquire), -1, state)
        .expect("sqlite3 connection state fits PocketPy userdata");
    return_value(instance)
}

fn connection(value: Value) -> Option<NonNull<rusqlite::Connection>> {
    if !value.is_instance(CONNECTION_TYPE.load(Ordering::Acquire)) {
        return None;
    }
    // SAFETY: the exact type check establishes ConnectionState userdata.
    let state = unsafe { &*value.userdata::<ConnectionState>() };
    NonNull::new(state.connection)
}

unsafe extern "C" fn connection_cursor(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let owner = arguments.get(0).expect("arity checked");
    if connection(owner).is_none() {
        return runtime_error(c"connection is closed");
    }
    new_cursor(owner)
}

fn new_cursor(owner: Value) -> bool {
    let mut roots = RootFrame::new();
    let Some(cursor) = roots.object_with_userdata(
        CURSOR_TYPE.load(Ordering::Acquire),
        2,
        CursorState {
            index: 0,
            closed: false,
        },
    ) else {
        return value_error(c"sqlite3 cursor state is too large");
    };
    let none = roots.none();
    cursor.set_slot(0, owner);
    cursor.set_slot(1, none);
    return_value(cursor)
}

unsafe extern "C" fn connection_execute(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 3) {
        return false;
    }
    let owner = arguments.get(0).expect("arity checked");
    if connection(owner).is_none() {
        return runtime_error(c"connection is closed");
    }
    let Some(sql) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"sql must be str");
    };
    let params = match python_params(arguments.get(2)) {
        Ok(params) => params,
        Err(()) => return false,
    };
    if !new_cursor(owner) {
        return false;
    }
    let mut roots = RootFrame::new();
    let cursor = roots.copy_returned();
    if !execute_statement(cursor, &sql, params) {
        return false;
    }
    return_value(cursor)
}

unsafe extern "C" fn connection_commit(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    if connection(arguments.get(0).expect("arity checked")).is_none() {
        return runtime_error(c"connection is closed");
    }
    // Connections operate in SQLite autocommit mode unless the caller starts
    // an explicit transaction, matching the previous compatibility module.
    let mut roots = RootFrame::new();
    let none = roots.none();
    return_value(none)
}

unsafe extern "C" fn connection_close(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let owner = arguments.get(0).expect("arity checked");
    if !owner.is_instance(CONNECTION_TYPE.load(Ordering::Acquire)) {
        return type_error(c"expected Connection");
    }
    // SAFETY: the exact type check establishes ConnectionState userdata.
    let state = unsafe { &mut *owner.userdata::<ConnectionState>() };
    close_connection(state);
    let mut roots = RootFrame::new();
    let none = roots.none();
    return_value(none)
}

unsafe extern "C" fn cursor_execute(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 3) {
        return false;
    }
    let cursor = arguments.get(0).expect("arity checked");
    if !cursor.is_instance(CURSOR_TYPE.load(Ordering::Acquire)) {
        return type_error(c"expected Cursor");
    }
    // SAFETY: the exact type check establishes CursorState userdata.
    let state = unsafe { &*cursor.userdata::<CursorState>() };
    if state.closed {
        return runtime_error(c"cursor is closed");
    }
    let Some(sql) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"sql must be str");
    };
    let params = match python_params(arguments.get(2)) {
        Ok(params) => params,
        Err(()) => return false,
    };
    execute_statement(cursor, &sql, params)
}

fn execute_statement(cursor: Value, sql: &str, params: Vec<SqlValue>) -> bool {
    let owner = cursor.slot(0);
    let Some(connection) = connection(owner) else {
        return runtime_error(c"connection is closed");
    };
    // SAFETY: `owner` is rooted by the cursor throughout this synchronous call.
    let connection = unsafe { connection.as_ref() };
    let Ok(mut statement) = connection.prepare(sql) else {
        return runtime_error(c"sqlite3 statement preparation failed");
    };
    let column_count = statement.column_count();
    let mut rows = Vec::new();
    let mut result_bytes = 0_usize;
    if column_count == 0 {
        if statement
            .execute(rusqlite::params_from_iter(params))
            .is_err()
        {
            return runtime_error(c"sqlite3 statement failed");
        }
    } else {
        let Ok(mut query) = statement.query(rusqlite::params_from_iter(params)) else {
            return runtime_error(c"sqlite3 query failed");
        };
        loop {
            let next = match query.next() {
                Ok(next) => next,
                Err(_) => return runtime_error(c"sqlite3 row fetch failed"),
            };
            let Some(row) = next else {
                break;
            };
            if rows.len() >= MAX_RESULT_ROWS {
                return runtime_error(c"sqlite3 result exceeds row limit");
            }
            let mut values = Vec::with_capacity(column_count);
            for index in 0..column_count {
                let Ok(value) = row.get::<_, SqlValue>(index) else {
                    return runtime_error(c"unsupported sqlite3 column value");
                };
                result_bytes = result_bytes.saturating_add(match &value {
                    SqlValue::Null => 0,
                    SqlValue::Integer(_) | SqlValue::Real(_) => 8,
                    SqlValue::Text(value) => value.len(),
                    SqlValue::Blob(value) => value.len(),
                });
                if result_bytes > MAX_RESULT_BYTES {
                    return runtime_error(c"sqlite3 result exceeds memory limit");
                }
                values.push(value);
            }
            rows.push(values);
        }
    }

    let mut roots = RootFrame::new();
    let output = roots.list();
    for row in rows {
        let Some(tuple) = roots.tuple(row.len()) else {
            return value_error(c"sqlite3 row is too large");
        };
        for (index, value) in row.into_iter().enumerate() {
            let Some(value) = python_value(&mut roots, value) else {
                return value_error(c"sqlite3 value is too large");
            };
            tuple.tuple_set(index, value);
        }
        output.list_append(tuple);
    }
    cursor.set_slot(1, output);
    // SAFETY: the exact cursor type was checked by both callers.
    let state = unsafe { &mut *cursor.userdata::<CursorState>() };
    state.index = 0;
    return_value(cursor)
}

fn python_params(value: Option<Value>) -> Result<Vec<SqlValue>, ()> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_none() {
        return Ok(Vec::new());
    }
    let length = value
        .tuple_len()
        .or_else(|| value.list_len())
        .ok_or_else(|| {
            type_error(c"parameters must be a tuple or list");
        })?;
    let mut params = Vec::with_capacity(length);
    for index in 0..length {
        let item = value
            .tuple_item(index)
            .or_else(|| value.list_item(index))
            .expect("index is in bounds");
        let converted = if item.is_none() {
            SqlValue::Null
        } else if let Some(value) = item.integer() {
            SqlValue::Integer(value)
        } else if item.is_type(ffi::py_PredefinedType_tp_float) {
            SqlValue::Real(item.number().expect("number checked"))
        } else if let Some(value) = item.string() {
            SqlValue::Text(value)
        } else if let Some(value) = item.bytes() {
            SqlValue::Blob(value)
        } else {
            type_error(c"unsupported sqlite3 parameter type");
            return Err(());
        };
        params.push(converted);
    }
    Ok(params)
}

fn python_value(roots: &mut RootFrame, value: SqlValue) -> Option<Value> {
    match value {
        SqlValue::Null => Some(roots.none()),
        SqlValue::Integer(value) => Some(roots.integer(value)),
        SqlValue::Real(value) => Some(roots.number(value)),
        SqlValue::Text(value) => roots.string(&value),
        SqlValue::Blob(value) => roots.bytes(&value),
    }
}

unsafe extern "C" fn cursor_fetchone(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let cursor = arguments.get(0).expect("arity checked");
    if !cursor.is_instance(CURSOR_TYPE.load(Ordering::Acquire)) {
        return type_error(c"expected Cursor");
    }
    // SAFETY: the exact type check establishes CursorState userdata.
    let state = unsafe { &mut *cursor.userdata::<CursorState>() };
    if state.closed {
        return runtime_error(c"cursor is closed");
    }
    let rows = cursor.slot(1);
    let Some(length) = rows.list_len() else {
        let mut roots = RootFrame::new();
        let none = roots.none();
        return return_value(none);
    };
    if state.index >= length {
        let mut roots = RootFrame::new();
        let none = roots.none();
        return return_value(none);
    }
    let row = rows.list_item(state.index).expect("index is in bounds");
    state.index += 1;
    return_value(row)
}

unsafe extern "C" fn cursor_fetchall(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let cursor = arguments.get(0).expect("arity checked");
    if !cursor.is_instance(CURSOR_TYPE.load(Ordering::Acquire)) {
        return type_error(c"expected Cursor");
    }
    // SAFETY: the exact type check establishes CursorState userdata.
    let state = unsafe { &mut *cursor.userdata::<CursorState>() };
    if state.closed {
        return runtime_error(c"cursor is closed");
    }
    let rows = cursor.slot(1);
    let Some(length) = rows.list_len() else {
        let mut roots = RootFrame::new();
        let output = roots.list();
        return return_value(output);
    };
    let mut roots = RootFrame::new();
    let output = roots.list();
    while state.index < length {
        output.list_append(rows.list_item(state.index).expect("index is in bounds"));
        state.index += 1;
    }
    return_value(output)
}

unsafe extern "C" fn cursor_close(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let cursor = arguments.get(0).expect("arity checked");
    if !cursor.is_instance(CURSOR_TYPE.load(Ordering::Acquire)) {
        return type_error(c"expected Cursor");
    }
    // SAFETY: the exact type check establishes CursorState userdata.
    let state = unsafe { &mut *cursor.userdata::<CursorState>() };
    state.closed = true;
    state.index = 0;
    let mut roots = RootFrame::new();
    let none = roots.none();
    cursor.set_slot(1, none);
    return_value(none)
}

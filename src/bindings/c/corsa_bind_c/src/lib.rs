#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::{
    cell::RefCell,
    ffi::{CStr, CString, c_char, c_int},
    ptr::{self, null_mut},
    sync::Mutex,
};

use corsa_bind_rs::{
    api::{
        ApiClient, ApiMode, ApiSpawnConfig, ManagedSnapshot, NodeHandle, ProjectHandle,
        SnapshotHandle, TypeHandle, UpdateSnapshotParams,
    },
    fast::{CompactString, FastMap},
    lsp::{VirtualChange, VirtualDocument},
    runtime::block_on,
    utils::{CorsaUtils, UnsafeTypeFlowInput},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpawnOptions {
    executable: String,
    cwd: Option<String>,
    mode: Option<String>,
    request_timeout_ms: Option<u64>,
    shutdown_timeout_ms: Option<u64>,
    outbound_capacity: Option<usize>,
    allow_unstable_upstream_calls: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotState<'a> {
    snapshot: &'a SnapshotHandle,
    projects: &'a [corsa_bind_rs::api::ProjectResponse],
    #[serde(skip_serializing_if = "Option::is_none")]
    changes: &'a Option<corsa_bind_rs::api::SnapshotChanges>,
}

#[repr(C)]
pub struct CorsaBindBytes {
    len: usize,
    ptr: *mut u8,
}

pub struct CorsaBindApiClient {
    inner: ApiClient,
    snapshots: Mutex<FastMap<CompactString, ManagedSnapshot>>,
}

pub struct CorsaBindVirtualDocument {
    inner: VirtualDocument,
}

const VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();

fn clear_last_error() {
    LAST_ERROR.with(|slot| {
        slot.borrow_mut().take();
    });
}

fn set_last_error(error: impl ToString) {
    let message = sanitize_cstring(error.to_string());
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = Some(message);
    });
}

fn sanitize_cstring(value: String) -> CString {
    match CString::new(value) {
        Ok(value) => value,
        Err(error) => {
            let filtered: Vec<u8> = error
                .into_vec()
                .into_iter()
                .filter(|byte| *byte != 0)
                .collect();
            CString::new(filtered).expect("CString filtering removed interior NULs")
        }
    }
}

unsafe fn cstr_arg<'a>(value: *const c_char, name: &str) -> Result<&'a str, String> {
    if value.is_null() {
        return Err(format!("{name} must not be null"));
    }
    CStr::from_ptr(value)
        .to_str()
        .map_err(|error| format!("invalid utf-8 in {name}: {error}"))
}

unsafe fn optional_cstr_arg<'a>(value: *const c_char) -> Result<Option<&'a str>, String> {
    if value.is_null() {
        return Ok(None);
    }
    CStr::from_ptr(value)
        .to_str()
        .map(Some)
        .map_err(|error| format!("invalid utf-8: {error}"))
}

unsafe fn parse_json_arg<T>(value: *const c_char, name: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let value = cstr_arg(value, name)?;
    serde_json::from_str(value).map_err(|error| format!("invalid {name}: {error}"))
}

unsafe fn optional_json_arg(value: *const c_char) -> Result<Value, String> {
    match optional_cstr_arg(value)? {
        Some(value) => {
            serde_json::from_str(value).map_err(|error| format!("invalid json: {error}"))
        }
        None => Ok(Value::Null),
    }
}

fn json_string<T>(value: &T) -> Result<*mut c_char, String>
where
    T: Serialize,
{
    let value = serde_json::to_string(value).map_err(|error| error.to_string())?;
    Ok(sanitize_cstring(value).into_raw())
}

fn owned_string(value: String) -> *mut c_char {
    sanitize_cstring(value).into_raw()
}

fn into_bytes(value: Option<Vec<u8>>) -> CorsaBindBytes {
    match value {
        Some(mut value) => {
            let ptr = value.as_mut_ptr();
            let len = value.len();
            std::mem::forget(value);
            CorsaBindBytes { len, ptr }
        }
        None => CorsaBindBytes {
            len: 0,
            ptr: null_mut(),
        },
    }
}

unsafe fn api_client<'a>(value: *mut CorsaBindApiClient) -> Result<&'a CorsaBindApiClient, String> {
    value
        .as_ref()
        .ok_or_else(|| "api client pointer must not be null".to_owned())
}

unsafe fn virtual_document<'a>(
    value: *mut CorsaBindVirtualDocument,
) -> Result<&'a mut CorsaBindVirtualDocument, String> {
    value
        .as_mut()
        .ok_or_else(|| "virtual document pointer must not be null".to_owned())
}

fn build_spawn_config(options: SpawnOptions) -> Result<ApiSpawnConfig, String> {
    let mut config = ApiSpawnConfig::new(options.executable);
    if let Some(cwd) = options.cwd {
        config = config.with_cwd(cwd);
    }
    if let Some(mode) = options.mode {
        config = config.with_mode(match mode.as_str() {
            "jsonrpc" => ApiMode::AsyncJsonRpcStdio,
            "msgpack" => ApiMode::SyncMsgpackStdio,
            _ => return Err("unknown Corsa API mode".to_owned()),
        });
    }
    if let Some(timeout_ms) = options.request_timeout_ms {
        config = config.with_request_timeout(Some(std::time::Duration::from_millis(timeout_ms)));
    }
    if let Some(timeout_ms) = options.shutdown_timeout_ms {
        config = config.with_shutdown_timeout(std::time::Duration::from_millis(timeout_ms));
    }
    if let Some(capacity) = options.outbound_capacity {
        config = config.with_outbound_capacity(capacity);
    }
    if let Some(allow) = options.allow_unstable_upstream_calls {
        config = config.with_allow_unstable_upstream_calls(allow);
    }
    Ok(config)
}

#[unsafe(no_mangle)]
pub extern "C" fn corsa_bind_version() -> *const c_char {
    VERSION.as_ptr().cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn corsa_bind_utils_version() -> *const c_char {
    corsa_bind_version()
}

#[unsafe(no_mangle)]
pub extern "C" fn corsa_bind_last_error_message() -> *const c_char {
    LAST_ERROR.with(|slot| {
        slot.borrow()
            .as_ref()
            .map_or(ptr::null(), |message| message.as_ptr())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_bytes_free(value: CorsaBindBytes) {
    if !value.ptr.is_null() {
        drop(Vec::from_raw_parts(value.ptr, value.len, value.len));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_is_unsafe_assignment(input_json: *const c_char) -> c_int {
    clear_last_error();
    match parse_json_arg::<UnsafeTypeFlowInput>(input_json, "input_json") {
        Ok(input) => c_int::from(CorsaUtils::is_unsafe_assignment(&input)),
        Err(error) => {
            set_last_error(error);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_is_unsafe_return(input_json: *const c_char) -> c_int {
    corsa_bind_is_unsafe_assignment(input_json)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_utils_is_unsafe_assignment(input_json: *const c_char) -> c_int {
    corsa_bind_is_unsafe_assignment(input_json)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_utils_is_unsafe_return(input_json: *const c_char) -> c_int {
    corsa_bind_is_unsafe_return(input_json)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_new(
    options_json: *const c_char,
) -> *mut CorsaBindApiClient {
    clear_last_error();
    let result = (|| -> Result<*mut CorsaBindApiClient, String> {
        let options = parse_json_arg::<SpawnOptions>(options_json, "options_json")?;
        let inner = block_on(ApiClient::spawn(build_spawn_config(options)?))
            .map_err(|error| error.to_string())?;
        Ok(Box::into_raw(Box::new(CorsaBindApiClient {
            inner,
            snapshots: Mutex::new(FastMap::default()),
        })))
    })();
    match result {
        Ok(client) => client,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_free(value: *mut CorsaBindApiClient) {
    if !value.is_null() {
        drop(Box::from_raw(value));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_initialize_json(
    value: *mut CorsaBindApiClient,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = api_client(value)?;
        let response = block_on(value.inner.initialize()).map_err(|error| error.to_string())?;
        json_string(response.as_ref())
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_parse_config_file_json(
    value: *mut CorsaBindApiClient,
    file: *const c_char,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = api_client(value)?;
        let file = cstr_arg(file, "file")?;
        let response =
            block_on(value.inner.parse_config_file(file)).map_err(|error| error.to_string())?;
        json_string(&response)
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_update_snapshot_json(
    value: *mut CorsaBindApiClient,
    params_json: *const c_char,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = api_client(value)?;
        let params = if params_json.is_null() {
            UpdateSnapshotParams::default()
        } else {
            parse_json_arg::<UpdateSnapshotParams>(params_json, "params_json")?
        };
        let snapshot =
            block_on(value.inner.update_snapshot(params)).map_err(|error| error.to_string())?;
        let handle = snapshot.handle.clone();
        let serialized = json_string(&SnapshotState {
            snapshot: &snapshot.handle,
            projects: snapshot.projects.as_slice(),
            changes: &snapshot.changes,
        })?;
        value
            .snapshots
            .lock()
            .map_err(|error| error.to_string())?
            .insert(CompactString::from(handle.as_str()), snapshot);
        Ok(serialized)
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_get_source_file(
    value: *mut CorsaBindApiClient,
    snapshot: *const c_char,
    project: *const c_char,
    file: *const c_char,
) -> CorsaBindBytes {
    clear_last_error();
    let result = (|| -> Result<CorsaBindBytes, String> {
        let value = api_client(value)?;
        let snapshot = SnapshotHandle::from(cstr_arg(snapshot, "snapshot")?);
        let project = ProjectHandle::from(cstr_arg(project, "project")?);
        let file = cstr_arg(file, "file")?.to_owned();
        let payload = block_on(value.inner.get_source_file(snapshot, project, file))
            .map_err(|error| error.to_string())?;
        Ok(into_bytes(payload.map(|payload| payload.into_bytes())))
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            CorsaBindBytes {
                len: 0,
                ptr: null_mut(),
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_get_string_type_json(
    value: *mut CorsaBindApiClient,
    snapshot: *const c_char,
    project: *const c_char,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = api_client(value)?;
        let snapshot = SnapshotHandle::from(cstr_arg(snapshot, "snapshot")?);
        let project = ProjectHandle::from(cstr_arg(project, "project")?);
        let response = block_on(value.inner.get_string_type(snapshot, project))
            .map_err(|error| error.to_string())?;
        json_string(&response)
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_type_to_string(
    value: *mut CorsaBindApiClient,
    snapshot: *const c_char,
    project: *const c_char,
    type_handle: *const c_char,
    location: *const c_char,
    flags: i32,
    has_flags: c_int,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = api_client(value)?;
        let snapshot = SnapshotHandle::from(cstr_arg(snapshot, "snapshot")?);
        let project = ProjectHandle::from(cstr_arg(project, "project")?);
        let type_handle = TypeHandle::from(cstr_arg(type_handle, "type_handle")?);
        let location = optional_cstr_arg(location)?.map(NodeHandle::from);
        let rendered = block_on(value.inner.type_to_string(
            snapshot,
            project,
            type_handle,
            location,
            if has_flags != 0 { Some(flags) } else { None },
        ))
        .map_err(|error| error.to_string())?;
        Ok(owned_string(rendered))
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_call_json(
    value: *mut CorsaBindApiClient,
    method: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = api_client(value)?;
        let method = cstr_arg(method, "method")?;
        let params = optional_json_arg(params_json)?;
        let response = block_on(value.inner.raw_json_request(method, params))
            .map_err(|error| error.to_string())?;
        json_string(&response)
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_call_binary(
    value: *mut CorsaBindApiClient,
    method: *const c_char,
    params_json: *const c_char,
) -> CorsaBindBytes {
    clear_last_error();
    let result = (|| -> Result<CorsaBindBytes, String> {
        let value = api_client(value)?;
        let method = cstr_arg(method, "method")?;
        let params = optional_json_arg(params_json)?;
        let response = block_on(value.inner.raw_binary_request(method, params))
            .map_err(|error| error.to_string())?;
        Ok(into_bytes(response.map(|payload| payload.into_bytes())))
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            CorsaBindBytes {
                len: 0,
                ptr: null_mut(),
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_release_handle(
    value: *mut CorsaBindApiClient,
    handle: *const c_char,
) -> c_int {
    clear_last_error();
    let result = (|| -> Result<(), String> {
        let value = api_client(value)?;
        let handle = cstr_arg(handle, "handle")?;
        if let Some(snapshot) = value
            .snapshots
            .lock()
            .map_err(|error| error.to_string())?
            .remove(handle)
        {
            return block_on(snapshot.release()).map_err(|error| error.to_string());
        }
        let params = serde_json::json!({ "handle": handle });
        let _ = block_on(value.inner.raw_json_request("release", params))
            .map_err(|error| error.to_string())?;
        Ok(())
    })();
    match result {
        Ok(()) => 0,
        Err(error) => {
            set_last_error(error);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_api_client_close(value: *mut CorsaBindApiClient) -> c_int {
    clear_last_error();
    let result = (|| -> Result<(), String> {
        let value = api_client(value)?;
        let snapshots =
            std::mem::take(&mut *value.snapshots.lock().map_err(|error| error.to_string())?);
        for (_, snapshot) in snapshots {
            block_on(snapshot.release()).map_err(|error| error.to_string())?;
        }
        block_on(value.inner.close()).map_err(|error| error.to_string())
    })();
    match result {
        Ok(()) => 0,
        Err(error) => {
            set_last_error(error);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_untitled(
    path: *const c_char,
    language_id: *const c_char,
    text: *const c_char,
) -> *mut CorsaBindVirtualDocument {
    clear_last_error();
    let result = (|| -> Result<*mut CorsaBindVirtualDocument, String> {
        let path = cstr_arg(path, "path")?;
        let language_id = cstr_arg(language_id, "language_id")?.to_owned();
        let text = cstr_arg(text, "text")?.to_owned();
        let inner = VirtualDocument::untitled(path, language_id, text)
            .map_err(|error| error.to_string())?;
        Ok(Box::into_raw(Box::new(CorsaBindVirtualDocument { inner })))
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_in_memory(
    authority: *const c_char,
    path: *const c_char,
    language_id: *const c_char,
    text: *const c_char,
) -> *mut CorsaBindVirtualDocument {
    clear_last_error();
    let result = (|| -> Result<*mut CorsaBindVirtualDocument, String> {
        let authority = cstr_arg(authority, "authority")?;
        let path = cstr_arg(path, "path")?;
        let language_id = cstr_arg(language_id, "language_id")?.to_owned();
        let text = cstr_arg(text, "text")?.to_owned();
        let inner = VirtualDocument::in_memory(authority, path, language_id, text)
            .map_err(|error| error.to_string())?;
        Ok(Box::into_raw(Box::new(CorsaBindVirtualDocument { inner })))
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_free(value: *mut CorsaBindVirtualDocument) {
    if !value.is_null() {
        drop(Box::from_raw(value));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_uri(
    value: *const CorsaBindVirtualDocument,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = value
            .as_ref()
            .ok_or_else(|| "virtual document pointer must not be null".to_owned())?;
        Ok(owned_string(value.inner.uri.to_string()))
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_language_id(
    value: *const CorsaBindVirtualDocument,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = value
            .as_ref()
            .ok_or_else(|| "virtual document pointer must not be null".to_owned())?;
        Ok(owned_string(value.inner.language_id.to_string()))
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_version(
    value: *const CorsaBindVirtualDocument,
) -> i32 {
    clear_last_error();
    match value.as_ref() {
        Some(value) => value.inner.version,
        None => {
            set_last_error("virtual document pointer must not be null");
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_text(
    value: *const CorsaBindVirtualDocument,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = value
            .as_ref()
            .ok_or_else(|| "virtual document pointer must not be null".to_owned())?;
        Ok(owned_string(value.inner.text.to_string()))
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_state_json(
    value: *const CorsaBindVirtualDocument,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = value
            .as_ref()
            .ok_or_else(|| "virtual document pointer must not be null".to_owned())?;
        json_string(&value.inner)
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_replace(
    value: *mut CorsaBindVirtualDocument,
    text: *const c_char,
) -> c_int {
    clear_last_error();
    let result = (|| -> Result<(), String> {
        let value = virtual_document(value)?;
        let text = cstr_arg(text, "text")?.to_owned();
        let changes = [VirtualChange::replace(text)];
        value
            .inner
            .apply_changes(&changes)
            .map(|_| ())
            .map_err(|error| error.to_string())
    })();
    match result {
        Ok(()) => 0,
        Err(error) => {
            set_last_error(error);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn corsa_bind_virtual_document_apply_changes_json(
    value: *mut CorsaBindVirtualDocument,
    changes_json: *const c_char,
) -> *mut c_char {
    clear_last_error();
    let result = (|| -> Result<*mut c_char, String> {
        let value = virtual_document(value)?;
        let changes = parse_json_arg::<Vec<VirtualChange>>(changes_json, "changes_json")?;
        let events = value
            .inner
            .apply_changes(changes.as_slice())
            .map_err(|error| error.to_string())?;
        json_string(&events)
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            set_last_error(error);
            null_mut()
        }
    }
}

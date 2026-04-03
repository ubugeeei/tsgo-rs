#extern
type OwnedCStr

#extern
type BorrowedCStr

#extern
type ApiClient

#extern
type VirtualDocument

fn string_to_c_bytes(s : String) -> Bytes {
  let arr = s.to_bytes().to_array()
  arr.push(0)
  Bytes::from_array(arr)
}

extern "C" fn OwnedCStr::to_string(self : OwnedCStr) -> String = "corsa_bind_mbt_take_string"
extern "C" fn BorrowedCStr::to_string(self : BorrowedCStr) -> String = "corsa_bind_mbt_copy_string"

extern "C" fn __version() -> BorrowedCStr = "corsa_bind_version"
extern "C" fn __last_error() -> BorrowedCStr = "corsa_bind_last_error_message"

fn version() -> String {
  __version().to_string()
}

fn last_error() -> String {
  __last_error().to_string()
}

#borrow(payload)
extern "C" fn __is_unsafe_assignment(payload : Bytes) -> Int = "corsa_bind_is_unsafe_assignment"

#borrow(payload)
extern "C" fn __is_unsafe_return(payload : Bytes) -> Int = "corsa_bind_is_unsafe_return"

fn check_bool_result(result : Int) -> Bool {
  if result < 0 {
    abort(last_error())
  }
  result != 0
}

fn is_unsafe_assignment(payload : String) -> Bool {
  check_bool_result(__is_unsafe_assignment(string_to_c_bytes(payload)))
}

fn is_unsafe_return(payload : String) -> Bool {
  check_bool_result(__is_unsafe_return(string_to_c_bytes(payload)))
}

#borrow(options)
extern "C" fn __api_client_new(options : Bytes) -> ApiClient = "corsa_bind_api_client_new"

extern "C" fn ApiClient::free(self : ApiClient) -> Unit = "corsa_bind_api_client_free"
extern "C" fn ApiClient::initialize_json(self : ApiClient) -> OwnedCStr = "corsa_bind_api_client_initialize_json"

#borrow(file)
extern "C" fn ApiClient::parse_config_file_json(self : ApiClient, file : Bytes) -> OwnedCStr = "corsa_bind_api_client_parse_config_file_json"

#borrow(params)
extern "C" fn ApiClient::update_snapshot_json(self : ApiClient, params : Bytes) -> OwnedCStr = "corsa_bind_api_client_update_snapshot_json"

#borrow(snapshot)
#borrow(project)
extern "C" fn ApiClient::get_string_type_json(self : ApiClient, snapshot : Bytes, project : Bytes) -> OwnedCStr = "corsa_bind_api_client_get_string_type_json"

#borrow(snapshot)
#borrow(project)
#borrow(type_handle)
extern "C" fn ApiClient::type_to_string(self : ApiClient, snapshot : Bytes, project : Bytes, type_handle : Bytes) -> OwnedCStr = "corsa_bind_api_client_type_to_string"

#borrow(method)
#borrow(params)
extern "C" fn ApiClient::call_json(self : ApiClient, method : Bytes, params : Bytes) -> OwnedCStr = "corsa_bind_api_client_call_json"

#borrow(handle)
extern "C" fn ApiClient::release_handle(self : ApiClient, handle : Bytes) -> Int = "corsa_bind_api_client_release_handle"
extern "C" fn ApiClient::close(self : ApiClient) -> Int = "corsa_bind_api_client_close"

fn api_client_new(options_json : String) -> ApiClient {
  __api_client_new(string_to_c_bytes(options_json))
}

fn ApiClient::initialize(self : ApiClient) -> String {
  self.initialize_json().to_string()
}

fn ApiClient::parse_config_file(self : ApiClient, file : String) -> String {
  self.parse_config_file_json(string_to_c_bytes(file)).to_string()
}

fn ApiClient::update_snapshot(self : ApiClient, params_json : String) -> String {
  self.update_snapshot_json(string_to_c_bytes(params_json)).to_string()
}

fn ApiClient::get_string_type(self : ApiClient, snapshot : String, project : String) -> String {
  self.get_string_type_json(string_to_c_bytes(snapshot), string_to_c_bytes(project)).to_string()
}

fn ApiClient::render_type(self : ApiClient, snapshot : String, project : String, type_handle : String) -> String {
  self.type_to_string(
    string_to_c_bytes(snapshot),
    string_to_c_bytes(project),
    string_to_c_bytes(type_handle),
  ).to_string()
}

fn ApiClient::call(self : ApiClient, method : String, params_json : String) -> String {
  self.call_json(string_to_c_bytes(method), string_to_c_bytes(params_json)).to_string()
}

fn ApiClient::release(self : ApiClient, handle : String) -> Unit {
  if self.release_handle(string_to_c_bytes(handle)) < 0 {
    abort(last_error())
  }
}

fn ApiClient::shutdown(self : ApiClient) -> Unit {
  if self.close() < 0 {
    abort(last_error())
  }
}

#borrow(path)
#borrow(language_id)
#borrow(text)
extern "C" fn __virtual_document_untitled(path : Bytes, language_id : Bytes, text : Bytes) -> VirtualDocument = "corsa_bind_virtual_document_untitled"

#borrow(authority)
#borrow(path)
#borrow(language_id)
#borrow(text)
extern "C" fn __virtual_document_in_memory(authority : Bytes, path : Bytes, language_id : Bytes, text : Bytes) -> VirtualDocument = "corsa_bind_virtual_document_in_memory"

extern "C" fn VirtualDocument::free(self : VirtualDocument) -> Unit = "corsa_bind_virtual_document_free"
extern "C" fn VirtualDocument::version(self : VirtualDocument) -> Int = "corsa_bind_virtual_document_version"
extern "C" fn VirtualDocument::text_json(self : VirtualDocument) -> OwnedCStr = "corsa_bind_virtual_document_text"
extern "C" fn VirtualDocument::state_json(self : VirtualDocument) -> OwnedCStr = "corsa_bind_virtual_document_state_json"

#borrow(text)
extern "C" fn VirtualDocument::replace_raw(self : VirtualDocument, text : Bytes) -> Int = "corsa_bind_virtual_document_replace"

#borrow(changes)
extern "C" fn VirtualDocument::apply_changes_json(self : VirtualDocument, changes : Bytes) -> OwnedCStr = "corsa_bind_virtual_document_apply_changes_json"

fn virtual_document_untitled(path : String, language_id : String, text : String) -> VirtualDocument {
  __virtual_document_untitled(
    string_to_c_bytes(path),
    string_to_c_bytes(language_id),
    string_to_c_bytes(text),
  )
}

fn virtual_document_in_memory(authority : String, path : String, language_id : String, text : String) -> VirtualDocument {
  __virtual_document_in_memory(
    string_to_c_bytes(authority),
    string_to_c_bytes(path),
    string_to_c_bytes(language_id),
    string_to_c_bytes(text),
  )
}

fn VirtualDocument::text(self : VirtualDocument) -> String {
  self.text_json().to_string()
}

fn VirtualDocument::state(self : VirtualDocument) -> String {
  self.state_json().to_string()
}

fn VirtualDocument::replace(self : VirtualDocument, text : String) -> Unit {
  if self.replace_raw(string_to_c_bytes(text)) < 0 {
    abort(last_error())
  }
}

fn VirtualDocument::apply_changes(self : VirtualDocument, changes_json : String) -> String {
  self.apply_changes_json(string_to_c_bytes(changes_json)).to_string()
}

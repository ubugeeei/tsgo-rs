package corsa_bind

/*
#cgo CFLAGS: -I../../c/corsa_bind_c/include
#cgo LDFLAGS: -lcorsa_bind_c
#include <stdlib.h>
#include "corsa_bind.h"
*/
import "C"

import (
	"encoding/json"
	"errors"
	"unsafe"
)

type ApiClientOptions struct {
	Executable                 string `json:"executable"`
	Cwd                        string `json:"cwd,omitempty"`
	Mode                       string `json:"mode,omitempty"`
	RequestTimeoutMs           uint64 `json:"requestTimeoutMs,omitempty"`
	ShutdownTimeoutMs          uint64 `json:"shutdownTimeoutMs,omitempty"`
	OutboundCapacity           uint64 `json:"outboundCapacity,omitempty"`
	AllowUnstableUpstreamCalls bool   `json:"allowUnstableUpstreamCalls,omitempty"`
}

type UnsafeTypeFlowInput struct {
	SourceTypeTexts []string `json:"sourceTypeTexts"`
	TargetTypeTexts []string `json:"targetTypeTexts,omitempty"`
}

type ApiClient struct {
	ptr *C.CorsaBindApiClient
}

type VirtualDocument struct {
	ptr *C.CorsaBindVirtualDocument
}

func Version() string {
	return C.GoString(C.corsa_bind_version())
}

func lastError() error {
	ptr := C.corsa_bind_last_error_message()
	if ptr == nil {
		return errors.New("corsa_bind call failed")
	}
	return errors.New(C.GoString(ptr))
}

func marshalCString(value any) (*C.char, error) {
	payload, err := json.Marshal(value)
	if err != nil {
		return nil, err
	}
	return C.CString(string(payload)), nil
}

func takeString(ptr *C.char) (string, error) {
	if ptr == nil {
		return "", lastError()
	}
	defer C.corsa_bind_string_free(ptr)
	return C.GoString(ptr), nil
}

func copyBytes(buffer C.struct_CorsaBindBytes) ([]byte, error) {
	if buffer.ptr == nil {
		if C.corsa_bind_last_error_message() != nil {
			return nil, lastError()
		}
		return nil, nil
	}
	defer C.corsa_bind_bytes_free(buffer)
	return C.GoBytes(unsafe.Pointer(buffer.ptr), C.int(buffer.len)), nil
}

func boolFromResult(result C.int) (bool, error) {
	if result < 0 {
		return false, lastError()
	}
	return result != 0, nil
}

func IsUnsafeAssignment(input UnsafeTypeFlowInput) (bool, error) {
	cJSON, err := marshalCString(input)
	if err != nil {
		return false, err
	}
	defer C.free(unsafe.Pointer(cJSON))
	return boolFromResult(C.corsa_bind_is_unsafe_assignment(cJSON))
}

func IsUnsafeReturn(input UnsafeTypeFlowInput) (bool, error) {
	cJSON, err := marshalCString(input)
	if err != nil {
		return false, err
	}
	defer C.free(unsafe.Pointer(cJSON))
	return boolFromResult(C.corsa_bind_is_unsafe_return(cJSON))
}

func SpawnApiClient(options ApiClientOptions) (*ApiClient, error) {
	cJSON, err := marshalCString(options)
	if err != nil {
		return nil, err
	}
	defer C.free(unsafe.Pointer(cJSON))
	ptr := C.corsa_bind_api_client_new(cJSON)
	if ptr == nil {
		return nil, lastError()
	}
	return &ApiClient{ptr: ptr}, nil
}

func (client *ApiClient) Free() {
	if client.ptr != nil {
		C.corsa_bind_api_client_free(client.ptr)
		client.ptr = nil
	}
}

func (client *ApiClient) InitializeJSON() (string, error) {
	return takeString(C.corsa_bind_api_client_initialize_json(client.ptr))
}

func (client *ApiClient) ParseConfigFileJSON(file string) (string, error) {
	cFile := C.CString(file)
	defer C.free(unsafe.Pointer(cFile))
	return takeString(C.corsa_bind_api_client_parse_config_file_json(client.ptr, cFile))
}

func (client *ApiClient) UpdateSnapshotJSON(params any) (string, error) {
	if params == nil {
		return takeString(C.corsa_bind_api_client_update_snapshot_json(client.ptr, nil))
	}
	cJSON, err := marshalCString(params)
	if err != nil {
		return "", err
	}
	defer C.free(unsafe.Pointer(cJSON))
	return takeString(C.corsa_bind_api_client_update_snapshot_json(client.ptr, cJSON))
}

func (client *ApiClient) GetSourceFile(snapshot string, project string, file string) ([]byte, error) {
	cSnapshot := C.CString(snapshot)
	cProject := C.CString(project)
	cFile := C.CString(file)
	defer C.free(unsafe.Pointer(cSnapshot))
	defer C.free(unsafe.Pointer(cProject))
	defer C.free(unsafe.Pointer(cFile))
	return copyBytes(C.corsa_bind_api_client_get_source_file(client.ptr, cSnapshot, cProject, cFile))
}

func (client *ApiClient) GetStringTypeJSON(snapshot string, project string) (string, error) {
	cSnapshot := C.CString(snapshot)
	cProject := C.CString(project)
	defer C.free(unsafe.Pointer(cSnapshot))
	defer C.free(unsafe.Pointer(cProject))
	return takeString(C.corsa_bind_api_client_get_string_type_json(client.ptr, cSnapshot, cProject))
}

func (client *ApiClient) TypeToString(snapshot string, project string, typeHandle string, location *string, flags *int32) (string, error) {
	cSnapshot := C.CString(snapshot)
	cProject := C.CString(project)
	cHandle := C.CString(typeHandle)
	defer C.free(unsafe.Pointer(cSnapshot))
	defer C.free(unsafe.Pointer(cProject))
	defer C.free(unsafe.Pointer(cHandle))

	var cLocation *C.char
	if location != nil {
		cLocation = C.CString(*location)
		defer C.free(unsafe.Pointer(cLocation))
	}

	var cFlags C.int32_t
	var hasFlags C.int
	if flags != nil {
		cFlags = C.int32_t(*flags)
		hasFlags = 1
	}

	return takeString(C.corsa_bind_api_client_type_to_string(
		client.ptr,
		cSnapshot,
		cProject,
		cHandle,
		cLocation,
		cFlags,
		hasFlags,
	))
}

func (client *ApiClient) CallJSON(method string, params any) (string, error) {
	cMethod := C.CString(method)
	defer C.free(unsafe.Pointer(cMethod))

	var cJSON *C.char
	if params != nil {
		var err error
		cJSON, err = marshalCString(params)
		if err != nil {
			return "", err
		}
		defer C.free(unsafe.Pointer(cJSON))
	}

	return takeString(C.corsa_bind_api_client_call_json(client.ptr, cMethod, cJSON))
}

func (client *ApiClient) CallBinary(method string, params any) ([]byte, error) {
	cMethod := C.CString(method)
	defer C.free(unsafe.Pointer(cMethod))

	var cJSON *C.char
	if params != nil {
		var err error
		cJSON, err = marshalCString(params)
		if err != nil {
			return nil, err
		}
		defer C.free(unsafe.Pointer(cJSON))
	}

	return copyBytes(C.corsa_bind_api_client_call_binary(client.ptr, cMethod, cJSON))
}

func (client *ApiClient) ReleaseHandle(handle string) error {
	cHandle := C.CString(handle)
	defer C.free(unsafe.Pointer(cHandle))
	if C.corsa_bind_api_client_release_handle(client.ptr, cHandle) < 0 {
		return lastError()
	}
	return nil
}

func (client *ApiClient) Close() error {
	if C.corsa_bind_api_client_close(client.ptr) < 0 {
		return lastError()
	}
	return nil
}

func UntitledVirtualDocument(path string, languageID string, text string) (*VirtualDocument, error) {
	cPath := C.CString(path)
	cLanguageID := C.CString(languageID)
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cPath))
	defer C.free(unsafe.Pointer(cLanguageID))
	defer C.free(unsafe.Pointer(cText))
	ptr := C.corsa_bind_virtual_document_untitled(cPath, cLanguageID, cText)
	if ptr == nil {
		return nil, lastError()
	}
	return &VirtualDocument{ptr: ptr}, nil
}

func InMemoryVirtualDocument(authority string, path string, languageID string, text string) (*VirtualDocument, error) {
	cAuthority := C.CString(authority)
	cPath := C.CString(path)
	cLanguageID := C.CString(languageID)
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cAuthority))
	defer C.free(unsafe.Pointer(cPath))
	defer C.free(unsafe.Pointer(cLanguageID))
	defer C.free(unsafe.Pointer(cText))
	ptr := C.corsa_bind_virtual_document_in_memory(cAuthority, cPath, cLanguageID, cText)
	if ptr == nil {
		return nil, lastError()
	}
	return &VirtualDocument{ptr: ptr}, nil
}

func (document *VirtualDocument) Free() {
	if document.ptr != nil {
		C.corsa_bind_virtual_document_free(document.ptr)
		document.ptr = nil
	}
}

func (document *VirtualDocument) URI() (string, error) {
	return takeString(C.corsa_bind_virtual_document_uri(document.ptr))
}

func (document *VirtualDocument) LanguageID() (string, error) {
	return takeString(C.corsa_bind_virtual_document_language_id(document.ptr))
}

func (document *VirtualDocument) Version() int32 {
	return int32(C.corsa_bind_virtual_document_version(document.ptr))
}

func (document *VirtualDocument) Text() (string, error) {
	return takeString(C.corsa_bind_virtual_document_text(document.ptr))
}

func (document *VirtualDocument) StateJSON() (string, error) {
	return takeString(C.corsa_bind_virtual_document_state_json(document.ptr))
}

func (document *VirtualDocument) Replace(text string) error {
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))
	if C.corsa_bind_virtual_document_replace(document.ptr, cText) < 0 {
		return lastError()
	}
	return nil
}

func (document *VirtualDocument) ApplyChangesJSON(changes any) (string, error) {
	cJSON, err := marshalCString(changes)
	if err != nil {
		return "", err
	}
	defer C.free(unsafe.Pointer(cJSON))
	return takeString(C.corsa_bind_virtual_document_apply_changes_json(document.ptr, cJSON))
}

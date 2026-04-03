const std = @import("std");

pub const CorsaBindApiClient = opaque {};
pub const CorsaBindVirtualDocument = opaque {};

pub const CorsaBindBytes = extern struct {
    ptr: ?[*]u8,
    len: usize,
};

extern fn corsa_bind_version() [*:0]const u8;
extern fn corsa_bind_last_error_message() ?[*:0]const u8;
extern fn corsa_bind_string_free(ptr: ?[*:0]u8) void;
extern fn corsa_bind_bytes_free(bytes: CorsaBindBytes) void;
extern fn corsa_bind_is_unsafe_assignment(input_json: [*:0]const u8) c_int;
extern fn corsa_bind_is_unsafe_return(input_json: [*:0]const u8) c_int;
extern fn corsa_bind_api_client_new(options_json: [*:0]const u8) ?*CorsaBindApiClient;
extern fn corsa_bind_api_client_free(client: ?*CorsaBindApiClient) void;
extern fn corsa_bind_api_client_initialize_json(client: ?*CorsaBindApiClient) ?[*:0]u8;
extern fn corsa_bind_api_client_parse_config_file_json(client: ?*CorsaBindApiClient, file: [*:0]const u8) ?[*:0]u8;
extern fn corsa_bind_api_client_update_snapshot_json(client: ?*CorsaBindApiClient, params_json: ?[*:0]const u8) ?[*:0]u8;
extern fn corsa_bind_api_client_get_source_file(client: ?*CorsaBindApiClient, snapshot: [*:0]const u8, project: [*:0]const u8, file: [*:0]const u8) CorsaBindBytes;
extern fn corsa_bind_api_client_get_string_type_json(client: ?*CorsaBindApiClient, snapshot: [*:0]const u8, project: [*:0]const u8) ?[*:0]u8;
extern fn corsa_bind_api_client_type_to_string(client: ?*CorsaBindApiClient, snapshot: [*:0]const u8, project: [*:0]const u8, type_handle: [*:0]const u8, location: ?[*:0]const u8, flags: i32, has_flags: c_int) ?[*:0]u8;
extern fn corsa_bind_api_client_call_json(client: ?*CorsaBindApiClient, method: [*:0]const u8, params_json: ?[*:0]const u8) ?[*:0]u8;
extern fn corsa_bind_api_client_call_binary(client: ?*CorsaBindApiClient, method: [*:0]const u8, params_json: ?[*:0]const u8) CorsaBindBytes;
extern fn corsa_bind_api_client_release_handle(client: ?*CorsaBindApiClient, handle: [*:0]const u8) c_int;
extern fn corsa_bind_api_client_close(client: ?*CorsaBindApiClient) c_int;
extern fn corsa_bind_virtual_document_untitled(path: [*:0]const u8, language_id: [*:0]const u8, text: [*:0]const u8) ?*CorsaBindVirtualDocument;
extern fn corsa_bind_virtual_document_in_memory(authority: [*:0]const u8, path: [*:0]const u8, language_id: [*:0]const u8, text: [*:0]const u8) ?*CorsaBindVirtualDocument;
extern fn corsa_bind_virtual_document_free(document: ?*CorsaBindVirtualDocument) void;
extern fn corsa_bind_virtual_document_uri(document: ?*const CorsaBindVirtualDocument) ?[*:0]u8;
extern fn corsa_bind_virtual_document_language_id(document: ?*const CorsaBindVirtualDocument) ?[*:0]u8;
extern fn corsa_bind_virtual_document_version(document: ?*const CorsaBindVirtualDocument) i32;
extern fn corsa_bind_virtual_document_text(document: ?*const CorsaBindVirtualDocument) ?[*:0]u8;
extern fn corsa_bind_virtual_document_state_json(document: ?*const CorsaBindVirtualDocument) ?[*:0]u8;
extern fn corsa_bind_virtual_document_replace(document: ?*CorsaBindVirtualDocument, text: [*:0]const u8) c_int;
extern fn corsa_bind_virtual_document_apply_changes_json(document: ?*CorsaBindVirtualDocument, changes_json: [*:0]const u8) ?[*:0]u8;

pub const Error = error{CorsaBindFailure};

pub fn version() []const u8 {
    return std.mem.span(corsa_bind_version());
}

pub fn lastError(allocator: std.mem.Allocator) ![]u8 {
    const ptr = corsa_bind_last_error_message() orelse return allocator.dupe(u8, "corsa_bind call failed");
    return allocator.dupe(u8, std.mem.span(ptr));
}

fn checkBool(result: c_int) Error!bool {
    if (result < 0) {
        return Error.CorsaBindFailure;
    }
    return result != 0;
}

fn takeOwnedString(allocator: std.mem.Allocator, ptr: ?[*:0]u8) Error![]u8 {
    const actual = ptr orelse return Error.CorsaBindFailure;
    defer corsa_bind_string_free(actual);
    return allocator.dupe(u8, std.mem.span(actual));
}

fn copyBytes(allocator: std.mem.Allocator, bytes: CorsaBindBytes) Error!?[]u8 {
    if (bytes.ptr == null) {
        if (corsa_bind_last_error_message() != null) {
            return Error.CorsaBindFailure;
        }
        return null;
    }
    defer corsa_bind_bytes_free(bytes);
    return allocator.dupe(u8, bytes.ptr.?[0..bytes.len]);
}

pub fn isUnsafeAssignment(c_json: [*:0]const u8) Error!bool {
    return checkBool(corsa_bind_is_unsafe_assignment(c_json));
}

pub fn isUnsafeReturn(c_json: [*:0]const u8) Error!bool {
    return checkBool(corsa_bind_is_unsafe_return(c_json));
}

pub const ApiClient = struct {
    handle: *CorsaBindApiClient,

    pub fn spawn(options_json: [*:0]const u8) Error!ApiClient {
        const handle = corsa_bind_api_client_new(options_json) orelse return Error.CorsaBindFailure;
        return .{ .handle = handle };
    }

    pub fn deinit(self: *ApiClient) void {
        corsa_bind_api_client_free(self.handle);
        self.handle = undefined;
    }

    pub fn initializeJson(self: ApiClient, allocator: std.mem.Allocator) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_api_client_initialize_json(self.handle));
    }

    pub fn parseConfigFileJson(self: ApiClient, allocator: std.mem.Allocator, file: [*:0]const u8) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_api_client_parse_config_file_json(self.handle, file));
    }

    pub fn updateSnapshotJson(self: ApiClient, allocator: std.mem.Allocator, params_json: ?[*:0]const u8) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_api_client_update_snapshot_json(self.handle, params_json));
    }

    pub fn getSourceFile(self: ApiClient, allocator: std.mem.Allocator, snapshot: [*:0]const u8, project: [*:0]const u8, file: [*:0]const u8) Error!?[]u8 {
        return copyBytes(allocator, corsa_bind_api_client_get_source_file(self.handle, snapshot, project, file));
    }

    pub fn getStringTypeJson(self: ApiClient, allocator: std.mem.Allocator, snapshot: [*:0]const u8, project: [*:0]const u8) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_api_client_get_string_type_json(self.handle, snapshot, project));
    }

    pub fn typeToString(self: ApiClient, allocator: std.mem.Allocator, snapshot: [*:0]const u8, project: [*:0]const u8, type_handle: [*:0]const u8, location: ?[*:0]const u8, flags: ?i32) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_api_client_type_to_string(self.handle, snapshot, project, type_handle, location, flags orelse 0, if (flags == null) 0 else 1));
    }

    pub fn callJson(self: ApiClient, allocator: std.mem.Allocator, method: [*:0]const u8, params_json: ?[*:0]const u8) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_api_client_call_json(self.handle, method, params_json));
    }

    pub fn callBinary(self: ApiClient, allocator: std.mem.Allocator, method: [*:0]const u8, params_json: ?[*:0]const u8) Error!?[]u8 {
        return copyBytes(allocator, corsa_bind_api_client_call_binary(self.handle, method, params_json));
    }

    pub fn releaseHandle(self: ApiClient, handle: [*:0]const u8) Error!void {
        if (corsa_bind_api_client_release_handle(self.handle, handle) < 0) {
            return Error.CorsaBindFailure;
        }
    }

    pub fn close(self: ApiClient) Error!void {
        if (corsa_bind_api_client_close(self.handle) < 0) {
            return Error.CorsaBindFailure;
        }
    }
};

pub const VirtualDocument = struct {
    handle: *CorsaBindVirtualDocument,

    pub fn untitled(path: [*:0]const u8, language_id: [*:0]const u8, text: [*:0]const u8) Error!VirtualDocument {
        const handle = corsa_bind_virtual_document_untitled(path, language_id, text) orelse return Error.CorsaBindFailure;
        return .{ .handle = handle };
    }

    pub fn inMemory(authority: [*:0]const u8, path: [*:0]const u8, language_id: [*:0]const u8, text: [*:0]const u8) Error!VirtualDocument {
        const handle = corsa_bind_virtual_document_in_memory(authority, path, language_id, text) orelse return Error.CorsaBindFailure;
        return .{ .handle = handle };
    }

    pub fn deinit(self: *VirtualDocument) void {
        corsa_bind_virtual_document_free(self.handle);
        self.handle = undefined;
    }

    pub fn uri(self: VirtualDocument, allocator: std.mem.Allocator) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_virtual_document_uri(self.handle));
    }

    pub fn languageId(self: VirtualDocument, allocator: std.mem.Allocator) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_virtual_document_language_id(self.handle));
    }

    pub fn version(self: VirtualDocument) i32 {
        return corsa_bind_virtual_document_version(self.handle);
    }

    pub fn text(self: VirtualDocument, allocator: std.mem.Allocator) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_virtual_document_text(self.handle));
    }

    pub fn stateJson(self: VirtualDocument, allocator: std.mem.Allocator) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_virtual_document_state_json(self.handle));
    }

    pub fn replace(self: VirtualDocument, text: [*:0]const u8) Error!void {
        if (corsa_bind_virtual_document_replace(self.handle, text) < 0) {
            return Error.CorsaBindFailure;
        }
    }

    pub fn applyChangesJson(self: VirtualDocument, allocator: std.mem.Allocator, changes_json: [*:0]const u8) Error![]u8 {
        return takeOwnedString(allocator, corsa_bind_virtual_document_apply_changes_json(self.handle, changes_json));
    }
};

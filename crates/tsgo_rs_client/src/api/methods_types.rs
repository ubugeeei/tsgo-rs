use base64::{Engine as _, engine::general_purpose::STANDARD};

use super::{
    ApiClient, DocumentIdentifier, EncodedPayload, NodeHandle, ProjectHandle, SnapshotHandle,
    TypeHandle, TypeResponse,
    encoded::PrintNodeOptions,
    requests_core::{IntrinsicTypeRequest, TypeNodeRequest},
    requests_symbols::{PositionBatchRequest, TypeProjectRequest},
    requests_types::{PrintNodeRequest, SignatureOfTypeRequest, TypeLocationRequest},
};
use crate::Result;

impl ApiClient {
    pub async fn get_type_at_location(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTypeAtLocation",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    pub async fn get_type_at_locations(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        locations: Vec<NodeHandle>,
    ) -> Result<Vec<Option<TypeResponse>>> {
        self.call(
            "getTypeAtLocations",
            super::requests_symbols::NodeBatchRequest {
                snapshot,
                project,
                locations,
            },
        )
        .await
    }

    pub async fn get_type_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTypeAtPosition",
            super::requests_core::SymbolAtPositionRequest {
                snapshot,
                project,
                file: file.into(),
                position,
            },
        )
        .await
    }

    pub async fn get_types_at_positions(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        positions: Vec<u32>,
    ) -> Result<Vec<Option<TypeResponse>>> {
        self.call(
            "getTypesAtPositions",
            PositionBatchRequest {
                snapshot,
                project,
                file: file.into(),
                positions,
            },
        )
        .await
    }

    pub async fn get_signatures_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
        kind: i32,
    ) -> Result<Vec<super::SignatureResponse>> {
        self.call(
            "getSignaturesOfType",
            SignatureOfTypeRequest {
                snapshot,
                project,
                r#type,
                kind,
            },
        )
        .await
    }

    pub async fn get_contextual_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getContextualType",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    pub async fn get_base_type_of_literal_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getBaseTypeOfLiteralType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    pub async fn get_any_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getAnyType", snapshot, project).await
    }

    pub async fn get_string_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getStringType", snapshot, project)
            .await
    }

    pub async fn get_number_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getNumberType", snapshot, project)
            .await
    }

    pub async fn get_boolean_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getBooleanType", snapshot, project)
            .await
    }

    pub async fn get_void_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getVoidType", snapshot, project).await
    }

    pub async fn get_undefined_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getUndefinedType", snapshot, project)
            .await
    }

    pub async fn get_null_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getNullType", snapshot, project).await
    }

    pub async fn get_never_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getNeverType", snapshot, project).await
    }

    pub async fn get_unknown_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getUnknownType", snapshot, project)
            .await
    }

    pub async fn get_big_int_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getBigIntType", snapshot, project)
            .await
    }

    pub async fn get_es_symbol_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getESSymbolType", snapshot, project)
            .await
    }

    pub async fn type_to_type_node(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
        location: Option<NodeHandle>,
        flags: Option<i32>,
    ) -> Result<Option<EncodedPayload>> {
        self.call_optional_binary(
            "typeToTypeNode",
            TypeNodeRequest {
                snapshot,
                project,
                r#type,
                location,
                flags,
            },
        )
        .await
    }

    pub async fn type_to_string(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
        location: Option<NodeHandle>,
        flags: Option<i32>,
    ) -> Result<String> {
        self.call(
            "typeToString",
            TypeNodeRequest {
                snapshot,
                project,
                r#type,
                location,
                flags,
            },
        )
        .await
    }

    pub async fn is_context_sensitive(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<bool> {
        self.call(
            "isContextSensitive",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    pub async fn print_node(
        &self,
        payload: &EncodedPayload,
        options: PrintNodeOptions,
    ) -> Result<String> {
        self.call(
            "printNode",
            PrintNodeRequest {
                data: STANDARD.encode(payload.as_bytes()),
                preserve_source_newlines: options.preserve_source_newlines,
                never_ascii_escape: options.never_ascii_escape,
                terminate_unterminated_literals: options.terminate_unterminated_literals,
            },
        )
        .await
    }

    async fn call_intrinsic(
        &self,
        method: &str,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call(method, IntrinsicTypeRequest { snapshot, project })
            .await
    }
}

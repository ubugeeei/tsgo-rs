use serde_json::json;

use super::{
    ApiClient, DocumentIdentifier, DocumentPosition, NodeHandle, ProjectHandle, SnapshotHandle,
    SymbolHandle, SymbolResponse, TypeResponse,
    requests_core::{
        ResolveNameRequest, ShorthandValueRequest, SymbolAtLocationRequest,
        SymbolAtPositionRequest, TypeOfSymbolAtLocationRequest,
    },
    requests_symbols::{
        NodeBatchRequest, PositionBatchRequest, SymbolBatchRequest, SymbolOnlyRequest,
    },
};
use crate::Result;

impl ApiClient {
    pub async fn get_symbol_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "getSymbolAtPosition",
            SymbolAtPositionRequest {
                snapshot,
                project,
                file: file.into(),
                position,
            },
        )
        .await
    }

    pub async fn get_symbols_at_positions(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        positions: Vec<u32>,
    ) -> Result<Vec<Option<SymbolResponse>>> {
        self.call(
            "getSymbolsAtPositions",
            PositionBatchRequest {
                snapshot,
                project,
                file: file.into(),
                positions,
            },
        )
        .await
    }

    pub async fn get_symbol_at_location(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "getSymbolAtLocation",
            SymbolAtLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    pub async fn get_symbols_at_locations(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        locations: Vec<NodeHandle>,
    ) -> Result<Vec<Option<SymbolResponse>>> {
        self.call(
            "getSymbolsAtLocations",
            NodeBatchRequest {
                snapshot,
                project,
                locations,
            },
        )
        .await
    }

    pub async fn get_type_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTypeOfSymbol",
            json!({ "snapshot": snapshot, "project": project, "symbol": symbol }),
        )
        .await
    }

    pub async fn get_types_of_symbols(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbols: Vec<SymbolHandle>,
    ) -> Result<Vec<Option<TypeResponse>>> {
        self.call(
            "getTypesOfSymbols",
            SymbolBatchRequest {
                snapshot,
                project,
                symbols,
            },
        )
        .await
    }

    pub async fn get_declared_type_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getDeclaredTypeOfSymbol",
            json!({ "snapshot": snapshot, "project": project, "symbol": symbol }),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn resolve_name(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        name: impl Into<String>,
        meaning: u32,
        location: Option<NodeHandle>,
        file: Option<DocumentIdentifier>,
        position: Option<u32>,
        exclude_globals: Option<bool>,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "resolveName",
            ResolveNameRequest {
                snapshot,
                project,
                name: name.into(),
                location,
                file,
                position,
                meaning,
                exclude_globals,
            },
        )
        .await
    }

    pub async fn resolve_name_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        name: impl Into<String>,
        meaning: u32,
        location: DocumentPosition,
        exclude_globals: Option<bool>,
    ) -> Result<Option<SymbolResponse>> {
        self.resolve_name(
            snapshot,
            project,
            name,
            meaning,
            None,
            Some(location.document),
            Some(location.position),
            exclude_globals,
        )
        .await
    }

    pub async fn get_shorthand_assignment_value_symbol(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "getShorthandAssignmentValueSymbol",
            ShorthandValueRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    pub async fn get_type_of_symbol_at_location(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
        location: NodeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTypeOfSymbolAtLocation",
            TypeOfSymbolAtLocationRequest {
                snapshot,
                project,
                symbol,
                location,
            },
        )
        .await
    }

    pub async fn get_parent_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional("getParentOfSymbol", SymbolOnlyRequest { snapshot, symbol })
            .await
    }

    pub async fn get_members_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        symbol: SymbolHandle,
    ) -> Result<Vec<SymbolResponse>> {
        self.call::<Option<Vec<SymbolResponse>>, _>(
            "getMembersOfSymbol",
            SymbolOnlyRequest { snapshot, symbol },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_exports_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        symbol: SymbolHandle,
    ) -> Result<Vec<SymbolResponse>> {
        self.call::<Option<Vec<SymbolResponse>>, _>(
            "getExportsOfSymbol",
            SymbolOnlyRequest { snapshot, symbol },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_export_symbol_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        symbol: SymbolHandle,
    ) -> Result<SymbolResponse> {
        self.call(
            "getExportSymbolOfSymbol",
            SymbolOnlyRequest { snapshot, symbol },
        )
        .await
    }
}

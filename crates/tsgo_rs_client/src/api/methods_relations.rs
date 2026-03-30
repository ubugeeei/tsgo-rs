use super::{
    ApiClient, IndexInfo, ProjectHandle, SignatureHandle, SnapshotHandle, TypeHandle,
    TypePredicateResponse, TypeResponse,
    requests_symbols::{SignatureOnlyRequest, TypeOnlyRequest, TypeProjectRequest},
};
use crate::Result;

impl ApiClient {
    pub async fn get_symbol_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<super::SymbolResponse>> {
        self.call_optional("getSymbolOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    pub async fn get_return_type_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getReturnTypeOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
    }

    pub async fn get_rest_type_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getRestTypeOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
    }

    pub async fn get_type_predicate_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Option<TypePredicateResponse>> {
        self.call_optional(
            "getTypePredicateOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
    }

    pub async fn get_base_types(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getBaseTypes",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_properties_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<super::SymbolResponse>> {
        self.call::<Option<Vec<super::SymbolResponse>>, _>(
            "getPropertiesOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_index_infos_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<IndexInfo>> {
        self.call::<Option<Vec<IndexInfo>>, _>(
            "getIndexInfosOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_constraint_of_type_parameter(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getConstraintOfTypeParameter",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    pub async fn get_type_arguments(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getTypeArguments",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_target_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getTargetOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    pub async fn get_types_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getTypesOfType",
            TypeOnlyRequest { snapshot, r#type },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_type_parameters_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getTypeParametersOfType",
            TypeOnlyRequest { snapshot, r#type },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_outer_type_parameters_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getOuterTypeParametersOfType",
            TypeOnlyRequest { snapshot, r#type },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_local_type_parameters_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getLocalTypeParametersOfType",
            TypeOnlyRequest { snapshot, r#type },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    pub async fn get_object_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getObjectTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    pub async fn get_index_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getIndexTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    pub async fn get_check_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getCheckTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    pub async fn get_extends_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getExtendsTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    pub async fn get_base_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getBaseTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    pub async fn get_constraint_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getConstraintOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }
}

use content_store::ContentHash;
use serde::{Deserialize, Serialize};

use strum::EnumDiscriminants;
use telemetry::prelude::*;

use crate::workspace_snapshot::content_address::ContentAddress;
use crate::{pk, StandardModel, Timestamp};
use crate::{AttributePrototypeId, SchemaVariantId};

// const LIST_FOR_ATTRIBUTE_PROTOTYPE_WITH_TAIL_COMPONENT_ID: &str = include_str!(
//     "../queries/external_provider/list_for_attribute_prototype_with_tail_component_id.sql"
// );
// const FIND_FOR_SCHEMA_VARIANT_AND_NAME: &str =
//     include_str!("../queries/external_provider/find_for_schema_variant_and_name.sql");
// const FIND_FOR_SOCKET: &str = include_str!("../queries/external_provider/find_for_socket.sql");
// const LIST_FOR_SCHEMA_VARIANT: &str =
//     include_str!("../queries/external_provider/list_for_schema_variant.sql");
// const LIST_FROM_INTERNAL_PROVIDER_USE: &str =
//     include_str!("../queries/external_provider/list_from_internal_provider_use.sql");

pk!(ExternalProviderId);

/// This provider can only provide data to external [`SchemaVariants`](crate::SchemaVariant). It can
/// only consume data within its own [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ExternalProvider {
    id: ExternalProviderId,

    #[serde(flatten)]
    pub timestamp: Timestamp,

    /// Indicates which [`SchemaVariant`](crate::SchemaVariant) this provider belongs to.
    schema_variant_id: SchemaVariantId,
    /// Indicates which transformation function should be used for "emit".
    attribute_prototype_id: Option<AttributePrototypeId>,

    /// Name for [`Self`] that can be used for identification.
    name: String,
    /// Definition of the data type (e.g. "JSONSchema" or "Number").
    type_definition: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct ExternalProviderGraphNode {
    id: ExternalProviderId,
    content_address: ContentAddress,
    content: ExternalProviderContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
pub enum ExternalProviderContent {
    V1(ExternalProviderContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExternalProviderContentV1 {
    #[serde(flatten)]
    pub timestamp: Timestamp,

    /// Indicates which [`SchemaVariant`](crate::SchemaVariant) this provider belongs to.
    pub schema_variant_id: SchemaVariantId,
    /// Indicates which transformation function should be used for "emit".
    pub attribute_prototype_id: Option<AttributePrototypeId>,

    /// Name for [`Self`] that can be used for identification.
    pub name: String,
    /// Definition of the data type (e.g. "JSONSchema" or "Number").
    pub type_definition: Option<String>,
}

impl ExternalProviderGraphNode {
    pub fn assemble(
        id: impl Into<ExternalProviderId>,
        content_hash: ContentHash,
        content: ExternalProviderContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::ExternalProvider(content_hash),
            content,
        }
    }
}

// impl ExternalProvider {
//     /// This function will also create an _output_ [`Socket`](crate::Socket).
//     #[allow(clippy::too_many_arguments)]
//     #[tracing::instrument(skip(ctx, name))]
//     pub async fn new_with_socket(
//         ctx: &DalContext,
//         schema_id: SchemaId,
//         schema_variant_id: SchemaVariantId,
//         name: impl AsRef<str>,
//         type_definition: Option<String>,
//         func_id: FuncId,
//         func_binding_id: FuncBindingId,
//         func_binding_return_value_id: FuncBindingReturnValueId,
//         arity: SocketArity,
//         frame_socket: bool,
//     ) -> ExternalProviderResult<(Self, Socket)> {
//         let name = name.as_ref();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM external_provider_create_v1($1, $2, $3, $4, $5, $6)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &schema_id,
//                     &schema_variant_id,
//                     &name,
//                     &type_definition,
//                 ],
//             )
//             .await?;

//         let mut external_provider: ExternalProvider =
//             standard_model::finish_create_from_row(ctx, row).await?;

//         let attribute_context = AttributeContext::builder()
//             .set_external_provider_id(external_provider.id)
//             .to_context()?;
//         let attribute_prototype = AttributePrototype::new(
//             ctx,
//             func_id,
//             func_binding_id,
//             func_binding_return_value_id,
//             attribute_context,
//             None,
//             None,
//         )
//         .await?;
//         external_provider
//             .set_attribute_prototype_id(ctx, Some(*attribute_prototype.id()))
//             .await?;

//         let socket = Socket::new(
//             ctx,
//             name,
//             match frame_socket {
//                 true => SocketKind::Frame,
//                 false => SocketKind::Provider,
//             },
//             &SocketEdgeKind::ConfigurationOutput,
//             &arity,
//             &DiagramKind::Configuration,
//             Some(schema_variant_id),
//         )
//         .await?;
//         socket
//             .set_external_provider(ctx, external_provider.id())
//             .await?;

//         Ok((external_provider, socket))
//     }

//     // Immutable fields.
//     standard_model_accessor_ro!(schema_id, SchemaId);
//     standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

//     // Mutable fields.
//     standard_model_accessor!(name, String, ExternalProviderResult);
//     standard_model_accessor!(type_definition, Option<String>, ExternalProviderResult);
//     standard_model_accessor!(
//         attribute_prototype_id,
//         Option<Pk(AttributePrototypeId)>,
//         ExternalProviderResult
//     );

//     // This is a 1-1 relationship, so the Vec<Socket> should be 1
//     standard_model_has_many!(
//         lookup_fn: sockets,
//         table: "socket_belongs_to_external_provider",
//         model_table: "sockets",
//         returns: Socket,
//         result: ExternalProviderResult,
//     );

//     /// Find all [`Self`] for a given [`SchemaVariant`](crate::SchemaVariant).
//     #[tracing::instrument(skip(ctx))]
//     pub async fn list_for_schema_variant(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> ExternalProviderResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_SCHEMA_VARIANT,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find [`Self`] with a provided [`SocketId`](crate::Socket).
//     #[instrument(skip_all)]
//     pub async fn find_for_socket(
//         ctx: &DalContext,
//         socket_id: SocketId,
//     ) -> ExternalProviderResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_FOR_SOCKET,
//                 &[ctx.tenancy(), ctx.visibility(), &socket_id],
//             )
//             .await?;
//         Ok(standard_model::object_option_from_row_option(row)?)
//     }

//     /// Find [`Self`] with a provided name, which is not only the name of [`Self`], but also of the
//     /// associated _output_ [`Socket`](crate::Socket).
//     #[instrument(skip_all)]
//     pub async fn find_for_schema_variant_and_name(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         name: impl AsRef<str>,
//     ) -> ExternalProviderResult<Option<Self>> {
//         let name = name.as_ref();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_FOR_SCHEMA_VARIANT_AND_NAME,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id, &name],
//             )
//             .await?;
//         Ok(standard_model::object_option_from_row_option(row)?)
//     }

//     /// Find all [`Self`] for a given [`AttributePrototypeId`](crate::AttributePrototype).
//     #[tracing::instrument(skip(ctx))]
//     pub async fn list_for_attribute_prototype_with_tail_component_id(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//         tail_component_id: ComponentId,
//     ) -> ExternalProviderResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_ATTRIBUTE_PROTOTYPE_WITH_TAIL_COMPONENT_ID,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_prototype_id,
//                     &tail_component_id,
//                 ],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find all [`Self`] that have
//     /// [`AttributePrototypeArguments`](crate::AttributePrototypeArgument) referencing the provided
//     /// [`InternalProviderId`](crate::InternalProvider).
//     #[tracing::instrument(skip(ctx))]
//     pub async fn list_from_internal_provider_use(
//         ctx: &DalContext,
//         internal_provider_id: InternalProviderId,
//     ) -> ExternalProviderResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FROM_INTERNAL_PROVIDER_USE,
//                 &[ctx.tenancy(), ctx.visibility(), &internal_provider_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }
// }

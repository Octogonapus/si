//! An [`AttributeValue`] represents which [`FuncBinding`](crate::func::binding::FuncBinding)
//! and [`FuncBindingReturnValue`] provide attribute's value. Moreover, it tracks whether the
//! value is proxied or not. Proxied values "point" to another [`AttributeValue`] to provide
//! the attribute's value.
//!
//! ## Updating [`AttributeValues`](AttributeValue)
//!
//! Let's say you want to update a
//! [`PropertyEditorValue`](crate::property_editor::values::PropertyEditorValue) in the UI or a
//! "field" on a [`Component`](crate::Component) in general. The key to doing so is the following
//! process:
//!
//! 1) Find the appropriate [`AttributeValue`] in a [`context`](crate::AttributeContext) that is
//!   either "exactly specific" to what you need or "less specific" than what you need (see the
//!   [`module`](crate::attribute::context) for more information)
//! 2) Find its parent, which almost all [`AttributeValues`](AttributeValue) should have if they are
//!   in the lineage of a [`RootProp`](crate::RootProp) (usually, the
//!   [`standard model accessor`](crate::standard_accessors) that contains the parent will suffice
//!   in finding the parent)
//! 3) Use [`AttributeValue::update_for_context()`] with the appropriate key and
//!   [`context`](crate::AttributeContext) while ensuring that if you reuse the key and/or
//!   [`context`](crate::AttributeContext) from the [`AttributeValue`](crate::AttributeValue)
//!   that you found, that it is _exactly_ what you need (i.e. if the key changes or the
//!   [`context`](crate::AttributeContext) is in a lesser specificity than what you need, you
//!   mutate them accordingly)
//!
//! Often, you may not have all the information necessary to find the [`AttributeValue`] that you
//! would like to update. Ideally, you would use one of the existing accessor methods off
//! [`AttributeValue`] with contextual information such as a [`PropId`](crate::Prop),
//! a [`ComponentId`](crate::Component)), a parent [`AttributeValue`], a key, etc.
//!
//! In situations where we do not have minimal information to find the _correct_ [`AttributeValue`]
//! from existing accessor queries, we can leveraging existing queries from other structs and write
//! new queries for those structs and specific use cases. For example, since most members of the
//! [`RootProp`](crate::RootProp) tree are stable across [`SchemaVariants`](crate::SchemaVariant),
//! we can use [`Component::root_prop_child_attribute_value_for_component()`](crate::Component::root_prop_child_attribute_value_for_component)
//! to find the [`AttributeValue`] whose [`context`](crate::AttributeContext) corresponds to a
//! direct child [`Prop`](crate::Prop) of the [`RootProp`](crate::RootProp).

use content_store::ContentHash;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use telemetry::prelude::*;

use crate::workspace_snapshot::content_address::ContentAddress;
use crate::{
    pk, StandardModel, Timestamp,
};

pub mod view;

// const CHILD_ATTRIBUTE_VALUES_FOR_CONTEXT: &str =
//     include_str!("../queries/attribute_value/child_attribute_values_for_context.sql");
// const FETCH_UPDATE_GRAPH_DATA: &str =
//     include_str!("../queries/attribute_value/fetch_update_graph_data.sql");
// const IS_FOR_INTERNAL_PROVIDER_OF_ROOT_PROP: &str =
//     include_str!("../queries/attribute_value/is_for_internal_provider_of_root_prop.sql");
// const FIND_PROP_FOR_VALUE: &str =
//     include_str!("../queries/attribute_value/find_prop_for_value.sql");
// const FIND_WITH_PARENT_AND_KEY_FOR_CONTEXT: &str =
//     include_str!("../queries/attribute_value/find_with_parent_and_key_for_context.sql");
// const FIND_WITH_PARENT_AND_PROTOTYPE_FOR_CONTEXT: &str =
//     include_str!("../queries/attribute_value/find_with_parent_and_prototype_for_context.sql");
// const LIST_FOR_CONTEXT: &str = include_str!("../queries/attribute_value/list_for_context.sql");
// const LIST_PAYLOAD_FOR_READ_CONTEXT: &str =
//     include_str!("../queries/attribute_value/list_payload_for_read_context.sql");
// const LIST_PAYLOAD_FOR_READ_CONTEXT_AND_ROOT: &str =
//     include_str!("../queries/attribute_value/list_payload_for_read_context_and_root.sql");

pk!(AttributeValueId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeValue {
    pub id: AttributeValueId,
    #[serde(flatten)]
    pub timestamp: Timestamp,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    pub unprocessed_value: Option<serde_json::Value>,
    /// The processed return value.
    /// Example: empty array.
    pub value: Option<serde_json::Value>,
}

impl AttributeValue {
    pub fn assemble(id: AttributeValueId, inner: &AttributeValueContentV1) -> Self {
        let inner = inner.to_owned();
        Self {
            id,
            timestamp: inner.timestamp,
            value: inner.value,
            unprocessed_value: inner.unprocessed_value,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AttributeValueGraphNode {
    id: AttributeValueId,
    content_address: ContentAddress,
    content: AttributeValueContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
pub enum AttributeValueContent {
    V1(AttributeValueContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AttributeValueContentV1 {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    pub unprocessed_value: Option<serde_json::Value>,
    /// The processed return value.
    /// Example: empty array.
    pub value: Option<serde_json::Value>,
}

impl From<AttributeValue> for AttributeValueContentV1 {
    fn from(value: AttributeValue) -> Self {
        Self {
            timestamp: value.timestamp,
            value: value.value,
            unprocessed_value: value.unprocessed_value,
        }
    }
}

impl AttributeValueGraphNode {
    pub fn assemble(
        id: impl Into<AttributeValueId>,
        content_hash: ContentHash,
        content: AttributeValueContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::AttributeValue(content_hash),
            content,
        }
    }
}

// impl AttributeValue {
//     standard_model_accessor!(
//         proxy_for_attribute_value_id,
//         Option<Pk(AttributeValueId)>,
//         AttributeValueResult
//     );
//     standard_model_accessor!(sealed_proxy, bool, AttributeValueResult);
//     standard_model_accessor!(func_binding_id, Pk(FuncBindingId), AttributeValueResult);
//     standard_model_accessor!(
//         func_binding_return_value_id,
//         Pk(FuncBindingReturnValueId),
//         AttributeValueResult
//     );
//     standard_model_accessor!(index_map, Option<IndexMap>, AttributeValueResult);
//     standard_model_accessor!(key, Option<String>, AttributeValueResult);

//     standard_model_belongs_to!(
//         lookup_fn: parent_attribute_value,
//         set_fn: set_parent_attribute_value_unchecked,
//         unset_fn: unset_parent_attribute_value,
//         table: "attribute_value_belongs_to_attribute_value",
//         model_table: "attribute_values",
//         belongs_to_id: AttributeValueId,
//         returns: AttributeValue,
//         result: AttributeValueResult,
//     );

//     standard_model_has_many!(
//         lookup_fn: child_attribute_values,
//         table: "attribute_value_belongs_to_attribute_value",
//         model_table: "attribute_values",
//         returns: AttributeValue,
//         result: AttributeValueResult,
//     );

//     standard_model_belongs_to!(
//         lookup_fn: attribute_prototype,
//         set_fn: set_attribute_prototype,
//         unset_fn: unset_attribute_prototype,
//         table: "attribute_value_belongs_to_attribute_prototype",
//         model_table: "attribute_prototypes",
//         belongs_to_id: AttributePrototypeId,
//         returns: AttributePrototype,
//         result: AttributeValueResult,
//     );

//     pub fn index_map_mut(&mut self) -> Option<&mut IndexMap> {
//         self.index_map.as_mut()
//     }

//     /// Returns the *unprocessed* [`serde_json::Value`] within the [`FuncBindingReturnValue`](crate::FuncBindingReturnValue)
//     /// corresponding to the field on [`Self`].
//     pub async fn get_unprocessed_value(
//         &self,
//         ctx: &DalContext,
//     ) -> AttributeValueResult<Option<serde_json::Value>> {
//         match FuncBindingReturnValue::get_by_id(ctx, &self.func_binding_return_value_id).await? {
//             Some(func_binding_return_value) => {
//                 Ok(func_binding_return_value.unprocessed_value().cloned())
//             }
//             None => Err(AttributeValueError::MissingFuncBindingReturnValue),
//         }
//     }

//     /// Returns the [`serde_json::Value`] within the [`FuncBindingReturnValue`](crate::FuncBindingReturnValue)
//     /// corresponding to the field on [`Self`].
//     pub async fn get_value(
//         &self,
//         ctx: &DalContext,
//     ) -> AttributeValueResult<Option<serde_json::Value>> {
//         match FuncBindingReturnValue::get_by_id(ctx, &self.func_binding_return_value_id).await? {
//             Some(func_binding_return_value) => Ok(func_binding_return_value.value().cloned()),
//             None => Err(AttributeValueError::MissingFuncBindingReturnValue),
//         }
//     }

//     pub async fn update_stored_index_map(&self, ctx: &DalContext) -> AttributeValueResult<()> {
//         standard_model::update(
//             ctx,
//             "attribute_values",
//             "index_map",
//             self.id(),
//             &self.index_map,
//             TypeHint::JsonB,
//         )
//         .await?;
//         Ok(())
//     }

//     /// Returns a list of child [`AttributeValues`](crate::AttributeValue) for a given
//     /// [`AttributeValue`] and [`AttributeReadContext`](crate::AttributeReadContext).
//     pub async fn child_attribute_values_for_context(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         attribute_read_context: AttributeReadContext,
//     ) -> AttributeValueResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 CHILD_ATTRIBUTE_VALUES_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_value_id,
//                     &attribute_read_context,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     pub async fn find_with_parent_and_prototype_for_context(
//         ctx: &DalContext,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         attribute_prototype_id: AttributePrototypeId,
//         context: AttributeContext,
//     ) -> AttributeValueResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_WITH_PARENT_AND_PROTOTYPE_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &attribute_prototype_id,
//                     &parent_attribute_value_id,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::option_object_from_row(row)?)
//     }

//     /// Find [`Self`] with a given parent value and key.
//     pub async fn find_with_parent_and_key_for_context(
//         ctx: &DalContext,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         key: Option<String>,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_WITH_PARENT_AND_KEY_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &parent_attribute_value_id,
//                     &key,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::option_object_from_row(row)?)
//     }

//     /// List [`AttributeValues`](crate::AttributeValue) for a provided
//     /// [`AttributeReadContext`](crate::AttributeReadContext).
//     ///
//     /// If you only anticipate one result to be returned and have an
//     /// [`AttributeReadContext`](crate::AttributeReadContext)
//     /// that is also a valid [`AttributeContext`](crate::AttributeContext), then you should use
//     /// [`Self::find_for_context()`] instead of this method.
//     ///
//     /// This does _not_ work for maps and arrays, barring the _first_ instance of the array or map
//     /// object themselves! For those objects, please use
//     /// [`Self::find_with_parent_and_key_for_context()`].
//     pub async fn list_for_context(
//         ctx: &DalContext,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_CONTEXT,
//                 &[ctx.tenancy(), ctx.visibility(), &context],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find one [`AttributeValue`](crate::AttributeValue) for a provided
//     /// [`AttributeReadContext`](crate::AttributeReadContext).
//     ///
//     /// This is a modified version of [`Self::list_for_context()`] that requires an
//     /// [`AttributeReadContext`](crate::AttributeReadContext)
//     /// that is also a valid [`AttributeContext`](crate::AttributeContext) _and_ "pops" the first
//     /// row off the rows found (which are sorted from most to least specific). Thus, the "popped"
//     /// row will corresponding to the most specific [`AttributeValue`] found.
//     ///
//     /// This does _not_ work for maps and arrays, barring the _first_ instance of the array or map
//     /// object themselves! For those objects, please use
//     /// [`Self::find_with_parent_and_key_for_context()`].
//     pub async fn find_for_context(
//         ctx: &DalContext,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Option<Self>> {
//         AttributeContextBuilder::from(context).to_context()?;
//         let mut rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_CONTEXT,
//                 &[ctx.tenancy(), ctx.visibility(), &context],
//             )
//             .await?;
//         let maybe_row = rows.pop();
//         Ok(standard_model::option_object_from_row(maybe_row)?)
//     }

//     /// Return the [`Prop`] that the [`AttributeValueId`] belongs to,
//     /// following the relationship through [`AttributePrototype`].
//     pub async fn find_prop_for_value(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//     ) -> AttributeValueResult<Prop> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 FIND_PROP_FOR_VALUE,
//                 &[ctx.tenancy(), ctx.visibility(), &attribute_value_id],
//             )
//             .await?;

//         Ok(standard_model::object_from_row(row)?)
//     }

//     /// List [`AttributeValuePayloads`](AttributeValuePayload) for a given
//     /// [`context`](crate::AttributeReadContext), which must specify a
//     /// [`ComponentId`](crate::Component).
//     pub async fn list_payload_for_read_context(
//         ctx: &DalContext,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Vec<AttributeValuePayload>> {
//         let schema_variant_id = match context.component_id {
//             Some(component_id) if component_id != ComponentId::NONE => {
//                 let component = Component::get_by_id(ctx, &component_id)
//                     .await?
//                     .ok_or(AttributeValueError::ComponentNotFoundById(component_id))?;
//                 let schema_variant = component
//                     .schema_variant(ctx)
//                     .await
//                     .map_err(|e| AttributeValueError::Component(e.to_string()))?
//                     .ok_or(AttributeValueError::SchemaVariantNotFoundForComponent(
//                         component_id,
//                     ))?;
//                 *schema_variant.id()
//             }
//             _ => {
//                 return Err(AttributeValueError::MissingComponentInReadContext(context));
//             }
//         };

//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_PAYLOAD_FOR_READ_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &schema_variant_id,
//                 ],
//             )
//             .await?;
//         let mut result = Vec::new();
//         for row in rows.into_iter() {
//             let func_binding_return_value_json: serde_json::Value = row.try_get("object")?;
//             let func_binding_return_value: Option<FuncBindingReturnValue> =
//                 serde_json::from_value(func_binding_return_value_json)?;

//             let prop_json: serde_json::Value = row.try_get("prop_object")?;
//             let prop: Prop = serde_json::from_value(prop_json)?;

//             let attribute_value_json: serde_json::Value = row.try_get("attribute_value_object")?;
//             let attribute_value: AttributeValue = serde_json::from_value(attribute_value_json)?;

//             let parent_attribute_value_id: Option<AttributeValueId> =
//                 row.try_get("parent_attribute_value_id")?;

//             result.push(AttributeValuePayload::new(
//                 prop,
//                 func_binding_return_value,
//                 attribute_value,
//                 parent_attribute_value_id,
//             ));
//         }
//         Ok(result)
//     }

//     /// This method is similar to [`Self::list_payload_for_read_context()`], but it leverages a
//     /// root [`AttributeValueId`](crate::AttributeValue) in order to find payloads at any
//     /// root [`Prop`](crate::Prop) corresponding to the provided context and root value.
//     ///
//     /// Requirements for the [`AttributeReadContext`](crate::AttributeReadContext):
//     /// - [`PropId`](crate::Prop) must be set to [`None`]
//     /// - Both providers fields must be unset
//     pub async fn list_payload_for_read_context_and_root(
//         ctx: &DalContext,
//         root_attribute_value_id: AttributeValueId,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Vec<AttributeValuePayload>> {
//         if context.has_prop_id()
//             || !context.has_unset_internal_provider()
//             || !context.has_unset_external_provider()
//         {
//             return Err(AttributeValueError::IncompatibleAttributeReadContext("incompatible attribute read context for query: prop must be empty and providers must be unset"));
//         }

//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_PAYLOAD_FOR_READ_CONTEXT_AND_ROOT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &root_attribute_value_id,
//                 ],
//             )
//             .await?;

//         let mut result = Vec::new();
//         for row in rows.into_iter() {
//             let func_binding_return_value_json: serde_json::Value = row.try_get("object")?;
//             let func_binding_return_value: Option<FuncBindingReturnValue> =
//                 serde_json::from_value(func_binding_return_value_json)?;

//             let prop_json: serde_json::Value = row.try_get("prop_object")?;
//             let prop: Prop = serde_json::from_value(prop_json)?;

//             let attribute_value_json: serde_json::Value = row.try_get("attribute_value_object")?;
//             let attribute_value: AttributeValue = serde_json::from_value(attribute_value_json)?;

//             let parent_attribute_value_id: Option<AttributeValueId> =
//                 row.try_get("parent_attribute_value_id")?;

//             result.push(AttributeValuePayload::new(
//                 prop,
//                 func_binding_return_value,
//                 attribute_value,
//                 parent_attribute_value_id,
//             ));
//         }
//         Ok(result)
//     }

//     /// Update the [`AttributeValue`] for a specific [`AttributeContext`] to the given value. If the
//     /// given [`AttributeValue`] is for a different [`AttributeContext`] than the one provided, a
//     /// new [`AttributeValue`] will be created for the given [`AttributeContext`].
//     ///
//     /// By passing in [`None`] as the `value`, the caller is explicitly saying "this value does not
//     /// exist here". This is potentially useful for "tombstoning" values that have been inherited
//     /// from a less-specific [`AttributeContext`]. For example, if a value has been set for a
//     /// [`SchemaVariant`](crate::SchemaVariant), but we do not want that value to exist for a
//     /// specific [`Component`](crate::Component), we can update the variant's value to [`None`] in
//     /// an [`AttributeContext`] specific to that component.
//     ///
//     /// This method returns the following:
//     /// - the [`Option<serde_json::Value>`] that was passed in
//     /// - the updated [`AttributeValueId`](Self)
//     pub async fn update_for_context(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         context: AttributeContext,
//         value: Option<serde_json::Value>,
//         // TODO: Allow updating the key
//         key: Option<String>,
//     ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
//         Self::update_for_context_raw(
//             ctx,
//             attribute_value_id,
//             parent_attribute_value_id,
//             context,
//             value,
//             key,
//             true,
//             true,
//         )
//         .await
//     }

//     pub async fn update_for_context_without_propagating_dependent_values(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         context: AttributeContext,
//         value: Option<serde_json::Value>,
//         // TODO: Allow updating the key
//         key: Option<String>,
//     ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
//         Self::update_for_context_raw(
//             ctx,
//             attribute_value_id,
//             parent_attribute_value_id,
//             context,
//             value,
//             key,
//             true,
//             false,
//         )
//         .await
//     }

//     pub async fn update_for_context_without_creating_proxies(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         context: AttributeContext,
//         value: Option<serde_json::Value>,
//         // TODO: Allow updating the key
//         key: Option<String>,
//     ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
//         Self::update_for_context_raw(
//             ctx,
//             attribute_value_id,
//             parent_attribute_value_id,
//             context,
//             value,
//             key,
//             false,
//             true,
//         )
//         .await
//     }

//     #[allow(clippy::too_many_arguments)]
//     async fn update_for_context_raw(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         context: AttributeContext,
//         value: Option<serde_json::Value>,
//         // TODO: Allow updating the key
//         key: Option<String>,
//         create_child_proxies: bool,
//         propagate_dependent_values: bool,
//     ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
//         // TODO(nick,paulo,zack,jacob): ensure we do not _have_ to do this in the future.
//         let ctx = &ctx.clone_without_deleted_visibility();

//         let row = ctx.txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT new_attribute_value_id FROM attribute_value_update_for_context_raw_v1($1, $2, $3, $4, $5, $6, $7, $8)",
//             &[
//                 ctx.tenancy(),
//                 ctx.visibility(),
//                 &attribute_value_id,
//                 &parent_attribute_value_id,
//                 &context,
//                 &value,
//                 &key,
//                 &create_child_proxies,
//             ],
//             ).await?;

//         let new_attribute_value_id: AttributeValueId = row.try_get("new_attribute_value_id")?;

//         // TODO(fnichol): we might want to fire off a status even at this point, however we've
//         // already updated the initial attribute value, so is there much value?

//         if propagate_dependent_values && !ctx.no_dependent_values() {
//             ctx.enqueue_job(DependentValuesUpdate::new(
//                 ctx.access_builder(),
//                 *ctx.visibility(),
//                 vec![new_attribute_value_id],
//             ))
//             .await?;
//         }

//         Ok((value, new_attribute_value_id))
//     }

//     /// Insert a new value under the parent [`AttributeValue`] in the given [`AttributeContext`]. This is mostly only
//     /// useful for adding elements to a [`PropKind::Array`], or to a [`PropKind::Map`]. Updating existing values in an
//     /// [`Array`](PropKind::Array), or [`Map`](PropKind::Map), and setting/updating all other [`PropKind`] should be
//     /// able to directly use [`update_for_context()`](AttributeValue::update_for_context()), as there will already be an
//     /// appropriate [`AttributeValue`] to use. By using this function,
//     /// [`update_for_context()`](AttributeValue::update_for_context()) is called after we have created an appropriate
//     /// [`AttributeValue`] to use.
//     #[instrument(skip_all, level = "debug")]
//     pub async fn insert_for_context(
//         ctx: &DalContext,
//         item_attribute_context: AttributeContext,
//         array_or_map_attribute_value_id: AttributeValueId,
//         value: Option<serde_json::Value>,
//         key: Option<String>,
//     ) -> AttributeValueResult<AttributeValueId> {
//         Self::insert_for_context_raw(
//             ctx,
//             item_attribute_context,
//             array_or_map_attribute_value_id,
//             value,
//             key,
//             true,
//         )
//         .await
//     }

//     #[instrument(skip_all, level = "debug")]
//     pub async fn insert_for_context_without_creating_proxies(
//         ctx: &DalContext,
//         parent_context: AttributeContext,
//         parent_attribute_value_id: AttributeValueId,
//         value: Option<serde_json::Value>,
//         key: Option<String>,
//     ) -> AttributeValueResult<AttributeValueId> {
//         Self::insert_for_context_raw(
//             ctx,
//             parent_context,
//             parent_attribute_value_id,
//             value,
//             key,
//             false,
//         )
//         .await
//     }

//     #[instrument(skip_all, level = "debug")]
//     async fn insert_for_context_raw(
//         ctx: &DalContext,
//         item_attribute_context: AttributeContext,
//         array_or_map_attribute_value_id: AttributeValueId,
//         value: Option<serde_json::Value>,
//         key: Option<String>,
//         create_child_proxies: bool,
//     ) -> AttributeValueResult<AttributeValueId> {
//         let row = ctx.txns().await?.pg().query_one(
//             "SELECT new_attribute_value_id FROM attribute_value_insert_for_context_raw_v1($1, $2, $3, $4, $5, $6, $7)",
//             &[
//                 ctx.tenancy(),
//                 ctx.visibility(),
//                 &item_attribute_context,
//                 &array_or_map_attribute_value_id,
//                 &value,
//                 &key,
//                 &create_child_proxies,
//             ],
//         ).await?;

//         let new_attribute_value_id: AttributeValueId = row.try_get("new_attribute_value_id")?;

//         if !ctx.no_dependent_values() {
//             ctx.enqueue_job(DependentValuesUpdate::new(
//                 ctx.access_builder(),
//                 *ctx.visibility(),
//                 vec![new_attribute_value_id],
//             ))
//             .await?;
//         }

//         Ok(new_attribute_value_id)
//     }

//     #[instrument(skip_all, level = "debug")]
//     pub async fn update_parent_index_map(&self, ctx: &DalContext) -> AttributeValueResult<()> {
//         let _row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 "SELECT attribute_value_update_parent_index_map_v1($1, $2, $3)",
//                 &[ctx.tenancy(), ctx.visibility(), &self.id],
//             )
//             .await?;

//         Ok(())
//     }

//     async fn populate_nested_values(
//         ctx: &DalContext,
//         parent_attribute_value_id: AttributeValueId,
//         update_context: AttributeContext,
//         unprocessed_value: serde_json::Value,
//     ) -> AttributeValueResult<()> {
//         let _row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 "SELECT attribute_value_populate_nested_values_v1($1, $2, $3, $4, $5)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &parent_attribute_value_id,
//                     &update_context,
//                     &unprocessed_value,
//                 ],
//             )
//             .await?;

//         Ok(())
//     }

//     /// Convenience method to determine if this [`AttributeValue`](Self) is for the implicit
//     /// [`InternalProvider`](crate::InternalProvider) that represents the "snapshot" of the entire
//     /// [`Component`](crate::Component). This means that the [`Prop`](crate::Prop) that the
//     /// [`InternalProvider`](crate::InternalProvider) is sourcing its data from does not have a
//     /// parent [`Prop`](crate::Prop).
//     #[allow(unused)]
//     async fn is_for_internal_provider_of_root_prop(
//         &mut self,
//         ctx: &DalContext,
//     ) -> AttributeValueResult<bool> {
//         let maybe_row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 IS_FOR_INTERNAL_PROVIDER_OF_ROOT_PROP,
//                 &[&ctx.tenancy(), ctx.visibility(), &self.context],
//             )
//             .await?;
//         if let Some(row) = maybe_row {
//             // If we got a row back, that means that we are an AttributeValue for an InternalProvider,
//             // and we should have gotten a row back from the query.
//             Ok(row.try_get("is_for_root_prop")?)
//         } else {
//             // If we didn't get a row back, that means that we didn't find an InternalProvider for the
//             // InternalProviderId in our AttributeContext. Likely because it is ident_nil_v1, indicating that we're
//             // not for an InternalProvider at all.
//             Ok(false)
//         }
//     }

//     #[instrument(skip(ctx), level = "debug")]
//     pub async fn create_dependent_values(
//         ctx: &DalContext,
//         attribute_value_ids: &[AttributeValueId],
//     ) -> AttributeValueResult<()> {
//         ctx.txns()
//             .await?
//             .pg()
//             .execute(
//                 "SELECT attribute_value_create_new_affected_values_v1($1, $2, $3)",
//                 &[&ctx.tenancy(), &ctx.visibility(), &attribute_value_ids],
//             )
//             .await?;
//         Ok(())
//     }

//     /// Returns a [`HashMap`] with key [`AttributeValueId`](Self) and value
//     /// [`Vec<AttributeValueId>`](Self) where the keys correspond to [`AttributeValues`](Self) that
//     /// are affected (directly and indirectly) by at least one of the provided
//     /// [`AttributeValueIds`](Self) having a new value. The [`Vec<AttributeValueId>`](Self)
//     /// correspond to the [`AttributeValues`](Self) that the key directly depends on that are also
//     /// affected by at least one of the provided [`AttributeValueIds`](Self) having a new value.
//     ///
//     /// **NOTE**: This has the side effect of **CREATING NEW [`AttributeValues`](Self)**
//     /// if this [`AttributeValue`] affects an [`AttributeContext`](crate::AttributeContext) where an
//     /// [`AttributePrototype`](crate::AttributePrototype) that uses it didn't already have an
//     /// [`AttributeValue`].
//     #[instrument(skip(ctx), level = "debug")]
//     pub async fn dependent_value_graph(
//         ctx: &DalContext,
//         attribute_value_ids: &[AttributeValueId],
//     ) -> AttributeValueResult<HashMap<AttributeValueId, Vec<AttributeValueId>>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FETCH_UPDATE_GRAPH_DATA,
//                 &[&ctx.tenancy(), ctx.visibility(), &attribute_value_ids],
//             )
//             .instrument(debug_span!("Graph SQL query"))
//             .await?;

//         let mut result: HashMap<AttributeValueId, Vec<AttributeValueId>> = HashMap::new();
//         for row in rows.into_iter() {
//             let attr_val_id: AttributeValueId = row.try_get("attribute_value_id")?;
//             let dependencies: Vec<AttributeValueId> =
//                 row.try_get("dependent_attribute_value_ids")?;
//             result.insert(attr_val_id, dependencies);
//         }

//         Ok(result)
//     }

//     pub async fn vivify_value_and_parent_values(
//         &self,
//         ctx: &DalContext,
//     ) -> AttributeValueResult<AttributeValueId> {
//         let row = ctx.txns().await?.pg().query_one(
//             "SELECT new_attribute_value_id FROM attribute_value_vivify_value_and_parent_values_raw_v1($1, $2, $3, $4, $5)",
//         &[
//             ctx.tenancy(),
//             ctx.visibility(),
//             &self.context,
//             &self.id,
//             &true
//         ]).await?;

//         Ok(row.try_get("new_attribute_value_id")?)
//     }

//     /// Re-evaluates the current `AttributeValue`'s `AttributePrototype` to update the
//     /// `FuncBinding`, and `FuncBindingReturnValue`, reflecting the current inputs to
//     /// the function.
//     ///
//     /// If the `AttributeValue` represents the `InternalProvider` for a `Prop` that
//     /// does not have a parent `Prop` (this is typically the `InternalProvider` for
//     /// the "root" `Prop` of a `SchemaVariant`), then it will also enqueue a
//     /// `CodeGeneration` job for the `Component`.
//     #[instrument(
//         name = "attribute_value.update_from_prototype_function",
//         skip_all,
//         level = "debug",
//         fields(
//             attribute_value.id = %self.id,
//             change_set_pk = %ctx.visibility().change_set_pk,
//         )
//     )]
//     pub async fn update_from_prototype_function(
//         &mut self,
//         ctx: &DalContext,
//     ) -> AttributeValueResult<()> {
//         // Check if this AttributeValue is for an implicit InternalProvider as they have special behavior that doesn't involve
//         // AttributePrototype and AttributePrototypeArguments.
//         if self
//             .context
//             .is_least_specific_field_kind_internal_provider()?
//         {
//             let internal_provider =
//                 InternalProvider::get_by_id(ctx, &self.context.internal_provider_id())
//                     .await?
//                     .ok_or_else(|| {
//                         AttributeValueError::InternalProviderNotFound(
//                             self.context.internal_provider_id(),
//                         )
//                     })?;
//             if internal_provider.is_internal_consumer() {
//                 // We don't care about the AttributeValue that comes back from implicit_emit, since we should already be
//                 // operating on an AttributeValue that has the correct AttributeContext, which means that a new one should
//                 // not need to be created.
//                 internal_provider
//                     .implicit_emit(ctx, self)
//                     .await
//                     .map_err(|e| AttributeValueError::InternalProvider(e.to_string()))?;

//                 debug!("InternalProvider is internal consumer");

//                 return Ok(());
//             }
//         } else if self.context.is_least_specific_field_kind_prop()? {
//             if let Some(parent_attribute_value) = self.parent_attribute_value(ctx).await? {
//                 parent_attribute_value
//                     .vivify_value_and_parent_values(ctx)
//                     .await?;
//             }
//         }

//         // The following should handle explicit "normal" Attributes, InternalProviders, and ExternalProviders already.
//         let attribute_prototype = self.attribute_prototype(ctx).await?.ok_or_else(|| {
//             AttributeValueError::AttributePrototypeNotFound(self.id, *ctx.visibility())
//         })?;

//         let mut func_binding_args: HashMap<String, Option<serde_json::Value>> = HashMap::new();
//         for mut argument_data in attribute_prototype
//             .argument_values(ctx, self.context)
//             .await
//             .map_err(|e| AttributeValueError::AttributePrototype(e.to_string()))?
//         {
//             match argument_data.values.len() {
//                 1 => {
//                     let argument = argument_data.values.pop().ok_or_else(|| {
//                         AttributeValueError::EmptyAttributePrototypeArgumentsForGroup(
//                             argument_data.argument_name.clone(),
//                         )
//                     })?;

//                     func_binding_args.insert(
//                         argument_data.argument_name,
//                         Some(serde_json::to_value(argument)?),
//                     );
//                 }
//                 2.. => {
//                     func_binding_args.insert(
//                         argument_data.argument_name,
//                         Some(serde_json::to_value(argument_data.values)?),
//                     );
//                 }
//                 _ => {
//                     return Err(
//                         AttributeValueError::EmptyAttributePrototypeArgumentsForGroup(
//                             argument_data.argument_name,
//                         ),
//                     );
//                 }
//             };
//         }

//         let func_id = attribute_prototype.func_id();
//         let (func_binding, mut func_binding_return_value) = match FuncBinding::create_and_execute(
//             ctx,
//             serde_json::to_value(func_binding_args.clone())?,
//             attribute_prototype.func_id(),
//         )
//         .instrument(debug_span!(
//             "Func execution",
//             "func.id" = %func_id,
//             ?func_binding_args,
//         ))
//         .await
//         {
//             Ok(function_return_value) => function_return_value,
//             Err(FuncBindingError::FuncBackendResultFailure {
//                 kind,
//                 message,
//                 backend,
//             }) => {
//                 return Err(AttributeValueError::FuncBackendResultFailure {
//                     kind,
//                     message,
//                     backend,
//                 })
//             }
//             Err(err) => Err(err)?,
//         };

//         self.set_func_binding_id(ctx, *func_binding.id()).await?;
//         self.set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
//             .await?;

//         // If the value we just updated was for a Prop, we might have run a function that
//         // generates a deep data structure. If the Prop is an Array/Map/Object, then the
//         // value should be an empty Array/Map/Object, while the unprocessed value contains
//         // the deep data structure.
//         if self.context.is_least_specific_field_kind_prop()? {
//             let processed_value = match func_binding_return_value.unprocessed_value().cloned() {
//                 Some(unprocessed_value) => {
//                     let prop = Prop::get_by_id(ctx, &self.context.prop_id())
//                         .await?
//                         .ok_or_else(|| AttributeValueError::PropNotFound(self.context.prop_id()))?;

//                     match prop.kind() {
//                         PropKind::Object | PropKind::Map => Some(serde_json::json!({})),
//                         PropKind::Array => Some(serde_json::json!([])),
//                         _ => Some(unprocessed_value),
//                     }
//                 }
//                 None => None,
//             };

//             func_binding_return_value
//                 .set_value(ctx, processed_value)
//                 .await?;
//         };
//         // If they are different from each other, then we know
//         // that we need to fully process the deep data structure, populating
//         // AttributeValues for the child Props.
//         // cannot be si:setArray / si:setMap / si:setObject
//         if self.context.prop_id() != PropId::NONE {
//             let prop = Prop::get_by_id(ctx, &self.context.prop_id())
//                 .await?
//                 .ok_or_else(|| AttributeValueError::PropNotFound(self.context.prop_id()))?;

//             if *prop.kind() == PropKind::Array
//                 || *prop.kind() == PropKind::Object
//                 || *prop.kind() == PropKind::Map
//             {
//                 let func_name = match *prop.kind() {
//                     PropKind::Array => "si:setArray",
//                     PropKind::Object => "si:setObject",
//                     PropKind::Map => "si:setMap",
//                     _ => unreachable!(),
//                 };

//                 let func = Func::find_by_attr(ctx, "name", &func_name)
//                     .await?
//                     .pop()
//                     .ok_or_else(|| AttributeValueError::MissingFunc(func_name.to_owned()))?;

//                 if attribute_prototype.func_id() != *func.id() {
//                     if let Some(unprocessed_value) =
//                         func_binding_return_value.unprocessed_value().cloned()
//                     {
//                         AttributeValue::populate_nested_values(
//                             ctx,
//                             self.id,
//                             self.context,
//                             unprocessed_value,
//                         )
//                         .await?;
//                     }
//                 }
//             }
//         }

//         Ok(())
//     }

//     pub async fn populate_child_proxies_for_value(
//         &self,
//         ctx: &DalContext,
//         less_specific_attribute_value_id: AttributeValueId,
//         more_specific_context: AttributeContext,
//     ) -> AttributeValueResult<Option<Vec<AttributeValueId>>> {
//         let row = ctx.txns().await?.pg().query_one(
//             "SELECT new_proxy_value_ids FROM attribute_value_populate_child_proxies_for_value_v1($1, $2, $3, $4, $5)",
//             &[
//                 ctx.tenancy(),
//                 ctx.visibility(),
//                 &less_specific_attribute_value_id,
//                 &more_specific_context,
//                 self.id(),
//             ]
//         ).await?;

//         // Are we part of a map or array? Be sure to update the index map
//         if self.key.is_some() {
//             ctx.txns()
//                 .await?
//                 .pg()
//                 .query_opt(
//                     "SELECT * FROM attribute_value_update_parent_index_map_v1($1, $2, $3)",
//                     &[ctx.tenancy(), ctx.visibility(), self.id()],
//                 )
//                 .await?;
//         }

//         Ok(row.try_get("new_proxy_value_ids")?)
//     }
// }

// #[derive(Debug, Clone)]
// pub struct AttributeValuePayload {
//     pub prop: Prop,
//     pub func_binding_return_value: Option<FuncBindingReturnValue>,
//     pub attribute_value: AttributeValue,
//     pub parent_attribute_value_id: Option<AttributeValueId>,
// }

// impl AttributeValuePayload {
//     pub fn new(
//         prop: Prop,
//         func_binding_return_value: Option<FuncBindingReturnValue>,
//         attribute_value: AttributeValue,
//         parent_attribute_value_id: Option<AttributeValueId>,
//     ) -> Self {
//         Self {
//             prop,
//             func_binding_return_value,
//             attribute_value,
//             parent_attribute_value_id,
//         }
//     }
// }

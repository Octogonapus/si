use dal::{DalContext, Prop, PropKind, Schema, SchemaVariant, StandardModel};
use dal_test::helpers::generate_fake_name;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext) {
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not find schema");
    let schema_variant_id = *schema
        .default_schema_variant_id()
        .expect("could not get default variant id");
    let domain_prop = SchemaVariant::find_prop_in_tree(ctx, schema_variant_id, &["root", "domain"])
        .await
        .expect("could not find prop");
    let prop = Prop::new(
        ctx,
        "coolness",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*domain_prop.id()),
        None,
    )
    .await
    .expect("cannot create prop");
    assert_eq!(prop.name(), "coolness");
    assert_eq!(prop.kind(), &PropKind::String);
}

#[test]
async fn parent_props(ctx: &DalContext) {
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not find schema");
    let schema_variant_id = *schema
        .default_schema_variant_id()
        .expect("could not get default variant id");
    let domain_prop = SchemaVariant::find_prop_in_tree(ctx, schema_variant_id, &["root", "domain"])
        .await
        .expect("could not find prop");

    let parent_prop = Prop::new(
        ctx,
        generate_fake_name(),
        PropKind::Object,
        None,
        schema_variant_id,
        Some(*domain_prop.id()),
        None,
    )
    .await
    .expect("cannot create prop");
    let child_prop = Prop::new(
        ctx,
        generate_fake_name(),
        PropKind::String,
        None,
        schema_variant_id,
        Some(*parent_prop.id()),
        None,
    )
    .await
    .expect("cannot create prop");
    let retrieved_parent_prop = child_prop
        .parent_prop(ctx)
        .await
        .expect("cannot get parent prop")
        .expect("there was no parent prop and we expected one!");
    assert_eq!(retrieved_parent_prop, parent_prop);

    let children = parent_prop
        .child_props(ctx)
        .await
        .expect("should have children");
    assert_eq!(children, vec![child_prop]);
}

#[test]
async fn parent_props_wrong_prop_kinds(ctx: &DalContext) {
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not find schema");
    let schema_variant = schema
        .default_variant(ctx)
        .await
        .expect("could not get default schema variant");
    let root_prop_id = schema_variant
        .root_prop_id()
        .expect("could not get root prop id");

    let parent_prop = Prop::new(
        ctx,
        generate_fake_name(),
        PropKind::String,
        None,
        *schema_variant.id(),
        Some(*root_prop_id),
        None,
    )
    .await
    .expect("cannot create prop");
    let result = Prop::new(
        ctx,
        generate_fake_name(),
        PropKind::Object,
        None,
        *schema_variant.id(),
        Some(*parent_prop.id()),
        None,
    )
    .await;
    result.expect_err("should have errored, and it did not");
}

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};
use crate::server::tracking::track;
use crate::service::pkg::{PkgError, PkgResult};
use axum::extract::OriginalUri;
use axum::Json;
use dal::{HistoryActor, User, Visibility, WsEvent};
use module_index_client::IndexClient;
use serde::{Deserialize, Serialize};
use si_pkg::SiPkg;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BeginImportFlow {
    pub id: Ulid,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CancelImportFlow {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn begin_approval_process(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    Json(request): Json<BeginImportFlow>,
) -> PkgResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(PkgError::ModuleIndexNotConfigured),
    };

    let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let pkg_data = module_index_client.download_module(request.id).await?;

    let pkg = SiPkg::load_from_bytes(pkg_data)?;
    let metadata = pkg.metadata()?;

    let user_pk = match ctx.history_actor() {
        HistoryActor::User(user_pk) => {
            let user = User::get_by_pk(&ctx, *user_pk)
                .await?
                .ok_or(PkgError::InvalidUser(*user_pk))?;

            Some(user.pk())
        }

        HistoryActor::SystemInit => None,
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "begin_approval_process",
        serde_json::json!({
            "how": "/pkg/begin_approval_process",
            "workspace_pk": ctx.tenancy().workspace_pk(),
        }),
    );

    WsEvent::workspace_import_begin_approval_process(
        &ctx,
        ctx.tenancy().workspace_pk(),
        user_pk,
        metadata.created_at(),
        metadata.created_by().to_string(),
        metadata.name().to_string(),
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    WsEvent::import_workspace_vote(
        &ctx,
        ctx.tenancy().workspace_pk(),
        user_pk.expect("A user was definitely found as per above"),
        "Approve".to_string(),
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    Ok(Json(()))
}

pub async fn cancel_approval_process(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CancelImportFlow>,
) -> PkgResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let user_pk = match ctx.history_actor() {
        HistoryActor::User(user_pk) => {
            let user = User::get_by_pk(&ctx, *user_pk)
                .await?
                .ok_or(PkgError::InvalidUser(*user_pk))?;

            Some(user.pk())
        }

        HistoryActor::SystemInit => None,
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "cancel_approval_process",
        serde_json::json!({
            "how": "/pkg/cancel_approval_process",
            "workspace_pk": ctx.tenancy().workspace_pk(),
        }),
    );

    WsEvent::workspace_import_cancel_approval_process(&ctx, ctx.tenancy().workspace_pk(), user_pk)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(()))
}

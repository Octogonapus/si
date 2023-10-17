use content_store::{ContentHash, Store};

use ulid::Ulid;

use crate::change_set_pointer::ChangeSetPointer;
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::{FuncContent, FuncContentV1, FuncGraphNode};

use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::{WorkspaceSnapshotError, WorkspaceSnapshotResult};
use crate::{
    DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncId, Timestamp,
    WorkspaceSnapshot,
};

// TODO(nick,jacob): when "updating content" to set the code, we need to do something like the following:
// code_base64 text,
// code_sha256 text GENERATED ALWAYS AS (COALESCE(ENCODE(DIGEST(code_base64, 'sha256'), 'hex'), '0')) STORE

impl WorkspaceSnapshot {
    pub async fn func_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        name: impl AsRef<str>,
        backend_kind: FuncBackendKind,
        backend_response_type: FuncBackendResponseType,
    ) -> WorkspaceSnapshotResult<Func> {
        let name = name.as_ref().to_string();
        let timestamp = Timestamp::now();
        let _finalized_once = false;

        let content = FuncContentV1 {
            timestamp,
            name: name.clone(),
            display_name: None,
            description: None,
            link: None,
            hidden: false,
            builtin: false,
            backend_kind,
            backend_response_type,
            handler: None,
            code_base64: None,
            code_sha256: "".to_string(),
        };

        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&FuncContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_func(change_set, id, name.clone(), hash)?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        let (_, func_category_index) = self
            .working_copy()?
            .get_category_child(CategoryNodeKind::Func)?;
        self.working_copy()?.add_edge(
            func_category_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            node_index,
        )?;

        Ok(Func::assemble(id.into(), &content))
    }

    pub async fn func_get_content(
        &mut self,
        ctx: &DalContext,
        func_id: FuncId,
    ) -> WorkspaceSnapshotResult<(ContentHash, FuncContentV1)> {
        let id: Ulid = func_id.into();
        let node_index = self.working_copy()?.get_node_index_by_id(id)?;
        let node_weight = self.working_copy()?.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: FuncContent = ctx
            .content_store()
            .lock()
            .await
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let FuncContent::V1(inner) = content;

        Ok((hash, inner))
    }

    pub async fn func_get_by_id(
        &mut self,
        ctx: &DalContext,
        func_id: FuncId,
    ) -> WorkspaceSnapshotResult<Func> {
        let (_, content) = self.func_get_content(ctx, func_id).await?;

        Ok(Func::assemble(func_id, &content))
    }

    pub fn func_find_intrinsic(
        &mut self,
        intrinsic: IntrinsicFunc,
    ) -> WorkspaceSnapshotResult<FuncId> {
        let name = intrinsic.name();
        Ok(self
            .func_find_by_name(name)?
            .ok_or(WorkspaceSnapshotError::IntrinsicFuncNotFound(
                name.to_owned(),
            ))?)
    }

    pub fn func_find_by_name(
        &mut self,
        name: impl AsRef<str>,
    ) -> WorkspaceSnapshotResult<Option<FuncId>> {
        let (_, func_category_index) = self
            .working_copy()?
            .get_category_child(CategoryNodeKind::Func)?;

        let func_id = self
            .working_copy()?
            .func_find_by_name(func_category_index, name)?;

        Ok(func_id.into())
    }

    pub async fn func_modify_by_id<L>(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        id: FuncId,
        lambda: L,
    ) -> WorkspaceSnapshotResult<Func>
    where
        L: FnOnce(&mut Func) -> WorkspaceSnapshotResult<()>,
    {
        let (_, inner) = dbg!(self.func_get_content(ctx, id).await)?;

        dbg!("got content", &inner);

        let mut func = Func::assemble(id, &inner);
        lambda(&mut func)?;
        let updated = FuncContentV1::from(func);

        dbg!("updated content", &updated);

        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&FuncContent::V1(updated.clone()))?;

        dbg!("added content");

        self.working_copy()?
            .update_content(change_set, id.into(), hash)?;

        dbg!("update content");

        Ok(Func::assemble(id, &updated))
    }
}

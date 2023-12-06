import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "./change_sets.store";
import { ComponentId, useComponentsStore } from "./components.store";
import { ActionKind, useFixesStore } from "./fixes.store";

export type ActionStatus = "failure" | "success";

export type ActionPrototypeId = string;
export type ActionInstanceId = string;

export type ProposedAction = ActionInstance & { kind: ActionKind };

export interface ActionPrototype {
  id: ActionPrototypeId;
  name: string;
  displayName: string;
}

export interface NewAction {
  id: never;
  prototypeId: ActionPrototypeId;
  name: string;
  componentId: ComponentId;
  displayName: string;
}

export type ActionId = string;
export interface ActionInstance {
  id: ActionId;
  actionPrototypeId: ActionPrototypeId;
  name: string;
  componentId: ComponentId;
  actor?: string;
  parents: ActionId[];
}

export type FullAction = {
  actionPrototypeId: ActionPrototypeId;
  actionInstanceId?: ActionId;
  componentId?: ComponentId;
  actor?: string;
} & Omit<ActionPrototype, "id">;

export const useActionsStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const componentsStore = useComponentsStore();

  return addStoreHooks(
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/actions`,
      {
        state: () => ({}),
        getters: {
          proposedActions(): ProposedAction[] {
            const graph = changeSetsStore.selectedChangeSet?.actions ?? {};
            const actions = [];
            while (_.keys(graph).length) {
              const removeIds = [];

              const sortedEntries = _.entries(graph);
              sortedEntries.sort(([a], [b]) => a.localeCompare(b));

              for (const [id, action] of sortedEntries) {
                if (action.parents.length === 0) {
                  actions.push(action);
                  removeIds.push(id);
                }
              }

              for (const removeId of removeIds) {
                delete graph[removeId];
                for (const childAction of _.values(graph)) {
                  const index = childAction.parents.findIndex(
                    (parentId) => parentId === removeId,
                  );
                  if (index !== -1) {
                    childAction.parents.splice(index);
                  }
                }
              }
            }
            return actions;
          },
          actionsByComponentId(): Record<ComponentId, FullAction[]> {
            return _.mapValues(
              componentsStore.componentsById,
              (component, componentId) => {
                return _.compact(
                  _.map(component.actions, (actionPrototype) => {
                    if (actionPrototype.name === "refresh") return;
                    const actionInstance: ActionInstance | undefined = _.find(
                      _.values(changeSetsStore.selectedChangeSet?.actions),
                      (pa) =>
                        pa.componentId === componentId &&
                        pa.actionPrototypeId === actionPrototype.id,
                    );

                    return {
                      actionPrototypeId: actionPrototype.id,
                      actionInstanceId: actionInstance?.id,
                      componentId: actionInstance?.componentId,
                      actor: actionInstance?.actor,
                      ..._.omit(actionPrototype, "id"),
                    };
                  }),
                );
              },
            );
          },

          actionHistoryByComponentId() {
            const fixesStore = useFixesStore();
            const allHistory = _.flatMap(
              fixesStore.fixBatches,
              (batch) => batch.fixes,
            );
            return _.groupBy(allHistory, (entry) => entry.componentId);
          },
        },
        actions: {
          async ADD_ACTION(
            componentId: ComponentId,
            actionPrototypeId: ActionPrototypeId,
          ) {
            return new ApiRequest({
              method: "post",
              url: "change_set/add_action",
              keyRequestStatusBy: [componentId, actionPrototypeId],
              params: {
                prototypeId: actionPrototypeId,
                componentId,
                visibility_change_set_pk: changeSetId,
              },
            });
          },
          async REMOVE_ACTION(id: ActionId) {
            return new ApiRequest<null>({
              method: "post",
              url: "change_set/remove_action",
              keyRequestStatusBy: id,
              params: {
                id,
                visibility_change_set_pk: changeSetId,
              },
            });
          },
        },
        onActivated() {
          if (!changeSetId) return;
        },
      },
    ),
  )();
};

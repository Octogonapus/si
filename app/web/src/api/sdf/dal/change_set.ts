import { ProposedAction, ActionId } from "@/store/actions.store";
import { UserId } from "@/store/auth.store";

export enum ChangeSetStatus {
  Open = "Open",
  Applied = "Applied",
  Failed = "Failed",
  Closed = "Closed",
  Abandoned = "Abandoned",
  NeedsApproval = "NeedsApproval",
}

export type ChangeSetId = string;
export interface ChangeSet {
  id: ChangeSetId;
  pk: ChangeSetId;
  name: string;
  actions: Record<ActionId, ProposedAction>;
  status: ChangeSetStatus;
  appliedByUserId?: UserId;
  appliedAt?: IsoDateString;
  mergeRequestedAt?: IsoDateString;
  mergeRequestedByUserId?: UserId;
}

export type ChangeStatus = "added" | "deleted" | "modified" | "unmodified";

export interface ComponentStatsGroup {
  componentId: string;
  componentName: string;
  componentStatus: ChangeStatus;
}

export interface ComponentStats {
  stats: ComponentStatsGroup[];
}

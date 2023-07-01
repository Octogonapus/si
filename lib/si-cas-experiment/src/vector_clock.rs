use std::collections::HashMap;

use ulid::Ulid;

use crate::{
    lamport_clock::LamportClock, change_set::ChangeSetPk, error::{DagResult, DagError},
};

// We keep a vector clock of every changeset that has impacted our given object id
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct VectorClock {
    pub object_id: Ulid,
    pub clock_entries: HashMap<ChangeSetPk, LamportClock>,
}

impl VectorClock {
    pub fn new(object_id: Ulid, change_set_pk: ChangeSetPk) -> VectorClock {
        let lc = LamportClock::new(change_set_pk);
        let mut clock_entries = HashMap::new();
        clock_entries.insert(change_set_pk, lc);
        VectorClock {
            object_id,
            clock_entries,
        }
    }

    pub fn inc(&mut self, change_set_pk: ChangeSetPk) {
        self.clock_entries
            .entry(change_set_pk)
            .and_modify(|lc| lc.inc())
            .or_insert(LamportClock::new(change_set_pk));
    }

    pub fn merge(&mut self, change_set_pk: ChangeSetPk, other: &VectorClock) -> DagResult<()> {
        if self.object_id != other.object_id {
            return Err(DagError::CannotMergeVectorClocksForDifferentObjects);
        }
        for (other_key, other_value) in other.clock_entries.iter() {
            self.clock_entries
                .entry(*other_key)
                .and_modify(|my_value| my_value.merge(other_value))
                .or_insert(other_value.clone());
        }
        self.inc(change_set_pk);
        Ok(())
    }

    pub fn fork(&self, change_set_pk: ChangeSetPk) -> DagResult<VectorClock> {
        let mut forked = self.clone();
        forked.inc(change_set_pk);
        Ok(forked)
    }

    // We are 'newer' than the other clock if we have seen all of the other clocks
    // change sets, and we are newer than they are.
    pub fn already_seen(&self, other: &VectorClock) -> bool {
        let mut is_newer = true;
        for other_clock in other.clock_entries.values() {
            if let Some(my_clock) = self.clock_entries.get(&other_clock.change_set_pk) {
                if my_clock < other_clock {
                    is_newer = false;
                }
            } else {
                is_newer = false;
            }
        }
        is_newer
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn already_seen() {
        let object_id = Ulid::new();
        let mut vector_clock_a = VectorClock::new(object_id, ChangeSetPk::new());
        let vector_clock_b = vector_clock_a.fork(ChangeSetPk::new()).unwrap();
        assert_eq!(vector_clock_b.already_seen(&vector_clock_a), true);
        assert_eq!(vector_clock_a.already_seen(&vector_clock_b), false);
        let change_set_pk = ChangeSetPk::new();
        vector_clock_a.merge(change_set_pk, &vector_clock_b).unwrap();
        assert_eq!(vector_clock_a.already_seen(&vector_clock_b), true);
    }
}

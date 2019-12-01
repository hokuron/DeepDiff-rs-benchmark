use std::hash::Hash;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Eq, PartialEq, Debug)]
enum Counter {
    Zero, One, Many
}

impl Counter {
    fn increment(&self) -> Counter {
        match self {
            Counter::Zero => Counter::One,
            Counter::One => Counter::Many,
            Counter::Many => Counter::Many
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct TableEntry {
    old_counter: Counter,
    new_counter: Counter,
    indexes_in_old: Vec<usize>,
}

impl TableEntry {
    fn new() -> Self {
        TableEntry{
            old_counter: Counter::Zero,
            new_counter: Counter::Zero,
            indexes_in_old: Vec::new()
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
enum ArrayEntry {
    TableEntry(Rc<RefCell<TableEntry>>),
    IndexInOther(usize),
}

pub enum Change<'a, T> {
    Insert(Insert<'a, T>),
    Delete(Delete<'a, T>),
    Replace(Replace<'a, T>),
    Move(Move<'a, T>)
}
pub struct Insert<'a, T> {
    item: &'a T,
    index: usize,
}

pub struct Delete<'a, T> {
    item: &'a T,
    index: usize,
}

pub struct Replace<'a, T> {
    old_item: &'a T,
    new_item: &'a T,
    index: usize,
}

pub struct Move<'a, T> {
    item: &'a T,
    from_index: usize,
    to_index: usize,
}

#[inline]
pub fn diff<'a, T: Eq + Hash>(old: &'a [T], new: &'a [T]) -> Vec<Change<'a, T>> {
    let mut table = HashMap::new();
    let mut old_array = Vec::new();
    let mut new_array = Vec::new();

    for item in new {
        let entry = table
            .entry(item)
            .or_insert(Rc::new(RefCell::new(TableEntry::new())));
        let mut e = entry.borrow_mut();
        e.new_counter = e.new_counter.increment();
        new_array.push(ArrayEntry::TableEntry(Rc::clone(entry)));
    }

    for (idx, item) in old.iter().enumerate() {
        let entry = table
            .entry(item)
            .or_insert(Rc::new(RefCell::new(TableEntry::new())));
        let mut e = entry.borrow_mut();
        e.old_counter = e.old_counter.increment();
        e.indexes_in_old.push(idx);
        old_array.push(ArrayEntry::TableEntry(Rc::clone(entry)));
    }

    for (new_idx, item) in new_array.iter_mut().enumerate() {
        match item.clone() {
            ArrayEntry::TableEntry(ref entry) => {
                let mut entry = entry.borrow_mut();

                if entry.indexes_in_old.is_empty() {
                    continue;
                }

                let old_idx = entry.indexes_in_old.remove(0);
                let is_observation1 = entry.new_counter == Counter::One && entry.old_counter == Counter::One;
                let is_observation2 = entry.new_counter != Counter::Zero && entry.old_counter != Counter::Zero && item == &mut old_array[old_idx];

                if is_observation1 || is_observation2 {
                    *item = ArrayEntry::IndexInOther(old_idx);
                    old_array[old_idx] = ArrayEntry::IndexInOther(new_idx);
                }
            },
            _ => continue
        }
    }

    let mut changes = Vec::new();
    let mut delete_offsets = vec![0; old.len()];

    let mut running_offset = 0;
    for (old_offset, entry) in old_array.iter().enumerate() {
        delete_offsets[old_offset] = running_offset;

        match entry {
            ArrayEntry::TableEntry(_te) => {
                let delete = Delete { item: &old[old_offset], index: old_offset };
                changes.push(Change::Delete(delete));

                running_offset += 1;
            },
            _ => continue
        }
    }

    running_offset = 0;
    for (new_offset, entry) in new_array.iter().enumerate() {
        match entry {
            ArrayEntry::TableEntry(_te) => {
                running_offset += 1;

                let insert = Insert { item: &new[new_offset], index: new_offset };
                changes.push(Change::Insert(insert));
            },
            ArrayEntry::IndexInOther(old_idx) => {
                if old[*old_idx] != new[new_offset] {
                    let replace = Replace { old_item: &old[*old_idx], new_item: &new[new_offset], index: new_offset };
                    changes.push(Change::Replace(replace));
                }

                let delete_offset = delete_offsets[*old_idx];
                if (old_idx - delete_offset + running_offset) != new_offset {
                    let r#move = Move { item: &new[new_offset], from_index: *old_idx, to_index: new_offset };
                    changes.push(Change::Move(r#move));
                }
            },
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a, T> Change<'a, T> {
        pub fn insert(&self) -> Option<&Insert<T>> {
            match self {
                Change::Insert(i) => Some(i),
                _ => None
            }
        }

        pub fn delete(&self) -> Option<&Delete<T>> {
            match self {
                Change::Delete(d) => Some(d),
                _ => None
            }
        }

        pub fn replace(&self) -> Option<&Replace<T>> {
            match self {
                Change::Replace(r) => Some(r),
                _ => None
            }
        }

        pub fn r#move(&self) -> Option<&Move<T>> {
            match self {
                Change::Move(m) => Some(m),
                _ => None
            }
        }
    }

    #[test]
    fn empty() {
        let old: Vec<String> = Vec::new();
        let changes = diff(&old, &[]);
        assert!(changes.is_empty());
    }

    #[test]
    fn all_insert() {
        let new = vec!["a", "b", "c"];
        let changes = diff(&[], &new);
        assert_eq!(changes.len(), 3);

        assert_eq!(changes[0].insert().unwrap().item, &"a");
        assert_eq!(changes[0].insert().unwrap().index, 0);
        assert_eq!(changes[1].insert().unwrap().item, &"b");
        assert_eq!(changes[1].insert().unwrap().index, 1);
        assert_eq!(changes[2].insert().unwrap().item, &"c");
        assert_eq!(changes[2].insert().unwrap().index, 2);
    }

    #[test]
    fn all_delete() {
        let old = vec!["a", "b", "c"];
        let changes = diff(&old, &[]);
        assert_eq!(changes.len(), 3);

        assert_eq!(changes[0].delete().unwrap().item, &"a");
        assert_eq!(changes[0].delete().unwrap().index, 0);
        assert_eq!(changes[1].delete().unwrap().item, &"b");
        assert_eq!(changes[1].delete().unwrap().index, 1);
        assert_eq!(changes[2].delete().unwrap().item, &"c");
        assert_eq!(changes[2].delete().unwrap().index, 2);
    }

    #[test]
    fn all_replace() {
        let old = vec!["a", "b", "c"];
        let new = vec!["A", "B", "C"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 6);

        assert!(changes[0].delete().is_some());
        assert!(changes[1].delete().is_some());
        assert!(changes[2].delete().is_some());
        assert!(changes[3].insert().is_some());
        assert!(changes[4].insert().is_some());
        assert!(changes[5].insert().is_some());
    }

    #[test]
    fn insert() {
        let old = vec!["a"];
        let new = vec!["b", "a"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 1);

        assert!(changes[0].insert().is_some());
    }

    #[test]
    fn replace() {
        let old = vec!["a", "b", "c"];
        let new = vec!["a", "B", "c"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 2);

        assert!(changes[0].delete().is_some());
        assert!(changes[1].insert().is_some());
    }

    #[test]
    fn same_prefix() {
        let old = vec!["a", "b", "c"];
        let new = vec!["a", "B"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 3);

        assert!(changes[0].delete().is_some());
        assert!(changes[1].delete().is_some());
        assert!(changes[2].insert().is_some());
    }

    #[test]
    fn reversed() {
        let old = vec!["a", "b", "c"];
        let new = vec!["c", "b", "a"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 2);

        assert!(changes[0].r#move().is_some());
        assert!(changes[1].r#move().is_some());
    }

    #[test]
    fn small_changes_at_edges() {
        let old = "sitting".chars().map(|c|c.to_string()).collect::<Vec<_>>();
        let new = "kitten".chars().map(|c|c.to_string()).collect::<Vec<_>>();
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 5);

        assert!(changes[0].delete().is_some());
        assert!(changes[1].delete().is_some());
        assert!(changes[2].delete().is_some());
        assert!(changes[3].insert().is_some());
        assert!(changes[4].insert().is_some());
    }

    #[test]
    fn same_postfix() {
        let old = vec!["a", "b", "c", "d", "e", "f"];
        let new = vec!["d", "e", "f"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 3);

        assert_eq!(changes[0].delete().unwrap().item, &"a");
        assert_eq!(changes[0].delete().unwrap().index, 0);

        assert_eq!(changes[1].delete().unwrap().item, &"b");
        assert_eq!(changes[1].delete().unwrap().index, 1);

        assert_eq!(changes[2].delete().unwrap().item, &"c");
        assert_eq!(changes[2].delete().unwrap().index, 2);
    }

    #[test]
    fn shifting() {
        let old = vec!["a", "b", "c", "d"];
        let new = vec!["c", "d", "e", "f"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 4);

        assert_eq!(changes[0].delete().unwrap().item, &"a");
        assert_eq!(changes[0].delete().unwrap().index, 0);

        assert_eq!(changes[1].delete().unwrap().item, &"b");
        assert_eq!(changes[1].delete().unwrap().index, 1);

        assert_eq!(changes[2].insert().unwrap().item, &"e");
        assert_eq!(changes[2].insert().unwrap().index, 2);

        assert_eq!(changes[3].insert().unwrap().item, &"f");
        assert_eq!(changes[3].insert().unwrap().index, 3);
    }

    #[test]
    fn replace_whole_new_word() {
        let old = vec!["a", "b", "c"];
        let new = vec!["d"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 4);

        assert!(changes[0].delete().is_some());
        assert!(changes[1].delete().is_some());
        assert!(changes[2].delete().is_some());
        assert!(changes[3].insert().is_some());
    }

    #[test]
    fn replace_one_character() {
        let old = vec!["a"];
        let new = vec!["b"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 2);

        assert!(changes[0].delete().is_some());
        assert!(changes[1].insert().is_some());
    }

    #[test]
    fn move_with_insert_delete() {
        let old = vec![1, 2, 3, 4, 5];
        let new = vec![1, 5, 2, 3 ,4];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 4);

        assert!(changes[0].r#move().is_some());
        assert!(changes[1].r#move().is_some());
        assert!(changes[2].r#move().is_some());
        assert!(changes[3].r#move().is_some());
    }

    #[test]
    fn move_with_delete_insert() {
        let old = vec![3, 2, 1];
        let new = vec![1, 4, 3];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 4);

        assert!(changes[0].delete().is_some());
        assert!(changes[1].r#move().is_some());
        assert!(changes[2].insert().is_some());
        assert!(changes[3].r#move().is_some());
    }

    #[test]
    fn replace_insert_replace_delete() {
        let old = vec![1, 3, 0 ,2];
        let new = vec![0, 2, 3, 1];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 4);

        assert!(changes[0].r#move().is_some());
        assert!(changes[1].r#move().is_some());
        assert!(changes[2].r#move().is_some());
        assert!(changes[3].r#move().is_some());
    }

    #[test]
    fn replace_move_replace() {
        let old = vec![2, 0, 1, 3];
        let new = vec![1, 3, 0, 2];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 4);

        assert!(changes[0].r#move().is_some());
        assert!(changes[1].r#move().is_some());
        assert!(changes[2].r#move().is_some());
        assert!(changes[3].r#move().is_some());
    }

    #[test]
    fn delete_until_one() {
        let old = vec!["a", "b", "c"];
        let new = vec!["a"];
        let changes = diff(&old, &new);
        assert_eq!(changes.len(), 2);

        assert_eq!(changes[0].delete().unwrap().item, &"b");
        assert_eq!(changes[0].delete().unwrap().index, 1);

        assert_eq!(changes[1].delete().unwrap().item, &"c");
        assert_eq!(changes[1].delete().unwrap().index, 2);
    }
}

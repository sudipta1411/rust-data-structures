use std::cmp::Ordering;

use super::store::{
    ARRAY_LIMIT, ArrayStore, BITMAP_BYTES, BITMAP_WORDS, BitmapStore, Run, RunStore, Store,
    values_to_runs,
};

#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Union,
    Intersection,
    Difference,
    SymmetricDifference,
}

pub(crate) fn combine(left: &Store, right: &Store, op: Operation) -> Store {
    match (left, right) {
        (Store::Array(a), Store::Array(b)) => combine_arrays(a, b, op),
        (Store::Bitmap(a), Store::Bitmap(b)) => combine_bitmaps(a, b, op),
        (Store::Run(a), Store::Run(b)) => combine_runs(a, b, op),
        _ => combine_general(left, right, op),
    }
}

fn combine_arrays(left: &ArrayStore, right: &ArrayStore, op: Operation) -> Store {
    let (left, right) = (left.values(), right.values());
    let mut out = Vec::with_capacity(match op {
        Operation::Union | Operation::SymmetricDifference => left.len() + right.len(),
        Operation::Intersection => left.len().min(right.len()),
        Operation::Difference => left.len(),
    });
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            Ordering::Less => {
                if matches!(
                    op,
                    Operation::Union | Operation::Difference | Operation::SymmetricDifference
                ) {
                    out.push(left[i])
                }
                i += 1
            }
            Ordering::Greater => {
                if matches!(op, Operation::Union | Operation::SymmetricDifference) {
                    out.push(right[j])
                }
                j += 1
            }
            Ordering::Equal => {
                if matches!(op, Operation::Union | Operation::Intersection) {
                    out.push(left[i])
                }
                i += 1;
                j += 1
            }
        }
    }
    if matches!(
        op,
        Operation::Union | Operation::Difference | Operation::SymmetricDifference
    ) {
        out.extend_from_slice(&left[i..]);
    }
    if matches!(op, Operation::Union | Operation::SymmetricDifference) {
        out.extend_from_slice(&right[j..]);
    }
    choose_values(out)
}

fn combine_bitmaps(left: &BitmapStore, right: &BitmapStore, op: Operation) -> Store {
    let mut words = Box::new([0; BITMAP_WORDS]);
    for (i, out) in words.iter_mut().enumerate() {
        *out = match op {
            Operation::Union => left.words()[i] | right.words()[i],
            Operation::Intersection => left.words()[i] & right.words()[i],
            Operation::Difference => left.words()[i] & !right.words()[i],
            Operation::SymmetricDifference => left.words()[i] ^ right.words()[i],
        };
    }
    choose_bitmap(BitmapStore::from_words(words))
}

fn combine_general(left: &Store, right: &Store, op: Operation) -> Store {
    combine_bitmaps(
        &left.clone().into_bitmap_store(),
        &right.clone().into_bitmap_store(),
        op,
    )
}

fn combine_runs(left: &RunStore, right: &RunStore, op: Operation) -> Store {
    let runs = match op {
        Operation::Union => union_runs(left, right),
        Operation::Intersection => intersect_runs(left, right),
        Operation::Difference => difference_runs(left, right),
        Operation::SymmetricDifference => {
            let a = RunStore::from_runs(difference_runs(left, right));
            let b = RunStore::from_runs(difference_runs(right, left));
            union_runs(&a, &b)
        }
    };
    choose_runs(runs)
}

fn intersect_runs(left: &RunStore, right: &RunStore) -> Vec<Run> {
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < left.runs().len() && j < right.runs().len() {
        let (a, b) = (left.runs()[i], right.runs()[j]);
        let (start, end) = (a.start.max(b.start), a.end.min(b.end));
        if start <= end {
            out.push(Run::new(start, end));
        }
        match a.end.cmp(&b.end) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                i += 1;
                j += 1
            }
        }
    }
    out
}

fn union_runs(left: &RunStore, right: &RunStore) -> Vec<Run> {
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::with_capacity(left.run_count() + right.run_count());
    while i < left.runs().len() || j < right.runs().len() {
        let next = if j == right.runs().len()
            || (i < left.runs().len() && left.runs()[i].start <= right.runs()[j].start)
        {
            let r = left.runs()[i];
            i += 1;
            r
        } else {
            let r = right.runs()[j];
            j += 1;
            r
        };
        push_merged(&mut out, next);
    }
    out
}

fn push_merged(out: &mut Vec<Run>, run: Run) {
    let Some(last) = out.last_mut() else {
        out.push(run);
        return;
    };
    if u32::from(run.start) <= u32::from(last.end) + 1 {
        last.end = last.end.max(run.end)
    } else {
        out.push(run)
    }
}

fn difference_runs(left: &RunStore, right: &RunStore) -> Vec<Run> {
    let mut out = Vec::new();
    let mut right_index = 0;
    for l in left.runs().iter().copied() {
        let mut cursor = u32::from(l.start);
        let end = u32::from(l.end);
        while right_index < right.runs().len() && u32::from(right.runs()[right_index].end) < cursor
        {
            right_index += 1
        }
        let mut scan = right_index;
        while scan < right.runs().len() {
            let r = right.runs()[scan];
            let (rs, re) = (u32::from(r.start), u32::from(r.end));
            if rs > end {
                break;
            }
            if rs > cursor {
                out.push(Run::new(cursor as u16, (rs - 1).min(end) as u16));
            }
            if re >= end {
                cursor = end + 1;
                break;
            }
            cursor = cursor.max(re + 1);
            scan += 1;
        }
        if cursor <= end {
            out.push(Run::new(cursor as u16, end as u16));
        }
    }
    out
}

fn choose_values(values: Vec<u16>) -> Store {
    if values.is_empty() {
        return Store::Array(ArrayStore::new());
    }
    let card = values.len() as u32;
    let runs = values_to_runs(values.iter().copied());
    let run_bytes = 2 + runs.len() * 4;
    let array_bytes = values.len() * 2;
    if run_bytes < array_bytes.min(BITMAP_BYTES) {
        Store::Run(RunStore::from_runs(runs))
    } else if card <= ARRAY_LIMIT {
        Store::Array(ArrayStore::from_sorted_values(values))
    } else {
        Store::Array(ArrayStore::from_sorted_values(values)).into_bitmap()
    }
}

fn choose_bitmap(bitmap: BitmapStore) -> Store {
    if bitmap.len() == 0 {
        return Store::Array(ArrayStore::new());
    }
    if bitmap.len() <= ARRAY_LIMIT {
        return choose_values(bitmap.iter().collect());
    }
    let runs = values_to_runs(bitmap.iter());
    if 2 + runs.len() * 4 < BITMAP_BYTES {
        Store::Run(RunStore::from_runs(runs))
    } else {
        Store::Bitmap(bitmap)
    }
}

fn choose_runs(runs: Vec<Run>) -> Store {
    if runs.is_empty() {
        return Store::Array(ArrayStore::new());
    }
    let card: u32 = runs.iter().map(|r| r.len()).sum();
    let run_bytes = 2 + runs.len() * 4;
    let array_bytes = card as usize * 2;
    if run_bytes < array_bytes.min(BITMAP_BYTES) {
        Store::Run(RunStore::from_runs(runs))
    } else if card <= ARRAY_LIMIT {
        Store::Array(ArrayStore::from_sorted_values(
            runs.iter().flat_map(|r| r.start..=r.end).collect(),
        ))
    } else {
        let mut bitmap = BitmapStore::new();
        for r in runs {
            bitmap.insert_range(r.start, r.end);
        }
        Store::Bitmap(bitmap)
    }
}

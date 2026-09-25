use super::store::{
    ARRAY_LIMIT, ArrayStore, BITMAP_BYTES, BITMAP_WORDS, BitmapStore, RunStore, Store,
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
    let a = left.clone().into_bitmap_store();
    let b = right.clone().into_bitmap_store();
    let mut words = Box::new([0; BITMAP_WORDS]);
    for (i, out) in words.iter_mut().enumerate() {
        *out = match op {
            Operation::Union => a.words()[i] | b.words()[i],
            Operation::Intersection => a.words()[i] & b.words()[i],
            Operation::Difference => a.words()[i] & !b.words()[i],
            Operation::SymmetricDifference => a.words()[i] ^ b.words()[i],
        };
    }
    choose(BitmapStore::from_words(words))
}

fn choose(bitmap: BitmapStore) -> Store {
    let card = bitmap.len();
    if card == 0 {
        return Store::Array(ArrayStore::new());
    }
    let runs = values_to_runs(bitmap.iter());
    let run_bytes = 2 + runs.len() * 4;
    let array_bytes = card as usize * 2;
    if run_bytes < array_bytes.min(BITMAP_BYTES) {
        Store::Run(RunStore::from_runs(runs))
    } else if card <= ARRAY_LIMIT {
        Store::Array(bitmap.into_array())
    } else {
        Store::Bitmap(bitmap)
    }
}

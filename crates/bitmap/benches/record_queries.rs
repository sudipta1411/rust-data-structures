use std::hint::black_box;
use std::time::Duration;

use bitmap::{ewah::EwahBitmap, roaring::RoaringBitmap};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

const COUNTRIES: usize = 32;
const PLANS: usize = 4;

#[derive(Clone, Copy)]
struct Record {
    country: usize,
    plan: usize,
    active: bool,
    payload: u64,
}

fn record_count() -> usize {
    std::env::var("BITMAP_BENCH_RECORDS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1000000)
}

fn generate_records(count: usize) -> Vec<Record> {
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    (0..count)
        .map(|id| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            Record {
                country: (state as usize) & (COUNTRIES - 1),
                plan: ((state >> 8) as usize) & (PLANS - 1),
                active: ((state >> 16) & 1) != 0,
                payload: state ^ id as u64,
            }
        })
        .collect()
}

struct RoaringIndexes {
    countries: Vec<RoaringBitmap>,
    plans: Vec<RoaringBitmap>,
    active: RoaringBitmap,
}

fn build_roaring(records: &[Record]) -> RoaringIndexes {
    let mut indexes = RoaringIndexes {
        countries: (0..COUNTRIES).map(|_| RoaringBitmap::new()).collect(),
        plans: (0..PLANS).map(|_| RoaringBitmap::new()).collect(),
        active: RoaringBitmap::new(),
    };

    for (id, record) in records.iter().enumerate() {
        let id = u32::try_from(id).expect("record count exceeds the roaring u32 ID limit");
        indexes.countries[record.country].insert(id);
        indexes.plans[record.plan].insert(id);
        if record.active {
            indexes.active.insert(id);
        }
    }

    indexes
}

fn set_bit(words: &mut [u64], id: usize) {
    let word_index = id / 64;
    let bit_index = id % 64;

    words[word_index] |= 1u64 << bit_index;
}

struct EwahIndexes {
    countries: Vec<EwahBitmap<u64>>,
    plans: Vec<EwahBitmap<u64>>,
    active: EwahBitmap<u64>,
}

fn build_ewah(records: &[Record]) -> EwahIndexes {
    let word_count = records.len().div_ceil(64);

    let mut countries = vec![vec![0u64; word_count]; COUNTRIES];

    let mut plans = vec![vec![0u64; word_count]; PLANS];

    let mut active = vec![0u64; word_count];

    for (id, record) in records.iter().enumerate() {
        set_bit(&mut countries[record.country], id);
        set_bit(&mut plans[record.plan], id);

        if record.active {
            set_bit(&mut active, id);
        }
    }

    EwahIndexes {
        countries: countries
            .iter()
            .map(|words| EwahBitmap::from_words(words))
            .collect(),

        plans: plans
            .iter()
            .map(|words| EwahBitmap::from_words(words))
            .collect(),

        active: EwahBitmap::from_words(&active),
    }
}

// country = 7 AND plan = 2 AND active = true
fn roaring_and(indexes: &RoaringIndexes) -> RoaringBitmap {
    &(&indexes.countries[7] & &indexes.plans[2]) & &indexes.active
}

fn ewah_and(indexes: &EwahIndexes) -> EwahBitmap<u64> {
    indexes.countries[7]
        .and(&indexes.plans[2])
        .unwrap()
        .and(&indexes.active)
        .unwrap()
}

// country IN (3, 7, 21)
fn roaring_or(indexes: &RoaringIndexes) -> RoaringBitmap {
    &(&indexes.countries[3] | &indexes.countries[7]) | &indexes.countries[21]
}

fn ewah_or(indexes: &EwahIndexes) -> EwahBitmap<u64> {
    indexes.countries[3]
        .or(&indexes.countries[7])
        .unwrap()
        .or(&indexes.countries[21])
        .unwrap()
}

fn materialize_roaring(result: &RoaringBitmap, records: &[Record]) -> u64 {
    result
        .iter()
        .map(|id| records[id as usize].payload)
        .fold(0, u64::wrapping_add)
}

fn materialize_ewah(result: &EwahBitmap<u64>, records: &[Record]) -> u64 {
    let mut total = 0u64;

    for (word_index, mut word) in result.to_words().into_iter().enumerate() {
        while word != 0 {
            let bit_index = word.trailing_zeros() as usize;
            let id = word_index * 64 + bit_index;

            if let Some(record) = records.get(id) {
                total = total.wrapping_add(record.payload);
            }

            // Clear the lowest set bit.
            word &= word - 1;
        }
    }

    total
}

fn ewah_ids(bitmap: &EwahBitmap<u64>) -> Vec<u32> {
    let mut ids = Vec::new();

    for (word_index, mut word) in bitmap.to_words().into_iter().enumerate() {
        while word != 0 {
            let bit = word.trailing_zeros() as usize;
            ids.push((word_index * 64 + bit) as u32);
            word &= word - 1;
        }
    }

    ids
}

fn scan_and_ids(records: &[Record]) -> Vec<u32> {
    records
        .iter()
        .enumerate()
        .filter(|(_, record)| record.country == 7 && record.plan == 2 && record.active)
        .map(|(id, _)| u32::try_from(id).expect("record ID exceeds u32::MAX"))
        .collect()
}

fn scan_or_ids(records: &[Record]) -> Vec<u32> {
    records
        .iter()
        .enumerate()
        .filter(|(_, record)| matches!(record.country, 3 | 7 | 21))
        .map(|(id, _)| u32::try_from(id).expect("record ID exceeds u32::MAX"))
        .collect()
}

fn scan_and_sum(records: &[Record]) -> u64 {
    records
        .iter()
        .filter(|record| record.country == 7 && record.plan == 2 && record.active)
        .map(|record| record.payload)
        .fold(0, u64::wrapping_add)
}

fn benchmark_queries(c: &mut Criterion) {
    let count = record_count();
    let records = generate_records(count);

    let roaring = build_roaring(&records);
    let ewah = build_ewah(&records);

    let roaring_and_result = roaring_and(&roaring);
    let ewah_and_result = ewah_and(&ewah);
    let roaring_or_result = roaring_or(&roaring);
    let ewah_or_result = ewah_or(&ewah);

    let expected_and = scan_and_ids(&records);
    let expected_or = scan_or_ids(&records);
    assert_eq!(roaring_and_result.iter().collect::<Vec<_>>(), expected_and);
    assert_eq!(ewah_ids(&ewah_and_result), expected_and);
    assert_eq!(roaring_or_result.iter().collect::<Vec<_>>(), expected_or);
    assert_eq!(ewah_ids(&ewah_or_result), expected_or);

    let expected_sum = scan_and_sum(&records);
    assert_eq!(
        materialize_roaring(&roaring_and_result, &records),
        expected_sum
    );
    assert_eq!(materialize_ewah(&ewah_and_result, &records), expected_sum);

    let mut query_group = c.benchmark_group(format!("record_queries/{count}_records"));
    query_group.throughput(Throughput::Elements(count as u64));

    query_group.bench_function(BenchmarkId::new("and_3_predicates", "roaring"), |b| {
        b.iter(|| black_box(roaring_and(black_box(&roaring))))
    });

    query_group.bench_function(BenchmarkId::new("and_3_predicates", "ewah"), |b| {
        b.iter(|| black_box(ewah_and(black_box(&ewah))))
    });

    query_group.bench_function(BenchmarkId::new("or_3_values", "roaring"), |b| {
        b.iter(|| black_box(roaring_or(black_box(&roaring))))
    });

    query_group.bench_function(BenchmarkId::new("or_3_values", "ewah"), |b| {
        b.iter(|| black_box(ewah_or(black_box(&ewah))))
    });
    query_group.finish();

    let mut materialize_group =
        c.benchmark_group(format!("record_materialization/{count}_records"));
    materialize_group.throughput(Throughput::Elements(roaring_and_result.len()));

    materialize_group.bench_function(BenchmarkId::new("materialize_and", "roaring"), |b| {
        b.iter(|| {
            black_box(materialize_roaring(
                black_box(&roaring_and_result),
                black_box(&records),
            ))
        })
    });

    materialize_group.bench_function(BenchmarkId::new("decode_and_materialize", "ewah"), |b| {
        b.iter(|| {
            black_box(materialize_ewah(
                black_box(&ewah_and_result),
                black_box(&records),
            ))
        })
    });
    materialize_group.finish();

    let mut end_to_end_group = c.benchmark_group(format!("end_to_end_and/{count}_records"));
    end_to_end_group.throughput(Throughput::Elements(count as u64));

    end_to_end_group.bench_function("linear_scan", |b| {
        b.iter(|| black_box(scan_and_sum(black_box(&records))))
    });

    end_to_end_group.bench_function("roaring", |b| {
        b.iter(|| {
            let result = roaring_and(black_box(&roaring));
            black_box(materialize_roaring(&result, black_box(&records)))
        })
    });

    end_to_end_group.bench_function("ewah", |b| {
        b.iter(|| {
            let result = ewah_and(black_box(&ewah));
            black_box(materialize_ewah(&result, black_box(&records)))
        })
    });

    end_to_end_group.finish();
}

fn benchmark_index_construction(c: &mut Criterion) {
    let count = record_count();
    let records = generate_records(count);
    let mut group = c.benchmark_group(format!("index_construction/{count}_records"));
    group.throughput(Throughput::Elements(count as u64));

    group.bench_function("roaring", |b| {
        b.iter(|| black_box(build_roaring(black_box(&records))))
    });

    group.bench_function("ewah", |b| {
        b.iter(|| black_box(build_ewah(black_box(&records))))
    });

    group.finish();
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(8))
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = benchmark_queries, benchmark_index_construction
}

criterion_main!(benches);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Run {
    pub(crate) start: u16,
    pub(crate) end: u16,
}

impl Run {
    pub(crate) fn new(start: u16, end: u16) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }

    pub(crate) fn len(self) -> u32 {
        u32::from(self.end) - u32::from(self.start) + 1
    }

    pub(crate) fn contains(self, value: u16) -> bool {
        self.start <= value && value <= self.end
    }
}

fn runs_are_normalized(runs: &[Run]) -> bool {
    runs.iter().all(|run| run.start <= run.end)
        && runs
            .windows(2)
            .all(|pair| u32::from(pair[0].end) + 1 < u32::from(pair[1].start))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RunStore {
    runs: Vec<Run>,
    cardinality: u32,
}

impl RunStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn from_range(start: u16, end: u16) -> Self {
        let run = Run::new(start, end);

        Self {
            runs: vec![run],
            cardinality: run.len(),
        }
    }

    pub(crate) fn from_runs(runs: Vec<Run>) -> Self {
        debug_assert!(runs_are_normalized(&runs));

        let cardinality = runs.iter().map(|run| run.len()).sum();

        Self { runs, cardinality }
    }

    pub(crate) fn len(&self) -> u32 {
        self.cardinality
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.cardinality == 0
    }

    pub(crate) fn run_count(&self) -> usize {
        self.runs.len()
    }

    pub(crate) fn runs(&self) -> &[Run] {
        &self.runs
    }

    pub(crate) fn contains(&self, value: u16) -> bool {
        self.runs
            .binary_search_by(|run| {
                if value < run.start {
                    std::cmp::Ordering::Greater
                } else if value > run.end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .is_ok()
    }

    pub(crate) fn insert(&mut self, value: u16) -> bool {
        self.insert_range(value, value) == 1
    }

    pub(crate) fn remove(&mut self, value: u16) -> bool {
        self.remove_range(value, value) == 1
    }

    pub(crate) fn insert_range(&mut self, start: u16, end: u16) -> u32 {
        assert!(start <= end);

        let mut merged_start = u32::from(start);
        let mut merged_end = u32::from(end);
        let requested = merged_end - merged_start + 1;
        let mut overlap = 0u32;

        let mut output = Vec::with_capacity(self.runs.len() + 1);
        let mut inserted = false;

        for run in self.runs().iter().copied() {
            let run_start = u32::from(run.start);
            let run_end = u32::from(run.end);

            if run_end + 1 < merged_start {
                output.push(run);
                continue;
            }
            if merged_end + 1 < run_start {
                if !inserted {
                    output.push(Run::new(merged_start as u16, merged_end as u16));
                    inserted = true;
                }
                output.push(run);
                continue;
            }
            let intersection_start = merged_start.max(run_start);
            let intersection_end = merged_end.min(run_end);
            if intersection_start <= intersection_end {
                overlap += intersection_end - intersection_start + 1;
            }
            merged_start = merged_start.min(run_start);
            merged_end = merged_end.max(run_end);
        }
        if !inserted {
            output.push(Run::new(merged_start as u16, merged_end as u16));
        }
        let added = requested - overlap;
        self.runs = output;
        self.cardinality += added;
        added
    }

    pub(crate) fn remove_range(&mut self, start: u16, end: u16) -> u32 {
        assert!(start <= end);
        let remove_start = u32::from(start);
        let remove_end = u32::from(end);
        let mut removed = 0u32;
        let mut output = Vec::with_capacity(self.runs.len() + 1);
        for run in self.runs.iter().copied() {
            let run_start = u32::from(run.start);
            let run_end = u32::from(run.end);

            if run_end < remove_start || run_start > remove_end {
                output.push(run);
                continue;
            }
            let overlap_start = run_start.max(remove_start);
            let overlap_end = run_end.min(remove_end);
            removed += overlap_end - overlap_start + 1;
            if run_start < remove_start {
                output.push(Run::new(run.start, (remove_start - 1) as u16));
            }
            if run_end > remove_end {
                output.push(Run::new((remove_end + 1) as u16, run.end));
            }
        }
        self.runs = output;
        self.cardinality -= removed;

        removed
    }

    pub(crate) fn iter(&self) -> RunIter<'_> {
        RunIter {
            runs: &self.runs,
            run_index: 0,
            current: self.runs.first().map(|run| run.start),
            remaining: self.cardinality as usize,
        }
    }

    pub(crate) fn validate(&self) -> bool {
        runs_are_normalized(&self.runs)
            && self.cardinality == self.runs.iter().map(|run| run.len()).sum()
    }
}

pub(crate) struct RunIter<'a> {
    runs: &'a [Run],
    run_index: usize,
    current: Option<u16>,
    remaining: usize,
}

impl Iterator for RunIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.current?;
        let run = self.runs[self.run_index];

        self.remaining -= 1;
        if value < run.end {
            self.current = Some(value + 1);
        } else {
            self.run_index += 1;
            self.current = self.runs.get(self.run_index).map(|next| next.start);
        }
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for RunIter<'_> {
    fn len(&self) -> usize {
        self.remaining
    }
}

impl std::iter::FusedIterator for RunIter<'_> {}

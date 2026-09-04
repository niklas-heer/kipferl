use std::fs;
use std::hint::black_box;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kipferl_format::Trailer;
use kipferl_loader::{inspect, prepare_path};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    bytes: Vec<u8>,
    file: PathBuf,
    cache: PathBuf,
}

impl Fixture {
    #[expect(
        clippy::expect_used,
        reason = "Invalid fixture sizes or filesystem failures must abort benchmark setup, not produce misleading timings"
    )]
    fn new(payload_size: usize) -> Self {
        let root = std::env::temp_dir().join(format!(
            "kipferl-loader-bench-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("create benchmark directory");
        let size = u64::try_from(payload_size).expect("benchmark fixture size fits u64");
        let trailer = Trailer {
            runtime_offset: 1,
            runtime_size: size,
            python_offset: size.checked_add(1).expect("benchmark offset fits u64"),
            python_size: size,
        };
        let bytes = [
            b"L".as_slice(),
            &vec![42; payload_size],
            &vec![43; payload_size],
            &trailer.encode(),
        ]
        .concat();
        let file = root.join("app");
        let cache = root.join("cache");
        fs::write(&file, &bytes).expect("write benchmark fixture");
        prepare_path(&file, &cache).expect("warm benchmark cache");
        Self {
            root,
            bytes,
            file,
            cache,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[expect(
    clippy::expect_used,
    reason = "Each measured operation must succeed on the validated fixture; failures invalidate benchmark results"
)]
fn loader(c: &mut Criterion) {
    let fixtures = [Fixture::new(1024), Fixture::new(1024 * 1024)];
    {
        let mut inspect_group = c.benchmark_group("loader/inspect");
        for fixture in &fixtures {
            let file_size = u64::try_from(fixture.bytes.len()).expect("fixture length fits u64");
            // Inspection samples bounded bytes regardless of total bundle size.
            inspect_group.bench_with_input(
                BenchmarkId::from_parameter(file_size),
                fixture,
                |b, f| {
                    b.iter(|| {
                        inspect(&mut Cursor::new(black_box(&f.bytes)), file_size)
                            .expect("valid fixture")
                    });
                },
            );
        }
        inspect_group.finish();
    }
    {
        let mut cache_group = c.benchmark_group("loader/cache_hit");
        for fixture in &fixtures {
            let file_size = u64::try_from(fixture.bytes.len()).expect("fixture length fits u64");
            cache_group.throughput(Throughput::Bytes(file_size));
            cache_group.bench_with_input(
                BenchmarkId::from_parameter(file_size),
                fixture,
                |b, f| {
                    b.iter(|| {
                        let prepared = prepare_path(black_box(&f.file), black_box(&f.cache))
                            .expect("valid cached fixture");
                        assert!(prepared.cache_hit);
                        black_box(prepared)
                    });
                },
            );
        }
        cache_group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(30).warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(2));
    targets = loader
}
criterion_main!(benches);

# rectiq-cli

Criterion benches are available for end-to-end CLI runs and path discovery.

## Datasets
- **S**: ~50 files, 1–5 KB
- **M**: ~500 files, 20–80 KB
- **L**: ~2000 files, 0.5–2 MB

## Running benches
```
RECTIQ_SILENT=1 cargo bench -p rectiq-cli -- --sample-size 10
```

p50 and p95 are reported in the Criterion summary for each benchmark.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use zvec_rust::*;
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let repo = fs::canonicalize(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.."))?;
    let run = repo.join(format!(
        "tests/.runtime/zvec-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
    ));
    fs::create_dir_all(&run)?;
    std::env::set_current_dir(&run)?;
    let bytes = fs::read(repo.join("tests/fixtures/zvec-snapshot.json"))?;
    let snapshot = format!("{:x}", Sha256::digest(&bytes));
    let fixture: Value = serde_json::from_slice(&bytes)?;
    fs::write(run.join("manifest.json"), &bytes)?;
    initialize(Some(&ConfigBuilder::new().num_threads(1)))?;
    let schema = CollectionSchema::builder("snapshot")
        .add_field(FieldSchema::new("body", DataType::String, false, 0)?)
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            2,
            IndexParams::flat(MetricType::L2)?,
        )
        .build()?;
    let path = run.join(&snapshot);
    let collection = Collection::create_and_open(path.to_str().unwrap(), &schema, None)?;
    for row in fixture["rows"].as_array().unwrap() {
        let mut doc = Doc::new()?;
        doc.set_pk(&format!(
            "{}@{}",
            row["key"].as_str().unwrap(),
            row["revision"]
        ));
        doc.add_string("body", row["body"].as_str().unwrap())?;
        doc.add_vector_f32(
            "embedding",
            &row["vector"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect::<Vec<_>>(),
        )?;
        let result = collection.insert(&[&doc])?;
        assert_eq!(result.error_count, 0);
        assert_eq!(result.success_count, 1);
    }
    collection.flush()?;
    collection.close()?;
    // A pending generation is built separately while the ready snapshot stays readable.
    let pending_path = run.join("pending-generation");
    let pending = Collection::create_and_open(pending_path.to_str().unwrap(), &schema, None)?;
    let mut replacement = Doc::new()?;
    replacement.set_pk("a@2");
    replacement.add_string("body", "unpublished replacement")?;
    replacement.add_vector_f32("embedding", &[0.0, 1.0])?;
    assert_eq!(pending.insert(&[&replacement])?.success_count, 1);
    pending.flush()?;
    let mut options = CollectionOptions::new()?;
    options.set_read_only(true)?;
    let query_vector = [1.0_f32, 0.0];
    // Retrieve all rows before tie-breaking and limiting; top-K alone can lose tied rows.
    let query = SearchQuery::new("embedding", &query_vector, 3)?;
    let expected = vec![
        ("a@1".to_owned(), 0.0_f32),
        ("b@1".to_owned(), 2.0),
        ("c@2".to_owned(), 2.0),
    ];
    for _ in 0..3 {
        let frozen = Collection::open(path.to_str().unwrap(), Some(&options))?;
        for _ in 0..100 {
            let mut results: Vec<_> = frozen
                .query(&query)?
                .iter()
                .map(|d| (d.get_pk().unwrap().to_owned(), d.get_score()))
                .collect();
            assert!(results.iter().all(|(_, score)| score.is_finite()));
            results.retain(|(_, score)| *score <= 2.0);
            results.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
            assert_eq!(results, expected);
            assert!(frozen.fetch(&["a@2"])?.is_empty());
            let old = frozen.fetch(&["a@1"])?;
            assert_eq!(
                old[0].get_string("body")?.as_deref(),
                Some("alpha revision one")
            );
            let full = frozen.fetch(&["c@2"])?;
            assert_eq!(
                full[0].get_string("body")?.as_deref(),
                Some("gamma revision two")
            );
        }
        frozen.close()?;
    }
    let evidence = json!({"snapshot":snapshot,"native_version":version(),"sdk":"0.7.1","repeated_queries":300,"read_only_opens":3,"ordered_results":expected});
    fs::write(
        run.join("result.json"),
        serde_json::to_vec_pretty(&evidence)?,
    )?;
    println!("{}\n{}", run.display(), evidence);
    Ok(())
}

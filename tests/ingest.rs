use ctx::{chunking::chunk_text, input::read_input};

#[tokio::test]
async fn read_input_requires_exactly_one_source() {
    let result = read_input(None, None, false).await;
    assert!(result.is_err());
}

#[test]
fn chunker_rejects_overlap_equal_to_size() {
    let result = chunk_text("hello", 10, 10);
    assert!(result.is_err());
}

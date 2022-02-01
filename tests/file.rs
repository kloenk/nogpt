#[test]
fn file() -> Result<(), nogpt::GptError> {
    let block = nogpt::std::BlockFile::open(&"tests/fixtures/gpt-linux-disk-01.img")?;

    let gpt = nogpt::Gpt::open(block)?;

    //println!("gpt: {:?}", gpt.num_parts);

    Ok(())
}

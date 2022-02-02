use nogpt::part::GptPartHeader;
use nogpt::GptRepair;

#[cfg(feature = "std")]
#[test]
fn file() -> Result<(), nogpt::GptError> {
    let block = nogpt::std::BlockFile::open(&"tests/fixtures/gpt-linux-disk-01.img")?;

    let gpt = nogpt::Gpt::open(block).fail()?;

    let part: GptPartHeader = gpt.get_partition(0)?;

    println!("part[0]: {:?}", part);

    Ok(())
}

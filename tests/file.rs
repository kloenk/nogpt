use block_device::BlockDevice;
use nogpt::mbr::MasterBootRecord;
use nogpt::part::GPTPartHeader;
use nogpt::{GPTError, GptRepair};

#[cfg(feature = "std")]
use nogpt::std::BlockFile;

#[cfg(feature = "std")]
#[test]
fn file() -> Result<(), GPTError> {
    let block = open_512_file()?;

    let gpt = nogpt::GPT::open(block).fail()?;

    let part: GPTPartHeader = gpt.get_partition(0)?;

    println!("part[0]: {:?}", part);

    Ok(())
}

#[cfg(feature = "std")]
#[test]
fn read_mbr() -> Result<(), GPTError> {
    let block = open_512_file()?;

    let mut buf = [0u8; 512];

    block.read(&mut buf, 0, 1)?;

    let mbr = unsafe { MasterBootRecord::from_buf(&buf) }?;

    assert_eq!(mbr.signature(), 0xaa55);

    assert!(!mbr.partition[0].is_empty());
    assert!(mbr.partition[1].is_empty());
    assert!(mbr.partition[2].is_empty());
    assert!(mbr.partition[3].is_empty());

    mbr.verify(None)?;
    mbr.verify(Some(96))?;
    mbr.verify(Some(0))
        .expect_err("MBR should error ouf if we have 0 lba's");

    Ok(())
}

#[cfg(feature = "std")]
fn open_512_file() -> Result<BlockFile, GPTError> {
    Ok(nogpt::std::BlockFile::open(
        &"tests/fixtures/gpt-linux-disk-01.img",
    )?)
}

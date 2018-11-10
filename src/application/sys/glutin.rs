use utils::time::Timestamp;

pub fn timestamp() -> Timestamp {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();

    let ms = duration.subsec_millis() as u64 + duration.as_secs() * 1000;
    Timestamp::from_millis(ms)
}

pub(crate) fn init() {}

pub(crate) fn run_forever<T>(mut func: T) -> Result<(), failure::Error>
where
    T: 'static + FnMut() -> Result<bool, failure::Error>,
{
    while func()? {}
    Ok(())
}

use utils::time::Timestamp;

pub fn timestamp() -> Timestamp {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();

    let ms = duration.subsec_millis() as u64 + duration.as_secs() * 1000;
    Timestamp::from_millis(ms)
}

pub(crate) fn init() {}

pub(crate) fn run_forever<F, F2>(mut advance: F, mut finished: F2) -> Result<(), failure::Error>
where
    F: FnMut() -> Result<bool, failure::Error> + 'static,
    F2: FnMut() -> Result<(), failure::Error> + 'static,
{
    while advance()? {}
    finished()
}

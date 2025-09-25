pub fn init_logging() {
    let _ = env_logger::builder().format_timestamp(None).try_init();
}
#[derive(Debug)]
pub struct InputPlugin {

    // config_name = "input"  Read it from plugin impl
    // type: String
    // codec
    // tags = []
    // add_field
    // plugin_type = input
    // threadable = false
    // stop_called = Concurrent::AtomicBoolean.new(false)

    // init(params: Hash)
    //     self.params = params

    // to override:
    // register()  - must be overriden
    // run()       - must be overriden
    // stop()      - optional (extra work on stopping)

    // API:
    // add_tag()
    // do_stop()
    // is_stopped() -> stop_called.value
    // decorate()
    // fix_streaming_codecs() ?
    // threads_count() -> u32 if threadable

    // From Plugin parent:
    // id()
    // do_close()
    // close()
}

impl Clone for InputPlugin {
    fn clone(&self) -> InputPlugin {
        InputPlugin {}
    }
}

impl InputPlugin {
    pub fn register(&mut self) {

    }

    pub fn run(&mut self) {

    }

    pub fn threads_count(&self) -> usize {
        1  // TODO: read it from the plugin impl
    }
}
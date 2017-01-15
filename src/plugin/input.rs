pub struct InputPlugin {
    // config_name = "input"
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

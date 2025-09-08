use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

pub fn get_subscriber(name: String, level: String) -> impl Subscriber + Send + Sync {
    // 日志过滤层
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    // 格式化层
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    // 创建订阅者
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // 初始化 log 到 tracing 的日志桥接器，将传统的 log 日志记录重定向到 tracing，确保使用 log crate 的第三方库的日志也能被 tracing 捕获
    LogTracer::init().expect("设置 Logger 失败");
    set_global_default(subscriber).expect("设置 subscriber 失败");
}

use cfg_if::cfg_if;
use worker::{console_log, Date, Request};

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().and_then(|cf| cf.coordinates()).unwrap_or_default(),
        req.cf()
            .and_then(|cf| cf.region())
            .unwrap_or_else(|| "unknown region".into())
    );
}

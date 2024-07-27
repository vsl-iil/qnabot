pub mod logging {
    use std::error::Error;

    pub fn non_fatal<T: Error>(error: T) {
        eprintln!("{error}");
    }

    pub fn fatal<T: Error>(error: T) {
        non_fatal(error);
        panic!("Произошла фатальная ошибка!");
    }

    pub fn info(message: &str) {
        eprintln!("{message}");
    }

    pub fn check_result<T, E: Error>(result: Result<T,E>, log_func: fn(E)) {
        if let Err(e) = result {
            log_func(e)
        }
    }

    pub fn check_pass<T, E: Error>(result: Result<T,E>, log_func: fn(E)) -> Option<T> {
        match result {
            Ok(res) => Some(res),
            Err(e)  => {
                log_func(e);
                None 
            }
        }
    }
}

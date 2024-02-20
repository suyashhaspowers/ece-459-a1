use curl::{Error};
use urlencoding::encode;

use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};
use std::collections::HashMap;
use std::time::Duration;
use std::str;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::Packages;

struct Collector(Box<String>);
impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        (*self.0).push_str(str::from_utf8(&data.to_vec()).unwrap());
        Ok(data.len())
    }
}

const DEFAULT_SERVER : &str = "ece459.patricklam.ca:4590";
impl Drop for Packages {
    fn drop(&mut self) {
        self.execute()
    }
}

static EASYKEY_COUNTER: AtomicI32 = AtomicI32::new(0);

pub struct AsyncState {
    server : String,
    easy_key_map: HashMap<i32, (String, String, i32)>,
    urls: Vec<(String, i32)>
}

impl AsyncState {
    pub fn new() -> AsyncState {
        AsyncState {
            server : String::from(DEFAULT_SERVER),
            easy_key_map: HashMap::new(),
            urls: Vec::new()
        }
    }
}

impl Packages {
    pub fn set_server(&mut self, new_server:&str) {
        self.async_state.server = String::from(new_server);
    }

    /// Retrieves the version number of pkg and calls enq_verify_with_version with that version number.
    pub fn enq_verify(&mut self, pkg:&str) {
        let version = self.get_available_debver(pkg);
        match version {
            None => { println!("Error: package {} not defined.", pkg); return },
            Some(v) => { 
                let vs = &v.to_string();
                self.enq_verify_with_version(pkg, vs); 
            }
        };
    }

    /// Enqueues a request for the provided version/package information. Stores any needed state to async_state so that execute() can handle the results and print out needed output.
    pub fn enq_verify_with_version(&mut self, pkg:&str, version:&str) {
        // URL encode the version
        let encoded_version = encode(version);

        // Create the full URL for the request
        let url = format!(
            "http://{}/rest/v1/checksums/{}/{}",
            self.async_state.server, pkg, encoded_version
        );

        // Increment the Easy2Handle counter
        let easy_key = EASYKEY_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Print the queued request
        println!("queueing request {}", url);

        let pkg_number = self.get_package_num_inserting(pkg);

        // Store the Easy2Handle key and store urls
        self.async_state.easy_key_map.insert(easy_key, (pkg.to_string(), version.to_string(), pkg_number));
        self.async_state.urls.push((url, easy_key));
    }

    /// Asks curl to perform all enqueued requests. For requests that succeed with response code 200, compares received MD5sum with local MD5sum (perhaps stored earlier). For requests that fail with 400+, prints error message.
    pub fn execute(&mut self) {
        let mut easys: Vec<(Easy2Handle<Collector>, i32)> = Vec::new();
        let mut multi: Multi = Multi::new();
        let urls = self.async_state.urls.clone();

        multi.pipelining(true, true).unwrap();

        for (url, easy_key) in urls {
            let result = self.init(&multi, url).unwrap();
            easys.push((result, easy_key))
        }

        while multi.perform().unwrap() > 0 {
            multi.wait(&mut [], Duration::from_secs(30)).unwrap();
        }

        // Iterate through all Easy2Handlesr
        for easy in easys.drain(..) {
            let mut handler_after: Easy2<Collector> = multi.remove2(easy.0).unwrap();
            let response_code = handler_after.response_code().unwrap();
            let easy_key = easy.1;
            let (pkg_name, version, pkg_number) = self.async_state.easy_key_map.get(&easy_key).unwrap();
        
            if response_code == 200 {
                // Check the received MD5sum with the local MD5sum
                match self.md5sums.get(&pkg_number) {
                    Some(md5_local) => {
                        let md5_api = handler_after.get_ref().0.as_ref().clone();
                        let match_md5 = &md5_api == md5_local;
                        println!("verifying {}, matches: {:?}", pkg_name, match_md5);
                    } None => {}
                }

            } else {
                // Print error message for response codes 400+
                println!(
                    "got error {} on request for package {} version {}",
                    response_code, pkg_name, version
                );
            }
        }
        // Set urls, multi, easys, map back to empty
        self.async_state.easy_key_map = HashMap::new();
        self.async_state.urls = Vec::new();
    }

    fn init(&mut self, multi:&Multi, url: String) -> Result<Easy2Handle<Collector>, Error> {
        let mut easy = Easy2::new(Collector(Box::new(String::new())));
        easy.url(&url)?;
        easy.verbose(false)?;
        Ok(multi.add2(easy).unwrap())
    }
}

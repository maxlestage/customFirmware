//! Le premier service Binder de customFirmware, écrit en Rust.
//!
//! Il s'enregistre sous le nom `customfirmware.hello` et répond à `greet`.
//! Compilé par AOSP (voir Android.bp), pas par Cargo.

use binder::{BinderFeatures, Interface, Result as BinderResult};
use customfirmware_hello::aidl::customfirmware::hello::IHello::{BnHello, IHello};

/// Le nom sous lequel le service s'enregistre auprès de servicemanager.
const SERVICE_NAME: &str = "customfirmware.hello";

struct Hello;

impl Interface for Hello {}

impl IHello for Hello {
    fn greet(&self, name: &str) -> BinderResult<String> {
        Ok(format!("Bonjour {name}, depuis Rust"))
    }
}

fn main() {
    let service = BnHello::new_binder(Hello, BinderFeatures::default());
    binder::add_service(SERVICE_NAME, service.as_binder())
        .expect("impossible d'enregistrer le service");
    binder::ProcessState::join_thread_pool();
}

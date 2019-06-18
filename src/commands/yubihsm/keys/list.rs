//! List keys inside the YubiHSM2

use abscissa::{Command, Runnable};
use std::process;
use tendermint::PublicKey;

/// The `yubihsm keys list` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct ListCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config", help = "path to tmkms.toml")]
    pub config: Option<String>,
}

impl Runnable for ListCommand {
    /// List all suitable Ed25519 keys in the HSM
    fn run(&self) {
        let hsm = crate::yubihsm::client();

        let serial_number = hsm
            .device_info()
            .unwrap_or_else(|e| {
                status_err!("couldn't get YubiHSM serial number: {}", e);
                process::exit(1);
            })
            .serial_number;

        let objects = hsm.list_objects(&[]).unwrap_or_else(|e| {
            status_err!("couldn't list YubiHSM objects: {}", e);
            process::exit(1);
        });

        let mut keys = objects
            .iter()
            .filter(|o| o.object_type == yubihsm::object::Type::AsymmetricKey)
            .collect::<Vec<_>>();

        keys.sort_by(|k1, k2| k1.object_id.cmp(&k2.object_id));

        if keys.is_empty() {
            status_err!("no keys in this YubiHSM (#{})", serial_number);
            process::exit(0);
        }

        println!("Listing keys in YubiHSM #{}:", serial_number);

        for key in &keys {
            let public_key = hsm.get_public_key(key.object_id).unwrap_or_else(|e| {
                status_err!(
                    "couldn't get public key for asymmetric key #{}: {}",
                    key.object_id,
                    e
                );
                process::exit(1);
            });

            let key_id = format!("- 0x#{:04x}", key.object_id);

            // TODO: support for non-Ed25519 keys
            if public_key.algorithm == yubihsm::asymmetric::Algorithm::Ed25519 {
                status_attr_ok!(
                    key_id,
                    PublicKey::from_raw_ed25519(&public_key.as_ref())
                        .unwrap()
                        .to_hex()
                );
            } else {
                status_attr_err!(key_id, "unsupported algorithm: {:?}", public_key.algorithm);
            }
        }
    }
}

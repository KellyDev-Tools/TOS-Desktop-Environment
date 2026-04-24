use clap::{Parser, Subcommand};
use cryptoki::context::Pkcs11;
use cryptoki::session::UserType;
use cryptoki::slot::Slot;
use cryptoki::object::{Attribute, ObjectClass, AttributeType};
use cryptoki::mechanism::Mechanism;
use secrecy::Secret;
use std::fs::File;
use std::io::{Read, Write};
use sha2::{Sha256, Digest};
use rsa::{RsaPublicKey};

/// Command line arguments for the TOS Release Signing Utility.
#[derive(Parser)]
#[command(name = "tos-signer")]
#[command(about = "TOS Release Signing Utility (PKCS#11 / HSM)", long_about = None)]
struct Cli {
    /// The command to execute.
    #[command(subcommand)]
    command: Commands,
}

/// Supported commands for HSM interaction and signing.
#[derive(Subcommand)]
enum Commands {
    /// Provision a new release key in the HSM.
    Provision {
        /// Path to the PKCS#11 module library.
        #[arg(long, env = "TOS_HSM_MODULE")]
        module: String,
        /// HSM slot ID to use.
        #[arg(long, env = "TOS_HSM_SLOT")]
        slot: u64,
        /// User PIN for the HSM.
        #[arg(long, env = "TOS_HSM_PIN")]
        pin: String,
        /// Label to assign to the generated key pair.
        #[arg(long)]
        label: String,
    },
    /// Sign a file using the HSM.
    Sign {
        /// Path to the PKCS#11 module library.
        #[arg(long, env = "TOS_HSM_MODULE")]
        module: String,
        /// HSM slot ID to use.
        #[arg(long, env = "TOS_HSM_SLOT")]
        slot: u64,
        /// User PIN for the HSM.
        #[arg(long, env = "TOS_HSM_PIN")]
        pin: String,
        /// Label of the private key in the HSM.
        #[arg(long)]
        label: String,
        /// Path to the file to sign.
        file: String,
    },
    /// Verify a file signature using a public key.
    Verify {
        /// Path to the file to verify.
        file: String,
        /// Path to the detached signature file.
        #[arg(long)]
        signature: String,
        /// Path to the public key metadata file (JSON).
        #[arg(long)]
        public_key: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Provision { module, slot, pin, label } => {
            println!("Provisioning release key '{}' in HSM slot {}...", label, slot);
            let ctx = Pkcs11::new(module)?;
            let slot = Slot::try_from(slot).map_err(|e| anyhow::anyhow!("Invalid slot: {}", e))?;
            let session = ctx.open_rw_session(slot)?;
            
            let pin_secret = Secret::new(pin);
            session.login(UserType::User, Some(&pin_secret))?;
            
            // Define templates for public and private keys
            let pub_label = format!("{}-public", label);
            let priv_label = format!("{}-private", label);

            let pub_template = vec![
                Attribute::Token(true),
                Attribute::Label(pub_label.as_bytes().to_vec()),
                Attribute::Class(ObjectClass::PUBLIC_KEY),
                Attribute::Verify(true),
                Attribute::ModulusBits(2048.into()),
                Attribute::PublicExponent(vec![0x01, 0x00, 0x01]), // 65537
            ];

            let priv_template = vec![
                Attribute::Token(true),
                Attribute::Label(priv_label.as_bytes().to_vec()),
                Attribute::Class(ObjectClass::PRIVATE_KEY),
                Attribute::Sign(true),
                Attribute::Sensitive(true),
                Attribute::Private(true),
            ];

            let mechanism = Mechanism::RsaPkcsKeyPairGen;
            let (pub_handle, _priv_handle) = session.generate_key_pair(&mechanism, &pub_template, &priv_template)?;

            // Extract public key for standalone verification distribution
            let pub_attr_template = vec![AttributeType::Modulus, AttributeType::PublicExponent];
            let pub_attrs = session.get_attributes(pub_handle, &pub_attr_template)?;
            
            let modulus = pub_attrs.iter()
                .find(|a| matches!(a.attribute_type(), AttributeType::Modulus))
                .and_then(|a| if let Attribute::Modulus(m) = a { Some(m) } else { None })
                .ok_or_else(|| anyhow::anyhow!("Failed to retrieve modulus"))?;
            
            let exponent = pub_attrs.iter()
                .find(|a| matches!(a.attribute_type(), AttributeType::PublicExponent))
                .and_then(|a| if let Attribute::PublicExponent(e) = a { Some(e) } else { None })
                .ok_or_else(|| anyhow::anyhow!("Failed to retrieve exponent"))?;

            let pub_key_file = format!("{}.pub.json", label);
            let mut f = File::create(&pub_key_file)?;
            let key_data = serde_json::json!({
                "label": label,
                "modulus": hex::encode(modulus),
                "exponent": hex::encode(exponent),
            });
            serde_json::to_writer_pretty(&mut f, &key_data)?;

            println!("SUCCESS: Key pair '{}' generated and persisted in HSM.", label);
            println!("Public key metadata saved to {}", pub_key_file);
        }
        Commands::Sign { module, slot, pin, label, file } => {
            let priv_label = format!("{}-private", label);
            println!("Signing {} with HSM key '{}'...", file, priv_label);
            
            let mut f = File::open(&file)?;
            let mut data = Vec::new();
            f.read_to_end(&mut data)?;
            
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let hash = hasher.finalize();

            let ctx = Pkcs11::new(module)?;
            let slot = Slot::try_from(slot).map_err(|e| anyhow::anyhow!("Invalid slot: {}", e))?;
            let session = ctx.open_ro_session(slot)?;
            
            let pin_secret = Secret::new(pin);
            session.login(UserType::User, Some(&pin_secret))?;

            // Find the private key by label
            let template = vec![
                Attribute::Label(priv_label.as_bytes().to_vec()),
                Attribute::Class(ObjectClass::PRIVATE_KEY),
            ];
            let objects = session.find_objects(&template)?;
            let key = objects.first().ok_or_else(|| anyhow::anyhow!("Key '{}' not found in HSM", priv_label))?;

            // Sign the hash using RSA PKCS#1
            let mechanism = Mechanism::RsaPkcs;
            let signature = session.sign(&mechanism, *key, &hash)?;

            let sig_file = format!("{}.sig", file);
            let mut out = File::create(&sig_file)?;
            out.write_all(&signature)?;
            
            println!("SUCCESS: Detached signature saved to {}", sig_file);
        }
        Commands::Verify { file, signature, public_key } => {
            println!("Verifying {} against signature {} using {}...", file, signature, public_key);
            
            // Read public key (JSON format generated by provision)
            let mut pk_file = File::open(&public_key)?;
            let pk_data: serde_json::Value = serde_json::from_reader(&mut pk_file)?;
            
            let modulus_hex = pk_data["modulus"].as_str().ok_or_else(|| anyhow::anyhow!("Invalid public key: missing modulus"))?;
            let exponent_hex = pk_data["exponent"].as_str().ok_or_else(|| anyhow::anyhow!("Invalid public key: missing exponent"))?;
            
            let modulus = hex::decode(modulus_hex)?;
            let exponent = hex::decode(exponent_hex)?;
            
            let n = rsa::BigUint::from_bytes_be(&modulus);
            let e = rsa::BigUint::from_bytes_be(&exponent);
            
            let pub_key = RsaPublicKey::new(n, e).map_err(|e| anyhow::anyhow!("Invalid RSA components: {}", e))?;
            
            // Read file and hash
            let mut f = File::open(&file)?;
            let mut data = Vec::new();
            f.read_to_end(&mut data)?;
            
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let hash = hasher.finalize();
            
            // Read signature
            let mut s_file = File::open(&signature)?;
            let mut sig_bytes = Vec::new();
            s_file.read_to_end(&mut sig_bytes)?;
            
            // Verify
            use rsa::Pkcs1v15Sign;
            pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hash, &sig_bytes)
                .map_err(|_| anyhow::anyhow!("Signature verification FAILED"))?;
            
            println!("SUCCESS: Signature is VALID.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::signature::{Signer, SignatureEncoding};
    use rsa::pkcs1v15::SigningKey;
    use rsa::RsaPrivateKey;
    use rand::thread_rng;

    #[test]
    fn test_verify_logic() -> anyhow::Result<()> {
        // Generate a temporary key for testing
        let mut rng = thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, 2048)?;
        let pub_key = RsaPublicKey::from(&priv_key);

        let data = b"hello tos release";
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        // Sign using the rsa crate (to simulate HSM signature)
        let signing_key = SigningKey::<Sha256>::new(priv_key);
        let signature = signing_key.sign(data);

        // Verify using our logic (RsaPublicKey::verify)
        use rsa::Pkcs1v15Sign;
        pub_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hash, &signature.to_bytes())
            .map_err(|_| anyhow::anyhow!("Verification failed"))?;

        Ok(())
    }

    #[test]
    fn test_public_key_json_roundtrip() -> anyhow::Result<()> {
        let n = rsa::BigUint::from(12345u32);
        let e = rsa::BigUint::from(65537u32);
        
        let json = serde_json::json!({
            "label": "test",
            "modulus": hex::encode(n.to_bytes_be()),
            "exponent": hex::encode(e.to_bytes_be()),
        });

        let modulus_hex = json["modulus"].as_str().unwrap();
        let exponent_hex = json["exponent"].as_str().unwrap();
        
        let n_decoded = rsa::BigUint::from_bytes_be(&hex::decode(modulus_hex)?);
        let e_decoded = rsa::BigUint::from_bytes_be(&hex::decode(exponent_hex)?);
        
        assert_eq!(n, n_decoded);
        assert_eq!(e, e_decoded);
        
        Ok(())
    }
}

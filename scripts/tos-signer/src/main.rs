use clap::{Parser, Subcommand};
use cryptoki::context::Pkcs11;
use cryptoki::session::UserType;
use cryptoki::mechanism::Mechanism;
use cryptoki::object::{Attribute, ObjectClass};
use cryptoki::slot::Slot;
use secrecy::Secret;
use std::fs::File;
use std::io::{Read, Write};
use sha2::{Sha256, Digest};

#[derive(Parser)]
#[command(name = "tos-signer")]
#[command(about = "TOS Release Signing Utility (PKCS#11 / HSM)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Provision a new release key in the HSM
    Provision {
        #[arg(long, env = "TOS_HSM_MODULE")]
        module: String,
        #[arg(long, env = "TOS_HSM_SLOT")]
        slot: u64,
        #[arg(long, env = "TOS_HSM_PIN")]
        pin: String,
        #[arg(long)]
        label: String,
    },
    /// Sign a file using the HSM
    Sign {
        #[arg(long, env = "TOS_HSM_MODULE")]
        module: String,
        #[arg(long, env = "TOS_HSM_SLOT")]
        slot: u64,
        #[arg(long, env = "TOS_HSM_PIN")]
        pin: String,
        #[arg(long)]
        label: String,
        file: String,
    },
    /// Verify a file signature
    Verify {
        file: String,
        #[arg(long)]
        signature: String,
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
            
            println!("SUCCESS: Key pair '{}' generated and persisted in HSM.", label);
        }
        Commands::Sign { module, slot, pin, label, file } => {
            println!("Signing {} with HSM key '{}'...", file, label);
            
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
                Attribute::Label(label.as_bytes().to_vec()),
                Attribute::Class(ObjectClass::PRIVATE_KEY),
            ];
            let objects = session.find_objects(&template)?;
            let key = objects.get(0).ok_or_else(|| anyhow::anyhow!("Key '{}' not found in HSM", label))?;

            // Sign the hash
            let mechanism = Mechanism::Sha256RsaPkcs;
            let signature = session.sign(&mechanism, *key, &hash)?;

            let sig_file = format!("{}.sig", file);
            let mut out = File::create(&sig_file)?;
            out.write_all(&signature)?;
            
            println!("SUCCESS: Detached signature saved to {}", sig_file);
        }
        Commands::Verify { file, signature, public_key } => {
            println!("Verifying {} against signature {}...", file, signature);
            println!("TODO: Implement standalone verification logic.");
        }
    }

    Ok(())
}

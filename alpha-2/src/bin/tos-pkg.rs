use std::path::PathBuf;
use tos_alpha2::services::marketplace::MarketplaceService;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("TOS-PKG // Package Management Utility");
        println!("Usage: tos-pkg <command> [args]");
        println!("Commands:");
        println!("  discover <path>  Inspect a module manifest");
        println!("  verify   <path>  Cryptographically verify a module");
        return Ok(());
    }

    let cmd = &args[1];
    match cmd.as_str() {
        "discover" => {
            if args.len() < 3 { return Err(anyhow::anyhow!("Missing path")); }
            let path = PathBuf::from(&args[2]);
            let manifest = MarketplaceService::discover_module_local(path)?;
            println!("MATCH FOUND:");
            println!("  ID:      {}", manifest.id);
            println!("  NAME:    {}", manifest.name);
            println!("  VERSION: {}", manifest.version);
            println!("  TYPE:    {}", manifest.module_type);
            println!("  AUTHOR:  {}", manifest.author);
        },
        "verify" => {
             if args.len() < 3 { return Err(anyhow::anyhow!("Missing path")); }
             let path = PathBuf::from(&args[2]);
             let manifest = MarketplaceService::discover_module_local(path)?;
             let pk = MarketplaceService::get_trusted_public_key()?;
             if MarketplaceService::verify_manifest_local(&manifest, &pk) {
                 println!("VERIFICATION: SUCCESS ✅ (Signed by TOS Core)");
             } else {
                 println!("VERIFICATION: FAILED ❌ (Invalid Signature)");
             }
        },
        _ => println!("ERROR: Unknown command '{}'", cmd),
    }

    Ok(())
}

use tos_alpha2::services::marketplace::{MarketplaceService, ModuleManifest};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand_core::OsRng;

#[test]
fn test_manifest_cryptographic_verification() {
    // 1. Generate a mock keypair for the test
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let public_key: VerifyingKey = signing_key.verifying_key();

    // 2. Create a mock manifest payload
    let mut manifest = ModuleManifest {
        id: "test.module.01".to_string(),
        name: "Security Validator".to_string(),
        version: "1.0.0".to_string(),
        module_type: "Test".to_string(),
        author: "TOS Core Team".to_string(),
        signature: None,
    };

    // 3. Generate the expected payload hash
    let payload = format!(
        "{}:{}:{}:{}:{}",
        manifest.id,
        manifest.name,
        manifest.version,
        manifest.module_type,
        manifest.author
    );

    // 4. Sign the payload and attach to manifest
    let signature = signing_key.sign(payload.as_bytes());
    manifest.signature = Some(hex::encode(signature.to_bytes()));

    // 5. Verify the manifest using the module's validation logic
    let is_valid = MarketplaceService::verify_manifest(&manifest, &public_key);
    assert!(is_valid, "Valid cryptographic signature was rejected");

    // 6. Test Tamper Detection: Modify the author field after signing
    manifest.author = "Malicious Actor".to_string();
    let is_valid_tampered = MarketplaceService::verify_manifest(&manifest, &public_key);
    assert!(!is_valid_tampered, "Tampered manifest bypassed cryptographic validation!");
}

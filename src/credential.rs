use anyhow::{Context, Result};
use windows::core::PCWSTR;
use windows::Win32::Foundation::FILETIME;
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredReadW, CredWriteW, CREDENTIALW, CREDENTIAL_ATTRIBUTEW, CRED_FLAGS,
    CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
};

const TARGET_NAME: &str = "ClipboardTranslator_APIKey";

/// Windows Credential ManagerにAPIキーを保存
pub fn save_api_key(api_key: &str) -> Result<()> {
    unsafe {
        let target_name = encode_wide(TARGET_NAME);
        let credential_blob = api_key.as_bytes();

        let mut cred = CREDENTIALW {
            Flags: CRED_FLAGS(0),
            Type: CRED_TYPE_GENERIC,
            TargetName: windows::core::PWSTR(target_name.as_ptr() as *mut u16),
            Comment: windows::core::PWSTR::null(),
            LastWritten: FILETIME::default(),
            CredentialBlobSize: credential_blob.len() as u32,
            CredentialBlob: credential_blob.as_ptr() as *mut u8,
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: std::ptr::null_mut::<CREDENTIAL_ATTRIBUTEW>(),
            TargetAlias: windows::core::PWSTR::null(),
            UserName: windows::core::PWSTR::null(),
        };

        CredWriteW(&mut cred, 0).context("Failed to write credential")?;
    }

    Ok(())
}

/// Windows Credential ManagerからAPIキーを読み込み
pub fn load_api_key() -> Result<String> {
    unsafe {
        let target_name = encode_wide(TARGET_NAME);
        let mut pcredential: *mut CREDENTIALW = std::ptr::null_mut();

        CredReadW(
            PCWSTR(target_name.as_ptr()),
            CRED_TYPE_GENERIC,
            0,
            &mut pcredential,
        )
        .context("Failed to read credential")?;

        if pcredential.is_null() {
            anyhow::bail!("Credential not found");
        }

        let cred = &*pcredential;
        let blob =
            std::slice::from_raw_parts(cred.CredentialBlob, cred.CredentialBlobSize as usize);
        let api_key = String::from_utf8(blob.to_vec()).context("Invalid UTF-8 in credential")?;

        // メモリ解放
        windows::Win32::Security::Credentials::CredFree(pcredential as *const _);

        Ok(api_key)
    }
}

/// Windows Credential ManagerからAPIキーを削除
pub fn delete_api_key() -> Result<()> {
    unsafe {
        let target_name = encode_wide(TARGET_NAME);
        CredDeleteW(
            PCWSTR(target_name.as_ptr()),
            CRED_TYPE_GENERIC,
            0,
        )
        .ok()
        .context("Failed to delete credential")?;
    }

    Ok(())
}

/// APIキーが保存されているかチェック
pub fn has_api_key() -> bool {
    load_api_key().is_ok()
}

/// UTF-16に変換（null終端付き）
fn encode_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

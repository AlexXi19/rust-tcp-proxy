extern crate tcp_proxy;
use tcp_proxy::proxy::crypto;

#[test]
fn test_cypto_identity() {
    let data = vec![1, 2, 3, 4, 5];
    let result = crypto::identity(data).expect("identity failed");
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_cypto_encryption() {
    let data = vec![1, 2, 3, 4, 5];
    std::env::set_var("AES_GCM_KEY", "01234567890123456789012345678901");
    let result = crypto::encrypt(data.clone()).expect("encryption failed");
    let decrypted_data = crypto::decrypt(result).expect("decryption failed");
    assert_eq!(decrypted_data, data);
}

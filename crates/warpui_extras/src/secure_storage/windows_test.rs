use super::SecureStorage;

#[test]
fn test_encrypt_decrypt_returns_same_value() {
    let key = String::from("key");
    let inputs = [
        "freckles grain uncaring strict stumbling reappear basil uproar",
        "ideology shifting overview cognition uniformed armory mummify editor",
        "",
        "{",
        "\'",
        "\"",
        "{\"test\"}",
        "defender french skating sweat neurotic extruding cadet mute headcount unaligned prognosis heroics geography deafening customer juicy scuttle blissful scrambler spleen embark engine shield banter botanist singing plutonium grafted carton playable approve astonish",
        "{\"api_keys\":{\"suggestions\":\"test-key\",\"acp\":\"test-key\"},\"settings\":{\"backend\":\"codex\",\"model\":\"gpt-5.5\",\"reasoning_effort\":\"xhigh\"}}",
    ].map(String::from);

    fn encrypt_then_decrypt(key: &str, input: String) -> String {
        let encrypted = SecureStorage::encrypt(key, input).unwrap();
        SecureStorage::decrypt(encrypted).unwrap()
    }

    for input in inputs {
        assert_eq!(
            encrypt_then_decrypt(&key, input.to_owned()),
            input,
            "Encrypting and decrypting {input:?}"
        );
    }
}

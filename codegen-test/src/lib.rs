pub mod models {
    include!(concat!(env!("OUT_DIR"), "/asyncapi.rs"));
}

#[cfg(test)]
mod test {
    use crate::models::SampleRequestPayload;

    #[test]
    fn test_parse_internally_tagged_enum_with_custom_value() {
        let raw_json = r#"{
            "id": "123",
            "kind": "request",
            "event": "deezNuts",
            "data": {
                "userId": "123"
            }
        }"#;
        let message_parse = serde_json::from_str::<SampleRequestPayload>(raw_json);
        println!("message_parse: {:?}", message_parse);

        assert!(message_parse.is_ok());
        let get_user = match message_parse.unwrap() {
            SampleRequestPayload::GetUser(x) => x,
            _ => panic!("Expected GetUser"),
        };
        assert_eq!(get_user.id, "123");
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    update_id: u64,
    pub message: Option<Message>,
    pub edited_message: Option<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub message_id: u64,
    from: User,
    pub chat: Chat,
    date: u64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: u64,
    is_bot: bool,
    first_name: String,
    username: String,
    language_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chat {
    pub id: u64,
    first_name: String,
    username: String,
    #[serde(rename = "type")]
    chat_type: String,
}

#[derive(Serialize, Debug)]
pub struct ResponseBody {
    pub method: String,
    pub chat_id: u64,
    pub text: String,
    pub reply_to_message_id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_deserialize_update_with_message() {
        let json = "{\"update_id\":459592837, \"message\":{\"message_id\":7,\"from\":{\"id\":247673932,\"is_bot\":false,\"first_name\":\"Liang\",\"username\":\"liul85\",\"language_code\":\"en\"},\"chat\":{\"id\":247673932,\"first_name\":\"Liang\",\"username\":\"liul85\",\"type\":\"private\"},\"date\":1631506802,\"text\":\"@KFC chicken 12.9 AUD CBA > food\"}}";
        let update: Update = serde_json::from_str(&json).unwrap();
        assert_eq!(update.update_id, 459592837);
        assert_eq!(
            update.message.unwrap().text,
            "@KFC chicken 12.9 AUD CBA > food"
        );
    }

    #[test]
    fn it_deserialize_update_with_edited_message() {
        let json = "{\"update_id\":459593047,\"edited_message\":{\"message_id\":276,\"from\":{\"id\":247673932,\"is_bot\":false,\"first_name\":\"Liang\",\"username\":\"liul85\",\"language_code\":\"en\"},\"chat\":{\"id\":247673932,\"first_name\":\"Liang\",\"username\":\"liul85\",\"type\":\"private\"},\"date\":1640933453,\"edit_date\":1640933464,\"text\":\"2021-12-30 @Coles 30 cba > food\",\"entities\":[{\"offset\":11,\"length\":6,\"type\":\"mention\"}]}}";
        let update: Update = serde_json::from_str(&json).unwrap();
        assert_eq!(update.update_id, 459593047);
        assert!(update.message.is_none());
        assert_eq!(
            update.edited_message.unwrap().text,
            "2021-12-30 @Coles 30 cba > food"
        );
    }
}

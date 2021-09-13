use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    update_id: u64,
    message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    message_id: u64,
    from: User,
    chat: Chat,
    date: u64,
    text: String,
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
    id: u64,
    first_name: String,
    username: String,
    #[serde(rename = "type")]
    chat_type: String,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_deserialize_update() {
        let json = "{\"update_id\":459592837, \"message\":{\"message_id\":7,\"from\":{\"id\":247673932,\"is_bot\":false,\"first_name\":\"Liang\",\"username\":\"liul85\",\"language_code\":\"en\"},\"chat\":{\"id\":247673932,\"first_name\":\"Liang\",\"username\":\"liul85\",\"type\":\"private\"},\"date\":1631506802,\"text\":\"@KFC chicken 12.9 AUD CBA > food\"}}";
        let update: Update = serde_json::from_str(&json).unwrap();
        assert_eq!(update.update_id, 459592837);
        assert_eq!(update.message.text, "@KFC chicken 12.9 AUD CBA > food");
    }
}

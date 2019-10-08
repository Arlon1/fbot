use qedchat::*;

use crate::abstractBot::Bot;

struct SimpleBot ();
impl Bot for SimpleBot {
    fn botlogic(&self, post: &RecvPost) -> Option<SendPost> {
        let mut post = post.post.clone();
        if post.message.starts_with("!caro") {
            post.name = "Caro(-line)".to_owned();
            post.message = "pong".to_owned();
            post.bottag = BotTag::Bot;
            post.delay += 1;
            Some(SendPost {
                post,
                publicid: true,
            })
        } else {
            None
        }
    }
}

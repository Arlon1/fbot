//use qedchat::{BotTag, Client, Post, SendPost};


//n botloop(channel: String) {

pub fn parse_post(post: qedchat::RecvPost) -> (qedchat::SendPost, bool) {
    let post = qedchat::SendPost {
        post: qedchat::Post {
            name: "Rita".to_owned(),
            message: "pong".to_owned(),
            channel: post.post.channel,
            bottag: qedchat::BotTag::Bot,
            delay: post.id + 1,
        },
        publicid: false,
    };
    if post.post.message == "!rita ping" {
        return (post, true);
    }
    return (post, false);
}

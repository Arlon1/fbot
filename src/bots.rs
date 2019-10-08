use anyhow::Result;
use qedchat::{BotTag, Post, RecvPost, SendPost};

pub trait Bot: Send + Sync {
    fn process(&self, post: &RecvPost) -> Result<Option<SendPost>>;
}

pub struct SimpleBot<T>(T);

impl<T> SimpleBot<T> {
    pub fn new(p: T) -> Self {
        SimpleBot(p)
    }
}

impl<T: Fn(&RecvPost) -> Result<Option<SendPost>> + Send + Sync> Bot for SimpleBot<T> {
    fn process(&self, post: &RecvPost) -> Result<Option<SendPost>> {
        (self.0)(post)
    }
}

pub fn ping_bot() -> impl Bot {
    SimpleBot::new(|post: &RecvPost| {
        Ok(
            if post.post.message == "!rita ping" && post.post.bottag == BotTag::Human {
                Some(SendPost {
                    post: Post {
                        name: "Rita".to_owned(),
                        message: "pong".to_owned(),
                        channel: post.post.channel.clone(),
                        bottag: BotTag::Bot,
                        delay: post.id + 1,
                    },
                    publicid: false,
                })
            } else {
                None
            },
        )
    })
}

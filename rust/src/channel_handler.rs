//extern crate qedchat;
//use qedchat::{BotTag, Client, Post, SendPost};


//n botloop(channel: String) {
//   let client = Client::new("FranzBots", "").unwrap();
//   let mut channel = client.listen_to_channel(channel, -10).unwrap();
//
//   loop {
//       let post = channel.receive().unwrap();
//       println!("[{}] [{}] {}", post.date, post.post.name, post.post.message);
//       if post.post.bottag == BotTag::Human && post.post.message.starts_with("!ping") {
//           channel
//               .send(&SendPost {
//                   post: Post {
//                       name: "fbot".to_owned(),
//                       message: "pong".to_owned(),
//                       channel: post.post.channel,
//                       bottag: BotTag::Bot,
//                       delay: post.id + 1,
//                   },
//                   publicid: false,
//               })
//               .unwrap();
//       }
//   }
//
pub struct ChannelHandler {
    pub channel_name: String,
}

impl ChannelHandler {
    pub fn botloop(&self) {
    }
}

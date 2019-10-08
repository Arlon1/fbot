// alle Bots müssen diesen Trait implementieren
trait Bot: Send + Sync {
    // unsere abstrakte Methode
    fn botlogic(&self, post: &RecvPost) -> Option<SendPost>;

    fn process(&self, post: &RecvPost) -> Option<SendPost> {
        if post.post.bottag == BotTag::Human {
            self.botlogic(post)
        }
        else {
            None
        }
    }
    
}

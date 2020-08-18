--let defaultChannels = [ "", "test" ]
let defaultChannels = [ "fbot" ]

in  { bots =
      { better_link_bot.channels = defaultChannels
      , rita_bot.channels = defaultChannels
      }
    , account = { user = env:fbot_user as Text, pass = env:fbot_pass as Text }
    }

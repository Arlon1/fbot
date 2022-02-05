--let defaultChannels = [ "", "test", "fbot" ]
let defaultChannels = [ "fbot" ]

in  { bots =
      { freiepunkte.channels = defaultChannels
      , nickname.channels = defaultChannels
      , ping_readdb.channels = defaultChannels
      , ping_sendtodb.channels = defaultChannels
      , ritabot.channels = defaultChannels
      , rubenbot.channels = defaultChannels
      }
    , account =
      { user = env:fbot_user as Text ? "", pass = env:fbot_pass as Text ? "" }
    , db =
      { user = "fbot", pass = "", hostname = "localhost", database = "fbot" }
    }

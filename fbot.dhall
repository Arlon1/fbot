--let defaultChannels = [ "", "test" ]
let defaultChannels = [ "fbot" ]

in  { bots =
      { rubenbot.channels = defaultChannels
      , ritabot.channels = defaultChannels
      }
    , account =
      { user = env:fbot_user as Text ? "", pass = env:fbot_pass as Text ? "" }
    }

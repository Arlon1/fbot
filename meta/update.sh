#!/bin/sh

set -e

gh_pages_url='https://arlon1.github.io/fbot'

systemctl stop fbot.service 

wget ${gh_pages_url}/rusty_rita -O /opt/fbot/rusty_rita
wget ${gh_pages_url}/fbot.service -O /etc/systemd/system/fbot.service

systemctl start fbot.service

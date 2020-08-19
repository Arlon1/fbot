#!/bin/sh

set -e
gh_pages_url='https://arlon1.github.io/fbot'
wget ${gh_pages_url}/rusty_rita -O /opt/fbot/rusty_rita
wget ${gh_pages_url}/fbot.service -O /etc/systemd/system/fbot.service
systemctl restart fbot.service

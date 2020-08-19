#!/bin/sh

set -e
gh-pages_url='https://arlon1.github.io/fbot/rusty_rita'
wget ${gh-pages_url}/rusty_rita -O /opt/fbot/rusty_rita
wget ${gh-pages_url}/fbot.service -O /etc/systemd/system/fbot.service
systemctl restart fbot.service

#!/bin/sh

URL="https://dieprobezeit.ch"
curl -f -s -o /dev/null --connect-timeout 10 $URL

if [ $? -ne 0 ]; then
    echo "HTTPS failed at $(date)" >> /var/log/https-monitor.log
    docker compose restart pingoo
fi

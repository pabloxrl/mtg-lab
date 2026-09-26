#!/usr/bin/env bash
set -euo pipefail
label="com.mtg-lab.symphony"
domain="gui/$(id -u)"
plist="$HOME/Library/LaunchAgents/$label.plist"
case "${1:-status}" in
  start) launchctl bootstrap "$domain" "$plist" ;;
  stop) launchctl bootout "$domain/$label" ;;
  restart) launchctl kickstart -k "$domain/$label" ;;
  status) launchctl print "$domain/$label" ;;
  *) echo "Usage: $0 {start|stop|restart|status}" >&2; exit 2 ;;
esac

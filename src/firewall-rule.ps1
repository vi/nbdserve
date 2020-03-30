New-NetFirewallRule -Name nbd -DisplayName 'net block device (nbd)' -Enabled True -Direction Inbound -Protocol TCP -Action Allow -LocalPort 10809
